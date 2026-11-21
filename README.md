# 🦈 OpenSharkX3

<p align="center">
  <img
    width="150"
    alt="OpenSharkX3"
    src="https://github.com/user-attachments/assets/0f401b0f-5df9-4a46-9445-87727d072f81"
  />
</p>

<h3 align="center">Native Linux Configuration Software for Attack Shark X3</h3>

<p align="center">
  <strong>A modern, open-source desktop companion for your Attack Shark X3 gaming mouse.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/Backend-Rust-orange?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Frontend-React%2019-61DAFB?style=for-the-badge&logo=react&logoColor=black" />
  <img src="https://img.shields.io/badge/Desktop-Tauri%202-FFC131?style=for-the-badge&logo=tauri&logoColor=black" />
  <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" />
  <img src="https://img.shields.io/badge/Open%20Source-Yes-success?style=for-the-badge&logo=github" />
</p>

<p align="center">
  <em>Built with Rust. Designed for Linux. Made for the community.</em>
</p>

---
## 📖 About

**OpenSharkX3** is an independent, open-source Linux desktop application designed to configure and monitor the **Attack Shark X3 wireless gaming mouse**.

The project aims to bring native Linux hardware support to Attack Shark X3 users without requiring the manufacturer's Windows-only configuration software, Wine, or proprietary background services.

Built with **Tauri 2, React 19, TypeScript and Rust**, OpenSharkX3 combines a modern graphical interface with low-level Linux HID communication.

The application communicates directly with compatible mouse receivers through the Linux HIDRAW subsystem, allowing configuration and device monitoring without detaching the kernel's HID driver.

Our goal is to provide Linux users with a complete, reliable and community-driven alternative to the manufacturer's software.

> Linux users deserve proper hardware support, too.

## ✨ Features

### 🖥️ Desktop Application

- Modern graphical desktop interface.
- Tauri 2 desktop architecture.
- React 19 frontend.
- Rust-powered hardware communication.
- Dark gaming-inspired interface.
- Local configuration and profile management.

### 🎯 DPI Management

- Multiple DPI stages.
- Sensitivity configuration interface.
- Active DPI stage monitoring.
- DPI stage management interface.
- Experimental hardware configuration support.

### ⚡ Performance

- 125 Hz polling rate.
- 250 Hz polling rate.
- 500 Hz polling rate.
- 1000 Hz polling rate.
- Actual polling-rate measurement.
- Sensor configuration interface.

All four polling-rate options have been physically measured on the tested 2.4 GHz receiver.

### 🎮 Button Configuration

- Mouse button mapping interface.
- Supported mouse actions.
- Keyboard shortcut assignments.
- Custom configuration profiles.

Hardware button remapping remains experimental until independently validated.

### 🔋 Device Monitoring

- Live device connection monitoring.
- Battery status events.
- DPI stage change events.
- Device acknowledgement monitoring.
- Connection diagnostics.
- HID report inspection.

Battery telemetry is based on actual device reports. Its percentage interpretation still requires further calibration against known charge states.

### 🛠️ Developer Diagnostics

- Automatic HIDRAW device discovery.
- USB VID/PID identification.
- HID report descriptor parsing.
- Input report monitoring.
- Feature report inspection.
- Polling-rate measurement.
- Device communication logs.
- Hardware diagnostic tools.

### 📁 Configuration Profiles

- Save configurations locally.
- Load existing profiles.
- Manage multiple configurations.
- Persistent local settings.

### 🔒 Safety First

- Read-only by default.
- Explicit hardware-write permission.
- Experimental feature protection.
- No automatic configuration writes on connection.
- No kernel HID driver detachment.
- No mandatory cloud account.
- Local-first configuration storage.

---

## 🧪 Hardware Validation

OpenSharkX3 is developed and tested against a real Attack Shark X3 receiver.

<details>
<summary><strong>Verified hardware information</strong></summary>

| Component | Value |
|---|---|
| Mouse | Attack Shark X3 |
| Receiver VID | `0x1D57` |
| Receiver PID | `0xFA60` |
| Product | Xenta / Beken 2.4G Wireless Device |
| Operating System | Arch Linux |
| Transport | Linux HIDRAW |
| Configuration Interface | Interface 02 |
| Kernel Driver | hid-generic / usbhid |

</details>

### Current verification status

