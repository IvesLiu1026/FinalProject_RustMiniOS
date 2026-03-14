#!/bin/zsh
set -euo pipefail

PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
GCC="$PACKAGES_DIR/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"

if [[ ! -x "$GCC" ]]; then
  echo "arm-none-eabi-gcc not found at: $GCC" >&2
  echo "Set PLATFORMIO_PACKAGES_DIR if your PlatformIO packages live elsewhere." >&2
  exit 1
fi

exec "$GCC" "$@"
