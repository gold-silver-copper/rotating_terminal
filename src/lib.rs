use std::f32::consts::FRAC_PI_6;

use bevy::{
    asset::{AssetPlugin, RenderAssetUsages},
    gltf::GltfMaterialName,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    scene::SceneInstanceReady,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Terminal,
    style::{Color as TuiColor, Style as TuiStyle},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use soft_ratatui::{
    EmbeddedGraphics, SoftBackend,
    embedded_graphics_unicodefonts::mono_4x6_atlas,
};
use tachyonfx::{CellFilter, Effect, EffectRenderer, EffectTimer, Interpolation, SimpleRng, fx};

const TERMINAL_ASSET_PATH: &str = "vintage_terminal/scene.gltf";
const CAMERA_ORBIT_SPEED_RADIANS_PER_SECOND: f32 = 0.18;
const CAMERA_ORBIT_RADIUS: f32 = 8.8;
const CAMERA_HEIGHT: f32 = 4.6;
const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -0.2, 0.0);
const ZOOM_CAMERA_START: Vec3 = Vec3::new(0.0, 4.8, 10.5);
const ZOOM_CAMERA_END: Vec3 = Vec3::new(0.0, 1.3, 4.2);
const ZOOM_CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -1.05, 0.22);
const ZOOM_DURATION_SECONDS: f32 = 28.0;
const TERMINAL_SCENE_OFFSET: Vec3 = Vec3::new(0.0, 0.25, 0.0);
const TERMINAL_SCALE: Vec3 = Vec3::splat(1.0);
const HIDDEN_SCENE_NODE_NAMES: &[&str] = &[];
const SCREEN_MATERIAL_NAME: &str = "Material.002";
const SCREEN_COLUMNS: u16 = 14;
const SCREEN_ROWS: u16 = 7;
const SCREEN_ATLAS_X: u32 = 10;
const SCREEN_ATLAS_Y: u32 = 463;
const SCREEN_ATLAS_WIDTH: u32 = 157;
const SCREEN_ATLAS_HEIGHT: u32 = 233;
const TERMINAL_TITLE_COLOR: TuiColor = TuiColor::Rgb(150, 255, 150);
const TERMINAL_BODY_COLOR: TuiColor = TuiColor::Rgb(92, 170, 92);
const TERMINAL_BORDER_COLOR: TuiColor = TuiColor::Rgb(70, 120, 70);

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

struct TerminalScreenRenderer {
    terminal: Terminal<SoftBackend<EmbeddedGraphics>>,
    title_effect: Effect,
    next_seed: u32,
    elapsed: std::time::Duration,
    frame_index: u64,
    atlas_width: u32,
    atlas_height: u32,
    atlas_template: Vec<u8>,
}

pub fn run_rotating_terminal() {
    app(CameraMode::Orbit).run();
}

pub fn run_stationary_terminal() {
    app(CameraMode::ZoomIn).run();
}

fn app(camera_mode: CameraMode) -> App {
    let mut app = App::new();
    app.insert_resource(camera_mode)
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: ".".to_string(),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, zoom_camera, animate_terminal_screen));
    app
}

fn setup(world: &mut World) {
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.85, 0.9, 1.0),
        brightness: 200.0,
        affects_lightmapped_meshes: true,
    });

    let mut renderer = build_terminal_screen_renderer();
    let initial_image = renderer.render_to_image(std::time::Duration::ZERO);
    let image_handle = {
        let mut images = world.resource_mut::<Assets<Image>>();
        images.add(initial_image)
    };
    world.insert_resource(TerminalScreenTexture(image_handle));
    world.insert_non_send_resource(renderer);

    let terminal_scene = {
        let asset_server = world.resource::<AssetServer>();
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(TERMINAL_ASSET_PATH))
    };
    let camera_mode = *world.resource::<CameraMode>();
    let mut commands = world.commands();

    commands
        .spawn((Name::new("Terminal Pivot"), Transform::default()))
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
        Name::new("Key Light"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, -FRAC_PI_6, -FRAC_PI_6)),
    ));

    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 900_000.0,
            range: 30.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 6.0, 5.0),
    ));

    match camera_mode {
        CameraMode::Orbit => {
            commands.spawn((
                Name::new("Orbit Camera"),
                OrbitCamera { angle: 0.0 },
                Camera3d::default(),
                Transform::from_xyz(0.0, CAMERA_HEIGHT, CAMERA_ORBIT_RADIUS)
                    .looking_at(CAMERA_LOOK_AT, Vec3::Y),
            ));
        }
        CameraMode::ZoomIn => {
            commands.spawn((
                Name::new("Zoom Camera"),
                ZoomCamera { progress: 0.0 },
                Camera3d::default(),
                Transform::from_translation(ZOOM_CAMERA_START).looking_at(ZOOM_CAMERA_LOOK_AT, Vec3::Y),
            ));
        }
    }
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
        *transform = Transform::from_translation(camera_position).looking_at(ZOOM_CAMERA_LOOK_AT, Vec3::Y);
    }
}

