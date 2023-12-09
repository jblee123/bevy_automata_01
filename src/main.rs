use bevy::input::mouse::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
// use bevy::sprite::MaterialMesh2dBundle;

const DEFAULT_GENERATION_RATE_HZ: f64 = 8.;
const MIN_GENERATION_RATE_HZ: f64 = 1.;
const MAX_GENERATION_RATE_HZ: f64 = 32.;

const WIN_BG_COLOR: Color = Color::SILVER;
const CELL_COLOR: Color = Color::BLUE;

const BASE_CELL_SIZE: f32 = 20.;

const BIRTH_RULES: [bool; 9] = [false, false, false, true, false, false, false, false, false];
const SURVIVAL_RULES: [bool; 9] = [false, false, true, true, false, false, false, false, false];

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CellCoord {
    pub x: i32,
    pub y: i32,
}

impl CellCoord {
    const MIN_X: i32 = i32::MIN;
    const MAX_X: i32 = i32::MAX;
    const MIN_Y: i32 = i32::MIN;
    const MAX_Y: i32 = i32::MAX;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Cell {
    pub coord: CellCoord,
}

impl Cell {
    pub fn new(coord: CellCoord) -> Self {
        Self { coord }
    }
}

#[derive(Component)]
struct Colony {
    pub cells: Vec<Cell>,
    pub new_cells: Vec<Cell>,
    pub neighbor_coords: Vec<CellCoord>,
}

impl Colony {
    pub fn new() -> Self {
        Self {
            cells: vec![],
            new_cells: vec![],
            neighbor_coords: vec![],
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
struct InfoLabelMarker;

fn main() {
    App::new()
        .insert_resource(ClearColor(WIN_BG_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Automata 01".into(),

                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keyboard)
        .add_systems(Update, update_camera)
        .add_systems(Update, update_colony)
        .add_systems(Update, update_display)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCameraMarker));

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

    let mut colony = Colony::new();
    colony.cells = vec![
        Cell::new(CellCoord { x: 0, y: 0 }),
        Cell::new(CellCoord { x: 1, y: 0 }),
        Cell::new(CellCoord { x: 4, y: 0 }),
        Cell::new(CellCoord { x: 5, y: 0 }),
        Cell::new(CellCoord { x: 6, y: 0 }),
        Cell::new(CellCoord { x: 3, y: 1 }),
        Cell::new(CellCoord { x: 1, y: 2 }),
    ];
    colony.cells.sort();
    commands.spawn(colony);
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
    mut colony_query: Query<&mut Colony>,
    mut sim_state_query: Query<&mut SimState>,
    time: Res<Time>,
) {
    let mut colony = colony_query.iter_mut().next().unwrap();

    let mut sim_state = sim_state_query.iter_mut().next().unwrap();

    let update_period_sec = 1. / sim_state.rate_hz;
    let next_gen_time = sim_state.last_generation_time_sec + update_period_sec;
    let now_sec = time.elapsed_seconds_f64();
    let run_step = sim_state.do_step || (now_sec >= next_gen_time && !sim_state.is_paused);

    if run_step {
        run_next_generation(&mut colony);
        sim_state.last_generation_time_sec = now_sec;
        sim_state.do_step = false;
        sim_state.generation += 1;
    }
}

fn update_display(
    mut commands: Commands,
    sprites_query: Query<Entity, With<Sprite>>,
    mut info_label_query: Query<&mut Text, With<InfoLabelMarker>>,
    colony_query: Query<&Colony>,
    sim_state_query: Query<&SimState>,
) {
    for sprites in &sprites_query {
        commands.entity(sprites).despawn();
    }

    let colony = colony_query.single();
    let mut num_cells = 0;

    for cell in &colony.cells {
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

fn run_next_generation(colony: &mut Colony) {
    // Pull out cells and new_cells so we're not working with them in the
    // context of the whole colony struct, which means easier borrow checking.
    let mut cells = std::mem::take(&mut colony.cells);
    let mut new_cells = std::mem::take(&mut colony.new_cells);
    let mut neighbor_coords = std::mem::take(&mut colony.neighbor_coords);

    new_cells.clear();
    new_cells.reserve(cells.len() * 2);

    const NEIGHBORS_PER_CELL: usize = 8;
    neighbor_coords.clear();
    neighbor_coords.reserve(cells.len() * NEIGHBORS_PER_CELL);

    for cell in &cells {
        let is_not_min_x = cell.coord.x != CellCoord::MIN_X;
        let is_not_max_x = cell.coord.x != CellCoord::MAX_X;
        let is_not_min_y = cell.coord.y != CellCoord::MIN_Y;
        let is_not_max_y = cell.coord.y != CellCoord::MAX_Y;

        if is_not_min_x && is_not_min_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x - 1,
                y: cell.coord.y - 1,
            });
        }
        if is_not_min_x {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x - 1,
                y: cell.coord.y,
            });
        }
        if is_not_min_x && is_not_max_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x - 1,
                y: cell.coord.y + 1,
            });
        }

        if is_not_min_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x,
                y: cell.coord.y - 1,
            });
        }
        if is_not_max_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x,
                y: cell.coord.y + 1,
            });
        }

        if is_not_max_x && is_not_min_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x + 1,
                y: cell.coord.y - 1,
            });
        }
        if is_not_max_x {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x + 1,
                y: cell.coord.y,
            });
        }
        if is_not_max_x && is_not_max_y {
            neighbor_coords.push(CellCoord {
                x: cell.coord.x + 1,
                y: cell.coord.y + 1,
            });
        }
    }

    neighbor_coords.sort();

    let mut cell_idx = 0;
    let mut neighbor_idx2 = 0;
    while neighbor_idx2 < neighbor_coords.len() {
        // Skip elements that are the same in neighbors list.
        let neighbor_idx1 = neighbor_idx2;
        neighbor_idx2 += 1;
        while neighbor_idx2 < neighbor_coords.len()
            && (neighbor_coords[neighbor_idx1] == neighbor_coords[neighbor_idx2])
        {
            neighbor_idx2 += 1;
        }

        // Find the first element in the cells >= the current element in the neighbors
        while (neighbor_coords[neighbor_idx1] > cells[cell_idx].coord)
            && ((cell_idx + 1) < cells.len())
        {
            cell_idx += 1;
        }

        let num_neighbors = neighbor_idx2 - neighbor_idx1;
        let was_alive = neighbor_coords[neighbor_idx1] == cells[cell_idx].coord;
        let is_alive = if was_alive {
            SURVIVAL_RULES[num_neighbors]
        } else {
            BIRTH_RULES[num_neighbors]
        };
        if is_alive {
            new_cells.push(Cell::new(neighbor_coords[neighbor_idx1]));
        }
    }

    // Replace our working vectors back into the colony, but switch cells and
    // new_cells.
    std::mem::swap(&mut colony.cells, &mut new_cells);
    std::mem::swap(&mut colony.new_cells, &mut cells);
    std::mem::swap(&mut colony.neighbor_coords, &mut neighbor_coords);
}
