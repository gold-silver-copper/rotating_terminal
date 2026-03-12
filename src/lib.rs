use std::f32::consts::{FRAC_PI_2, FRAC_PI_6, PI, TAU};

use bevy::{
    app::AppExit,
    asset::{AssetPlugin, RenderAssetUsages},
    camera::RenderTarget,
    core_pipeline::tonemapping::Tonemapping,
    gltf::GltfMaterialName,
    image::{CompressedImageFormats, ImagePlugin, ImageSampler, ImageType},
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel},
    post_process::bloom::Bloom,
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::Hdr,
    },
    scene::SceneInstanceReady,
};
use bevy_image_export::{ImageExport, ImageExportPlugin, ImageExportSettings, ImageExportSource};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect as TuiRect},
    prelude::Terminal,
    style::{Color as TuiColor, Style as TuiStyle},
    text::{Line as TuiLine, Text as TuiText},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use soft_ratatui::{
    EmbeddedGraphics, SoftBackend,
    embedded_graphics_unicodefonts::mono_4x6_atlas,
};
const TERMINAL_ASSET_PATH: &str = "vintage_terminal/scene.gltf";
const CAMERA_ORBIT_SPEED_RADIANS_PER_SECOND: f32 = 0.28;
const CAMERA_ORBIT_RADIUS: f32 = 8.0;
const CAMERA_HEIGHT: f32 = 1.0;
const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -0.2, 0.0);
const CAMERA_ORBIT_VERTICAL_SWAY: f32 = 0.35;
const ZOOM_CAMERA_START: Vec3 = Vec3::new(-0.35, 0.55, 8.4);
const ZOOM_CAMERA_END: Vec3 = Vec3::new(-0.6, 0.0, 2.5);
const ZOOM_CAMERA_LOOK_AT_START: Vec3 = Vec3::new(-0.1, -0.75, 0.35);
const ZOOM_CAMERA_LOOK_AT_END: Vec3 = Vec3::new(-0.18, 0.5, 0.62);
const ZOOM_DURATION_SECONDS: f32 = 16.0;
const TERMINAL_SCENE_OFFSET: Vec3 = Vec3::new(0.0, 0.25, 0.0);
const TERMINAL_SCALE: Vec3 = Vec3::splat(1.0);
const TERMINAL_FOOTPRINT_WIDTH: f32 = 3.265587;
const TERMINAL_FOOTPRINT_DEPTH: f32 = 2.057711;
const TERMINAL_CLUSTER_CLEARANCE: f32 = 0.04;
const TERMINAL_CLUSTER_GAP: f32 = TERMINAL_FOOTPRINT_WIDTH + TERMINAL_CLUSTER_CLEARANCE;
const TERMINAL_CLUSTER_OFFSET: f32 =
    (TERMINAL_CLUSTER_GAP + TERMINAL_FOOTPRINT_DEPTH) * 0.5;
const FLOOR_Y: f32 = -0.884;
const HIDDEN_SCENE_NODE_NAMES: &[&str] = &[];
const SCREEN_MATERIAL_NAME: &str = "Material.002";
const SCREEN_COLUMNS: u16 = 14;
const SCREEN_ROWS: u16 = 7;
const SCREEN_ATLAS_X: u32 = 10;
const SCREEN_ATLAS_Y: u32 = 463;
const SCREEN_ATLAS_WIDTH: u32 = 157;
const SCREEN_ATLAS_HEIGHT: u32 = 233;
const TERMINAL_BORDER_COLOR: TuiColor = TuiColor::Rgb(70, 120, 70);
const BLOOM_INTENSITY: f32 = 0.06;
const TERMINAL_SURFACE_MIN_ROUGHNESS: f32 = 0.97;
const TERMINAL_SURFACE_MAX_METALLIC: f32 = 0.0;
const TERMINAL_SURFACE_MAX_REFLECTANCE: f32 = 0.01;
const TUI_UPDATE_FPS: f64 = 15.0;
const LIVE_FIXED_FPS: f64 = 60.0;
const EXPORT_ROTATION_WIDTH: u32 = 1920;
const EXPORT_ROTATION_HEIGHT: u32 = 1080;
const EXPORT_ZOOM_WIDTH: u32 = 1920;
const EXPORT_ZOOM_HEIGHT: u32 = 1080;
const EXPORT_FPS: f64 = 60.0;
const EXPORT_ROTATION_FRAMES: u32 = 420;
const EXPORT_ZOOM_FRAMES: u32 = ((ZOOM_DURATION_SECONDS * 2.0) as u32) * EXPORT_FPS as u32;
const INTERMISSION_LOOP_SECONDS: f32 = 6.4;
const INTERMISSION_SPINNER_STEP_UPDATES: u64 = 8;
const EXPORT_INTERMISSION_FRAMES: u32 = (INTERMISSION_LOOP_SECONDS * EXPORT_FPS as f32) as u32;
const EXPORT_SHUTDOWN_FRAMES: u32 = ((SHUTDOWN_DURATION_SECONDS * 2.0) * EXPORT_FPS as f32) as u32;
const EXPORT_WARMUP_FRAMES: u32 = 60;
const INTERMISSION_CAMERA_BASE: Vec3 = Vec3::new(0.19, CAMERA_HEIGHT, CAMERA_ORBIT_RADIUS - 0.003);
const INTERMISSION_CAMERA_LOOK_AT: Vec3 = Vec3::new(0.02, -0.2, 0.08);
const SHUTDOWN_CAMERA_START: Vec3 = Vec3::new(-0.12, 1.25, 7.1);
const SHUTDOWN_CAMERA_END: Vec3 = Vec3::new(-0.25, 1.6, 8.9);
const SHUTDOWN_CAMERA_LOOK_AT: Vec3 = Vec3::new(-0.04, -0.22, 0.14);
const SHUTDOWN_DURATION_SECONDS: f32 = 8.0;

