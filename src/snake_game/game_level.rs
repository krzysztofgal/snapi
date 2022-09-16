pub struct GameLevel {
    height: usize,
    width: usize,
    level: Vec<Tile>,
}

#[derive(Copy, Clone, Default)]
pub struct Tile {
    index: usize,
    r#type: TileType,
}

impl Tile {
    #[inline]
    pub fn get_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn tile_type(&self) -> TileType {
        self.r#type
    }

    #[inline]
    pub fn set_to(&mut self, tile_type: TileType) {
        self.r#type = tile_type;
    }
}

#[derive(Copy, Clone, Default)]
pub enum TileType {
    #[default]
    Empty,
    Snake,
    Fruit,
}

impl GameLevel {
    pub fn new(width: usize, height: usize) -> Self {
        let size = height * width;
        let level = (0..size)
            .into_iter()
            .map(|i| Tile {
                index: i,
                ..Default::default()
            })
            .collect();

        Self {
            height,
            width,
            level,
        }
    }

    pub fn level_coordinates(&self) -> LevelCoordinates {
        // max -1 because counting from 0
        LevelCoordinates {
            x_min: 0,
            y_min: 0,
            x_max: self.width - 1,
            y_max: self.height - 1,
        }
    }

    pub fn level_dimensions(&self) -> LevelDimensions {
        LevelDimensions {
            width: self.width,
            height: self.height,
        }
    }

    pub fn level(&self) -> &[Tile] {
        self.level.as_slice()
    }

    pub fn get_tile_position(&self, tile: &Tile) -> TileXY {
        let i = tile.get_index();

        // usize division will "floor" value
        let y = i / self.width;
        let x = i - (y * self.width);

        TileXY { x, y }
    }

    pub fn get_tile(&self, index: usize) -> Option<&Tile> {
        self.level.get(index)
    }

    pub fn get_tile_mut(&mut self, index: usize) -> Option<&mut Tile> {
        self.level.get_mut(index)
    }

    fn is_tile_in_level_bounds(&self, x: usize, y: usize) -> bool {
        // hide dimensional logic here
        let d = self.level_coordinates();
        x <= d.x_max && y <= d.y_max
    }

    pub fn get_tile_on(&self, x: usize, y: usize) -> Option<&Tile> {
        if self.is_tile_in_level_bounds(x, y) {
            self.level.get(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_tile_mut_on(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if self.is_tile_in_level_bounds(x, y) {
            self.level.get_mut(y * self.width + x)
        } else {
            None
        }
    }

    pub fn tile_sibling(&self, tile: &Tile, on_position: SiblingPosition) -> Option<&Tile> {
        use SiblingPosition::*;
        let TileXY { x, y } = self.get_tile_position(tile);

        match on_position {
            Up => {
                if y > 0 {
                    self.get_tile_on(x, y - 1)
                } else {
                    None
                }
            }
            Down => self.get_tile_on(x, y + 1),
            Left => {
                if x > 0 {
                    self.get_tile_on(x - 1, y)
                } else {
                    None
                }
            }
            Right => self.get_tile_on(x + 1, y),
        }
    }

    // put fruit on arbitrary position
    // may fail silently
    pub fn put_fruit(&mut self, x: usize, y: usize) {
        if let Some(tile) = self.get_tile_mut_on(x, y) {
            if matches!(tile.tile_type(), TileType::Empty) {
                tile.set_to(TileType::Fruit);
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum SiblingPosition {
    Up,
    Down,
    Left,
    Right,
}

pub struct TileXY {
    pub x: usize,
    pub y: usize,
}

pub struct LevelDimensions {
    pub width: usize,
    pub height: usize,
}

pub struct LevelCoordinates {
    pub x_min: usize,
    pub y_min: usize,
    pub x_max: usize,
    pub y_max: usize,
}

#[cfg(test)]
mod tests {
    use super::GameLevel;

    #[test]
    fn level_tile_coordinates() {
        let mut level = GameLevel::new(20, 10);
        let center_by_pos = level.get_tile_mut_on(10, 5).unwrap().get_index();
        let center_by_index = level.get_tile(110).unwrap().get_index();

        assert_eq!(center_by_index, center_by_pos)
    }
}
