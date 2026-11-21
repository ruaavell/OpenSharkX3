# Hardware validation — Attack Shark X3 (1d57:fa60)

Validation log for the physical receiver used during development. Everything
here was observed on that unit; nothing is inferred from the X11 protocol.

## Environment

| Item | Value |
|---|---|
| Device | Attack Shark X3 |
| Receiver | USB `1d57:fa60` |
| OS | Arch Linux |
| Transport | native Linux hidraw, interface 02 |
| Wine | not usable for configuration (see below) |

## Confirmed results

### 1. Wine is not a viable configuration transport

Wine's HID descriptor parser reports a **1-byte feature report** for the
configuration collection. The vendor preamble cannot be sent through it, so all
Wine-based attempts were abandoned. This is a Wine limitation, not a device
failure.

### 2. Native hidraw communication works

- Interface **02** is the configuration interface (vendor-defined feature
  reports plus the `0x03` input channel).
- `HIDIOCSFEATURE` reaches the firmware; `HIDIOCGFEATURE` blocks until timeout
  and is not used in the normal path.
- The kernel driver stays bound. No `USBDEVFS_DISCONNECT`, no libusb claim.

### 3. The 10-byte preamble is accepted

```text
sent: 0C 0A 01 FE 01 FE 00 00 00 00
```

The descriptor declares report `0x0C` as 6 bytes. The firmware accepted the
10-byte buffer. Without this preamble a configuration report is acknowledged and
then silently ignored.

### 4. Device acknowledgement

```text
received: 03 10 50 00 0C
```

Code `0x5010`, acknowledged report `0x0C`. The same frame shape is used for
other reports, with the report ID in the high value byte.

### 5. Polling rate — written and physically measured (2026-11-21)

Each write is a 10-byte preamble followed by report `0x06`, then a 2–4 second
measurement on the mouse interface (`tools/x3_lab.py measure`). The measurement
opens `/dev/hidraw5` (interface 1) read-only and timestamps every real input
report; it never writes.

| Setting written | Report sent | Median interval | Jitter (MAD) | Measured | Samples |
|---|---|---|---|---|---|
| 1000 Hz (baseline) | — | 1000.9 µs | 9.8 µs | **999.1 Hz** | 952 |
| 125 Hz | `06 09 01 08 F7 00 00 00 00` | 8002.0 µs | 3.9 µs | **125.0 Hz** | 106 |
| 250 Hz | `06 09 01 04 FB 00 00 00 00` | 4000.4 µs | 6.2 µs | **250.0 Hz** | 358 |
| 500 Hz | `06 09 01 02 FD 00 00 00 00` | 1999.6 µs | 12.4 µs | **500.1 Hz** | 1941 |
| 1000 Hz (verify) | `06 09 01 01 FE 00 00 00 00` | 1015.9 µs | 39.4 µs | **984.4 Hz** | 787 |
| 1000 Hz (final restore) | `06 09 01 01 FE 00 00 00 00` | 1006.3 µs | 18.5 µs | **993.8 Hz** | 888 |

All four supported rates therefore have a **physically verified** effect on this
receiver. The slight under-run at 1000 Hz (984–994 Hz instead of 1000.0) is
normal scheduling/wireless jitter, not a protocol error.

#### A dropped report is not a protocol error

During the matrix, one 500 Hz write and later one 1000 Hz write returned success
at the USB layer but produced **no `0x5010` acknowledgement**, and the next
measurement still showed the previous rate (250.0 Hz and 500.3 Hz respectively).
Re-sending the same report produced the expected acknowledgement and the correct
measured rate. This is why the application records `acknowledged` separately
from `verified`, and why a polling write is only called verified after an
independent measurement.

The device was left at **1000 Hz**, confirmed by the final 993.8 Hz measurement.


### 6. Battery / status heartbeat

```text
received: 03 10 40 01 07
```

Repeats roughly every 2 seconds. Code `0x4010`, level `0x07` (70%), sleep flag
`0x00` (awake).

### 7. DPI button event

```text
received: 03 00 10 0N 00
```

`N` cycles 1..6, proving six DPI stages. The UI converts this to 0-based
indices.

## Evidence tier per report

| Report | Built | Transmitted | Acknowledged | Physically verified |
|---|---|---|---|---|
| Preamble `0x0C` | yes | yes | yes | yes (enables later writes) |
| Polling `0x06` | yes | yes | yes | **yes** (all four rates measured) |
| DPI `0x04` | yes | no | no | no |
| Prefs `0x05` | yes | no | no | no |
| Buttons `0x08` | yes | no | no | no |
| Custom macro | no (refused) | no | no | no |

The desktop UI exposes exactly these tiers: `LOCAL`, `TRANSMITTED`, `ACK`,
`VERIFIED`. A receipt is never labelled `VERIFIED` automatically.

## Write safety rules enforced in code

- Default mode is **read-only** (`OPENSHARKX3_WRITE_MODE` unset).
- Connecting to the device never sends a configuration report.
- Every write requires an explicit user action (an Apply button or a single
  `write_setting` call).
- DPI, lighting/power and button reports additionally require
  `OPENSHARKX3_EXPERIMENTAL=1`.
- The custom-macro upload is refused outright.
- `0x0B` is never sent.

## Remaining hardware tasks

These require the physical mouse and have **not** been done:

1. Send one DPI stage change and confirm the cursor speed changes and survives a
   power cycle.
2. Send a lighting change and confirm the LED actually changes.
3. Send a button remap with a harmless target and confirm it in `evtest`.
4. Capture the vendor software (Windows/USBPcap) while changing each setting to
   compare the FA60 report layout with the upstream structure.
5. Confirm the battery level byte against a known charge state (charging vs
   discharging).
6. Custom macro layout capture before enabling any upload path.
7. Characterise the occasional RF-dropped configuration report (no `0x5010`
   ack) and decide whether the app should retry once automatically; for now the
   receipt says it was not acknowledged.

## Reproduction

```bash
# read-only inspection
python3 tools/x3_lab.py descriptor
python3 tools/x3_lab.py listen --seconds 10

# read-only: observe the real input interval (move the mouse during the window)
python3 tools/x3_lab.py measure --seconds 2

# dry-run: prints the exact bytes, sends nothing
python3 tools/x3_lab.py polling --hz 125

# explicit write (only with a deliberate reason)
python3 tools/x3_lab.py polling --hz 125 --yes
```

`tools/x3_lab.py` validates every non-preamble report against the live
descriptor before writing.