#[derive(Resource, Clone, Copy)]
enum SceneMode {
    Orbit,
    ZoomIn,
    Intermission,
    ShutdownOutro,
    ShutdownLoop,
}

#[derive(Component)]
struct SceneCamera;

#[derive(Resource)]
struct TerminalScreenTexture(Handle<Image>);

#[derive(Resource)]
struct TerminalScreenMaterial(Handle<StandardMaterial>);

struct TerminalScreenRenderer {
    terminal: Terminal<SoftBackend<EmbeddedGraphics>>,
    demo_app: TerminalDemoApp,
    scene_mode: SceneMode,
    atlas_width: u32,
    atlas_height: u32,
    atlas_template: Vec<u8>,
}

struct TerminalDemoApp {
    frame_count: u64,
    elapsed_secs: f32,
}

#[derive(Resource)]
struct TerminalAnimationCadence {
    accumulator_secs: f32,
}

#[derive(Resource, Clone)]
struct ExportRotationConfig {
    output_dir: String,
    frames: u32,
    width: u32,
    height: u32,
    warmup_frames: u32,
}

#[derive(Resource)]
struct ExportCaptureState {
    output_texture_handle: Handle<Image>,
    exporter_started: bool,
    warmup_frames_remaining: u32,
    current_frame: u32,
}

#[derive(Component)]
struct ExportCaptureCamera;

struct TerminalInstance {
    name: &'static str,
    position: Vec3,
    rotation: Quat,
}

impl TerminalDemoApp {
    fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed_secs: 0.0,
        }
    }

    fn on_tick(&mut self, delta: std::time::Duration) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.elapsed_secs += delta.as_secs_f32();
    }
}

pub fn run_rotating_terminal() {
    app(SceneMode::Orbit).run();
}

pub fn run_stationary_terminal() {
    app(SceneMode::ZoomIn).run();
}

pub fn run_intermission() {
    app(SceneMode::Intermission).run();
}

pub fn run_shutdown_outro() {
    app(SceneMode::ShutdownOutro).run();
}

pub fn run_export_rotation(output_dir: String) {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    export_app(
        output_dir,
        export_plugin,
        SceneMode::Orbit,
        EXPORT_ROTATION_FRAMES,
        EXPORT_ROTATION_WIDTH,
        EXPORT_ROTATION_HEIGHT,
    )
    .run();
    export_threads.finish();
}

pub fn run_export_zoom(output_dir: String) {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    export_app(
        output_dir,
        export_plugin,
        SceneMode::ZoomIn,
        EXPORT_ZOOM_FRAMES,
        EXPORT_ZOOM_WIDTH,
        EXPORT_ZOOM_HEIGHT,
    )
    .run();
    export_threads.finish();
}

pub fn run_export_intermission(output_dir: String) {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    export_app(
        output_dir,
        export_plugin,
        SceneMode::Intermission,
        EXPORT_INTERMISSION_FRAMES,
        EXPORT_ZOOM_WIDTH,
        EXPORT_ZOOM_HEIGHT,
    )
    .run();
    export_threads.finish();
}

pub fn run_export_shutdown(output_dir: String) {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    export_app(
        output_dir,
        export_plugin,
        SceneMode::ShutdownLoop,
        EXPORT_SHUTDOWN_FRAMES,
        EXPORT_ZOOM_WIDTH,
        EXPORT_ZOOM_HEIGHT,
    )
    .run();
    export_threads.finish();
}

