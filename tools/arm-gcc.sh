#!/bin/zsh
set -euo pipefail

if [[ -n "${MINIOS_ARM_GCC:-}" ]]; then
  GCC="$MINIOS_ARM_GCC"
elif command -v arm-none-eabi-gcc >/dev/null 2>&1; then
  GCC="$(command -v arm-none-eabi-gcc)"
else
  PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
  GCC="$PACKAGES_DIR/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"
fi

if [[ ! -x "$GCC" ]]; then
  echo "arm-none-eabi-gcc not found." >&2
  echo "Set MINIOS_ARM_GCC, install it on PATH, or set PLATFORMIO_PACKAGES_DIR." >&2
  exit 1
fi

exec "$GCC" "$@"
