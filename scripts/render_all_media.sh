#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RENDERS_DIR="${1:-$ROOT_DIR/renders}"
FPS="${FPS:-60}"
WEBP_FPS="${WEBP_FPS:-30}"
WEBP_SCALE_WIDTH="${WEBP_SCALE_WIDTH:-320}"
WEBP_QUALITY="${WEBP_QUALITY:-90}"
WEBP_DELAY_MS="${WEBP_DELAY_MS:-$(( (1000 + (WEBP_FPS / 2)) / WEBP_FPS ))}"
WEBP_TEMP_ROOT="${WEBP_TEMP_ROOT:-$RENDERS_DIR/.webp_frames}"

cleanup() {
  rm -rf "$WEBP_TEMP_ROOT"
}

trap cleanup EXIT

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

render_and_convert() {
  local label="$1"
  local render_script="$2"
  local frames_dir="$3"
  local mp4_path="$4"
  local webp_path="$5"
  local temp_frames_dir="$WEBP_TEMP_ROOT/$label"

  echo "==> Rendering $label"
  FPS="$FPS" "$render_script" "$frames_dir" "$mp4_path"

  echo "==> Converting $label MP4 to WebP"
  rm -rf "$temp_frames_dir"
  mkdir -p "$temp_frames_dir"
  ffmpeg \
    -y \
    -i "$mp4_path" \
    -vf "fps=${WEBP_FPS},scale=${WEBP_SCALE_WIDTH}:-1" \
    "$temp_frames_dir/frame%04d.png"
  img2webp \
    -loop 0 \
    -d "$WEBP_DELAY_MS" \
    -q "$WEBP_QUALITY" \
    "$temp_frames_dir"/frame*.png \
    -o "$webp_path"
  rm -rf "$temp_frames_dir"
}

require_cmd cargo
require_cmd ffmpeg
require_cmd img2webp

mkdir -p "$RENDERS_DIR"
mkdir -p "$WEBP_TEMP_ROOT"

render_and_convert \
  "rotation" \
  "$ROOT_DIR/scripts/render_rotation_loop.sh" \
  "$RENDERS_DIR/rotation_frames" \
  "$RENDERS_DIR/rotating_terminal_loop.mp4" \
  "$ROOT_DIR/rotating_terminal_loop.webp"

render_and_convert \
  "zoom" \
  "$ROOT_DIR/scripts/render_zoom_in.sh" \
  "$RENDERS_DIR/zoom_frames" \
  "$RENDERS_DIR/terminal_zoom_loop.mp4" \
  "$ROOT_DIR/terminal_zoom_loop.webp"

render_and_convert \
  "intermission" \
  "$ROOT_DIR/scripts/render_intermission_loop.sh" \
  "$RENDERS_DIR/intermission_frames" \
  "$RENDERS_DIR/intermission_loop.mp4" \
  "$ROOT_DIR/intermission_loop.webp"

render_and_convert \
  "shutdown" \
  "$ROOT_DIR/scripts/render_outro_loop.sh" \
  "$RENDERS_DIR/shutdown_frames" \
  "$RENDERS_DIR/shutdown_loop.mp4" \
  "$ROOT_DIR/shutdown_loop.webp"
