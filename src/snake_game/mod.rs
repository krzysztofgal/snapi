pub mod fruit;
mod game;
mod game_level;
pub mod renderer;
pub mod snake;
#[cfg(test)]
mod tests;

pub use game::Game;
pub use game_level::{GameLevel, Tile, TileType};

#[derive(Debug)]
pub enum GameError {
    GameOver,
    InvalidInternalState,
    RenderingError,
}

pub trait SnakeBehavior {
    fn put_on(&mut self, level: &mut GameLevel, tail_size: usize) -> Result<(), GameError>;
    fn make_move(&mut self, level: &mut GameLevel) -> Result<(), GameError>;
    fn direction(&self) -> MovementDirection;
    fn set_direction(&mut self, new_direction: MovementDirection) -> Result<(), GameError>;
    /// total snake length (with head)
    fn len(&self) -> usize;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

pub trait GameDisplay<S: SnakeBehavior, F: FruitBehavior> {
    type Output;
    type Error;

    fn render(&self, game: &Game<S, F>) -> Result<Self::Output, Self::Error>;
}

// testing impls
struct NullSnake;
impl SnakeBehavior for NullSnake {
    fn put_on(&mut self, _level: &mut GameLevel, _tail_size: usize) -> Result<(), GameError> {
        Ok(())
    }

    fn make_move(&mut self, _level: &mut GameLevel) -> Result<(), GameError> {
        Ok(())
    }

    fn direction(&self) -> MovementDirection {
        MovementDirection::Right
    }

    fn set_direction(&mut self, _new_direction: MovementDirection) -> Result<(), GameError> {
        Ok(())
    }

    fn len(&self) -> usize {
        0
    }
}

struct NullFruit;
impl FruitBehavior for NullFruit {
    fn put_on(&mut self, _level: &mut GameLevel) -> Result<(), GameError> {
        Ok(())
    }
}
