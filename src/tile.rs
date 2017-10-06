use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
    pub explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
        }
    }
}

/// Abstraction over simple vector of tiles.
pub struct TileMap {
    data: Vec<Tile>,
    map_width: i32,
}

impl TileMap {
    pub fn from_data(data: Vec<Tile>, map_width: i32) -> TileMap {
        TileMap {
            data: data,
            map_width: map_width,
        }
    }
}

impl Index<(i32, i32)> for TileMap {
    type Output = Tile;

    fn index(&self, (x, y): (i32, i32)) -> &Tile {
        &self.data[(x + y * self.map_width) as usize]
    }
}

impl IndexMut<(i32, i32)> for TileMap {
    fn index_mut<'a>(&'a mut self, (x, y): (i32, i32)) -> &'a mut Tile {
        &mut self.data[(x + y * self.map_width) as usize]
    }
}
