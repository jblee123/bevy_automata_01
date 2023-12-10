pub mod conway;
pub mod line_color_material;
pub mod mesh;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::input::mouse::*;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::RenderLayers;
use bevy::sprite::*;
use bevy::window::PrimaryWindow;

use line_color_material::*;
use mesh::*;

const DEFAULT_GENERATION_RATE_HZ: f64 = 8.;
const MIN_GENERATION_RATE_HZ: f64 = 1.;
const MAX_GENERATION_RATE_HZ: f64 = 32.;

const WIN_BG_COLOR: Color = Color::SILVER;
const CELL_COLOR: Color = Color::BLUE;

const BASE_CELL_SIZE: f32 = 20.;

#[derive(Component)]
struct ColonyComponent {
    pub colony: conway::Colony,
}

impl ColonyComponent {
    pub fn new() -> Self {
        Self {
            colony: conway::Colony::new(),
        }
    }
}

#[derive(Component)]
struct SimState {
    pub rate_hz: f64,
    pub last_generation_time_sec: f64,
    pub is_paused: bool,
    pub do_step: bool,
    pub generation: usize,
}

impl SimState {
    pub fn new() -> Self {
        Self {
            last_generation_time_sec: 0.,
            rate_hz: DEFAULT_GENERATION_RATE_HZ,
            is_paused: true,
            do_step: false,
            generation: 0,
        }
    }
}

#[derive(Component)]
struct GuiState {
    pub drag_start: Vec2,
    pub drag_offset: Vec2,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            drag_start: Vec2::default(),
            drag_offset: Vec2::default(),
        }
    }
}

#[derive(Component)]
struct MainCameraMarker;

#[derive(Component)]
struct DebugCameraMarker;

#[derive(Component)]
struct InfoLabelMarker;

fn main() {
    App::new()
        .insert_resource(ClearColor(WIN_BG_COLOR))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Automata 01".into(),

                    ..default()
                }),
                ..default()
            }),
            Material2dPlugin::<LineColorMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keyboard)
        .add_systems(Update, update_camera)
        .add_systems(Update, update_colony)
        .add_systems(Update, update_display)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), MainCameraMarker));
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                // no "background color", we need to see the main camera's output
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera: Camera {
                // renders after / on top of the main camera
                order: 100,
                ..default()
            },
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scaling_mode: ScalingMode::Fixed {
                    width: 1.,
                    height: 1.,
                },
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        DebugCameraMarker,
    ));

    commands.spawn(SimState::new());
    commands.spawn(GuiState::new());

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 24.0,
                color: Color::BLACK,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        InfoLabelMarker,
    ));

    let mut colony_comp = ColonyComponent::new();
    colony_comp.colony.cells = vec![
        conway::Cell::new(conway::CellCoord { x: 0, y: 0 }),
        conway::Cell::new(conway::CellCoord { x: 1, y: 0 }),
        conway::Cell::new(conway::CellCoord { x: 4, y: 0 }),
        conway::Cell::new(conway::CellCoord { x: 5, y: 0 }),
        conway::Cell::new(conway::CellCoord { x: 6, y: 0 }),
        conway::Cell::new(conway::CellCoord { x: 3, y: 1 }),
        conway::Cell::new(conway::CellCoord { x: 1, y: 2 }),
    ];
    colony_comp.colony.cells.sort();
    commands.spawn(colony_comp);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(LineList {
                    lines: vec![
                        (Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, 0.0)),
                        (Vec3::new(-0.5, 0.5, 0.0), Vec3::new(0.5, -0.5, 0.0)),
                    ],
                }))
                .into(),
            material: line_materials.add(LineColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(Vec3::new(0., 0., 2.)),
            ..default()
        },
        RenderLayers::layer(1),
    ));
}

fn handle_keyboard(keys: Res<Input<KeyCode>>, mut sim_state_query: Query<&mut SimState>) {
    let mut sim_state = sim_state_query.iter_mut().next().unwrap();

    if keys.just_pressed(KeyCode::Space) {
        sim_state.is_paused = !sim_state.is_paused;
    }

    if keys.just_pressed(KeyCode::Minus) {
        sim_state.rate_hz /= 2.;
        if sim_state.rate_hz < MIN_GENERATION_RATE_HZ {
            sim_state.rate_hz = MIN_GENERATION_RATE_HZ;
        }
    }
    if keys.just_pressed(KeyCode::Equals) {
        sim_state.rate_hz *= 2.;
        if sim_state.rate_hz > MAX_GENERATION_RATE_HZ {
            sim_state.rate_hz = MAX_GENERATION_RATE_HZ;
        }
    }

    sim_state.do_step = keys.just_pressed(KeyCode::N);
}

