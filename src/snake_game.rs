use std::collections::VecDeque;

pub struct Game<S> {
    snake: S,
    level: GameLevel,
}

impl<S: SnakeBehavior> Game<S> {
    pub fn new(level: GameLevel, snake: S) -> Self {
        Self { snake, level }
    }
}

pub trait SnakeBehavior {
    fn put_on(&mut self, level: &mut GameLevel, tail_size: usize) -> Result<(), GameError>;
    fn make_move(&mut self, level: &mut GameLevel) -> Result<(), GameError>;
    fn direction(&self) -> MovementDirection;
    fn set_direction(&mut self, new_direction: MovementDirection) -> Result<(), GameError>;
    /// total snake length (with head)
    fn len(&self) -> usize;
}

pub trait FruitBehavior {
    fn put_on(&mut self, level: &mut GameLevel) -> Result<(), GameError>;
}

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

#[derive(Debug)]
pub struct SnakeUnbounded {
    tail: VecDeque<usize>,
    movement_direction: MovementDirection,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
}

impl MovementDirection {
    pub fn is_opposite_to(&self, new_direction: &Self) -> bool {
        use MovementDirection::*;
        matches!(
            (self, new_direction),
            (Left, Right) | (Right, Left) | (Up, Down) | (Down, Up)
        )
    }
}

#[derive(Debug)]
pub enum GameError {
    GameOver,
    InvalidInternalState,
}

enum MovementResult<'a> {
    Ok(&'a Tile),
    GrowOn(&'a Tile),
    SnakeCollision,
    LevelBoundary,
}

impl SnakeUnbounded {
    pub fn new(initial_direction: MovementDirection) -> Self {
        Self {
            tail: VecDeque::new(),
            movement_direction: initial_direction,
        }
    }

    fn grow_on(&mut self, tile: &mut Tile) {
        tile.set_to(TileType::Snake);
        self.tail.push_front(tile.get_index());
    }

    fn try_move_to<'l>(&self, to_tile: &Option<&'l Tile>) -> MovementResult<'l> {
        match to_tile {
            Some(tile) => match tile.tile_type() {
                TileType::Empty => MovementResult::Ok(tile),
                TileType::Fruit => MovementResult::GrowOn(tile),
                TileType::Snake => MovementResult::SnakeCollision,
            },
            None => MovementResult::LevelBoundary,
        }
    }
}

impl SnakeBehavior for SnakeUnbounded {
    fn put_on(&mut self, level: &mut GameLevel, tail_size: usize) -> Result<(), GameError> {
        use MovementDirection::*;

        if tail_size < 1 {
            return Err(GameError::InvalidInternalState);
        }

        let d = level.level_coordinates();
        let x_center = d.x_max / 2;
        let y_center = d.y_max / 2;

        // get head tile
        let head = level
            .get_tile_on(x_center, y_center)
            .ok_or(GameError::InvalidInternalState)?;

        let get_tail_index_on = |on_tile: &Option<&Tile>| -> Result<usize, GameError> {
            match on_tile {
                Some(tile) => Ok(tile.get_index()),
                None => Err(GameError::InvalidInternalState),
            }
        };

        // set on which tiles snake lives
        let mut tail = Vec::new();
        tail.push(head.get_index());
        let mut siblings = level.tile_siblings(head);
        for _ in 0..tail_size {
            // tail tile needs to be on opposite direction of movement
            let tail_tile_index = match self.movement_direction {
                Up => get_tail_index_on(&siblings.down),
                Down => get_tail_index_on(&siblings.up),
                Left => get_tail_index_on(&siblings.right),
                Right => get_tail_index_on(&siblings.left),
            }?;

            tail.push(tail_tile_index);
            let tile = level.get_tile(tail_tile_index).unwrap();
            siblings = level.tile_siblings(tile);
        }

        // put snake on selected tiles
        for tile_index in tail.iter() {
            let tile = level.get_tile_mut(*tile_index).unwrap();
            tile.set_to(TileType::Snake);
        }

        // save tail tiles to track movement
        self.tail = tail.into_iter().collect::<VecDeque<usize>>();

        Ok(())
    }

