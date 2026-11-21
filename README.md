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
- Sensitivity configuration.
- Active DPI stage monitoring.
- Stage management interface.
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
 <img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-39-16" src="https://github.com/user-attachments/assets/1118f527-da7e-4e7d-808d-782cbe27d6a6" />
  <img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-42-12" src="https://github.com/user-attachments/assets/f1bc3b0d-7868-44b4-9b23-708e86b31c10" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-42-05" src="https://github.com/user-attachments/assets/326b3fe8-50d5-470a-9dcc-7b4d4d558d4f" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-41-59" src="https://github.com/user-attachments/assets/1457d1f7-ade7-46b5-adaa-208aafe5aa7b" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-41-55" src="https://github.com/user-attachments/assets/7843513b-8658-4d2c-844f-255df8ce2eba" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-41-50" src="https://github.com/user-attachments/assets/23428f58-d5f9-4369-af99-bc69d147f846" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-41-45" src="https://github.com/user-attachments/assets/e9f75589-5c40-4c3e-bf36-b3d2aadfe69c" />
<img width="1282" height="864" alt="Ekran Görüntüsü 2026-11-21 03-41-37" src="https://github.com/user-attachments/assets/be454008-63bd-42b8-927f-7f0ee836737b" />

</p>

<p align="center">
  <em>OpenSharkX3 graphical configuration interface.</em>
</p>

The application combines device monitoring, configuration controls, profiles and hardware diagnostics in a unified desktop experience.
## 📦 Installation

OpenSharkX3 is primarily developed and hardware-tested on Arch Linux. The instructions below cover source builds and installation of generated packages.

### Arch Linux / CachyOS

Install the required dependencies:

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

Bluetooth packages are only needed for Bluetooth-related functionality. USB and 2.4 GHz receiver communication use Linux HIDRAW.

### Clone the repository

```bash
git clone https://github.com/ruaavell/OpenSharkX3.git
cd OpenSharkX3
```

### Install frontend dependencies

```bash
npm install
```

### Configure HIDRAW permissions

Install the supplied udev rules:

```bash
sudo install -Dm644 \
  tools/70-attack-shark-x3-hidraw.rules \
  /etc/udev/rules.d/70-attack-shark-x3-hidraw.rules

sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug and reconnect the receiver.

The rules cover the tested receiver (`1d57:fa60`) and USB cable device (`1d57:fa55`). Access is granted to the logged-in desktop user through `uaccess`. Running the application with `sudo` should not be necessary.

Verify device detection:

```bash
ls -l /dev/hidraw*
lsusb | grep -i '1d57'
```

### Build from source

Check the Rust backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Build the frontend:

```bash
npm run build:renderer
```

Build the complete desktop application:

```bash
npm run build
```

Build artifacts are generated under:

```text
src-tauri/target/release/bundle/
├── appimage/
└── deb/
```

### Install the AppImage

After a successful build:

```bash
./install.sh user
```

This installs the application into `~/.local/bin` and creates a desktop launcher.

The installed launcher must start in read-only mode.

For a system-wide installation:

```bash
sudo ./install.sh system
```

### Debian / Ubuntu

Build the Debian package and install it:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/OpenSharkX3_2.0.1_amd64.deb
```

The `.deb` package is intended for Debian-based distributions. Arch Linux and CachyOS users should use the AppImage or native packaging.

### Development mode

Run the application from the source tree:

```bash
npm run dev
```

The optional `start.sh` script is an Arch Linux development bootstrap. It checks dependencies, configures udev when needed, runs read-only diagnostics and starts the development application. It is not a release installer.

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

---

## 🛠️ Development and Testing

Run the Rust tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Check the Rust backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Run TypeScript checks:

```bash
npx tsc --noEmit
```

Build the frontend:

```bash
npm run build:renderer
```

Build the complete application:

```bash
npm run build
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
git clone https://github.com/YOUR_USERNAME/OpenSharkX3.git

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

This project builds upon and acknowledges the original OpenSharkX11 project and other open-source hardware research referenced in its documentation.

---

<p align="center">
  <strong>🦈 OpenSharkX3</strong>
  <br />
  Bringing the Attack Shark X3 experience to Linux.
  <br /><br />
  <sub>Built with Rust. Made for Linux. Powered by the community. thanks for https://github.com/iago-fragnan/attack-shark-x11-linux - </sub>
</p>
