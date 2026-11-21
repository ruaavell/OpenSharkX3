# Attack Shark X3 — FA60 protocol

This document describes the **receiver** identity `1d57:fa60` (2.4 GHz dongle)
and the wired identity `1d57:fa55`. It records only what has been observed on
this unit or is explicitly labelled as unverified.

> Read [`HARDWARE_VALIDATION.md`](HARDWARE_VALIDATION.md) before sending
> anything. A USB acknowledgement is **not** proof that a setting took effect.

## 1. Transport

| Property | Value |
|---|---|
| Vendor / product | `1d57:fa60` (wireless), `1d57:fa55` (USB-C) |
| Interface | **02** — vendor-defined configuration interface |
| Linux transport | `/dev/hidrawN` + `HIDIOCSFEATURE` / `HIDIOCGFEATURE` |
| Report descriptor length | 236 bytes |
| Input channel | report `0x03`, 5 bytes (`03 <code> <value>`) |
| Kernel driver | stays bound; the interface is never detached or claimed |

`GET_FEATURE` on interface 2 blocks until the USB timeout on this firmware.
Configuration is write-only in practice; live state arrives only as unsolicited
input reports.

Discovery in `src-tauri/src/hidraw.rs` is by sysfs identity
(`HID_ID` + `bInterfaceNumber`), never by a hard-coded device number.

### Report lengths declared by the FA60 descriptor

| Report | Bytes | Report | Bytes |
|---|---|---|---|
| `0x04` | 52 | `0x0A` | 8 |
| `0x05` | 13 | `0x0B` | 8 |
| `0x06` | 9 | `0x0C` | 6 |
| `0x07` | 8 | `0x10` | 8 |
| `0x08` | 59 | `0x22` | 132 |
| `0x09` | 64 | `0x23` | 262 |
| | | `0xA0` | 8 |

## 2. Write sequence

Every configuration write is preceded by a 10-byte preamble and separated from
the next report by at least 120 ms.

```text
preamble (10 bytes, feature report 0x0C)
0C 0A 01 FE 01 FE 00 00 00 00

wait 120 ms

report  (feature report <id>, length byte = total length)
```

The descriptor declares report `0x0C` as **6** bytes. The vendor software sends
10 and this receiver acknowledges it. hidraw passes the full buffer to the USB
stack; the firmware, not the descriptor, decides what it accepts.

**Status of the preamble: accepted and acknowledged on hardware.**

## 3. Device acknowledgements and events

Input reports on interface 2, report ID `0x03`:

```text
byte 0   0x03                report ID
byte 1   code low
byte 2   code high
byte 3   value low
byte 4   value high
```

| Bytes | Code | Meaning | Status |
|---|---|---|---|
| `03 10 50 00 0C` | `0x5010` | write acknowledgement for report `0x0C` | observed |
| `03 10 50 00 06` | `0x5010` | write acknowledgement for polling report `0x06` | observed |
| `03 10 40 01 07` | `0x4010` | battery heartbeat, level `0x07`, sleep flag `0x00` | observed every ~2 s |
| `03 00 10 0N 00` | `0x1000` | DPI button pressed, stage `N` (1..6) | observed |
| `03 10 40 00 05` | `0x4010` | battery heartbeat, level `0x05`, sleep flag `0x01` | format inferred |

Battery level is the high byte of the value, 1..10 in 10% steps. A level
outside `1..=10` is **not** displayed as a percentage.

`0x5010` acknowledges the report ID in its high value byte. It does not say the
setting was applied.

## 4. Polling rate — report `0x06`

9 bytes. Byte 3 is the divider against 1000 Hz; byte 4 is its one's complement.

```text
06 09 01 <div> <~div & 0xFF> 00 00 00 00
```

| Rate | Divider | Report |
|---|---|---|
| 125 Hz | `0x08` | `06 09 01 08 F7 00 00 00 00` |
| 250 Hz | `0x04` | `06 09 01 04 FB 00 00 00 00` |
| 500 Hz | `0x02` | `06 09 01 02 FD 00 00 00 00` |
| 1000 Hz | `0x01` | `06 09 01 01 FE 00 00 00 00` |

- **All four rates are physically verified on this receiver** by measuring real
  input-report intervals after each write: 125 → 125.0 Hz, 250 → 250.0 Hz,
  500 → 500.1 Hz, 1000 → 984–994 Hz. The device was restored to 1000 Hz.
- A report can be accepted by the USB stack but never acknowledged over RF;
  the next measurement then still shows the previous rate. Re-sending works.
  The app therefore records `acknowledged` and `verified` separately and measures
  the interval independently (`measure_polling_rate`, or
  `tools/x3_lab.py measure`).
- Measurement is read-only: it observes input reports on the mouse interface and
  never writes.

## 5. DPI — report `0x04`

52 bytes, atomic (it carries every stage, the active stage and all colours).

```text
0     0x04      report ID
1     52        length
2     0x01      command / profile
3     lod       0 = 1 mm, 1 = 2 mm
4     ripple    ripple control
5     mask      enabled-stage bitmask (0x3F = six stages)
6     angle     angle snap
7     motion    motion sync
8..15           DPI low bytes, stages 0..7
16..23          DPI high bytes, stages 0..7
24    active    active stage + 1 (1..6)
25..48          8 × RGB triplets
49    0x01      constant
50..51          checksum, big-endian, sum of bytes 3..=49
```

DPI encoding: `wire = dpi / 50 - 1`, 16-bit little-endian split across the two
byte planes. This is **not** the X11 lookup table.

**Status: structure preserved from the upstream project and checked against the
FA60 descriptor. Not physically verified on this unit.** The desktop app gates
this report behind `OPENSHARKX3_EXPERIMENTAL=1`.

## 6. Lighting / power / debounce — report `0x05`

13 bytes on FA60 (upstream vendors build 15 with trailing padding). The layout
in `protocols/user_prefs.rs` follows the upstream structure: mode in the high
nibble, inverted speed, debounce in 2 ms units, deep-sleep split across nibbles,
checksum over bytes 3..10.

**Not physically verified on this unit.** Gated as experimental.

## 7. Button remap — report `0x08`

59 bytes on FA60. Eight button slots at the upstream offsets, with a checksum
over bytes 3..56. The app refuses to build a map that has no left click.

**Not physically verified on this unit.** Gated as experimental.

## 8. Custom macros — not implemented

The upstream four-packet macro layout (`0x09` packets) has not been verified on
this firmware. `build_custom_macro` exists for reference, but the driver refuses
to send it. The UI stores macros locally only.

**Never send report `0x0B`.** Upstream reports that it unpairs the 2.4 GHz
dongle.

## 9. Validation rules used by the app

1. The report ID must exist as a feature report in the live descriptor.
2. The buffer length must equal the declared length, except the 10-byte
   preamble, which is the documented vendor exception.
3. Byte 1 in every report is the total length.
4. A write requires an explicit user action and a write policy that permits it.
5. Connecting, discovering and reading never write anything.