fn app(scene_mode: SceneMode) -> App {
    let mut app = App::new();
    app.insert_resource(scene_mode)
        .insert_resource(Time::<Fixed>::from_hz(LIVE_FIXED_FPS))
        .insert_resource(TerminalAnimationCadence {
            accumulator_secs: 0.0,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ".".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, update_scene_camera)
        .add_systems(FixedUpdate, animate_terminal_screen);
    app
}

fn export_app(
    output_dir: String,
    export_plugin: ImageExportPlugin,
    scene_mode: SceneMode,
    frames: u32,
    width: u32,
    height: u32,
) -> App {
    let mut app = App::new();
    app.insert_resource(scene_mode)
        .insert_resource(TerminalAnimationCadence {
            accumulator_secs: 0.0,
        })
        .insert_resource(ExportRotationConfig {
            output_dir,
            frames,
            width,
            height,
            warmup_frames: EXPORT_WARMUP_FRAMES,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ".".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (width, height).into(),
                        visible: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                }),
        )
        .add_plugins(export_plugin)
        .add_systems(Startup, (setup, setup_export_capture))
        .add_systems(Update, update_scene_camera)
        .add_systems(Update, (animate_terminal_screen_export, drive_export_capture));
    app
}

fn setup(world: &mut World) {
    world.insert_resource(ClearColor(Color::srgb(0.01, 0.0, 0.03)));
    world.insert_resource(DirectionalLightShadowMap { size: 4096 });
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.85, 0.9, 1.0),
        brightness: 140.0,
        affects_lightmapped_meshes: true,
    });

    let scene_mode = *world.resource::<SceneMode>();
    let mut renderer = build_terminal_screen_renderer(scene_mode);
    let initial_image = renderer.render_to_image(std::time::Duration::ZERO);
    let image_handle = {
        let mut images = world.resource_mut::<Assets<Image>>();
        images.add(initial_image)
    };
    world.insert_resource(TerminalScreenTexture(image_handle.clone()));
    let screen_material_handle = {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(image_handle.clone()),
            emissive: LinearRgba::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        })
    };
    world.insert_resource(TerminalScreenMaterial(screen_material_handle));
    world.insert_non_send_resource(renderer);

    let terminal_scene = {
        let asset_server = world.resource::<AssetServer>();
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(TERMINAL_ASSET_PATH))
    };
    let floor_mesh = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        meshes.add(Circle::new(20.0).mesh().resolution(96))
    };
    let floor_material = {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.005, 0.009, 0.028),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            reflectance: 0.02,
            ..default()
        })
    };
    let mut commands = world.commands();

    commands
        .spawn((
            Name::new("Terminal Pivot"),
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for terminal in terminal_instances() {
                parent.spawn((
                    Name::new(terminal.name),
                    SceneRoot(terminal_scene.clone()),
                    Transform::from_translation(TERMINAL_SCENE_OFFSET + terminal.position)
                        .with_rotation(terminal.rotation)
                        .with_scale(TERMINAL_SCALE),
                ))
                .observe(configure_terminal_scene_when_ready);
            }
        });

    commands.spawn((
        Name::new("Floor"),
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, FLOOR_Y, 0.0).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    ));

    commands.spawn((
        Name::new("Key Light"),
        DirectionalLight {
            illuminance: 24_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, -FRAC_PI_6, -FRAC_PI_6)),
        CascadeShadowConfigBuilder {
            maximum_distance: 24.0,
            first_cascade_far_bound: 8.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 1_300_000.0,
            range: 36.0,
            shadows_enabled: false,
            color: Color::srgb(0.4, 0.9, 1.0),
            ..default()
        },
        Transform::from_xyz(4.5, 5.5, 4.0),
    ));

    commands.spawn((
        Name::new("Rim Light"),
        SpotLight {
            intensity: 3_500_000.0,
            range: 28.0,
            inner_angle: 0.55,
            outer_angle: 0.85,
            shadows_enabled: false,
            color: Color::srgb(0.12, 0.42, 0.2),
            ..default()
        },
        Transform::from_xyz(-5.5, 4.0, -4.0).looking_at(Vec3::new(0.0, -0.7, 0.1), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Scene Camera"),
        SceneCamera,
        Camera3d::default(),
        Msaa::Off,
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: BLOOM_INTENSITY,
            ..Bloom::NATURAL
        },
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            ..default()
        },
        DistanceFog {
            color: Color::srgba(0.03, 0.01, 0.07, 1.0),
            directional_light_color: Color::srgba(0.03, 0.12, 0.06, 0.2),
            falloff: FogFalloff::Linear {
                start: 9.0,
                end: 28.0,
            },
            ..default()
        },
        camera_transform(scene_mode, 0.0),
    ));
}

