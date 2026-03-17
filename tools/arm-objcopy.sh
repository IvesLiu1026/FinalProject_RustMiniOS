#!/bin/zsh
set -euo pipefail

if [[ -n "${MINIOS_ARM_OBJCOPY:-}" ]]; then
  OBJCOPY="$MINIOS_ARM_OBJCOPY"
elif command -v arm-none-eabi-objcopy >/dev/null 2>&1; then
  OBJCOPY="$(command -v arm-none-eabi-objcopy)"
else
  PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
  OBJCOPY="$PACKAGES_DIR/toolchain-gccarmnoneeabi/bin/arm-none-eabi-objcopy"
fi

if [[ ! -x "$OBJCOPY" ]]; then
  echo "arm-none-eabi-objcopy not found." >&2
  echo "Set MINIOS_ARM_OBJCOPY, install it on PATH, or set PLATFORMIO_PACKAGES_DIR." >&2
  exit 1
fi

exec "$OBJCOPY" "$@"
