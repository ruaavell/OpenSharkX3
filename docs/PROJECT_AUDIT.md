# Project audit — OpenSharkX3

Audit of the working tree at the point this document was written. The project is
a fork of the MIT-licensed OpenSharkX11 configurator (Clevs), retargeted to the
Attack Shark X3 receiver `1d57:fa60` on Linux.

The directory is **not a Git repository**; there is no commit history or diff to
compare against. This audit describes the files as they exist on disk.

## Build and test status

| Check | Command | Result |
|---|---|---|
| Rust typecheck | `cargo check` | pass (warnings only) |
| Rust tests | `cargo test` | 41 passed, 0 failed |
| TypeScript | `npx tsc --noEmit` | pass |
| Renderer build | `npx vite build` | pass |
| Release binary | `cargo build --release` (via `tauri build`) | pass |
| `.deb` bundle | `npx tauri build --bundles deb` | pass |
| AppImage bundle | `npm run build` | pass (via the local gdk-pixbuf shim in `scripts/tauri-build.sh`) |

> This Arch install has no `/usr/lib/gdk-pixbuf-2.0/2.10.0` loaders directory
> (gdk-pixbuf 2.44 with built-in loaders), so linuxdeploy's GTK plugin aborted.
> `scripts/tauri-build.sh` detects that and puts a pkg-config shim first on
> `PKG_CONFIG_PATH` for the bundle step only; on systems with a normal loaders
> directory it runs `tauri build` unchanged.

## Architecture

```text
src/                      React 19 UI (Vite, JSX + one TS IPC module)
  app.jsx                 root state, connection lifecycle, explicit writes
  api.ts                  typed Tauri invoke/listen surface
  sections.jsx            dashboard, DPI, lighting, buttons, macros,
                          performance, profiles, diagnostics, settings
  data.jsx                constants, icons, button geometry, provenance tiers
  i18n.jsx                pt / en / zh strings
  style.css               cockpit design system

src-tauri/src/
  hidraw.rs               sysfs discovery, descriptor parser, HIDIOCSFEATURE
  driver/attack_shark.rs  USB transport, telemetry listener, write receipts
  driver/attack_shark_ble.rs  BLE GATT transport (zbus/BlueZ)
  protocols/              report builders: preamble, polling, DPI, prefs,
                          buttons, macros, status decoder
  commands.rs             Tauri commands
  safety.rs               read-only / dry-run / hardware policy
  state.rs                ~/.config/opensharkx3 persistence + profiles
  tray.rs                 system tray
```

## Completed

- **Native hidraw backend.** Discovery by VID/PID/interface, interface 02
  preference, descriptor parsing, feature writes, input monitoring. No USB
  interface is detached and no device number is hard-coded.
- **FA60 protocol.** Verified 10-byte preamble, polling report `0x06` (all four
  rates physically measured after writing), `0x5010` acknowledgement decoding,
  `0x4010` battery heartbeat, `0x1000` DPI stage events. DPI/prefs/button
  builders keep the upstream structure and are explicitly marked unverified.
- **Write safety.** Read-only by default; hardware mode is opt-in; experimental
  reports need a second gate; connecting never writes; custom macro upload
  refused; `0x0B` never sent.
- **Tauri integration.** `connect`, `disconnect`, `battery`, `device_info`,
  `discover_devices`, `device_status`, `descriptor_dump`, `diagnose`,
  `telemetry`, `measure_polling_rate`, `get_config`, `save_config`,
  `apply_config`, `write_setting`, `reset_config`, profile commands.
- **React interface.** Dashboard, connection state, battery, DPI stages,
  polling rate, sensor toggles, buttons, macros, profiles, diagnostics, settings
  and a diagnostic console, all reading real backend values.
- **Provenance display.** Every write receipt is `local`, `transmitted`,
  `acknowledged` or `verified`; the last is never set automatically.

## Fixed during this pass

- `sections.jsx` was missing `MacroSection` and `DiagnosticsSection`, which made
  the Vite production build fail. Both are implemented.
- The DPI auto-apply path called `apply_config` (DPI + lighting + polling +
  buttons). It now calls `write_setting("dpi", …)` so a DPI edit writes only the
  DPI report.
- Profile saving sent the renderer's configuration shape where the backend
  expected `AppState`. `profiles_save` now merges the patch over the current
  state, so saved profiles are complete and loadable.
- Added `save_config`: local-only persistence with no device access, used by the
  macro editor.
- Battery placeholders removed: an unreported battery shows `—`, not `0%`.
- Stale `X3_ALLOW_WRITES` references replaced with `OPENSHARKX3_WRITE_MODE`
  (`start.sh`, BLE driver). `start.sh` now defaults to read-only.
- Settings and profile panels now name the X3 and the `opensharkx3` directory.
- Removed the unused `rusb` dependency (hidraw replaced libusb).
- AUR package installs the hidraw udev rule (the X11 USB rule alone does not
  grant `/dev/hidraw*` access).
- AppImage bundling fixed on this install by `scripts/tauri-build.sh`, a
  conditional gdk-pixbuf pkg-config shim used only when the loaders directory
  is missing.
- Added unit tests for `save_config`, renderer-patch/profile merging, value
  clamping and the explicit `write_setting` targets (41 tests total).
- Added a read-only `measure` command to `tools/x3_lab.py` and measured the
  real polling interval at every supported rate; results are in
  `HARDWARE_VALIDATION.md`.

## Known limitations

- DPI, lighting/power and button reports are not physically verified. The app
  says so wherever they appear.
- Custom macro upload is not implemented; the UI stores macros locally.
- `GET_FEATURE` readback is not usable on this firmware, so settings cannot be
  read back from the device.
- BLE writes are not implemented for the X3 (GATT layout unverified).
- The project still uses the package/binary name `opensharkx11` for continuity
  with the upstream packaging; user-facing strings say X3.
- Rust build emits dead-code warnings for protocol helpers kept for reference.

## Next hardware steps

See the "Remaining hardware tasks" section of
[`HARDWARE_VALIDATION.md`](HARDWARE_VALIDATION.md).
