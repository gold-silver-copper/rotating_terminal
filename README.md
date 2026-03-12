# rotating_terminal

Bevy 0.18.1 project that loads a `vintage_terminal` 3D model and drives its screen with a live Ratatui render.

The project currently includes:

- an orbiting-camera scene
- a stationary-terminal scene with a slow zoom-in camera
- an offscreen export path for rendering either a full rotation or a zoom-in to an image sequence
- shell scripts that turn exported frames into high-quality MP4 files with `ffmpeg`

## Requirements

- Rust and Cargo
- `ffmpeg` for video export via the shell script

## Install

```bash
cargo check --bins
```

This validates all binaries in the project.

## Binaries

### `rotating_terminal`

Runs the main interactive scene with the camera orbiting around the terminal.

```bash
cargo run --bin rotating_terminal
```

### `stationary_terminal`

Runs a stationary-terminal scene where the camera slowly zooms toward the screen.

```bash
cargo run --bin stationary_terminal
```

### `export_rotation`

Renders one full orbit to a numbered PNG sequence using `bevy_image_export`.

Default output directory:

- `renders/rotation_frames`

Usage:

```bash
cargo run --release --bin export_rotation
```

Custom output directory:

```bash
cargo run --release --bin export_rotation -- /absolute/or/relative/output_dir
```

Export settings currently baked into the app:

- resolution: `1920x1920`
- frame rate: `60 fps`
- frame count: `420`
- motion: exactly one full orbit

### `export_zoom`

Renders the zoom-in camera move to a numbered PNG sequence using `bevy_image_export`.

Default output directory:

- `renders/zoom_frames`

Usage:

```bash
cargo run --release --bin export_zoom
```

Custom output directory:

```bash
cargo run --release --bin export_zoom -- /absolute/or/relative/output_dir
```

Export settings currently baked into the app:

- resolution: `1920x1920`
- frame rate: `60 fps`
- frame count: `1680`
- motion: one full scripted zoom-in pass

## Scripts

### `scripts/render_rotation_loop.sh`

Runs the export binary, then encodes the resulting image sequence into an MP4 with `ffmpeg`.

Default usage:

```bash
scripts/render_rotation_loop.sh
```

Default outputs:

- frames: `renders/rotation_frames`
- video: `renders/rotating_terminal_loop.mp4`

Custom paths:

```bash
scripts/render_rotation_loop.sh /path/to/frames /path/to/output.mp4
```

Optional environment variable:

```bash
FPS=30 scripts/render_rotation_loop.sh
```

What the script does:

1. Deletes the existing frame directory.
2. Runs `cargo run --release --bin export_rotation`.
3. Encodes `00001.png`, `00002.png`, ... into a high-quality H.264 MP4.

Encoding settings:

- codec: `libx264`
- preset: `veryslow`
- quality: `crf 12`

### `scripts/render_zoom_in.sh`

Runs the zoom export binary, then encodes the resulting image sequence into an MP4 with `ffmpeg`.

Default usage:

```bash
scripts/render_zoom_in.sh
```

Default outputs:

- frames: `renders/zoom_frames`
- video: `renders/terminal_zoom.mp4`

Custom paths:

```bash
scripts/render_zoom_in.sh /path/to/frames /path/to/output.mp4
```

Optional environment variable:

```bash
FPS=30 scripts/render_zoom_in.sh
```

What the script does:

1. Deletes the existing frame directory.
2. Runs `cargo run --release --bin export_zoom`.
3. Encodes `00001.png`, `00002.png`, ... into a high-quality H.264 MP4.

Encoding settings:

- codec: `libx264`
- preset: `veryslow`
- quality: `crf 12`

## Common Commands

Check everything:

```bash
cargo check --bins
```

Run the orbiting scene:

```bash
cargo run --bin rotating_terminal
```

Run the zoom scene:

```bash
cargo run --bin stationary_terminal
```

Export one full rotation to PNG frames:

```bash
cargo run --release --bin export_rotation
```

Export one zoom-in pass to PNG frames:

```bash
cargo run --release --bin export_zoom
```

Export rotation frames and build the MP4:

```bash
scripts/render_rotation_loop.sh
```

Export zoom frames and build the MP4:

```bash
scripts/render_zoom_in.sh
```

## Project Layout

- [Cargo.toml](/Users/kisaczka/Desktop/code/rotating_terminal/Cargo.toml): dependencies and package metadata
- [src/lib.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/lib.rs): shared scene setup, screen rendering, and export logic
- [src/bin/rotating_terminal.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/rotating_terminal.rs): orbiting-camera entry point
- [src/bin/stationary_terminal.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/stationary_terminal.rs): zoom-in entry point
- [src/bin/export_rotation.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_rotation.rs): image-sequence export entry point
- [src/bin/export_zoom.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_zoom.rs): zoom image-sequence export entry point
- [scripts/render_rotation_loop.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_rotation_loop.sh): export-and-encode helper
- [scripts/render_zoom_in.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_zoom_in.sh): zoom export-and-encode helper
- `vintage_terminal/`: model assets and textures

## Notes

- The export scripts expect `ffmpeg` to be available on your `PATH`.
- The export bins write numbered PNGs, which makes them easy to re-encode into other formats later.
