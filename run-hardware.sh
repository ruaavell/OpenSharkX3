#!/usr/bin/env bash
# Launch OpenSharkX3 with hardware writes enabled.
# DPI/sensor/button writes are experimental; also set OPENSHARKX3_EXPERIMENTAL=1.
export OPENSHARKX3_WRITE_MODE=hardware
export OPENSHARKX3_EXPERIMENTAL=1
exec "${HOME}/.local/bin/opensharkx3" "$@"