| Feature | Status |
|---|---|
| USB device discovery | ✅ Verified |
| HID interface discovery | ✅ Verified |
| HID descriptor parsing | ✅ Verified |
| Configuration preamble | ✅ Acknowledged |
| 125 Hz polling rate | ✅ Physically verified |
| 250 Hz polling rate | ✅ Physically verified |
| 500 Hz polling rate | ✅ Physically verified |
| 1000 Hz polling rate | ✅ Physically verified |
| Battery heartbeat | ✅ Observed |
| DPI stage events | ✅ Observed |
| DPI configuration writes | 🧪 Experimental |
| Sensor configuration | 🧪 Experimental |
| Button remapping | 🧪 Experimental |
| Custom macro upload | ❌ Not implemented |
| Bluetooth configuration writes | ❌ Not implemented |

<sub>Verification status refers to the tested FA60 receiver. Compatibility with other hardware revisions and connection modes must be established separately.</sub>

### Communication protocol

The tested receiver uses a vendor-specific HID Feature Report communication channel.

The configuration interface requires a preliminary command before configuration reports are applied:

```text
0C 0A 01 FE 01 FE 00 00 00 00
```

A successful transmission is not automatically considered proof that a configuration was applied.

OpenSharkX3 distinguishes between four operation states:

| State | Meaning |
|---|---|
| `LOCAL` | Configuration exists locally |
| `TRANSMITTED` | Report was transmitted |
| `ACK` | Device acknowledged the report |
| `VERIFIED` | Physical hardware behavior was independently verified |

This distinction helps prevent misleading hardware status indicators.

For detailed technical information, see:

- [Hardware Validation](docs/HARDWARE_VALIDATION.md)
- [Protocol Documentation](docs/PROTOCOL.md)
- [Project Audit](docs/PROJECT_AUDIT.md)

---

## 🖥️ Interface

<p align="center">
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/1118f527-da7e-4e7d-808d-782cbe27d6a6" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/f1bc3b0d-7868-44b4-9b23-708e86b31c10" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/326b3fe8-50d5-470a-9dcc-7b4d4d558d4f" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/1457d1f7-ade7-46b5-adaa-208aafe5aa7b" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/7843513b-8658-4d2c-844f-255df8ce2eba" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/23428f58-d5f9-4369-af99-bc69d147f846" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/e9f75589-5c40-4c3e-bf36-b3d2aadfe69c" />
  <img width="1282" height="864" alt="OpenSharkX3 Interface" src="https://github.com/user-attachments/assets/be454008-63bd-42b8-927f-7f0ee836737b" />
</p>

<p align="center">
  <em>OpenSharkX3 graphical configuration interface.</em>
</p>

The application combines device monitoring, configuration controls, profiles and hardware diagnostics in a unified desktop experience.

---

## 🐧 Linux Compatibility

OpenSharkX3 is designed for Linux desktop environments.

| Distribution | Support Target |
|---|---|
| Arch Linux | Primary development platform |
| CachyOS | Supported target |
| Debian | Planned / testing |
| Ubuntu | Planned / testing |
| Fedora | Planned / testing |
| Linux Mint | Planned / testing |
| openSUSE | Planned / testing |

The tested development environment is Arch Linux.

Compatibility with other distributions depends on WebKitGTK, GTK, HIDRAW permissions, BlueZ availability and hardware revisions.

### Desktop environments

- GNOME
- KDE Plasma
- Other GTK/WebKit-compatible desktop environments

Wayland and X11 environments are considered in the application design.

---

## 📦 Installation

### Arch Linux / CachyOS

Arch Linux is the primary development and hardware-testing platform for OpenSharkX3.

#### 1. Install system dependencies

Update your system:

```bash
sudo pacman -Syu
```

Install the required packages:

```bash
sudo pacman -S --needed \
  base-devel \
  webkit2gtk-4.1 \
  gtk3 \
  libayatana-appindicator \
  libusb \
  bluez \
  bluez-utils \
  nodejs \
  npm \
  rust \
  cargo \
  pkgconf \
  git
```

These packages provide the Linux desktop runtime, Rust toolchain, Node.js environment and build dependencies.

Bluetooth packages are intended for Bluetooth-related functionality. The tested 2.4 GHz receiver communicates through Linux HIDRAW.

#### 2. Clone the repository

```bash
git clone https://github.com/ruaavell/OpenSharkX3.git

cd OpenSharkX3
```

#### 3. Install frontend dependencies

```bash
npm install
```

#### 4. Configure HIDRAW permissions

OpenSharkX3 communicates with the mouse receiver through `/dev/hidraw*`.

Install the supplied udev rule:

```bash
sudo install -Dm644 \
  tools/70-attack-shark-x3-hidraw.rules \
  /etc/udev/rules.d/70-attack-shark-x3-hidraw.rules
```