    fn make_move(&mut self, level: &mut GameLevel) -> Result<(), GameError> {
        use MovementDirection::*;

        let head_index = self.tail.front().ok_or(GameError::InvalidInternalState)?;
        let head = level
            .get_tile(*head_index)
            .ok_or(GameError::InvalidInternalState)?;
        let siblings = level.tile_siblings(head);

        let movement_result = match self.movement_direction {
            Up => self.try_move_to(&siblings.up),
            Down => self.try_move_to(&siblings.down),
            Left => self.try_move_to(&siblings.left),
            Right => self.try_move_to(&siblings.right),
        };

        // normal movement - true, set to false on snake grow.
        let mut delete_tail_end = true;

        match movement_result {
            MovementResult::Ok(tile) => {
                let tile = level
                    .get_tile_mut(tile.get_index())
                    .ok_or(GameError::InvalidInternalState)?;
                tile.set_to(TileType::Snake);
                self.tail.push_front(tile.get_index());
            }
            MovementResult::GrowOn(tile) => {
                let tile = level
                    .get_tile_mut(tile.get_index())
                    .ok_or(GameError::InvalidInternalState)?;
                self.grow_on(tile);
                delete_tail_end = false;
            }
            MovementResult::SnakeCollision => return Err(GameError::GameOver),
            // "globe" level behavior logic
            MovementResult::LevelBoundary => {
                let head_pos = level.get_tile_position(head);
                let level_dim = level.level_coordinates();

                let next_tile = match self.movement_direction {
                    Up => level.get_tile_mut_on(head_pos.x, level_dim.y_max),
                    Down => level.get_tile_mut_on(head_pos.x, level_dim.y_min),
                    Left => level.get_tile_mut_on(level_dim.x_max, head_pos.y),
                    Right => level.get_tile_mut_on(level_dim.x_min, head_pos.y),
                }
                .ok_or(GameError::InvalidInternalState)?;

                next_tile.set_to(TileType::Snake);
                self.tail.push_front(next_tile.get_index())
            }
        }

        // delete last segment
        if delete_tail_end {
            let tail_end_index = self
                .tail
                .pop_back()
                .ok_or(GameError::InvalidInternalState)?;
            let tail_end = level
                .get_tile_mut(tail_end_index)
                .ok_or(GameError::InvalidInternalState)?;
            tail_end.set_to(TileType::Empty);
        }

        Ok(())
    }

    fn direction(&self) -> MovementDirection {
        self.movement_direction
    }

    fn set_direction(&mut self, new_direction: MovementDirection) -> Result<(), GameError> {
        if self.movement_direction.is_opposite_to(&new_direction) {
            return Err(GameError::GameOver);
        }
        self.movement_direction = new_direction;
        Ok(())
    }

    fn len(&self) -> usize {
        self.tail.len()
    }
}

pub struct FruitRandomLimited {
    limit: usize,
    chance: f64,
}

impl FruitRandomLimited {
    pub fn new(limit: usize, chance: f64) -> Self {
        if !(0.01..=1.0).contains(&chance) {
            panic!("Invalid configuration: chance must be in range of 0.01 - 1.00")
        }

        Self { limit, chance }
    }
}

impl FruitBehavior for FruitRandomLimited {
    fn put_on(&mut self, level: &mut GameLevel) -> Result<(), GameError> {
        use rand::prelude::*;

        // count fruits on level
        let count_fruits = level
            .level()
            .iter()
            .filter(|t| matches!(t.tile_type(), TileType::Fruit))
            .count();

        // if under limit then draw a chance to put one fruit
        if count_fruits < self.limit {
            let draw = thread_rng().gen_range(0.01..1.0);
            if self.chance >= draw {
                let empty_tiles = level
                    .level()
                    .iter()
                    .filter(|t| matches!(t.tile_type(), TileType::Empty))
                    .collect::<Vec<_>>();

                // there is nowhere to put fruit
                if empty_tiles.is_empty() {
                    return Ok(());
                }

                // put fruit on empty field
                let random_index = thread_rng().gen_range(0..empty_tiles.len());
                let tile = level.get_tile_mut(random_index).unwrap();
                tile.set_to(TileType::Fruit);
            }
        }

        Ok(())
    }
}

pub trait GameDisplay {
    type Output;
    type Error;

    fn render(&self, level: &GameLevel) -> Result<Self::Output, Self::Error>;
}

