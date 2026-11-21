> **Attack Shark X3 (1d57:fa60) — experimental fork.** This tree targets the X3
> receiver on Linux. Writes are **read-only by default**; connecting never
> applies a saved configuration. Read [`docs/HARDWARE_VALIDATION.md`](docs/HARDWARE_VALIDATION.md)
> before enabling hardware writes. Original project: clevim/OpenSharkX11 (MIT).

<div align="center">
  <img src="assets/icon.png" width="108" alt="OpenSharkX3">
  <h1>OpenSharkX3</h1>
  <p>Native Linux configurator for the <strong>Attack Shark X3</strong> mouse</p>

  <p>
    <img src="https://img.shields.io/badge/platform-linux-informational?style=flat-square&logo=linux&logoColor=white">
    <img src="https://img.shields.io/badge/tauri-2-ffc131?style=flat-square&logo=tauri&logoColor=black">
    <img src="https://img.shields.io/badge/rust-backend-dea584?style=flat-square&logo=rust&logoColor=white">
    <img src="https://img.shields.io/badge/react-19-61dafb?style=flat-square&logo=react&logoColor=black">
    <img src="https://img.shields.io/badge/license-MIT-brightgreen?style=flat-square">
  </p>
</div>

---

Desktop app (Tauri 2 + Rust backend + React 19 UI) to configure the **Attack Shark X3** (receiver `1d57:fa60`, cable `1d57:fa55`) on Linux — no Windows software needed. Works via **2.4 GHz dongle**, **wired USB-C**, and **Bluetooth 5.0 (BLE)**.

Premium dark desktop interface: Tux branding, a persistent device-status rail, real battery telemetry, a clickable X3 button map and a technical diagnostics console. Turkish is the default language, with English, Portuguese and Chinese available. USB uses the **native Linux hidraw** transport (the kernel driver stays bound; no interface is detached) and BLE uses zbus/BlueZ. Everything runs in Rust — no Node.js runtime.

### Hardware status

| Area | State |
|---|---|
| hidraw discovery, interface 02, descriptor parsing | **verified on hardware** |
| 10-byte vendor preamble `0C 0A 01 FE 01 FE 00 00 00 00` | **accepted + acknowledged** |
| Polling report `0x06` | **verified**: 125 / 250 / 500 / 1000 Hz each measured after writing (125.0, 250.0, 500.1, 984–994 Hz); 1000 Hz restored |
| Battery heartbeat `03 10 40 01 07` | **verified** (level byte) |
| DPI `0x04`, lighting/power `0x05`, buttons `0x08` | built, **not physically verified** — gated behind `OPENSHARKX3_EXPERIMENTAL=1` |
| Custom macro upload | not implemented (layout unverified) |

Every value in the UI is tagged `LOCAL`, `TRANSMITTED`, `ACK` or `VERIFIED`. An acknowledgement is never presented as physical verification.

---

## Features

| Section | What you can configure |
|---|---|
| **DPI** | 6 independent stages · up to 26,000 DPI · color per stage · Angle Snap · Ripple Control |
| **Lighting** | 7 modes (Off, Static, Breathing, Neon, ColorBreathing, StaticDPI, BreathingDPI) · speed · global color |
| **Buttons** | Remap all 8 buttons: native mouse actions, custom keyboard shortcuts |
| **Performance** | Polling rate (125 / 250 / 500 / 1000 Hz) · debounce (4–50 ms) · 250–1000 Hz disabled in USB-C mode (max 125 Hz) |
| **Profiles** | Save, load, rename, duplicate, delete, import and export configuration profiles (never writes to the device) |
| **Battery** <img width="75" height="33" alt="Ekran Görüntüsü 2026-11-21 17-01-29" src="https://github.com/user-attachments/assets/d57032fd-312b-4a9c-b9a1-d7c47254202e" />| Real-time monitor · automatic LED override at critical level |
| **Macros** | Local macro library and timeline editor (key, delay, release, reorder) — stored locally, never uploaded |
| **Diagnostics** | hidraw node discovery · report descriptor dump · real polling-interval measurement |
| **Dashboard** | Live connection, battery, polling, DPI and activity overview · auto-detects 2.4 GHz, USB-C, and Bluetooth |
| **Settings** | Turkish / English / Portuguese / Chinese · dark and light themes · accent color · about and credits |

> **Tri-mode support**: the "Search mouse" button automatically tries 2.4 GHz dongle → USB-C cable → Bluetooth BLE in sequence. A dedicated BT button is also available when disconnected.

---

## Requirements

**Runtime:**
- **Arch / CachyOS**: `sudo pacman -S webkit2gtk-4.1 gtk3 libayatana-appindicator bluez bluez-utils`
- Other distros: **WebKitGTK 4.1** (≥ 2.44 recommended), **GTK 3**, and **BlueZ** (for BLE)
- BLE requires the mouse to be paired before launching (`bluetoothctl pair <MAC>`)
- hidraw access requires the udev rule (installed automatically by the packages)

**Build (from source):**
- `rust` / `cargo`, `nodejs` / `npm`, `pkg-config`, plus the runtime deps above with `-dev`/`-devel` headers
- Arch / CachyOS: `sudo pacman -S rust nodejs npm pkg-config webkit2gtk-4.1`