Reload the udev rules:

```bash
sudo udevadm control --reload-rules

sudo udevadm trigger
```

Unplug and reconnect the USB receiver.

The supplied rule is intended for the tested hardware identifiers. Check the actual rule file before assuming that another receiver revision is covered.

Verify device detection:

```bash
ls -l /dev/hidraw*

lsusb | grep -i '1d57'
```

The udev rule uses the logged-in desktop user's device access mechanism. Running OpenSharkX3 with `sudo` should not be necessary.

#### 5. Check the Rust backend

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

#### 6. Build the frontend

```bash
npm run build:renderer
```

#### 7. Build the complete application

```bash
npm run build
```

The configured Tauri bundle targets are generated under:

```text
src-tauri/target/release/bundle/
├── appimage/
└── deb/
```

The exact bundle formats depend on the Tauri build configuration.

#### 8. Run the application

For development mode:

```bash
npm run dev
```

For production, use the generated package or AppImage.

If an AppImage is generated, make it executable:

```bash
chmod +x ./OpenSharkX3_*.AppImage
```

Then run it:

```bash
./OpenSharkX3_*.AppImage
```

### Debian / Ubuntu

Debian-based distributions are planned testing targets.

The application may require additional runtime dependencies, including GTK and WebKitGTK packages. Package names and versions differ between distributions.

If a Debian package is generated by the Tauri build:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/OpenSharkX3_2.0.3_amd64.deb
```

This package should only be considered compatible after testing on the target distribution.

---

## 🔧 Hardware Write Modes

OpenSharkX3 prioritizes safe hardware communication.

The default mode is read-only.

| Mode | Description |
|---|---|
| `readonly` | No hardware configuration writes |
| `dryrun` | Build and display reports without sending them |
| `hardware` | Allow supported writes following explicit user action |

### Read-only mode

The default mode is intended for device discovery, monitoring and diagnostics.

From a development checkout:

```bash
npm run dev
```

### Dry-run mode

Build and inspect configuration reports without sending them:

```bash
OPENSHARKX3_WRITE_MODE=dryrun npm run dev
```

### Hardware mode

Hardware writes should only be enabled when the user explicitly intends to modify the device.

```bash
OPENSHARKX3_WRITE_MODE=hardware npm run dev
```

The application must still require an explicit user action for each hardware configuration write.

### Experimental features

Unverified hardware operations may additionally require:

```bash
OPENSHARKX3_EXPERIMENTAL=1
```

For example:

```bash
OPENSHARKX3_WRITE_MODE=hardware \
OPENSHARKX3_EXPERIMENTAL=1 \
npm run dev
```

Do not enable experimental operations for routine use.

Read `docs/HARDWARE_VALIDATION.md` before enabling experimental configuration.

**Important:** Never send HID report `0x0B`. The protocol research identifies it as a command that can unpair the 2.4 GHz receiver.

---

## 🏗️ Technology Stack

| Technology | Purpose |
|---|---|
| Rust | Hardware communication and application backend |
| Tauri 2 | Desktop application framework |
| React 19 | User interface |
| TypeScript | Frontend integration |
| Vite | Frontend development and bundling |
| Linux HIDRAW | Native USB HID communication |
| BlueZ / zbus | Bluetooth infrastructure |
| Cargo | Rust dependency and build management |
| GTK / WebKitGTK | Linux desktop runtime |

---

## 📂 Project Structure

```text
OpenSharkX3/
│
├── src/
│   ├── app.jsx
│   ├── api.ts
│   ├── sections.jsx
│   ├── data.jsx
│   ├── i18n.jsx
│   └── style.css
│
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs
│   │   ├── hidraw.rs
│   │   ├── driver/
│   │   ├── protocols/
│   │   ├── safety.rs
│   │   ├── state.rs
│   │   └── tray.rs
│   │
│   ├── tests/
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── assets/
│
├── tools/
│   ├── x3_lab.py
│   └── 70-attack-shark-x3-hidraw.rules
│
├── docs/
│   ├── HARDWARE_VALIDATION.md
│   ├── INSTALLATION_ARCH.md
│   ├── PROJECT_AUDIT.md
│   └── PROTOCOL.md
│
├── aur/
├── scripts/
│
├── package.json
├── Cargo.lock
├── README.md
└── LICENSE
```

<sub>This is a representative project layout. Individual files and directories may change as development progresses.</sub>

---

## 🛠️ Development and Testing

### Rust tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Rust backend check

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

### TypeScript checks

```bash
npx tsc --noEmit
```

### Frontend build

```bash
npm run build:renderer
```

### Complete application build

```bash
npm run build
```

### Development server

```bash
npm run dev
```

The project aims to maintain reliable hardware communication through unit tests, controlled USB experiments and physical validation.

---

## 🔒 Privacy and Security

OpenSharkX3 is designed around local hardware communication.

- No mandatory cloud account.
- No proprietary Windows driver dependency.
- No mandatory external configuration server.
- No automatic uploading of mouse profiles.
- No requirement to run a privileged background daemon.
- No unnecessary USB kernel driver detachment.

Configuration profiles are stored locally on the user's system.

Hardware writes are separated from read-only device monitoring to reduce the risk of unintended configuration changes.

---

## 🗺️ Roadmap

- [x] Rust hardware communication foundation
- [x] Linux HIDRAW discovery
- [x] FA60 receiver descriptor parsing
- [x] Native polling-rate configuration
- [x] Physical polling-rate verification
- [x] Battery/status heartbeat monitoring
- [x] DPI stage event monitoring
- [x] Tauri desktop integration
- [x] React graphical interface
- [x] Local profile management
- [x] Hardware-write safety system
- [ ] Physical DPI configuration verification
- [ ] Physical sensor-setting verification
- [ ] Physical button remapping verification
- [ ] Further battery calibration
- [ ] Custom macro protocol investigation
- [ ] Bluetooth configuration support
- [ ] Additional hardware revision testing
- [ ] Expanded distribution testing
- [ ] Automated release builds

---

## 🤝 Contributing

Contributions are welcome!

Whether you're a Rust developer, Linux enthusiast, hardware researcher or Attack Shark X3 owner, your contribution can help improve Linux peripheral support.

You can help by:

1. Reporting bugs and compatibility problems.
2. Testing hardware settings on compatible devices.
3. Improving the React interface.
4. Improving the Rust HID implementation.
5. Investigating undocumented HID reports.
6. Testing different Linux distributions.
7. Submitting pull requests.

### Development setup

```bash
git clone https://github.com/ruaavell/OpenSharkX3.git

