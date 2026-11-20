# 🦈 OpenSharkX3

### Open-source Attack Shark X3 configuration software for Linux

<p align="center">
  <strong>A native, Rust-powered Linux companion for the Attack Shark X3 gaming mouse.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" />
  <img src="https://img.shields.io/badge/Open%20Source-Yes-success?style=for-the-badge&logo=github" />
</p>

---

## 📖 About

**OpenSharkX3** is an independent, open-source Linux driver and graphical configuration application for the **Attack Shark X3 wireless gaming mouse**.

The goal is simple: bring proper mouse configuration support to Linux users without depending on the manufacturer's Windows-only software, Wine, or proprietary background services.

Built entirely with **Rust**, OpenSharkX3 provides a native desktop experience, combining low-level hardware communication with an intuitive graphical interface.

Whether you're using Arch Linux, Fedora, Debian, Ubuntu, or another Linux distribution, OpenSharkX3 aims to make configuring your Attack Shark X3 straightforward.

> Linux users deserve proper hardware support, too.

## ✨ Features

- 🦀 **Written in Rust:** A modern, efficient and memory-safe implementation.
- 🖥️ **Graphical Interface:** Configure your mouse through a dedicated desktop application.
- 🐧 **Linux Native:** Designed specifically for Linux environments.
- 🔌 **Direct Hardware Communication:** Communicates with the mouse through its HID interface.
- 🎯 **DPI Configuration:** Manage mouse sensitivity and DPI stages.
- ⚡ **Polling Rate:** Configure supported polling-rate options.
- 🎮 **Button Configuration:** Customize supported mouse button assignments.
- 🔋 **Device Information:** Access supported device status information.
- 🛠️ **Open Source:** Transparent development, community contributions and extensibility.
- 🚫 **No Wine Required:** No need to run the manufacturer's Windows application.

*Feature availability depends on the implemented hardware protocol and supported device revision.*

## 🖥️ Interface

<p align="center">
  <img src="docs/images/interface.png" alt="OpenSharkX3 graphical interface" width="850" />
</p>

<p align="center">
  <i>OpenSharkX3 graphical configuration interface.</i>
</p>

## 🐧 Linux Compatibility

OpenSharkX3 is developed for the Linux desktop ecosystem.

| Distribution | Platform |
|---|---|
| Arch Linux | Linux |
| Debian | Linux |
| Ubuntu | Linux |
| Fedora | Linux |
| Linux Mint | Linux |
| openSUSE | Linux |

Compatibility may vary depending on HID permissions, system libraries, desktop environment and hardware revision.

## 📦 Installation

### 1. Download the source code

```bash
git clone https://github.com/YOUR_USERNAME/OpenSharkX3.git
cd OpenSharkX3
```

### 2. Install Rust

If Rust is not already installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Reload your shell:

```bash
source "$HOME/.cargo/env"
```

### 3. Build the application

```bash
cargo build --release
```

### 4. Run OpenSharkX3

```bash
cargo run --release
```

Alternatively, run the compiled executable:

```bash
./target/release/opensharkx3
```

*Note: The executable name and additional system dependencies should match the actual project configuration.*

## 🔧 Hardware

| Component | Description |
|---|---|
| Device | Attack Shark X3 |
| Device Type | Wireless Gaming Mouse |
| Communication | USB HID |
| Connection | USB / 2.4 GHz receiver |
| Software | OpenSharkX3 |
| Implementation | Rust |

OpenSharkX3 is an independent implementation designed to communicate with compatible Attack Shark X3 hardware.

## 🏗️ Technology

The project is built around the Rust ecosystem.

| Technology | Purpose |
|---|---|
| Rust | Core application and hardware communication |
| Cargo | Dependency and build management |
| HID | Mouse communication |
| Linux | Target operating system |
| GUI Framework | Native graphical interface |

## 📂 Project Structure

```text
OpenSharkX3/
│
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── device/
│   ├── hid/
│   ├── config/
│   └── gui/
│
├── assets/
│   └── icons/
│
├── docs/
│   └── images/
│
├── tests/
│
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

*The structure above illustrates the intended organization. Adapt it to the actual repository layout.*

## 🔒 Open Source & Privacy

OpenSharkX3 is designed around local hardware configuration.

- No proprietary Windows driver dependency.
- No mandatory cloud account.
- No requirement to send mouse configuration data to an external server.
- Source code is publicly available for inspection and contributions.

The application communicates with the connected hardware locally.

## 🤝 Contributing

Contributions are welcome!

Whether you're a Rust developer, Linux enthusiast, hardware researcher, or Attack Shark X3 owner, your contributions can help improve Linux hardware compatibility.

You can contribute by:

1. Reporting bugs and compatibility issues.
2. Suggesting new features.
3. Improving the graphical interface.
4. Testing on different Linux distributions.
5. Improving HID communication and device support.
6. Submitting pull requests.

### Getting Started

```bash
git clone https://github.com/YOUR_USERNAME/OpenSharkX3.git

cd OpenSharkX3

cargo build

cargo test
```

Before submitting a pull request, please make sure your changes compile and existing tests pass.

## 🐛 Bug Reports

Found a problem?

Please open an issue on GitHub and include:

- Linux distribution and version.
- Kernel version.
- Attack Shark X3 hardware revision, if known.
- Steps to reproduce the issue.
- Relevant application logs.

## 🗺️ Roadmap

- [x] Rust-based application foundation
- [x] Graphical user interface
- [x] Linux-oriented development
- [ ] Expanded hardware configuration
- [ ] Improved device detection
- [ ] Additional Linux distribution testing
- [ ] Application packaging
- [ ] Automated release builds
- [ ] Expanded hardware compatibility

*Roadmap items should be updated as development progresses.*

## ⚖️ Disclaimer

OpenSharkX3 is an independent, community-driven project.

It is not affiliated with, endorsed by, or officially supported by ATTACK SHARK.

ATTACK SHARK and X3 are trademarks or product names belonging to their respective owners.

This project is developed for Linux compatibility, hardware interoperability and open-source software development.

## 📜 License

OpenSharkX3 is distributed under the **MIT License**.

See [LICENSE](LICENSE) for more information.

---

<p align="center">
  <strong>🦈 OpenSharkX3</strong>
  <br />
  Bringing the Attack Shark X3 experience to Linux.
  <br /><br />
  <sub>Built with Rust. Made for Linux. Powered by the community.</sub>
</p>
