#!/bin/zsh
set -euo pipefail

ELF_PATH="${1:?missing firmware path}"
PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
OPENOCD="$PACKAGES_DIR/tool-openocd/bin/openocd"
SCRIPT_DIR="$PACKAGES_DIR/tool-openocd/openocd/scripts"

if [[ ! -x "$OPENOCD" ]]; then
  echo "OpenOCD not found at: $OPENOCD" >&2
  echo "Set PLATFORMIO_PACKAGES_DIR if your PlatformIO packages live elsewhere." >&2
  exit 1
fi

"$OPENOCD" \
  -s "$SCRIPT_DIR" \
  -f interface/stlink.cfg \
  -f target/stm32f4x.cfg \
  -c "program $ELF_PATH verify reset exit"
