use bevy::prelude::*;
// use bevy::sprite::MaterialMesh2dBundle;

const GENERATION_PERIOD_HZ: f64 = 8.;
const GENERATION_PERIOD_SEC: f64 = 1. / GENERATION_PERIOD_HZ;

const WIN_BG_COLOR: Color = Color::GRAY;
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
    pub alive: bool,
}

impl Cell {
    pub fn new(coord: CellCoord) -> Self {
        Self { coord, alive: true }
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
    pub is_paused: bool,
    pub last_generation_time_sec: f64,
}

impl SimState {
    pub fn new() -> Self {
        Self {
            is_paused: true,
            last_generation_time_sec: 0.,
        }
    }
}

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
        .add_systems(Update, update_colony)
        .add_systems(Update, update_display)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SimState::new());

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

fn update_colony(
    mut colony_query: Query<&mut Colony>,
    mut sim_state_query: Query<&mut SimState>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let colony = colony_query.iter_mut().next();
    if colony.is_none() {
        return;
    }

    let sim_state = sim_state_query.iter_mut().next();
    if sim_state.is_none() {
        return;
    }

    let mut sim_state = sim_state.unwrap();

    if keys.just_pressed(KeyCode::Space) {
        sim_state.is_paused = !sim_state.is_paused;
    }

    let step_next = keys.just_pressed(KeyCode::N);

    let next_gen_time = sim_state.last_generation_time_sec + GENERATION_PERIOD_SEC;
    let now_sec = time.elapsed_seconds_f64();
    let run_step = step_next || (now_sec >= next_gen_time && !sim_state.is_paused);

    let mut colony = colony.unwrap();

    if run_step {
        run_next_generation(&mut colony);
        sim_state.last_generation_time_sec = now_sec;
    }
}

fn update_display(
    mut commands: Commands,
    entities: Query<Entity, With<Sprite>>,
    colony_query: Query<&Colony>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }

    for colony in &colony_query {
        let living_cells = colony.cells.iter().filter(|cell| cell.alive);

        for cell in living_cells {
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
        }
    }
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
        if !cell.alive {
            continue;
        }

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
        neighbor_idx2 = neighbor_idx2 + 1;
        while neighbor_idx2 < neighbor_coords.len()
            && (neighbor_coords[neighbor_idx1] == neighbor_coords[neighbor_idx2])
        {
            neighbor_idx2 = neighbor_idx2 + 1;
        }

        // Find the first element in the cells >= the current element in the neighbors
        while ((neighbor_coords[neighbor_idx1] > cells[cell_idx].coord) || !cells[cell_idx].alive)
            && ((cell_idx + 1) < cells.len())
        {
            cell_idx = cell_idx + 1;
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