Full Arch setup, permissions and packaging: [`docs/INSTALLATION_ARCH.md`](docs/INSTALLATION_ARCH.md).

---

## Installation

### Arch / CachyOS — AUR (recommended)

Installs with app menu icon, `.desktop` shortcut, udev rule, and binary at `/usr/bin/opensharkx11`.

**With yay / paru:**
```bash
yay -S opensharkx11-git
# or
paru -S opensharkx11-git
```

**Manual build from the cloned repository:**
```bash
cd aur
makepkg -si
```

To remove:
```bash
sudo pacman -R opensharkx11
```

---

### AppImage — any Linux distro

```bash
# 1. Download OpenSharkX3_2.0.1_amd64.AppImage from the releases page
chmod +x OpenSharkX3_2.0.1_amd64.AppImage

# 2. Install the udev rule (USB access without root)
sudo cp tools/70-attack-shark-x3-hidraw.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger

# 3. Run
./OpenSharkX3_2.0.1_amd64.AppImage
```

---

### .deb — Ubuntu / Debian

```bash
# Download OpenSharkX3_2.0.1_amd64.deb from the releases page
sudo dpkg -i OpenSharkX3_2.0.1_amd64.deb
# udev rule is installed automatically
```

---

### Development — hot reload

```bash
npm install      # frontend deps + Tauri CLI (once)
npm run dev      # tauri dev — compiles the Rust backend and starts Vite HMR
```

The first `npm run dev` compiles the Rust backend (a few minutes); subsequent runs are incremental.

---

## Build distribution packages

```bash
npm run build
# outputs, under src-tauri/target/release/bundle/:
#   appimage/OpenSharkX3_2.0.1_amd64.AppImage
#   deb/OpenSharkX3_2.0.1_amd64.deb
```

> On rolling-release distros (CachyOS/Arch) the AppImage step needs these env vars so the
> bundled `linuxdeploy` works with a modern toolchain:
> `APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 NO_STRIP=1 npm run build`

---

## Commands

```bash
npm run dev              # tauri dev (Rust backend + Vite HMR)
npm run build            # tauri build → AppImage + .deb
npm run dev:renderer     # Vite only (frontend, no Rust — fast UI iteration)
npm run build:renderer   # Vite production build only
```

---

## Project structure

```
src/                    — React 19 frontend (Vite)
├── app.jsx             — root component and global state
├── api.ts              — Tauri IPC wrapper (invoke / listen)
├── sections.jsx        — tab sections (incl. macros + diagnostics)
├── data.jsx            — constants, SVG icons, mouse data, provenance tiers
├── i18n.jsx            — translations and accent color themes
└── style.css           — cockpit design system

src-tauri/              — Rust backend (Tauri 2)
├── src/
│   ├── commands.rs     — #[tauri::command] IPC handlers
│   ├── hidraw.rs       — sysfs discovery, descriptor parser, HIDIOCSFEATURE
│   ├── driver/         — hidraw (USB/2.4 GHz) + BLE (zbus/BlueZ) drivers
│   ├── protocols/      — payload builders (preamble, polling, DPI, prefs, buttons)
│   ├── safety.rs       — read-only / dry-run / hardware write policy
│   ├── state.rs        — config persistence (~/.config/opensharkx3)
│   └── tray.rs         — system tray (battery + connection mode)
├── tests/data/         — captured FA60 interface-2 report descriptor
└── tauri.conf.json

tools/                  — x3_lab.py, probe/read helpers, udev rule
assets/                 — icons + opensharkx11.desktop
aur/                    — PKGBUILD for Arch / CachyOS
docs/                   — protocol, validation, audit, installation
scripts/                — .deb udev install/remove hooks
```

---

## Documentation

- [`docs/PROTOCOL.md`](docs/PROTOCOL.md) — FA60 protocol: transport, preamble, verified polling report, status events, unverified report layouts
- [`docs/HARDWARE_VALIDATION.md`](docs/HARDWARE_VALIDATION.md) — exactly what was observed on hardware and what was not
- [`docs/PROJECT_AUDIT.md`](docs/PROJECT_AUDIT.md) — current implementation status and known limitations
- [`docs/INSTALLATION_ARCH.md`](docs/INSTALLATION_ARCH.md) — Arch install, udev permissions, packaging, write modes
- [`docs/protocol/PROTOCOL_EN.md`](docs/protocol/PROTOCOL_EN.md) — original X11 reverse-engineering notes (also [Português](docs/protocol/PROTOCOL.md) · [中文](docs/protocol/PROTOCOL_ZH.md))

**Never send report `0x0B`** — upstream reports it unpairs the 2.4 GHz dongle.

---

## Credits

- USB driver base: [HarukaYamamoto0/attack-shark-x11-driver](https://github.com/HarukaYamamoto0/attack-shark-x11-driver) (MIT)
- Protocol fixes: [dressedinblack5/attack-shark-x11-electron](https://github.com/dressedinblack5/attack-shark-x11-electron) (MIT)

---

## License

[MIT](LICENSE) © Ruavell
