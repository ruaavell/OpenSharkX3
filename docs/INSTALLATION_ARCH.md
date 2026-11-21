# Arch Linux installation, packaging and permissions

The application is a Tauri 2 desktop binary (`opensharkx11`) with a WebKitGTK
frontend and a Rust backend. It talks to `/dev/hidraw*` directly, so **no root
daemon and no libusb claim** are involved at runtime.

## 1. Dependencies

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 gtk3 libayatana-appindicator \
  bluez bluez-utils \
  nodejs npm rust cargo pkgconf git
```

`bluez`/`bluez-utils` are only needed for the Bluetooth (BLE) transport. USB and
2.4 GHz use hidraw and need no extra library.

## 2. hidraw permissions (required)

Write access to the configuration interface goes through a udev rule:

```bash
sudo install -Dm644 tools/70-attack-shark-x3-hidraw.rules \
  /etc/udev/rules.d/70-attack-shark-x3-hidraw.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
# unplug and re-plug the receiver / cable
```

The rule matches `1d57:fa60` (receiver) and `1d57:fa55` (USB-C cable), sets mode
`0660` and tags the node with `uaccess` so the logged-in seat user gets access.

The file number **must be below 73**. `uaccess` is applied by
`73-seat-late.rules`; a rule named `99-…` is parsed after it and the ACL is never
set. This is why the rule is `70-…`.

Verify:

```bash
ls -l /dev/hidraw*
python3 tools/x3_lab.py descriptor   # read-only, lists feature reports
```

If only root can open the node, the rule did not apply — re-plug the device.

## 3. Build from source

```bash
git clone <repo> OpenSharkX3
cd OpenSharkX3
npm install
npm run build:renderer    # optional: verify the UI alone
cargo check --manifest-path src-tauri/Cargo.toml
npm run build             # tauri build → AppImage + .deb
```

Artifacts land in `src-tauri/target/release/bundle/`.

`npm run build` runs `scripts/tauri-build.sh`, which sets `NO_STRIP=1` and
`APPIMAGE_EXTRACT_AND_RUN=1`. If the system has no gdk-pixbuf loaders directory
(`/usr/lib/gdk-pixbuf-2.0/2.10.0`), linuxdeploy's GTK plugin aborts; the script
detects that and uses a local pkg-config shim for the bundling step. Systems with
a normal loaders directory are unaffected.

Package-specific bundle:

```bash
npm run build -- --bundles deb
```

For development with hot reload:

```bash
npm run dev
```

## 4. Install the AUR package locally

```bash
cd aur
makepkg -si
```

The PKGBUILD builds from the working tree (it does not download sources) and
installs:

- `/usr/bin/opensharkx11`
- `.desktop` entry and icons
- `/usr/lib/udev/rules.d/70-attack-shark-x3-hidraw.rules`
- `/usr/lib/udev/rules.d/99-attack-shark-x11.rules` (USB-level, kept for
  compatibility)

The package name and binary keep the `opensharkx11` name for continuity; the
application itself is the X3 build.

## 5. Write modes

Set through the environment before launching. The default is the safest one.

| `OPENSHARKX3_WRITE_MODE` | Behaviour |
|---|---|
| unset / `readonly` | every write is refused (default) |
| `dryrun` | reports are built and logged, never sent |
| `hardware` | verified reports are sent on explicit user action |

DPI, lighting/power and button reports also require:

```bash
export OPENSHARKX3_EXPERIMENTAL=1
```

Connecting to the mouse is always read-only; no saved configuration is applied
on connect.

Examples:

```bash
# normal, read-only
opensharkx11

# simulate writes
OPENSHARKX3_WRITE_MODE=dryrun opensharkx11

# send the verified polling report on explicit action only
OPENSHARKX3_WRITE_MODE=hardware opensharkx11
```

`./start.sh` wraps this: no flag = read-only, `--dryrun`, or `--write` (asks for
confirmation and sets hardware mode).

## 6. Packaging notes

- The Rust crate is named `opensharkx11`; the Tauri window title and UI say X3.
- State lives in `~/.config/opensharkx3/state.json` and `profiles.json`. A
  pre-rename `~/.config/opensharkx11` file is copied once, never moved.
- No secrets, tokens or passwords are used or stored anywhere in the project.
- `sudo` is only needed for the udev rule and `makepkg -si`.

## 7. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `configuration interface … not found` | receiver unplugged, or no `1d57` hidraw node; check `lsusb` and `ls /sys/class/hidraw` |
| `cannot open /dev/hidrawN read-only` | udev rule missing or not reloaded; re-plug |
| Descriptor shows no feature reports | wrong interface; the app selects interface 02 with feature reports |
| Battery shows `—` | the device has not sent its `0x4010` heartbeat yet; wait a few seconds |
| Write refused | read-only mode; start with `OPENSHARKX3_WRITE_MODE=dryrun`/`hardware` |
| Experimental report refused | set `OPENSHARKX3_EXPERIMENTAL=1` after reading `HARDWARE_VALIDATION.md` |
| Polling measurement reports too few samples | move the mouse during the measurement window |
| Wayland protocol error on launch | the app already sets `WEBKIT_DISABLE_COMPOSITING_MODE=1`; remove the override only if your compositor is fixed |
