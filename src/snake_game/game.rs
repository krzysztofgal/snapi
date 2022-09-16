use super::{
    game_level::GameLevel, FruitBehavior, GameDisplay, GameError, MovementDirection, SnakeBehavior,
};
pub struct Game<S, F> {
    snake: S,
    fruit: F,
    level: GameLevel,
}

impl<S: SnakeBehavior, F: FruitBehavior> Game<S, F> {
    pub fn new(level: GameLevel, snake: S, fruit: F) -> Self {
        Self {
            snake,
            fruit,
            level,
        }
    }

    pub fn put_snake(&mut self, tail_size: usize) -> Result<(), GameError> {
        self.snake.put_on(&mut self.level, tail_size)
    }

    pub fn set_snake_direction(
        &mut self,
        new_direction: MovementDirection,
    ) -> Result<(), GameError> {
        self.snake.set_direction(new_direction)
    }

    pub fn level(&self) -> &GameLevel {
        &self.level
    }

    pub fn snake(&self) -> &dyn SnakeBehavior {
        &self.snake
    }

    pub fn render<O, E>(
        &self,
        renderer: &dyn GameDisplay<S, F, Output = O, Error = E>,
    ) -> Result<O, E> {
        renderer.render(self)
    }

    pub fn try_move(&mut self) -> Result<(), GameError> {
        self.snake.make_move(&mut self.level)?;
        self.fruit.put_on(&mut self.level)?;

        Ok(())
    }
}
