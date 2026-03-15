#!/bin/zsh
set -euo pipefail
unsetopt xtrace verbose 2>/dev/null || true

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MEDIA_ROOT="$ROOT_DIR/assets/test_media"
IMAGES_IN="$MEDIA_ROOT/images"
GIFS_IN="$MEDIA_ROOT/gifs"
OUT_ROOT="$MEDIA_ROOT/converted"
IMAGE_OUT="$OUT_ROOT/previews"
GIF_FRAMES_OUT="$OUT_ROOT/gif_frames"
GIF_PREVIEW_OUT="$OUT_ROOT/gif_previews"
MANIFEST_OUT="$OUT_ROOT/manifests"
FIRMWARE_ROOT="$OUT_ROOT/firmware"
FIRMWARE_STILLS_OUT="$FIRMWARE_ROOT/stills"
FIRMWARE_MOTION_OUT="$FIRMWARE_ROOT/motion"

CANVAS_W="${CANVAS_W:-320}"
CANVAS_H="${CANVAS_H:-240}"
FOREGROUND_W="${FOREGROUND_W:-296}"
FOREGROUND_H="${FOREGROUND_H:-216}"
GIF_FRAME_STEP="${GIF_FRAME_STEP:-2}"
GIF_DELAY_CS="${GIF_DELAY_CS:-10}"
MOTION_FRAME_MAX="${MOTION_FRAME_MAX:-12}"
BACKGROUND_TINT="${BACKGROUND_TINT:-#08101e}"
BACKGROUND_COLORIZE="${BACKGROUND_COLORIZE:-34}"
FRAME_BORDER="${FRAME_BORDER:-2}"
FRAME_BORDER_COLOR="${FRAME_BORDER_COLOR:-#dfe8ff}"
STILL_FW_W="${STILL_FW_W:-120}"
STILL_FW_H="${STILL_FW_H:-90}"
MOTION_FW_W="${MOTION_FW_W:-80}"
MOTION_FW_H="${MOTION_FW_H:-60}"

