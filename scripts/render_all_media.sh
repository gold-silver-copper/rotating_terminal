#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RENDERS_DIR="${1:-$ROOT_DIR/renders}"
FPS="${FPS:-60}"

mkdir -p "$RENDERS_DIR"

FPS="$FPS" "$ROOT_DIR/scripts/render_rotation_loop.sh" \
  "$RENDERS_DIR/rotation_frames" \
  "$RENDERS_DIR/rotating_terminal_loop.mp4" \
  "$ROOT_DIR/rotating_terminal_loop.webp"

FPS="$FPS" "$ROOT_DIR/scripts/render_zoom_in.sh" \
  "$RENDERS_DIR/zoom_frames" \
  "$RENDERS_DIR/terminal_zoom_loop.mp4" \
  "$ROOT_DIR/terminal_zoom_loop.webp"

FPS="$FPS" "$ROOT_DIR/scripts/render_intermission_loop.sh" \
  "$RENDERS_DIR/intermission_frames" \
  "$RENDERS_DIR/intermission_loop.mp4" \
  "$ROOT_DIR/intermission_loop.webp"

FPS="$FPS" "$ROOT_DIR/scripts/render_outro_loop.sh" \
  "$RENDERS_DIR/shutdown_frames" \
  "$RENDERS_DIR/shutdown_loop.mp4" \
  "$ROOT_DIR/shutdown_loop.webp"