impl GameLevel {
    fn new(width: usize, height: usize) -> Self {
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

    pub fn get_tile_on(&self, x: usize, y: usize) -> Option<&Tile> {
        self.level.get(y * self.width + x)
    }

    pub fn get_tile_mut_on(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.level.get_mut(y * self.width + x)
    }

    pub fn tile_siblings(&self, tile: &Tile) -> TileSiblings {
        // hide dimensional logic here
        let d = self.level_coordinates();
        let TileXY { x, y } = self.get_tile_position(tile);
        let up = if y > 0 {
            self.get_tile_on(x, y - 1)
        } else {
            None
        };
        let down = if y < d.y_max {
            self.get_tile_on(x, y + 1)
        } else {
            None
        };
        let left = if x > 0 {
            self.get_tile_on(x - 1, y)
        } else {
            None
        };
        let right = if x < d.x_max {
            self.get_tile_on(x + 1, y)
        } else {
            None
        };

        TileSiblings {
            up,
            down,
            left,
            right,
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

pub struct TileSiblings<'l> {
    pub up: Option<&'l Tile>,
    pub down: Option<&'l Tile>,
    pub left: Option<&'l Tile>,
    pub right: Option<&'l Tile>,
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
    use super::{GameDisplay, GameLevel, SnakeUnbounded, TileType};
    use crate::snake_game::SnakeBehavior;

    struct GameDisplaySimplePrint;

    impl GameDisplay for GameDisplaySimplePrint {
        type Output = String;
        type Error = std::convert::Infallible;

        fn render(&self, level: &GameLevel) -> Result<Self::Output, Self::Error> {
            use std::fmt::Write;

            let level_box = level.level_coordinates();
            let dimensions = level.level_dimensions();
            let tiles = level.level();

            let mut output = String::with_capacity(tiles.len());

            // vertical "wall"
            let v_wall = (level_box.x_min..=level_box.x_max)
                .into_iter()
                .map(|_| '#')
                .collect::<String>();

            // top wall
            let _ = write!(output, "#{}#\n\r#", &v_wall);
            for (index, tile) in tiles.iter().enumerate() {
                let char = match tile.tile_type() {
                    TileType::Empty => ' ',
                    TileType::Fruit => '@',
                    TileType::Snake => '\u{2588}',
                };

                if index > 0 && index % dimensions.width == 0 {
                    // wall at end of line and start of new line
                    let _ = write!(output, "#\n\r#");
                }

                let _ = write!(output, "{}", char);
            }
            // bottom wall
            let _ = write!(output, "#\n\r#{}#", &v_wall);

            Ok(output)
        }
    }

    #[test]
    fn level_tile_coordinates() {
        let mut level = GameLevel::new(20, 10);
        let center_by_pos = level.get_tile_mut_on(10, 5).unwrap().get_index();
        let center_by_index = level.get_tile(110).unwrap().get_index();

        assert_eq!(center_by_index, center_by_pos)
    }

    #[test]
    fn level_render() {
        let level = GameLevel::new(20, 10);
        let output = GameDisplaySimplePrint.render(&level).unwrap();
        println!("{output}");

        let expected = "######################\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              #                    #\n\r\
                              ######################";

        assert_eq!(expected, output)
    }

    #[test]
    fn snake_movement_and_grow() {
        use super::GameError;
        use super::MovementDirection;
        use super::SnakeBehavior;

        let mut level = GameLevel::new(20, 10);
        let mut snake = SnakeUnbounded::new(MovementDirection::Right);
        level.put_fruit(5, 8);
        level.put_fruit(5, 2);
        snake.put_on(&mut level, 3).unwrap();

        let start_len = snake.len();

        let output = GameDisplaySimplePrint.render(&level).unwrap();
        println!("{output}");

        for s in 0..63 {
            snake.make_move(&mut level).unwrap();
            let output = GameDisplaySimplePrint.render(&level).unwrap();
            println!("Step: {s}");
            println!("{output}");

            match s {
                // go through entire level
                15 => snake.set_direction(MovementDirection::Up).unwrap(),
                17 => {
                    // test grow
                    assert_eq!(snake.len(), start_len + 1);
                }
                30 => snake.set_direction(MovementDirection::Left).unwrap(),
                45 => snake.set_direction(MovementDirection::Down).unwrap(),
                // go to collision
                60 => snake.set_direction(MovementDirection::Left).unwrap(),
                61 => snake.set_direction(MovementDirection::Up).unwrap(),
                // make collision
                62 => {
                    snake.set_direction(MovementDirection::Right).unwrap();
                    let result = snake.make_move(&mut level);
                    let mut game_over = false;
                    if let Err(GameError::GameOver) = result {
                        game_over = true;
                    }
                    assert!(game_over);
                    break;
                }
                _ => (),
            }
        }
    }
}