fn update_camera(
    mut gui_state_query: Query<&mut GuiState>,
    mut camera_query: Query<(&mut OrthographicProjection, &mut Transform), With<MainCameraMarker>>,
    windows_query: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let (mut projection, mut transform) = camera_query.single_mut();

    let win = windows_query.single();
    let mut gui_state = gui_state_query.single_mut();

    let mouse_pos = win.cursor_position();
    if mouse_pos.is_none() {
        return;
    }

    let mouse_pos = mouse_pos.unwrap();

    if mouse_buttons.just_pressed(MouseButton::Left) {
        gui_state.drag_start = mouse_pos;
        gui_state.drag_offset = Vec2::ZERO;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        for _ in motion_evr.read() {
            let new_offset = mouse_pos - gui_state.drag_start;
            let delta = new_offset - gui_state.drag_offset;

            transform.translation.x -= delta.x * projection.scale;
            transform.translation.y += delta.y * projection.scale;
            gui_state.drag_offset = new_offset;
        }
    }

    const MAX_SCALE: f32 = 16.0; // 2^4
    const MIN_SCALE: f32 = 0.0625; // 1/(2^4)

    let start_scale = projection.scale;

    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line | MouseScrollUnit::Pixel => {
                // Really should handle these separately...
                if ev.y > 0. {
                    // zoom in
                    projection.scale /= 2.;
                    if projection.scale < MIN_SCALE {
                        projection.scale = MIN_SCALE;
                    }
                } else {
                    // zoom out
                    projection.scale *= 2.;
                    if projection.scale > MAX_SCALE {
                        projection.scale = MAX_SCALE;
                    }
                }
            }
        }
    }

    // We want the point under the mouse cursor to stay where it is, so that
    // means moving the camera. This will shift the camera to the mouse
    // cursor (taking the old scale into account) and then shift it back in
    // the opposite direction a distance based on the same number of pixels
    // but at the new scale.
    let win_center = Vec2::new(win.width(), win.height()) * 0.5;
    let mut pix_offset = mouse_pos - win_center;
    pix_offset.y = -pix_offset.y; // screen coords to space coords
    let cam_offset = pix_offset * (start_scale - projection.scale);
    transform.translation.x += cam_offset.x;
    transform.translation.y += cam_offset.y;
}

fn update_colony(
    mut colony_query: Query<&mut ColonyComponent>,
    mut sim_state_query: Query<&mut SimState>,
    time: Res<Time>,
) {
    let mut colony_comp = colony_query.iter_mut().next().unwrap();

    let mut sim_state = sim_state_query.iter_mut().next().unwrap();

    let update_period_sec = 1. / sim_state.rate_hz;
    let next_gen_time = sim_state.last_generation_time_sec + update_period_sec;
    let now_sec = time.elapsed_seconds_f64();
    let run_step = sim_state.do_step || (now_sec >= next_gen_time && !sim_state.is_paused);

    if run_step {
        conway::run_next_generation(&mut colony_comp.colony);
        sim_state.last_generation_time_sec = now_sec;
        sim_state.do_step = false;
        sim_state.generation += 1;
    }
}

fn update_display(
    mut commands: Commands,
    sprites_query: Query<Entity, With<Sprite>>,
    mut info_label_query: Query<&mut Text, With<InfoLabelMarker>>,
    colony_query: Query<&ColonyComponent>,
    sim_state_query: Query<&SimState>,
) {
    for sprites in &sprites_query {
        commands.entity(sprites).despawn();
    }

    let colony_comp = colony_query.single();
    let mut num_cells = 0;

    for cell in &colony_comp.colony.cells {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: CELL_COLOR,
                custom_size: Some(Vec2::new(BASE_CELL_SIZE, BASE_CELL_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                BASE_CELL_SIZE * cell.coord.x as f32,
                BASE_CELL_SIZE * cell.coord.y as f32,
                0.,
            )),
            ..default()
        });

        num_cells += 1;
    }

    let sim_state = sim_state_query.single();
    let mut info_label = info_label_query.single_mut();
    info_label.sections[0].value = format!(
        "Generation: {0}\nCell count: {1}",
        sim_state.generation, num_cells
    )
    .into();
}
