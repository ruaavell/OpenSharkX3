
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
