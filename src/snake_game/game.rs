use super::{
    game_level::GameLevel, FruitBehavior, GameDisplay, GameError, MovementDirection, SnakeBehavior,
};
pub struct Game<S, F, R> {
    snake: S,
    fruit: F,
    renderer: R,
    level: GameLevel,
}

impl<S: SnakeBehavior, F: FruitBehavior, R: GameDisplay> Game<S, F, R> {
    pub fn new(renderer: R, level: GameLevel, snake: S, fruit: F) -> Self {
        Self {
            snake,
            fruit,
            renderer,
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

    pub fn render(&self) -> Result<R::Output, R::Error> {
        self.renderer.render(&self.level)
    }

    pub fn try_move(&mut self) -> Result<(), GameError> {
        self.snake.make_move(&mut self.level)?;
        self.fruit.put_on(&mut self.level)?;

        Ok(())
    }
}
