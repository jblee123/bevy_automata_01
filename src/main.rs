use bevy::prelude::*;
// use bevy::sprite::MaterialMesh2dBundle;

const GENERATION_PERIOD_HZ: f64 = 8.;
const GENERATION_PERIOD_SEC: f64 = 1. / GENERATION_PERIOD_HZ;

const WIN_BG_COLOR: Color = Color::GRAY;
const CELL_COLOR: Color = Color::BLUE;

const BASE_CELL_SIZE: f32 = 20.;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CellCoord {
    pub x: i32,
    pub y: i32,
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
}

#[derive(Component)]
struct SimState {
    pub last_generation_time_sec: f64,
}

impl SimState {
    pub fn new() -> Self {
        Self {
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

    commands.spawn(Colony {
        cells: vec![
            Cell::new(CellCoord { x: 0, y: 0 }),
            Cell::new(CellCoord { x: 1, y: 0 }),
            Cell::new(CellCoord { x: 3, y: 1 }),
        ],
        new_cells: vec![],
    });
}

fn update_colony(
    mut colony_query: Query<&mut Colony>,
    mut sim_state_query: Query<&mut SimState>,
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
    let next_gen_time = sim_state.last_generation_time_sec + GENERATION_PERIOD_SEC;
    let now_sec = time.elapsed_seconds_f64();
    if now_sec < next_gen_time {
        return;
    }

    sim_state.last_generation_time_sec = now_sec;

    let mut colony = colony.unwrap();

    // Pull out cells and new_cells so we're not working with them in the
    // context of the whole colony struct, which means easier borrow checking.
    let mut cells = Vec::<Cell>::new();
    std::mem::swap(&mut colony.cells, &mut cells);

    let mut new_cells = Vec::<Cell>::new();
    std::mem::swap(&mut colony.new_cells, &mut new_cells);

    new_cells.clear();

    let living_cells = cells.iter().filter(|cell| cell.alive);
    for cell in living_cells {
        new_cells.push(*cell);
        new_cells.last_mut().unwrap().coord.x *= -1;
    }

    // Replace our working vectors back into the colony, but switch cells and
    // new_cells.
    std::mem::swap(&mut colony.cells, &mut new_cells);
    std::mem::swap(&mut colony.new_cells, &mut cells);
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