usage() {
  cat <<EOF
Usage: $(basename "$0")

Preprocesses media under:
  $IMAGES_IN
  $GIFS_IN

Generated outputs:
  $IMAGE_OUT
  $GIF_FRAMES_OUT
  $GIF_PREVIEW_OUT
  $MANIFEST_OUT
  $FIRMWARE_ROOT

Environment overrides:
  CANVAS_W, CANVAS_H
  FOREGROUND_W, FOREGROUND_H
  GIF_FRAME_STEP, GIF_DELAY_CS, MOTION_FRAME_MAX
  STILL_FW_W, STILL_FW_H
  MOTION_FW_W, MOTION_FW_H
  BACKGROUND_TINT, BACKGROUND_COLORIZE
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v magick >/dev/null 2>&1; then
  echo "ImageMagick 'magick' is required but was not found in PATH." >&2
  exit 1
fi

mkdir -p \
  "$IMAGE_OUT" \
  "$GIF_FRAMES_OUT" \
  "$GIF_PREVIEW_OUT" \
  "$MANIFEST_OUT" \
  "$FIRMWARE_STILLS_OUT" \
  "$FIRMWARE_MOTION_OUT"

compose_frame() {
  local input="$1"
  local output="$2"

  magick \
    \( "$input" \
      -auto-orient \
      -colorspace sRGB \
      -strip \
      -resize "${CANVAS_W}x${CANVAS_H}^" \
      -gravity center \
      -extent "${CANVAS_W}x${CANVAS_H}" \
      -blur 0x18 \
      -fill "$BACKGROUND_TINT" \
      -colorize "$BACKGROUND_COLORIZE" \
    \) \
    \( "$input" \
      -auto-orient \
      -colorspace sRGB \
      -strip \
      -resize "${FOREGROUND_W}x${FOREGROUND_H}" \
      -unsharp 0x0.8+0.8+0.02 \
      -bordercolor "$FRAME_BORDER_COLOR" \
      -border "$FRAME_BORDER" \
    \) \
    -gravity center \
    -compose over \
    -composite \
    "$output"
}

to_rgb565le() {
  local input="$1"
  local width="$2"
  local height="$3"
  local output="$4"

  magick "$input" \
    -colorspace sRGB \
    -strip \
    -filter Lanczos \
    -resize "${width}x${height}!" \
    -depth 8 \
    rgb:- \
    | perl -e '
        use strict;
        use warnings;
        binmode STDIN;
        binmode STDOUT;
        my $buf;
        while (read(STDIN, $buf, 3) == 3) {
          my ($r, $g, $b) = unpack("C3", $buf);
          my $value = (($r & 0xF8) << 8) | (($g & 0xFC) << 3) | ($b >> 3);
          print pack("v", $value);
        }
      ' > "$output"
}

write_still_manifest() {
  local stem="$1"
  local source_name="$2"
  local preview_name="$3"
  local firmware_name="$4"

  cat >"$MANIFEST_OUT/${stem}.txt" <<EOF
type=still
source=${source_name}
canvas=${CANVAS_W}x${CANVAS_H}
foreground_max=${FOREGROUND_W}x${FOREGROUND_H}
strategy=contain_with_blurred_backdrop
preview=${preview_name}
firmware_format=rgb565le
firmware_size=${STILL_FW_W}x${STILL_FW_H}
firmware_scale=2
firmware_file=${firmware_name}
EOF
}

write_gif_manifest() {
  local stem="$1"
  local source_name="$2"
  local preview_name="$3"
  local frame_dir_name="$4"
  local frames_kept="$5"

  cat >"$MANIFEST_OUT/${stem}.txt" <<EOF
type=gif
source=${source_name}
canvas=${CANVAS_W}x${CANVAS_H}
foreground_max=${FOREGROUND_W}x${FOREGROUND_H}
strategy=contain_with_blurred_backdrop
frame_step=${GIF_FRAME_STEP}
frame_delay_cs=${GIF_DELAY_CS}
preview=${preview_name}
frames_kept=${frames_kept}
firmware_format=rgb565le
firmware_size=${MOTION_FW_W}x${MOTION_FW_H}
firmware_scale=3
firmware_frame_dir=${frame_dir_name}
EOF
}

process_still() {
  local input="$1"
  local stem
  stem="$(basename "$input")"
  stem="${stem%.*}"

  local preview="$IMAGE_OUT/${stem}_320x240.png"
  local firmware="$FIRMWARE_STILLS_OUT/${stem}.rgb565"

  compose_frame "$input" "$preview"
  to_rgb565le "$preview" "$STILL_FW_W" "$STILL_FW_H" "$firmware"
  write_still_manifest "$stem" "$(basename "$input")" "$(basename "$preview")" "$(basename "$firmware")"
}

process_gif() {
  local input="$1"
  local stem
  stem="$(basename "$input")"
  stem="${stem%.*}"

  local tmpdir
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/rustminios_media.XXXXXX")"
  local raw_dir="$tmpdir/raw"
  local preview_dir="$GIF_FRAMES_OUT/$stem"
  local firmware_dir="$FIRMWARE_MOTION_OUT/$stem"

  mkdir -p "$raw_dir" "$preview_dir" "$firmware_dir"
  find "$preview_dir" -maxdepth 1 -type f -name 'frame_*.png' -delete
  find "$firmware_dir" -maxdepth 1 -type f -name 'frame_*.rgb565' -delete

  magick "$input" -coalesce "$raw_dir/frame_%04d.png"

  local -a selected_frames=()
  local index=0
  local frame
  for frame in "$raw_dir"/frame_*.png; do
    [[ -e "$frame" ]] || continue
    if (( index % GIF_FRAME_STEP == 0 )); then
      selected_frames+=("$frame")
    fi
    (( index += 1 ))
  done

  local selected_count="${#selected_frames[@]}"
  if (( selected_count == 0 )); then
    write_gif_manifest "$stem" "$(basename "$input")" "$(basename "$GIF_PREVIEW_OUT/${stem}_320x240.gif")" "$stem" 0
    find "$tmpdir" -mindepth 1 -delete
    rmdir "$tmpdir"
    return
  fi

  local stride=1
  if (( selected_count > MOTION_FRAME_MAX )); then
    stride=$(( (selected_count + MOTION_FRAME_MAX - 1) / MOTION_FRAME_MAX ))
  fi

  local kept=0
  local selected_index=0
  for frame in "${selected_frames[@]}"; do
    if (( selected_index % stride != 0 )); then
      (( selected_index += 1 ))
      continue
    fi
    if (( kept >= MOTION_FRAME_MAX )); then
      break
    fi

    local frame_id
    frame_id=$(printf "%04d" "$kept")
    local preview_frame="$preview_dir/frame_${frame_id}.png"
    local firmware_frame="$firmware_dir/frame_${frame_id}.rgb565"

    compose_frame "$frame" "$preview_frame"
    to_rgb565le "$preview_frame" "$MOTION_FW_W" "$MOTION_FW_H" "$firmware_frame"

    (( kept += 1 ))
    (( selected_index += 1 ))
  done

  if (( kept > 0 )); then
    magick -delay "$GIF_DELAY_CS" -loop 0 \
      "$preview_dir"/frame_*.png \
      "$GIF_PREVIEW_OUT/${stem}_320x240.gif"
  fi

  write_gif_manifest \
    "$stem" \
    "$(basename "$input")" \
    "$(basename "$GIF_PREVIEW_OUT/${stem}_320x240.gif")" \
    "$stem" \
    "$kept"

  find "$tmpdir" -mindepth 1 -delete
  rmdir "$tmpdir"
}

processed_any=0

process_local_image() {
  local input="$1"
  echo "Processing still: $(basename "$input")"
  process_still "$input"
  processed_any=1
}

process_local_gif() {
  local input="$1"
  echo "Processing GIF: $(basename "$input")"
  process_gif "$input"
  processed_any=1
}

for input in "$IMAGES_IN"/*; do
  [[ -f "$input" ]] || continue
  case "${input:l}" in
    *.png|*.jpg|*.jpeg|*.bmp)
      process_local_image "$input"
      ;;
  esac
done

for input in "$GIFS_IN"/*; do
  [[ -f "$input" ]] || continue
  case "${input:l}" in
    *.gif)
      process_local_gif "$input"
      ;;
  esac
done

if (( processed_any == 0 )); then
  echo "No supported files found in $MEDIA_ROOT"
  exit 1
fi

echo
echo "Finished."
echo "Still previews: $IMAGE_OUT"
echo "GIF frames: $GIF_FRAMES_OUT"
echo "GIF previews: $GIF_PREVIEW_OUT"
echo "Firmware stills: $FIRMWARE_STILLS_OUT"
echo "Firmware motion: $FIRMWARE_MOTION_OUT"
echo "Manifests: $MANIFEST_OUT"
