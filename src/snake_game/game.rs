use super::{game_level::GameLevel, FruitBehavior, GameDisplay, SnakeBehavior};

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
}
