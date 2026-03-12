#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPORT_BIN="$1"
FRAMES_DIR="$2"
OUTPUT_MP4="$3"
OUTPUT_WEBP="$4"
WEBP_TEMP_DIR="$5"
FPS="${FPS:-60}"
FINAL_WIDTH="${FINAL_WIDTH:-1920}"
FINAL_HEIGHT="${FINAL_HEIGHT:-1080}"
WEBP_FPS="${WEBP_FPS:-30}"
WEBP_SCALE_WIDTH="${WEBP_SCALE_WIDTH:-320}"
WEBP_QUALITY="${WEBP_QUALITY:-90}"
WEBP_DELAY_MS="${WEBP_DELAY_MS:-$(( (1000 + (WEBP_FPS / 2)) / WEBP_FPS ))}"
FRAME_PATTERN="$FRAMES_DIR"/%05d.png

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

fail() {
  echo "render_scene_media.sh: $*" >&2
  exit 1
}

count_pngs() {
  local dir="$1"
  find "$dir" -maxdepth 1 -type f -name '*.png' | wc -l | tr -d ' '
}

cleanup() {
  rm -rf "$WEBP_TEMP_DIR"
}

trap cleanup EXIT

require_cmd cargo
require_cmd ffmpeg
require_cmd img2webp

mkdir -p "$(dirname "$FRAMES_DIR")"
mkdir -p "$(dirname "$OUTPUT_MP4")"
mkdir -p "$(dirname "$OUTPUT_WEBP")"
rm -rf "$FRAMES_DIR"
rm -rf "$WEBP_TEMP_DIR"
rm -f "$OUTPUT_MP4" "$OUTPUT_WEBP"
mkdir -p "$WEBP_TEMP_DIR"

cd "$ROOT_DIR"

cargo run --release --bin "$EXPORT_BIN" -- "$FRAMES_DIR"

if [ ! -d "$FRAMES_DIR" ]; then
  fail "export bin '$EXPORT_BIN' did not create frames directory '$FRAMES_DIR'"
fi

if [ "$(count_pngs "$FRAMES_DIR")" -eq 0 ]; then
  fail "export bin '$EXPORT_BIN' produced no PNG frames in '$FRAMES_DIR'"
fi

ffmpeg \
  -y \
  -framerate "$FPS" \
  -i "$FRAME_PATTERN" \
  -vf "scale=${FINAL_WIDTH}:${FINAL_HEIGHT}:flags=lanczos" \
  -c:v libx264 \
  -preset veryslow \
  -crf 10 \
  -profile:v high \
  -pix_fmt yuv420p \
  -movflags +faststart \
  "$OUTPUT_MP4"

[ -f "$OUTPUT_MP4" ] || fail "ffmpeg did not create MP4 '$OUTPUT_MP4'"

ffmpeg \
  -y \
  -i "$OUTPUT_MP4" \
  -vf "fps=${WEBP_FPS},scale=${WEBP_SCALE_WIDTH}:-1" \
  "$WEBP_TEMP_DIR/frame%04d.png"

if [ "$(count_pngs "$WEBP_TEMP_DIR")" -eq 0 ]; then
  fail "ffmpeg did not extract any WebP frames into '$WEBP_TEMP_DIR'"
fi

img2webp \
  -loop 0 \
  -d "$WEBP_DELAY_MS" \
  -q "$WEBP_QUALITY" \
  "$WEBP_TEMP_DIR"/frame*.png \
  -o "$OUTPUT_WEBP"

[ -f "$OUTPUT_WEBP" ] || fail "img2webp did not create WebP '$OUTPUT_WEBP'"
