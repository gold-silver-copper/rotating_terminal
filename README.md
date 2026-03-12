# rotating_terminal

Bevy 0.18.1 project that loads a `vintage_terminal` 3D model and drives its screen with a live Ratatui render.

The project currently includes:

- an orbiting hero shot
- a stationary terminal shot with a looping zoom in/out camera
- an intermission variant
- a shutdown/outro variant
- offscreen exporters for all current loopable shots
- shell scripts that render PNG sequences and encode high-quality looping MP4s with `ffmpeg`

## Requirements

- Rust and Cargo
- `ffmpeg` for video export

## Install

```bash
cargo check --bins
```

## Interactive Binaries

### `rotating_terminal`

Orbiting camera around the terminal.

```bash
cargo run --bin rotating_terminal
```

### `stationary_terminal`

Stationary terminal with a looping zoom in/out camera move.

```bash
cargo run --bin stationary_terminal
```

### `intermission`

Intermission/standby scene variant.

```bash
cargo run --bin intermission
```

### `shutdown_outro`

Shutdown/outro scene variant.

```bash
cargo run --bin shutdown_outro
```

## Export Binaries

All exports render numbered PNGs with `bevy_image_export`.

### `export_rotation`

```bash
cargo run --release --bin export_rotation
```

Custom output directory:

```bash
cargo run --release --bin export_rotation -- /path/to/output_dir
```

Settings:

- resolution: `1920x1080`
- frame rate: `60 fps`
- frame count: `420`
- motion: one seamless full orbit loop

### `export_zoom`

```bash
cargo run --release --bin export_zoom
```

Custom output directory:

```bash
cargo run --release --bin export_zoom -- /path/to/output_dir
```

Settings:

- resolution: `1920x1080`
- frame rate: `60 fps`
- frame count: `3360`
- motion: seamless zoom in/out ping-pong loop

### `export_intermission`

```bash
cargo run --release --bin export_intermission
```

Custom output directory:

```bash
cargo run --release --bin export_intermission -- /path/to/output_dir
```

Settings:

- resolution: `1920x1080`
- frame rate: `60 fps`
- motion: seamless intermission loop

### `export_shutdown`

```bash
cargo run --release --bin export_shutdown
```

Custom output directory:

```bash
cargo run --release --bin export_shutdown -- /path/to/output_dir
```

Settings:

- resolution: `1920x1080`
- frame rate: `60 fps`
- motion: seamless outro loop based on the shutdown camera/text cycle

## Render Scripts

The scripts below are executable and can be run directly with `./scripts/...`.

### `./scripts/render_rotation_loop.sh`

Default outputs:

- frames: `renders/rotation_frames`
- video: `renders/rotating_terminal_loop.mp4`

Usage:

```bash
./scripts/render_rotation_loop.sh
./scripts/render_rotation_loop.sh /path/to/frames /path/to/output.mp4
```

### `./scripts/render_zoom_in.sh`

Default outputs:

- frames: `renders/zoom_frames`
- video: `renders/terminal_zoom_loop.mp4`

Usage:

```bash
./scripts/render_zoom_in.sh
./scripts/render_zoom_in.sh /path/to/frames /path/to/output.mp4
```

### `./scripts/render_intermission_loop.sh`

Default outputs:

- frames: `renders/intermission_frames`
- video: `renders/intermission_loop.mp4`

Usage:

```bash
./scripts/render_intermission_loop.sh
./scripts/render_intermission_loop.sh /path/to/frames /path/to/output.mp4
```

### `./scripts/render_outro_loop.sh`

Default outputs:

- frames: `renders/shutdown_frames`
- video: `renders/shutdown_loop.mp4`

Usage:

```bash
./scripts/render_outro_loop.sh
./scripts/render_outro_loop.sh /path/to/frames /path/to/output.mp4
```

### Shared script options

Optional environment variable:

```bash
FPS=30 ./scripts/render_rotation_loop.sh
```

All scripts:

1. delete the existing frame directory
2. run the matching `export_*` binary
3. encode the numbered PNG sequence to H.264 MP4 with `ffmpeg`

Encoding settings:

- codec: `libx264`
- preset: `veryslow`
- quality: `crf 12`

## Common Commands

Check all binaries:

```bash
cargo check --bins
```

Run the main scenes:

```bash
cargo run --bin rotating_terminal
cargo run --bin stationary_terminal
cargo run --bin intermission
cargo run --bin shutdown_outro
```

Render looping MP4s:

```bash
./scripts/render_rotation_loop.sh
./scripts/render_zoom_in.sh
./scripts/render_intermission_loop.sh
./scripts/render_outro_loop.sh
```

## Project Layout

- [Cargo.toml](/Users/kisaczka/Desktop/code/rotating_terminal/Cargo.toml): dependencies and package metadata
- [src/lib.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/lib.rs): shared scene setup, TUI rendering, camera motion, and export logic
- [src/bin/rotating_terminal.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/rotating_terminal.rs): orbit scene
- [src/bin/stationary_terminal.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/stationary_terminal.rs): looping zoom scene
- [src/bin/intermission.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/intermission.rs): intermission scene
- [src/bin/shutdown_outro.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/shutdown_outro.rs): outro scene
- [src/bin/export_rotation.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_rotation.rs): rotation exporter
- [src/bin/export_zoom.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_zoom.rs): zoom-loop exporter
- [src/bin/export_intermission.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_intermission.rs): intermission exporter
- [src/bin/export_shutdown.rs](/Users/kisaczka/Desktop/code/rotating_terminal/src/bin/export_shutdown.rs): outro exporter
- [scripts/render_rotation_loop.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_rotation_loop.sh): rotation render script
- [scripts/render_zoom_in.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_zoom_in.sh): zoom render script
- [scripts/render_intermission_loop.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_intermission_loop.sh): intermission render script
- [scripts/render_outro_loop.sh](/Users/kisaczka/Desktop/code/rotating_terminal/scripts/render_outro_loop.sh): outro render script
- `vintage_terminal/`: model assets and textures

## Notes

- The TUI updates at a lower cadence than the camera/render loop so screen motion reads more deliberately.
- The export scripts expect `ffmpeg` to be available on your `PATH`.
- Export bins write numbered PNGs first, which makes it easy to re-encode later.