fn animate_terminal_screen(
    time: Res<Time>,
    terminal_screen_texture: Res<TerminalScreenTexture>,
    mut terminal_screen_renderer: NonSendMut<TerminalScreenRenderer>,
    mut images: ResMut<Assets<Image>>,
) {
    let updated_image = terminal_screen_renderer.render_to_image(time.delta());
    if let Some(image) = images.get_mut(&terminal_screen_texture.0) {
        *image = updated_image;
    }
}

fn configure_terminal_scene_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<&Name>,
    mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    terminal_screen_texture: Res<TerminalScreenTexture>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
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

        let Ok((material_handle, material_name)) = mesh_materials.get(descendant) else {
            continue;
        };
        if material_name.0.as_str() != SCREEN_MATERIAL_NAME {
            continue;
        }
        let Some(material) = asset_materials.get_mut(material_handle.id()) else {
            continue;
        };

        let mut animated_material = material.clone();
        animated_material.base_color = Color::WHITE;
        animated_material.base_color_texture = Some(terminal_screen_texture.0.clone());
        animated_material.emissive_texture = Some(terminal_screen_texture.0.clone());
        animated_material.emissive = LinearRgba::rgb(1.4, 1.4, 1.4);
        animated_material.unlit = true;
        animated_material.cull_mode = None;
        commands
            .entity(descendant)
            .insert(MeshMaterial3d(asset_materials.add(animated_material)));
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
        title_effect: make_title_dissolve_effect(7),
        next_seed: 8,
        elapsed: std::time::Duration::ZERO,
        frame_index: 0,
        atlas_width,
        atlas_height,
        atlas_template: atlas_template
            .data
            .expect("screen atlas template should contain decoded image data"),
    }
}

fn draw_terminal_screen(frame: &mut Frame, elapsed: std::time::Duration, frame_index: u64) {
    let area = frame.area();
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(TuiStyle::default().fg(TERMINAL_BORDER_COLOR));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let spinner = ['|', '/', '-', '\\'][(frame_index as usize) % 4];
    let pulse_word = if (elapsed.as_millis() / 300).is_multiple_of(2) {
        "ONLINE"
    } else {
        "SYNCING"
    };

    frame.render_widget(
        Paragraph::new("TERMINAL\nCOLLECTIVE")
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(TERMINAL_TITLE_COLOR)),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!("{spinner} {pulse_word}"))
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(TERMINAL_BODY_COLOR)),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new("SOFT RATATUI\nSIGNAL LIVE")
            .alignment(Alignment::Center)
            .style(TuiStyle::default().fg(TERMINAL_BODY_COLOR))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

impl TerminalScreenRenderer {
    fn render_to_image(&mut self, delta: std::time::Duration) -> Image {
        self.elapsed += delta;
        self.frame_index = self.frame_index.wrapping_add(1);

        let terminal = &mut self.terminal;
        let title_effect = &mut self.title_effect;
        let elapsed = self.elapsed;
        let frame_index = self.frame_index;
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_terminal_screen(frame, elapsed, frame_index);
                frame.render_effect(title_effect, area, delta.into());
            })
            .expect("soft_ratatui screen should render");

        if !self.title_effect.running() {
            self.title_effect = make_title_dissolve_effect(self.next_seed);
            self.next_seed = self.next_seed.wrapping_add(1);
        }

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

fn make_title_dissolve_effect(seed: u32) -> Effect {
    fx::dissolve(EffectTimer::from_ms(1100, Interpolation::SineInOut))
        .with_rng(SimpleRng::new(seed))
        .with_filter(CellFilter::AllOf(vec![
            CellFilter::Text,
            CellFilter::FgColor(TERMINAL_TITLE_COLOR),
        ]))
}
