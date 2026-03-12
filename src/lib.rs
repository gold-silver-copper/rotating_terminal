use std::f32::consts::{FRAC_PI_6, TAU};

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
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    prelude::Terminal,
    style::{Color as TuiColor, Style as TuiStyle},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use soft_ratatui::{
    EmbeddedGraphics, SoftBackend,
    embedded_graphics_unicodefonts::mono_4x6_atlas,
};
const TERMINAL_ASSET_PATH: &str = "vintage_terminal/scene.gltf";
const CAMERA_ORBIT_SPEED_RADIANS_PER_SECOND: f32 = 0.12;
const CAMERA_ORBIT_RADIUS: f32 = 8.8;
const CAMERA_HEIGHT: f32 = 3.8;
const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -0.2, 0.0);
const ZOOM_CAMERA_START: Vec3 = Vec3::new(-0.35, 0.55, 8.4);
const ZOOM_CAMERA_END: Vec3 = Vec3::new(-0.6, 0.0, 2.5);
const ZOOM_CAMERA_LOOK_AT_START: Vec3 = Vec3::new(-0.1, -0.75, 0.35);
const ZOOM_CAMERA_LOOK_AT_END: Vec3 = Vec3::new(-0.18, 0.5, 0.62);
const ZOOM_DURATION_SECONDS: f32 = 28.0;
const TERMINAL_SCENE_OFFSET: Vec3 = Vec3::new(0.0, 0.25, 0.0);
const TERMINAL_SCALE: Vec3 = Vec3::splat(1.0);
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
const EXPORT_ROTATION_WIDTH: u32 = 1920;
const EXPORT_ROTATION_HEIGHT: u32 = 1080;
const EXPORT_ZOOM_WIDTH: u32 = 1920;
const EXPORT_ZOOM_HEIGHT: u32 = 1080;
const EXPORT_FPS: f64 = 60.0;
const EXPORT_ROTATION_FRAMES: u32 = 420;
const EXPORT_ZOOM_FRAMES: u32 = (ZOOM_DURATION_SECONDS as u32) * EXPORT_FPS as u32;
const EXPORT_WARMUP_FRAMES: u32 = 24;

#[derive(Resource, Clone, Copy)]
enum CameraMode {
    Orbit,
    ZoomIn,
}

#[derive(Component)]
struct OrbitCamera {
    angle: f32,
}

#[derive(Component)]
struct ZoomCamera {
    progress: f32,
}

#[derive(Resource)]
struct TerminalScreenTexture(Handle<Image>);

#[derive(Resource)]
struct TerminalScreenMaterial(Handle<StandardMaterial>);

struct TerminalScreenRenderer {
    terminal: Terminal<SoftBackend<EmbeddedGraphics>>,
    demo_app: TerminalDemoApp,
    atlas_width: u32,
    atlas_height: u32,
    atlas_template: Vec<u8>,
}

struct TerminalDemoApp {
    frame_count: u64,
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

impl TerminalDemoApp {
    fn new() -> Self {
        Self { frame_count: 0 }
    }

    fn on_tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

pub fn run_rotating_terminal() {
    app(CameraMode::Orbit).run();
}

pub fn run_stationary_terminal() {
    app(CameraMode::ZoomIn).run();
}

pub fn run_export_rotation(output_dir: String) {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    export_app(
        output_dir,
        export_plugin,
        CameraMode::Orbit,
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
        CameraMode::ZoomIn,
        EXPORT_ZOOM_FRAMES,
        EXPORT_ZOOM_WIDTH,
        EXPORT_ZOOM_HEIGHT,
    )
    .run();
    export_threads.finish();
}

fn app(camera_mode: CameraMode) -> App {
    let mut app = App::new();
    app.insert_resource(camera_mode)
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ".".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, zoom_camera))
        .add_systems(FixedUpdate, animate_terminal_screen);
    app
}