cd OpenSharkX3

npm install

cargo check --manifest-path src-tauri/Cargo.toml

cargo test --manifest-path src-tauri/Cargo.toml

npm run dev
```

Please include hardware evidence when reporting new protocol findings.

---

## 🐛 Bug Reports

When opening an issue, please include:

- Linux distribution and version.
- Kernel version.
- Desktop environment.
- Connection type.
- USB VID/PID.
- OpenSharkX3 version.
- Steps to reproduce the problem.
- Relevant diagnostic logs.

Never include passwords, access tokens or private credentials in diagnostic reports.

---

## 📚 Documentation

| Document | Description |
|---|---|
| [Hardware Validation](docs/HARDWARE_VALIDATION.md) | Real hardware test results |
| [Protocol](docs/PROTOCOL.md) | HID protocol and command structure |
| [Project Audit](docs/PROJECT_AUDIT.md) | Architecture and development status |
| [Arch Installation](docs/INSTALLATION_ARCH.md) | Arch Linux installation and permissions |

---

## ⚖️ Disclaimer

OpenSharkX3 is an independent, community-driven open-source project.

It is not affiliated with, endorsed by or officially supported by ATTACK SHARK.

ATTACK SHARK and X3 are trademarks or product names belonging to their respective owners.

This project is developed for Linux compatibility, hardware interoperability and open-source software development.

---

## 📜 License

OpenSharkX3 is distributed under the **MIT License**.

See [LICENSE](LICENSE) for additional information.

---

## 🙏 Credits

OpenSharkX3 builds upon publicly available research, protocol investigations and community contributions related to Attack Shark mouse hardware.

Special thanks to the developers and contributors behind these projects:

- **[HarukaYamamoto0/attack-shark-x11-driver](https://github.com/HarukaYamamoto0/attack-shark-x11-driver)**  
  For contributing to the Linux driver research and hardware compatibility ecosystem.

- **[yourchocomate/attacksharkx3](https://github.com/yourchocomate/attacksharkx3)**  
  For additional Attack Shark X3 hardware and protocol research.

We appreciate the work of everyone contributing to open-source Linux hardware support.

OpenSharkX3 is an independent project. It is not an official fork or endorsed release of the projects listed above.

---

<p align="center">
  <strong>🦈 OpenSharkX3</strong>
  <br />
  Bringing the Attack Shark X3 experience to Linux.
  <br /><br />
  <sub>Built with Rust. Made for Linux. Powered by the community.</sub>
</p>
