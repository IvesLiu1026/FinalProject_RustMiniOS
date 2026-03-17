#!/bin/zsh
set -euo pipefail

if [[ -n "${MINIOS_ARM_SIZE:-}" ]]; then
  SIZE="$MINIOS_ARM_SIZE"
elif command -v arm-none-eabi-size >/dev/null 2>&1; then
  SIZE="$(command -v arm-none-eabi-size)"
else
  PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
  SIZE="$PACKAGES_DIR/toolchain-gccarmnoneeabi/bin/arm-none-eabi-size"
fi

if [[ ! -x "$SIZE" ]]; then
  echo "arm-none-eabi-size not found." >&2
  echo "Set MINIOS_ARM_SIZE, install it on PATH, or set PLATFORMIO_PACKAGES_DIR." >&2
  exit 1
fi

exec "$SIZE" "$@"
