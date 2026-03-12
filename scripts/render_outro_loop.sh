#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAMES_DIR="${1:-$ROOT_DIR/renders/shutdown_frames}"
OUTPUT_MP4="${2:-$ROOT_DIR/renders/shutdown_loop.mp4}"
FPS="${FPS:-60}"
OUTPUT_WEBP="${3:-$ROOT_DIR/shutdown_loop.webp}"
WEBP_FPS="${WEBP_FPS:-30}"
WEBP_SCALE_WIDTH="${WEBP_SCALE_WIDTH:-320}"
WEBP_QUALITY="${WEBP_QUALITY:-90}"
WEBP_DELAY_MS="${WEBP_DELAY_MS:-$(( (1000 + (WEBP_FPS / 2)) / WEBP_FPS ))}"
WEBP_TEMP_DIR="${WEBP_TEMP_DIR:-$ROOT_DIR/renders/.webp_frames_shutdown}"

mkdir -p "$(dirname "$FRAMES_DIR")"
mkdir -p "$(dirname "$OUTPUT_MP4")"
mkdir -p "$(dirname "$OUTPUT_WEBP")"
rm -rf "$FRAMES_DIR"
rm -rf "$WEBP_TEMP_DIR"
mkdir -p "$WEBP_TEMP_DIR"

cd "$ROOT_DIR"

cargo run --release --bin export_shutdown -- "$FRAMES_DIR"

ffmpeg \
  -y \
  -framerate "$FPS" \
  -i "$FRAMES_DIR/%05d.png" \
  -c:v libx264 \
  -preset veryslow \
  -crf 12 \
  -profile:v high \
  -pix_fmt yuv420p \
  -movflags +faststart \
  "$OUTPUT_MP4"

ffmpeg \
  -y \
  -i "$OUTPUT_MP4" \
  -vf "fps=${WEBP_FPS},scale=${WEBP_SCALE_WIDTH}:-1" \
  "$WEBP_TEMP_DIR/frame%04d.png"

img2webp \
  -loop 0 \
  -d "$WEBP_DELAY_MS" \
  -q "$WEBP_QUALITY" \
  "$WEBP_TEMP_DIR"/frame*.png \
  -o "$OUTPUT_WEBP"

rm -rf "$WEBP_TEMP_DIR"