fn setup_export_capture(
    mut commands: Commands,
    scene_mode: Res<SceneMode>,
    export_config: Res<ExportRotationConfig>,
    mut images: ResMut<Assets<Image>>,
) {
    let output_texture_handle = {
        let size = Extent3d {
            width: export_config.width,
            height: export_config.height,
            depth_or_array_layers: 1,
        };
        let mut export_texture = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("terminal-export-texture"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            sampler: ImageSampler::nearest(),
            asset_usage: RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            ..default()
        };
        export_texture.resize(size);
        images.add(export_texture)
    };

    commands.insert_resource(ExportCaptureState {
        output_texture_handle: output_texture_handle.clone(),
        exporter_started: false,
        warmup_frames_remaining: export_config.warmup_frames,
        current_frame: 0,
    });

    commands.spawn((
        Name::new("Export Capture Camera"),
        ExportCaptureCamera,
        Camera3d::default(),
        Msaa::Off,
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: BLOOM_INTENSITY,
            ..Bloom::NATURAL
        },
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            ..default()
        },
        DistanceFog {
            color: Color::srgba(0.03, 0.01, 0.07, 1.0),
            directional_light_color: Color::srgba(0.03, 0.12, 0.06, 0.2),
            falloff: FogFalloff::Linear {
                start: 9.0,
                end: 28.0,
            },
            ..default()
        },
        Camera {
            order: 1,
            ..default()
        },
        RenderTarget::Image(output_texture_handle.into()),
        camera_transform(*scene_mode, 0.0),
    ));
}

fn drive_export_capture(
    mut commands: Commands,
    scene_mode: Res<SceneMode>,
    export_config: Res<ExportRotationConfig>,
    mut export_state: ResMut<ExportCaptureState>,
    mut capture_camera: Query<&mut Transform, With<ExportCaptureCamera>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if export_state.warmup_frames_remaining > 0 {
        export_state.warmup_frames_remaining -= 1;
        return;
    }

    if !export_state.exporter_started {
        commands.spawn((
            ImageExport(export_sources.add(export_state.output_texture_handle.clone())),
            ImageExportSettings {
                output_dir: export_config.output_dir.clone(),
                extension: "png".to_string(),
            },
        ));
        export_state.exporter_started = true;
    }

    if export_state.current_frame >= export_config.frames {
        app_exit.write(AppExit::Success);
        return;
    }

    for mut transform in &mut capture_camera {
        let progress = match *scene_mode {
            SceneMode::Orbit | SceneMode::Intermission => {
                cyclic_export_progress(export_state.current_frame, export_config.frames)
            }
            SceneMode::ZoomIn => zoom_export_progress(export_state.current_frame, export_config.frames),
            SceneMode::ShutdownLoop => {
                ping_pong_progress(export_state.current_frame as f32 / export_config.frames as f32 * 2.0)
            }
            _ if export_config.frames <= 1 => 1.0,
            _ => export_state.current_frame as f32 / (export_config.frames - 1) as f32,
        };
        *transform = camera_transform(*scene_mode, progress);
    }

    export_state.current_frame += 1;
}

fn update_scene_camera(
    time: Res<Time>,
    scene_mode: Res<SceneMode>,
    mut query: Query<&mut Transform, (With<SceneCamera>, Without<ExportCaptureCamera>)>,
) {
    for mut transform in &mut query {
        *transform = camera_transform(*scene_mode, runtime_camera_progress(*scene_mode, &time));
    }
}

fn runtime_camera_progress(scene_mode: SceneMode, time: &Time) -> f32 {
    let elapsed = time.elapsed_secs();
    match scene_mode {
        SceneMode::Orbit => elapsed * CAMERA_ORBIT_SPEED_RADIANS_PER_SECOND / TAU,
        SceneMode::ZoomIn => ping_pong_progress(elapsed / ZOOM_DURATION_SECONDS),
        SceneMode::Intermission => elapsed / INTERMISSION_LOOP_SECONDS,
        SceneMode::ShutdownOutro => (elapsed / SHUTDOWN_DURATION_SECONDS).min(1.0),
        SceneMode::ShutdownLoop => ping_pong_progress(elapsed / SHUTDOWN_DURATION_SECONDS),
    }
}

