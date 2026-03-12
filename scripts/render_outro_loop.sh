#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAMES_DIR="${1:-$ROOT_DIR/renders/shutdown_frames}"
OUTPUT_MP4="${2:-$ROOT_DIR/renders/shutdown_loop.mp4}"
OUTPUT_WEBP="${3:-$ROOT_DIR/shutdown_loop.webp}"
WEBP_TEMP_DIR="${WEBP_TEMP_DIR:-$ROOT_DIR/renders/.webp_frames_shutdown}"

"$ROOT_DIR/scripts/render_scene_media.sh" \
  "export_shutdown" \
  "$FRAMES_DIR" \
  "$OUTPUT_MP4" \
  "$OUTPUT_WEBP" \
  "$WEBP_TEMP_DIR"
