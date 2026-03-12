#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAMES_DIR="${1:-$ROOT_DIR/renders/intermission_frames}"
OUTPUT_MP4="${2:-$ROOT_DIR/renders/intermission_loop.mp4}"
FPS="${FPS:-60}"

mkdir -p "$(dirname "$FRAMES_DIR")"
mkdir -p "$(dirname "$OUTPUT_MP4")"
rm -rf "$FRAMES_DIR"

cd "$ROOT_DIR"

cargo run --release --bin export_intermission -- "$FRAMES_DIR"

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