fn camera_transform(scene_mode: SceneMode, progress: f32) -> Transform {
    match scene_mode {
        SceneMode::Orbit => {
            let angle = progress * TAU;
            let position = Vec3::new(
                angle.sin() * CAMERA_ORBIT_RADIUS,
                CAMERA_HEIGHT + (angle * 2.0).sin() * CAMERA_ORBIT_VERTICAL_SWAY,
                angle.cos() * CAMERA_ORBIT_RADIUS,
            );
            Transform::from_translation(position).looking_at(CAMERA_LOOK_AT, Vec3::Y)
        }
        SceneMode::ZoomIn => {
            let position = ZOOM_CAMERA_START.lerp(ZOOM_CAMERA_END, progress);
            let look_at = ZOOM_CAMERA_LOOK_AT_START.lerp(ZOOM_CAMERA_LOOK_AT_END, progress);
            Transform::from_translation(position).looking_at(look_at, Vec3::Y)
        }
        SceneMode::Intermission => {
            let angle = progress * TAU;
            let drift = Vec3::new(angle.cos() * 0.14, (angle * 0.35).sin() * 0.05, angle.sin() * 0.18);
            let look_at =
                INTERMISSION_CAMERA_LOOK_AT + Vec3::new((angle * 0.4).sin() * 0.02, 0.01, 0.0);
            Transform::from_translation(INTERMISSION_CAMERA_BASE + drift).looking_at(look_at, Vec3::Y)
        }
        SceneMode::ShutdownOutro => {
            let eased = ease_in_out_cubic(progress);
            let position = SHUTDOWN_CAMERA_START.lerp(SHUTDOWN_CAMERA_END, eased);
            Transform::from_translation(position).looking_at(SHUTDOWN_CAMERA_LOOK_AT, Vec3::Y)
        }
        SceneMode::ShutdownLoop => {
            let eased = ease_in_out_cubic(progress);
            let position = SHUTDOWN_CAMERA_START.lerp(SHUTDOWN_CAMERA_END, eased);
            Transform::from_translation(position).looking_at(SHUTDOWN_CAMERA_LOOK_AT, Vec3::Y)
        }
    }
}

fn ping_pong_progress(progress: f32) -> f32 {
    let wrapped = progress.rem_euclid(2.0);
    if wrapped <= 1.0 {
        wrapped
    } else {
        2.0 - wrapped
    }
}

fn cyclic_export_progress(frame_index: u32, total_frames: u32) -> f32 {
    if total_frames == 0 {
        0.0
    } else {
        frame_index as f32 / total_frames as f32
    }
}

fn zoom_export_progress(frame_index: u32, total_frames: u32) -> f32 {
    if total_frames == 0 {
        return 0.0;
    }
    ping_pong_progress(frame_index as f32 / total_frames as f32 * 2.0)
}

fn ease_in_out_cubic(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * progress * progress * progress
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(3) * 0.5
    }
}

fn animate_terminal_screen(
    time: Res<Time>,
    mut cadence: ResMut<TerminalAnimationCadence>,
    mut terminal_screen_texture: ResMut<TerminalScreenTexture>,
    terminal_screen_material: Res<TerminalScreenMaterial>,
    mut terminal_screen_renderer: NonSendMut<TerminalScreenRenderer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    animate_terminal_screen_with_delta(
        time.delta_secs(),
        &mut cadence,
        &mut terminal_screen_texture,
        &terminal_screen_material,
        &mut terminal_screen_renderer,
        &mut materials,
        &mut images,
    );
}

fn animate_terminal_screen_export(
    mut cadence: ResMut<TerminalAnimationCadence>,
    mut terminal_screen_texture: ResMut<TerminalScreenTexture>,
    terminal_screen_material: Res<TerminalScreenMaterial>,
    mut terminal_screen_renderer: NonSendMut<TerminalScreenRenderer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    animate_terminal_screen_with_delta(
        (1.0 / EXPORT_FPS) as f32,
        &mut cadence,
        &mut terminal_screen_texture,
        &terminal_screen_material,
        &mut terminal_screen_renderer,
        &mut materials,
        &mut images,
    );
}

