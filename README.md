# rotating_terminal

Bevy 0.18.1 project that loads a `vintage_terminal` 3D model and drives its screen with a `ratatui` UI rendered through [`soft_ratatui`](https://github.com/gold-silver-copper/soft_ratatui).

The `vintage_terminal` asset is from [Sketchfab: Vintage Terminal](https://sketchfab.com/3d-models/vintage-terminal-847c174b86e24d868d25217ca2297886).

Current project surface:

- orbiting hero shot
- stationary zoom loop
- intermission/standby loop
- shutdown/outro loop
- PNG exporters for every scene
- `ffmpeg` helper scripts that encode high-quality H.264 MP4 loops

## Requirements

- Rust and Cargo
- `ffmpeg` for the render scripts

## Install / Verify

```bash
cargo check --bins
```

## Interactive Binaries

### `rotating_terminal`

Orbiting hero shot around the terminal.

```bash
cargo run --bin rotating_terminal
```

### `stationary_terminal`

Stationary shot with a looping zoom in / zoom out camera move.

```bash
cargo run --bin stationary_terminal
```

### `intermission`

Standby/intermission scene with the same low camera height and radius envelope as the orbit scene, plus a faster waiting-pattern spinner on the terminal screen.

```bash
cargo run --bin intermission
```

### `shutdown_outro`

Shutdown/outro scene with friendly flashing farewell copy on the terminal screen.

```bash
cargo run --bin shutdown_outro
```

## Export Binaries

All exporters render numbered PNGs with `bevy_image_export`.

Shared export settings:

- resolution: `1920x1080`
- output cadence: `60 fps`
- warmup before capture: `60` frames

### `export_rotation`

```bash
cargo run --release --bin export_rotation
```

Custom output directory:

```bash
cargo run --release --bin export_rotation -- /path/to/output_dir
```

Settings:

- frame count: `420`
- duration: `7.0s`
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

- frame count: `1920`
- duration: `32.0s`
- motion: seamless zoom in / zoom out ping-pong loop

### `export_intermission`

```bash
cargo run --release --bin export_intermission
```

Custom output directory:

```bash
cargo run --release --bin export_intermission -- /path/to/output_dir
```

Settings:

- frame count: `384`
- duration: `6.4s`
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

- frame count: `960`
- duration: `16.0s`
- motion: seamless shutdown ping-pong loop based on the shutdown camera and screen text cycle

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

### Shared script option

Override the encoded playback framerate:

```bash
FPS=30 ./scripts/render_rotation_loop.sh
```

Each script:

1. deletes the existing frame directory
2. runs the matching `export_*` binary
3. encodes the PNG sequence to an H.264 MP4 with `ffmpeg`

Encoding settings:

- codec: `libx264`
- preset: `veryslow`
- quality: `crf 12`
- pixel format: `yuv420p`

## Common Commands

Check everything:

```bash
cargo check --bins
```

Run the scenes:

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

- [Cargo.toml](Cargo.toml): package metadata and dependencies
- [src/lib.rs](src/lib.rs): shared scene setup, TUI rendering, camera motion, material tuning, and export logic
- [src/bin/rotating_terminal.rs](src/bin/rotating_terminal.rs): orbit scene
- [src/bin/stationary_terminal.rs](src/bin/stationary_terminal.rs): zoom-loop scene
- [src/bin/intermission.rs](src/bin/intermission.rs): intermission scene
- [src/bin/shutdown_outro.rs](src/bin/shutdown_outro.rs): shutdown/outro scene
- [src/bin/export_rotation.rs](src/bin/export_rotation.rs): orbit exporter
- [src/bin/export_zoom.rs](src/bin/export_zoom.rs): zoom-loop exporter
- [src/bin/export_intermission.rs](src/bin/export_intermission.rs): intermission exporter
- [src/bin/export_shutdown.rs](src/bin/export_shutdown.rs): shutdown exporter
- [scripts/render_rotation_loop.sh](scripts/render_rotation_loop.sh): orbit render helper
- [scripts/render_zoom_in.sh](scripts/render_zoom_in.sh): zoom render helper
- [scripts/render_intermission_loop.sh](scripts/render_intermission_loop.sh): intermission render helper
- [scripts/render_outro_loop.sh](scripts/render_outro_loop.sh): shutdown render helper
- [vintage_terminal/](vintage_terminal/): GLTF model and source textures
- [bevy_cube_colors/](bevy_cube_colors/): separate Bevy scratch/example app in the repo

## Notes

- The terminal screen is a `14x7` `soft_ratatui` surface composited back into the model's screen texture atlas.
- TUI content updates at `15 fps`; the live scene runs at `30 fps`; exports render at `60 fps`.
- The export/live TUI clock advances using the accumulated render interval so intermission and shutdown text timing matches the final renders.
- The terminal body materials are intentionally matte and bloom is kept low to avoid blown-out highlights on the shell.
- Exporters write PNG sequences first, which makes re-encoding with different `ffmpeg` settings straightforward.
