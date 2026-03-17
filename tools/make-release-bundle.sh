#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
ROOT_DIR="${SCRIPT_DIR:h}"
TARGET_TRIPLE="thumbv7em-none-eabihf"
PACKAGE_NAME="finalproject_rustminios"
VERSION="$(
  sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1
)"
if [[ -z "$VERSION" ]]; then
  echo "Unable to determine package version from Cargo.toml." >&2
  exit 1
fi

BUNDLE_NAME="${PACKAGE_NAME}-v${VERSION}-stm32f407zg"
OUT_DIR="${ROOT_DIR}/dist/${BUNDLE_NAME}"
FIRMWARE_DIR="${OUT_DIR}/firmware"
ELF_SRC="${ROOT_DIR}/target/${TARGET_TRIPLE}/release/${PACKAGE_NAME}"

rm -rf "$OUT_DIR"
mkdir -p "$FIRMWARE_DIR" "$OUT_DIR/tools"

(cd "$ROOT_DIR" && cargo build --release)

cp "$ELF_SRC" "$FIRMWARE_DIR/${PACKAGE_NAME}.elf"
"$ROOT_DIR/tools/arm-objcopy.sh" -O binary "$ELF_SRC" "$FIRMWARE_DIR/${PACKAGE_NAME}.bin"
"$ROOT_DIR/tools/arm-objcopy.sh" -O ihex "$ELF_SRC" "$FIRMWARE_DIR/${PACKAGE_NAME}.hex"

cp "$ROOT_DIR/flash.sh" "$OUT_DIR/flash.sh"
cp "$ROOT_DIR/tools/flash-openocd.sh" "$OUT_DIR/tools/flash-openocd.sh"
cp "$ROOT_DIR/tools/flash-probe-rs.sh" "$OUT_DIR/tools/flash-probe-rs.sh"
cp "$ROOT_DIR/tools/flash-windows.ps1" "$OUT_DIR/tools/flash-windows.ps1"
cp "$ROOT_DIR/tools/bundle-quickstart.md" "$OUT_DIR/QUICKSTART.md"

chmod +x \
  "$OUT_DIR/flash.sh" \
  "$OUT_DIR/tools/flash-openocd.sh" \
  "$OUT_DIR/tools/flash-probe-rs.sh"

(cd "$OUT_DIR" && shasum -a 256 firmware/* > checksums.txt)
(cd "$ROOT_DIR/dist" && zip -qr "${BUNDLE_NAME}.zip" "${BUNDLE_NAME}")

echo "Bundle ready:"
echo "  $OUT_DIR"
echo "Zip archive:"
echo "  $ROOT_DIR/dist/${BUNDLE_NAME}.zip"