fn animate_terminal_screen_with_delta(
    delta_secs: f32,
    cadence: &mut TerminalAnimationCadence,
    terminal_screen_texture: &mut TerminalScreenTexture,
    terminal_screen_material: &TerminalScreenMaterial,
    terminal_screen_renderer: &mut TerminalScreenRenderer,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    cadence.accumulator_secs += delta_secs;
    let frame_interval = (1.0 / TUI_UPDATE_FPS) as f32;
    if cadence.accumulator_secs + f32::EPSILON < frame_interval {
        return;
    }
    let mut elapsed_since_last_draw = 0.0;
    while cadence.accumulator_secs + f32::EPSILON >= frame_interval {
        cadence.accumulator_secs -= frame_interval;
        elapsed_since_last_draw += frame_interval;
    }

    let updated_image = terminal_screen_renderer
        .render_to_image(std::time::Duration::from_secs_f32(elapsed_since_last_draw));
    let new_texture_handle = images.add(updated_image);
    let previous_texture_handle =
        std::mem::replace(&mut terminal_screen_texture.0, new_texture_handle.clone());

    let Some(material) = materials.get_mut(&terminal_screen_material.0) else {
        return;
    };
    material.base_color_texture = Some(new_texture_handle.clone());
    material.emissive_texture = Some(new_texture_handle);

    let _ = images.remove(previous_texture_handle.id());
}

fn configure_terminal_scene_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<&Name>,
    mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terminal_screen_material: Res<TerminalScreenMaterial>,
) {
    for descendant in children.iter_descendants(scene_ready.entity) {
        if let Ok(name) = names.get(descendant) {
            if HIDDEN_SCENE_NODE_NAMES
                .iter()
                .any(|hidden_name| name.as_str() == *hidden_name)
            {
                commands.entity(descendant).despawn();
            }
        }

        let Ok((_material_handle, material_name)) = mesh_materials.get(descendant) else {
            continue;
        };
        if material_name.0.as_str() != SCREEN_MATERIAL_NAME {
            if let Some(material) = materials.get_mut(&_material_handle.0) {
                material.metallic_roughness_texture = None;
                material.emissive_texture = None;
                material.emissive = LinearRgba::BLACK;
                material.perceptual_roughness =
                    material.perceptual_roughness.max(TERMINAL_SURFACE_MIN_ROUGHNESS);
                material.metallic = material.metallic.min(TERMINAL_SURFACE_MAX_METALLIC);
                material.reflectance = material.reflectance.min(TERMINAL_SURFACE_MAX_REFLECTANCE);
            }
            continue;
        }
        commands
            .entity(descendant)
            .insert(MeshMaterial3d(terminal_screen_material.0.clone()));
    }
}

fn terminal_instances() -> [TerminalInstance; 4] {
    [
        TerminalInstance {
            name: "Vintage Terminal North",
            position: Vec3::new(0.0, 0.0, TERMINAL_CLUSTER_OFFSET),
            rotation: Quat::IDENTITY,
        },
        TerminalInstance {
            name: "Vintage Terminal East",
            position: Vec3::new(TERMINAL_CLUSTER_OFFSET, 0.0, 0.0),
            rotation: Quat::from_rotation_y(FRAC_PI_2),
        },
        TerminalInstance {
            name: "Vintage Terminal South",
            position: Vec3::new(0.0, 0.0, -TERMINAL_CLUSTER_OFFSET),
            rotation: Quat::from_rotation_y(PI),
        },
        TerminalInstance {
            name: "Vintage Terminal West",
            position: Vec3::new(-TERMINAL_CLUSTER_OFFSET, 0.0, 0.0),
            rotation: Quat::from_rotation_y(-FRAC_PI_2),
        },
    ]
}

fn build_terminal_screen_renderer(scene_mode: SceneMode) -> TerminalScreenRenderer {
    let font_regular = mono_4x6_atlas();
    let backend = SoftBackend::<EmbeddedGraphics>::new(
        SCREEN_COLUMNS,
        SCREEN_ROWS,
        font_regular,
        None,
        None,
    );
    let terminal = Terminal::new(backend).expect("soft_ratatui backend should initialize");
    let atlas_template = load_screen_atlas_template();
    let atlas_width = atlas_template.texture_descriptor.size.width;
    let atlas_height = atlas_template.texture_descriptor.size.height;

    TerminalScreenRenderer {
        terminal,
        demo_app: TerminalDemoApp::new(),
        scene_mode,
        atlas_width,
        atlas_height,
        atlas_template: atlas_template
            .data
            .expect("screen atlas template should contain decoded image data"),
    }
}

