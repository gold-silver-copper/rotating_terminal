#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAMES_DIR="${1:-$ROOT_DIR/renders/intermission_frames}"
OUTPUT_MP4="${2:-$ROOT_DIR/renders/intermission_loop.mp4}"
OUTPUT_WEBP="${3:-$ROOT_DIR/intermission_loop.webp}"
WEBP_TEMP_DIR="${WEBP_TEMP_DIR:-$ROOT_DIR/renders/.webp_frames_intermission}"

"$ROOT_DIR/scripts/render_scene_media.sh" \
  "export_intermission" \
  "$FRAMES_DIR" \
  "$OUTPUT_MP4" \
  "$OUTPUT_WEBP" \
  "$WEBP_TEMP_DIR"