fn export_app(
    output_dir: String,
    export_plugin: ImageExportPlugin,
    camera_mode: CameraMode,
    frames: u32,
    width: u32,
    height: u32,
) -> App {
    let mut app = App::new();
    app.insert_resource(camera_mode)
        .insert_resource(Time::<Fixed>::from_hz(EXPORT_FPS))
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
        .add_systems(Update, (orbit_camera, zoom_camera))
        .add_systems(FixedUpdate, (animate_terminal_screen, drive_export_capture));
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

    let mut renderer = build_terminal_screen_renderer();
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
            emissive_texture: Some(image_handle.clone()),
            emissive: LinearRgba::rgb(1.4, 1.4, 1.4),
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
    let camera_mode = *world.resource::<CameraMode>();
    let floor_mesh = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        meshes.add(Plane3d::default().mesh().size(40.0, 40.0))
    };
    let floor_material = {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.02, 0.08),
            perceptual_roughness: 0.92,
            metallic: 0.02,
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
            parent.spawn((
                Name::new("Vintage Terminal"),
                SceneRoot(terminal_scene),
                Transform::from_translation(TERMINAL_SCENE_OFFSET)
                    .with_rotation(Quat::IDENTITY)
                    .with_scale(TERMINAL_SCALE),
            ))
            .observe(configure_terminal_scene_when_ready);
        });

    commands.spawn((
        Name::new("Floor"),
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, FLOOR_Y, 0.0),
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
            color: Color::srgb(1.0, 0.25, 0.85),
            ..default()
        },
        Transform::from_xyz(-5.5, 4.0, -4.0).looking_at(Vec3::new(0.0, -0.7, 0.1), Vec3::Y),
    ));

    match camera_mode {
        CameraMode::Orbit => {
            commands.spawn((
                Name::new("Orbit Camera"),
                OrbitCamera { angle: 0.0 },
                Camera3d::default(),
                Msaa::Off,
                Hdr,
                Tonemapping::TonyMcMapface,
                Bloom::NATURAL,
                ScreenSpaceAmbientOcclusion {
                    quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
                    ..default()
                },
                DistanceFog {
                    color: Color::srgba(0.03, 0.01, 0.07, 1.0),
                    directional_light_color: Color::srgba(0.2, 0.05, 0.2, 0.2),
                    falloff: FogFalloff::Linear {
                        start: 10.0,
                        end: 28.0,
                    },
                    ..default()
                },
                Transform::from_xyz(0.0, CAMERA_HEIGHT, CAMERA_ORBIT_RADIUS)
                    .looking_at(CAMERA_LOOK_AT, Vec3::Y),
            ));
        }
        CameraMode::ZoomIn => {
            commands.spawn((
                Name::new("Zoom Camera"),
                ZoomCamera { progress: 0.0 },
                Camera3d::default(),
                Msaa::Off,
                Hdr,
                Tonemapping::TonyMcMapface,
                Bloom::NATURAL,
                ScreenSpaceAmbientOcclusion {
                    quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
                    ..default()
                },
                DistanceFog {
                    color: Color::srgba(0.03, 0.01, 0.07, 1.0),
                    directional_light_color: Color::srgba(0.2, 0.05, 0.2, 0.2),
                    falloff: FogFalloff::Linear {
                        start: 9.0,
                        end: 24.0,
                    },
                    ..default()
                },
                Transform::from_translation(ZOOM_CAMERA_START)
                    .looking_at(ZOOM_CAMERA_LOOK_AT_START, Vec3::Y),
            ));
        }
    }
}

fn setup_export_capture(
    mut commands: Commands,
    camera_mode: Res<CameraMode>,
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
        Bloom::NATURAL,
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            ..default()
        },
        DistanceFog {
            color: Color::srgba(0.03, 0.01, 0.07, 1.0),
            directional_light_color: Color::srgba(0.2, 0.05, 0.2, 0.2),
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
        match *camera_mode {
            CameraMode::Orbit => Transform::from_xyz(0.0, CAMERA_HEIGHT, CAMERA_ORBIT_RADIUS)
                .looking_at(CAMERA_LOOK_AT, Vec3::Y),
            CameraMode::ZoomIn => Transform::from_translation(ZOOM_CAMERA_START)
                .looking_at(ZOOM_CAMERA_LOOK_AT_START, Vec3::Y),
        },
    ));
}

fn drive_export_capture(
    mut commands: Commands,
    camera_mode: Res<CameraMode>,
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
        *transform = match *camera_mode {
            CameraMode::Orbit => {
                let angle = export_state.current_frame as f32 / export_config.frames as f32 * TAU;
                let capture_position = Vec3::new(
                    angle.sin() * CAMERA_ORBIT_RADIUS,
                    CAMERA_HEIGHT,
                    angle.cos() * CAMERA_ORBIT_RADIUS,
                );
                Transform::from_translation(capture_position).looking_at(CAMERA_LOOK_AT, Vec3::Y)
            }
            CameraMode::ZoomIn => {
                let progress = if export_config.frames <= 1 {
                    1.0
                } else {
                    export_state.current_frame as f32 / (export_config.frames - 1) as f32
                };
                let capture_position = ZOOM_CAMERA_START.lerp(ZOOM_CAMERA_END, progress);
                let look_at = ZOOM_CAMERA_LOOK_AT_START.lerp(ZOOM_CAMERA_LOOK_AT_END, progress);
                Transform::from_translation(capture_position).looking_at(look_at, Vec3::Y)
            }
        };
    }

    export_state.current_frame += 1;
}

