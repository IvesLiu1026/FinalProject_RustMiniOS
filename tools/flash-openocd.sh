#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
ROOT_DIR="${SCRIPT_DIR:h}"
DEFAULT_ELF="$ROOT_DIR/firmware/finalproject_rustminios.elf"
ELF_PATH="${1:-}"

if [[ -z "$ELF_PATH" && -f "$DEFAULT_ELF" ]]; then
  ELF_PATH="$DEFAULT_ELF"
fi

if [[ -z "$ELF_PATH" ]]; then
  echo "Usage: tools/flash-openocd.sh <path-to-firmware.elf>" >&2
  exit 1
fi

OPENOCD=""
SCRIPT_ROOT=""
if [[ -n "${MINIOS_OPENOCD:-}" ]]; then
  OPENOCD="$MINIOS_OPENOCD"
  SCRIPT_ROOT="${OPENOCD_SCRIPTS:-}"
elif command -v openocd >/dev/null 2>&1; then
  OPENOCD="$(command -v openocd)"
  SCRIPT_ROOT="${OPENOCD_SCRIPTS:-}"
else
  PACKAGES_DIR="${PLATFORMIO_PACKAGES_DIR:-$HOME/.platformio/packages}"
  OPENOCD="$PACKAGES_DIR/tool-openocd/bin/openocd"
  SCRIPT_ROOT="$PACKAGES_DIR/tool-openocd/openocd/scripts"
fi

if [[ ! -x "$OPENOCD" ]]; then
  echo "OpenOCD not found." >&2
  echo "Set MINIOS_OPENOCD, install openocd on PATH, or set PLATFORMIO_PACKAGES_DIR." >&2
  exit 1
fi

OPENOCD_ARGS=()
if [[ -n "$SCRIPT_ROOT" && -d "$SCRIPT_ROOT" ]]; then
  OPENOCD_ARGS+=(-s "$SCRIPT_ROOT")
fi

exec "$OPENOCD" \
  "${OPENOCD_ARGS[@]}" \
  -f interface/stlink.cfg \
  -f target/stm32f4x.cfg \
  -c "program $ELF_PATH verify reset exit"
