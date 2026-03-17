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
  echo "Usage: tools/flash-probe-rs.sh <path-to-firmware.elf>" >&2
  exit 1
fi

PROBE_RS="${MINIOS_PROBE_RS_BIN:-}"
if [[ -z "$PROBE_RS" ]]; then
  PROBE_RS="$(command -v probe-rs 2>/dev/null || true)"
fi

if [[ -z "$PROBE_RS" || ! -x "$PROBE_RS" ]]; then
  echo "probe-rs not found." >&2
  echo "Install probe-rs from https://probe.rs/docs/getting-started/installation" >&2
  exit 1
fi

CHIP="${MINIOS_PROBE_RS_CHIP:-STM32F407ZGTx}"

exec "$PROBE_RS" run --chip "$CHIP" "$ELF_PATH"