fn orbit_camera(time: Res<Time>, mut query: Query<(&mut Transform, &mut OrbitCamera)>) {
    for (mut transform, mut orbit) in &mut query {
        orbit.angle += CAMERA_ORBIT_SPEED_RADIANS_PER_SECOND * time.delta_secs();
        let camera_position = Vec3::new(
            orbit.angle.sin() * CAMERA_ORBIT_RADIUS,
            CAMERA_HEIGHT,
            orbit.angle.cos() * CAMERA_ORBIT_RADIUS,
        );
        *transform = Transform::from_translation(camera_position).looking_at(CAMERA_LOOK_AT, Vec3::Y);
    }
}

fn zoom_camera(time: Res<Time>, mut query: Query<(&mut Transform, &mut ZoomCamera)>) {
    for (mut transform, mut zoom) in &mut query {
        zoom.progress = (zoom.progress + time.delta_secs() / ZOOM_DURATION_SECONDS).min(1.0);
        let camera_position = ZOOM_CAMERA_START.lerp(ZOOM_CAMERA_END, zoom.progress);
        let look_at = ZOOM_CAMERA_LOOK_AT_START.lerp(ZOOM_CAMERA_LOOK_AT_END, zoom.progress);
        *transform = Transform::from_translation(camera_position).looking_at(look_at, Vec3::Y);
    }
}

fn animate_terminal_screen(
    time: Res<Time>,
    mut terminal_screen_texture: ResMut<TerminalScreenTexture>,
    terminal_screen_material: Res<TerminalScreenMaterial>,
    mut terminal_screen_renderer: NonSendMut<TerminalScreenRenderer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let updated_image = terminal_screen_renderer.render_to_image(time.delta());
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
            continue;
        }
        commands
            .entity(descendant)
            .insert(MeshMaterial3d(terminal_screen_material.0.clone()));
    }
}

fn build_terminal_screen_renderer() -> TerminalScreenRenderer {
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
        atlas_width,
        atlas_height,
        atlas_template: atlas_template
            .data
            .expect("screen atlas template should contain decoded image data"),
    }
}

fn draw_terminal_screen(frame: &mut Frame, app: &mut TerminalDemoApp) {
    let area = frame.area();
    let frame_index = app.frame_count;
    let flash_on = (frame_index / 8).is_multiple_of(2);
    let background = TuiColor::Black;
    let title_color = TuiColor::Rgb(150, 255, 150);
    let loader_color = TuiColor::Rgb(110, 255, 110);
    let live_color = if flash_on {
        TuiColor::Rgb(255, 240, 170)
    } else {
        TuiColor::Rgb(80, 70, 30)
    };

    frame.render_widget(
        Block::new().style(TuiStyle::default().bg(background)),
        area,
    );

    let frame_area = area.inner(Margin::new(1, 0));
    let block = Block::new()
        .borders(Borders::ALL)
        .style(TuiStyle::default().bg(background))
        .border_style(TuiStyle::default().fg(TERMINAL_BORDER_COLOR).bg(background));
    let inner = block.inner(frame_area);
    frame.render_widget(block, frame_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let spinner = ['|', '/', '-', '\\'][(frame_index as usize) % 4];
    let sweep = (frame_index as usize) % 8;
    let loader_bar: String = (0..8)
        .map(|idx| if idx == sweep { '#' } else { '-' })
        .collect();

    frame.render_widget(
        Paragraph::new("TERMINAL\nCOLLECTIVE")
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(title_color).bg(background)),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!("{spinner}{loader_bar}{spinner}"))
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(loader_color).bg(background))
            .wrap(Wrap { trim: false }),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new("LIVE SOON")
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(live_color).bg(background))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

impl TerminalScreenRenderer {
    fn render_to_image(&mut self, _delta: std::time::Duration) -> Image {
        self.demo_app.on_tick();
        let terminal = &mut self.terminal;
        let demo_app = &mut self.demo_app;
        terminal
            .draw(|frame| {
                draw_terminal_screen(frame, demo_app);
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
