pub mod fruit;
mod game;
mod game_level;
pub mod renderer;
pub mod snake;
#[cfg(test)]
mod tests;

pub use game_level::{GameLevel, Tile, TileType};

#[derive(Debug)]
pub enum GameError {
    GameOver,
    InvalidInternalState,
}

pub trait SnakeBehavior {
    fn put_on(&mut self, level: &mut GameLevel, tail_size: usize) -> Result<(), GameError>;
    fn make_move(&mut self, level: &mut GameLevel) -> Result<(), GameError>;
    fn direction(&self) -> MovementDirection;
    fn set_direction(&mut self, new_direction: MovementDirection) -> Result<(), GameError>;
    /// total snake length (with head)
    fn len(&self) -> usize;
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

pub trait FruitBehavior {
    fn put_on(&mut self, level: &mut GameLevel) -> Result<(), GameError>;
}

pub trait GameDisplay {
    type Output;
    type Error;

    fn render(&self, level: &GameLevel) -> Result<Self::Output, Self::Error>;
}
