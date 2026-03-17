#!/bin/zsh
set -euo pipefail

if [[ -n "${MINIOS_ARM_NM:-}" ]]; then
  NM="$MINIOS_ARM_NM"
elif command -v arm-none-eabi-nm >/dev/null 2>&1; then
  NM="$(command -v arm-none-eabi-nm)"
else
  PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
  NM="$PACKAGES_DIR/toolchain-gccarmnoneeabi/bin/arm-none-eabi-nm"
fi

if [[ ! -x "$NM" ]]; then
  echo "arm-none-eabi-nm not found." >&2
  echo "Set MINIOS_ARM_NM, install it on PATH, or set PLATFORMIO_PACKAGES_DIR." >&2
  exit 1
fi

exec "$NM" "$@"
