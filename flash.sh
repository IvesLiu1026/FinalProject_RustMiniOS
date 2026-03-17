#!/bin/zsh
set -euo pipefail

ROOT_DIR="${0:A:h}"
DEFAULT_ELF="$ROOT_DIR/firmware/finalproject_rustminios.elf"
ELF_PATH="${1:-}"

if [[ -z "$ELF_PATH" && -f "$DEFAULT_ELF" ]]; then
  ELF_PATH="$DEFAULT_ELF"
fi

if [[ -z "$ELF_PATH" ]]; then
  echo "Usage: ./flash.sh <path-to-firmware.elf>" >&2
  echo "Tip: release bundles can run ./flash.sh directly from the bundle root." >&2
  exit 1
fi

FLASH_TOOL="${MINIOS_FLASH_TOOL:-auto}"

case "$FLASH_TOOL" in
  auto)
    if command -v probe-rs >/dev/null 2>&1; then
      exec "$ROOT_DIR/tools/flash-probe-rs.sh" "$ELF_PATH"
    fi
    exec "$ROOT_DIR/tools/flash-openocd.sh" "$ELF_PATH"
    ;;
  probe-rs | probe)
    exec "$ROOT_DIR/tools/flash-probe-rs.sh" "$ELF_PATH"
    ;;
  openocd)
    exec "$ROOT_DIR/tools/flash-openocd.sh" "$ELF_PATH"
    ;;
  *)
    echo "Unknown MINIOS_FLASH_TOOL='$FLASH_TOOL' (expected auto, probe-rs, or openocd)." >&2
    exit 1
    ;;
esac
