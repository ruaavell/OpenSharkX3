#!/usr/bin/env bash
# OpenSharkX3 installer
# Installs the built AppImage (extracted, no FUSE required) onto the system.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPIMAGE="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage/OpenSharkX3_2.0.4_amd64.AppImage"
DEB="$SCRIPT_DIR/src-tauri/target/release/bundle/deb/OpenSharkX3_2.0.4_amd64.deb"

MODE="${1:-user}"   # user | system | deb

install_extracted() {
  local appdir="$1"
  local bindir="$2"
  rm -rf "$appdir"
  mkdir -p "$appdir" "$bindir"
  (cd "$appdir" && "$APPIMAGE" --appimage-extract >/dev/null 2>&1)
  mv "$appdir/squashfs-root" "$appdir/appimage"
}

case "$MODE" in
  user)
    BIN_DIR="${HOME}/.local/bin"
    APP_DIR="${HOME}/.local/share/opensharkx3"
    APP_DIR_IMG="${APP_DIR}/appimage"
    APP_DIR_DATA="${APP_DIR}"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
    mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"
    install_extracted "$APP_DIR" "$BIN_DIR"
    cat > "$BIN_DIR/opensharkx3" <<EOF
#!/usr/bin/env bash
exec "${APP_DIR_IMG}/AppRun" "\$@"
EOF
    chmod +x "$BIN_DIR/opensharkx3"
    # Hardware-mode launcher so end users can apply settings without env vars.
    cat > "$BIN_DIR/opensharkx3-hardware" <<'EOF'
#!/usr/bin/env bash
export OPENSHARKX3_WRITE_MODE=hardware
export OPENSHARKX3_EXPERIMENTAL=1
exec "${HOME}/.local/bin/opensharkx3" "$@"
EOF
    chmod +x "$BIN_DIR/opensharkx3-hardware"
    # icon (if present)
    if [ -f "$SCRIPT_DIR/src-tauri/icons/512x512.png" ]; then
      cp "$SCRIPT_DIR/src-tauri/icons/512x512.png" "$ICON_DIR/opensharkx3.png"
    fi
    cat > "${HOME}/.local/share/applications/opensharkx3.desktop" <<EOF
[Desktop Entry]
Name=OpenSharkX3
Comment=Mouse configuration utility for X3
Exec=$BIN_DIR/opensharkx3-hardware
Icon=opensharkx3
Type=Application
Categories=Utility;HardwareSettings;
Terminal=false
EOF
    echo "Installed to $BIN_DIR/opensharkx3"
    echo "Make sure $BIN_DIR is in your PATH."
    ;;
  system)
    if [ ! -w /usr/local/bin ]; then
      echo "Need root. Re-run with: sudo $0 system" >&2
      exit 1
    fi
    APP_DIR="/usr/local/share/opensharkx3"
    install_extracted "$APP_DIR" "/usr/local/bin"
    cat > /usr/local/bin/opensharkx3 <<EOF
#!/usr/bin/env bash
exec "${APP_DIR}/appimage/AppRun" "\$@"
EOF
    chmod +x /usr/local/bin/opensharkx3
    cat > /usr/local/bin/opensharkx3-hardware <<EOF
#!/usr/bin/env bash
export OPENSHARKX3_WRITE_MODE=hardware
export OPENSHARKX3_EXPERIMENTAL=1
exec /usr/local/bin/opensharkx3 "\$@"
EOF
    chmod +x /usr/local/bin/opensharkx3-hardware
    if [ -f "$SCRIPT_DIR/src-tauri/icons/512x512.png" ]; then
      cp "$SCRIPT_DIR/src-tauri/icons/512x512.png" /usr/share/icons/hicolor/512x512/apps/opensharkx3.png 2>/dev/null || true
    fi
    cat > /usr/share/applications/opensharkx3.desktop <<EOF
[Desktop Entry]
Name=OpenSharkX3
Comment=Mouse configuration utility for X3
Exec=/usr/local/bin/opensharkx3-hardware
Icon=opensharkx3
Type=Application
Categories=Utility;HardwareSettings;
Terminal=false
EOF
    echo "Installed system-wide to /usr/local/bin/opensharkx3"
    ;;
  deb)
    if [ ! -f "$DEB" ]; then
      echo "deb not found: $DEB" >&2
      exit 1
    fi
    sudo dpkg -i "$DEB" || sudo apt-get install -f -y
    ;;
  *)
    echo "Usage: $0 [user|system|deb]" >&2
    exit 1
    ;;
esac