fn draw_terminal_screen(frame: &mut Frame, app: &mut TerminalDemoApp, scene_mode: SceneMode) {
    let area = frame.area();
    let frame_index = app.frame_count;
    let background = TuiColor::Black;
    let title_color = TuiColor::Rgb(150, 255, 150);
    let horizontal_margin = match scene_mode {
        SceneMode::ShutdownOutro | SceneMode::ShutdownLoop => 0,
        _ => 1,
    };

    frame.render_widget(
        Block::new().style(TuiStyle::default().bg(background)),
        area,
    );

    let frame_area = area.inner(Margin::new(horizontal_margin, 0));
    let block = Block::new()
        .borders(Borders::ALL)
        .style(TuiStyle::default().bg(background))
        .border_style(TuiStyle::default().fg(TERMINAL_BORDER_COLOR).bg(background));
    let inner = block.inner(frame_area);
    frame.render_widget(block, frame_area);

    match scene_mode {
        SceneMode::Orbit | SceneMode::ZoomIn => draw_live_soon_screen(frame, inner, frame_index, title_color),
        SceneMode::Intermission => draw_intermission_screen(frame, inner, frame_index, title_color),
        SceneMode::ShutdownOutro | SceneMode::ShutdownLoop => {
            draw_shutdown_outro_screen(frame, inner, app, scene_mode, title_color)
        }
    }
}

fn draw_live_soon_screen(frame: &mut Frame, inner: TuiRect, frame_index: u64, title_color: TuiColor) {
    let flash_on = (frame_index / 8).is_multiple_of(2);
    let loader_color = TuiColor::Rgb(110, 255, 110);
    let live_color = if flash_on {
        TuiColor::Rgb(255, 240, 170)
    } else {
        TuiColor::Rgb(80, 70, 30)
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1), Constraint::Length(2)])
        .split(inner);
    let spinner = ['|', '/', '-', '\\'][(frame_index as usize) % 4];
    let sweep = (frame_index as usize) % 8;
    let loader_bar: String = (0..8)
        .map(|idx| if idx == sweep { '#' } else { '-' })
        .collect();

    frame.render_widget(
        Paragraph::new(title_text())
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(title_color).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!("{spinner}{loader_bar}{spinner}"))
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(loader_color).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new("LIVE SOON")
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(live_color).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

fn draw_intermission_screen(
    frame: &mut Frame,
    inner: TuiRect,
    frame_index: u64,
    title_color: TuiColor,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);
    let loop_updates = (INTERMISSION_LOOP_SECONDS * TUI_UPDATE_FPS as f32) as u64;
    let phase_updates = if loop_updates == 0 { 0 } else { frame_index % loop_updates };
    let spinner = ['.', 'o', 'O', 'o']
        [((frame_index / INTERMISSION_SPINNER_STEP_UPDATES.max(1)) % 4) as usize];
    let phrase_phase = (phase_updates * 4) / loop_updates.max(1);
    let phrase = match phrase_phase % 4 {
        0 => "RECESS\nSTANDBY",
        1 => "HOLD\nPAUSE",
        2 => "RECESS\nHOLD",
        _ => "STANDBY\nPAUSE",
    };
    frame.render_widget(
        Paragraph::new(title_text())
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(title_color).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!("{spinner} WAITING PATTERN {spinner}"))
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(TuiColor::Rgb(92, 180, 100)).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(phrase)
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(TuiColor::Rgb(215, 225, 180)).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

fn draw_shutdown_outro_screen(
    frame: &mut Frame,
    inner: TuiRect,
    app: &TerminalDemoApp,
    scene_mode: SceneMode,
    title_color: TuiColor,
) {
    let progress = match scene_mode {
        SceneMode::ShutdownOutro => (app.elapsed_secs / SHUTDOWN_DURATION_SECONDS).min(1.0),
        SceneMode::ShutdownLoop => ping_pong_progress(app.elapsed_secs / SHUTDOWN_DURATION_SECONDS),
        _ => 0.0,
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1), Constraint::Length(2)])
        .split(inner);
    let flash_on = ((app.elapsed_secs * 2.0) as u64).is_multiple_of(2);

    frame.render_widget(
        Paragraph::new(title_text())
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(title_color).bg(TuiColor::Black))
            .wrap(Wrap { trim: false }),
        rows[0],
    );

    if progress < 0.34 {
        frame.render_widget(
            Paragraph::new("THANKS FOR\nWATCHING")
                .alignment(Alignment::Center)
                .style(
                    TuiStyle::default()
                        .fg(if flash_on {
                            TuiColor::Rgb(215, 225, 180)
                        } else {
                            TuiColor::Rgb(110, 118, 94)
                        })
                        .bg(TuiColor::Black),
                )
                .wrap(Wrap { trim: false }),
            rows[2],
        );
    } else if progress < 0.68 {
        frame.render_widget(
            Paragraph::new("GOODBYE\nSEE YOU SOON")
                .alignment(Alignment::Center)
                .style(
                    TuiStyle::default()
                        .fg(if flash_on {
                            TuiColor::Rgb(108, 205, 108)
                        } else {
                            TuiColor::Rgb(52, 98, 52)
                        })
                        .bg(TuiColor::Black),
                )
                .wrap(Wrap { trim: false }),
            rows[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new("HAVE A\nNICE DAY")
                .alignment(Alignment::Center)
                .style(
                    TuiStyle::default()
                        .fg(if flash_on {
                            TuiColor::Rgb(82, 145, 82)
                        } else {
                            TuiColor::Rgb(36, 64, 36)
                        })
                        .bg(TuiColor::Black),
                )
                .wrap(Wrap { trim: false }),
            rows[2],
        );
    }
}

