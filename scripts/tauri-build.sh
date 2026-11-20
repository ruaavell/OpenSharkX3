#!/usr/bin/env bash
# Build the Tauri app (AppImage + deb) with a workaround for systems whose
# gdk-pixbuf package ships no external loaders directory.
#
# linuxdeploy's GTK plugin runs roughly:
#   cp "$(realpath --relative-to=/usr/lib "$(pkg-config --variable=gdk_pixbuf_binarydir gdk-pixbuf-2.0)")"
# When that directory does not exist (as on an Arch install with gdk-pixbuf
# 2.44 where the loaders are built into the library), realpath fails and the
# plugin aborts with "cp: '' ...". The app itself is unaffected; only the
# AppImage bundling step fails.
#
# If the real directory is missing, this script puts a local pkg-config file
# first on PKG_CONFIG_PATH that points linuxdeploy at an existing, tiny
# directory. The generated loaders.cache is empty, which is correct for a
# gdk-pixbuf with built-in loaders. On systems with a normal loaders directory
# the shim is not used at all.
#
# Usage:
#   npm run build
#   npm run build -- --bundles deb
set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")/.."

export NO_STRIP=1
export APPIMAGE_EXTRACT_AND_RUN=1

moduledir="$(pkg-config --variable=gdk_pixbuf_moduledir gdk-pixbuf-2.0 2>/dev/null || true)"
if [ -z "$moduledir" ] || [ ! -d "$moduledir" ]; then
  shim="$(pwd)/src-tauri/target/gdk-pixbuf-shim"
  mkdir -p "$shim"
  # Keep the real package metadata (version, Requires, Libs, Cflags) so cargo's
  # gdk-pixbuf-sys probe still succeeds; only redirect the two directories that
  # linuxdeploy tries to copy.
  real_pc="$(pkg-config --variable=pcfiledir gdk-pixbuf-2.0)/gdk-pixbuf-2.0.pc"
  cp "$real_pc" "$shim/gdk-pixbuf-2.0.pc"
  sed -i \
    -e 's|^gdk_pixbuf_binarydir=.*|gdk_pixbuf_binarydir=/usr/lib/pkgconfig|' \
    -e 's|^gdk_pixbuf_moduledir=.*|gdk_pixbuf_moduledir=${gdk_pixbuf_binarydir}|' \
    -e 's|^gdk_pixbuf_cache_file=.*|gdk_pixbuf_cache_file=${gdk_pixbuf_binarydir}/loaders.cache|' \
    "$shim/gdk-pixbuf-2.0.pc"
  echo "warning: gdk-pixbuf loaders dir '${moduledir:-<unset>}' is missing;" >&2
  echo "warning: using the local pkg-config shim for linuxdeploy" >&2
  export PKG_CONFIG_PATH="$shim${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi

exec npx tauri build "$@"
