const BIRTH_RULES: [bool; 9] = [false, false, false, true, false, false, false, false, false];
const SURVIVAL_RULES: [bool; 9] = [false, false, true, true, false, false, false, false, false];

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct CellCoord {
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
pub struct Cell {
    pub coord: CellCoord,
}

impl Cell {
    pub fn new(coord: CellCoord) -> Self {
        Self { coord }
    }
}

pub struct Colony {
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

pub fn run_next_generation(colony: &mut Colony) {
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