fn title_text() -> TuiText<'static> {
    TuiText::from(vec![
        TuiLine::from("TERMINAL"),
        TuiLine::from("COLLECTIVE"),
    ])
}

impl TerminalScreenRenderer {
    fn render_to_image(&mut self, delta: std::time::Duration) -> Image {
        self.demo_app.on_tick(delta);
        let terminal = &mut self.terminal;
        let demo_app = &mut self.demo_app;
        let scene_mode = self.scene_mode;
        terminal
            .draw(|frame| {
                draw_terminal_screen(frame, demo_app, scene_mode);
            })
            .expect("soft_ratatui screen should render");

        let backend = terminal.backend();
        let screen_width = backend.get_pixmap_width() as u32;
        let screen_height = backend.get_pixmap_height() as u32;
        let screen_data = rotate_rgba_90_ccw(
            screen_width,
            screen_height,
            &flip_rgba_rows(screen_width, screen_height, &backend.get_pixmap_data_as_rgba()),
        );
        let rotated_screen_width = screen_height;
        let rotated_screen_height = screen_width;
        let atlas_data = composite_screen_into_atlas(
            &self.atlas_template,
            self.atlas_width,
            self.atlas_height,
            SCREEN_ATLAS_X,
            SCREEN_ATLAS_Y,
            SCREEN_ATLAS_WIDTH,
            SCREEN_ATLAS_HEIGHT,
            rotated_screen_width,
            rotated_screen_height,
            &screen_data,
        );
        build_screen_image(self.atlas_width, self.atlas_height, atlas_data)
    }
}

fn build_screen_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;
    image.sampler = ImageSampler::nearest();
    image
}

fn load_screen_atlas_template() -> Image {
    Image::from_buffer(
        include_bytes!("../vintage_terminal/textures/Material.002_baseColor.png"),
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .expect("screen atlas template should decode")
}

fn composite_screen_into_atlas(
    atlas_template: &[u8],
    atlas_width: u32,
    atlas_height: u32,
    target_x: u32,
    target_y: u32,
    target_width: u32,
    target_height: u32,
    screen_width: u32,
    screen_height: u32,
    screen_data: &[u8],
) -> Vec<u8> {
    let mut atlas = atlas_template.to_vec();

    for dest_y in 0..target_height {
        let source_y = dest_y * screen_height / target_height;
        for dest_x in 0..target_width {
            let source_x = dest_x * screen_width / target_width;
            let source_index = ((source_y * screen_width + source_x) * 4) as usize;
            let atlas_x = target_x + dest_x;
            let atlas_y = target_y + dest_y;
            if atlas_x >= atlas_width || atlas_y >= atlas_height {
                continue;
            }
            let atlas_index = ((atlas_y * atlas_width + atlas_x) * 4) as usize;
            atlas[atlas_index..atlas_index + 4]
                .copy_from_slice(&screen_data[source_index..source_index + 4]);
        }
    }

    atlas
}

fn flip_rgba_rows(width: u32, height: u32, source: &[u8]) -> Vec<u8> {
    let row_len = (width * 4) as usize;
    let mut flipped = vec![0; source.len()];
    for y in 0..height as usize {
        let src_start = y * row_len;
        let dst_start = (height as usize - 1 - y) * row_len;
        flipped[dst_start..dst_start + row_len]
            .copy_from_slice(&source[src_start..src_start + row_len]);
    }
    flipped
}

fn rotate_rgba_90_ccw(width: u32, height: u32, source: &[u8]) -> Vec<u8> {
    let mut rotated = vec![0; source.len()];
    for y in 0..height {
        for x in 0..width {
            let src_index = ((y * width + x) * 4) as usize;
            let dest_x = y;
            let dest_y = width - 1 - x;
            let dest_index = ((dest_y * height + dest_x) * 4) as usize;
            rotated[dest_index..dest_index + 4].copy_from_slice(&source[src_index..src_index + 4]);
        }
    }
    rotated
}
