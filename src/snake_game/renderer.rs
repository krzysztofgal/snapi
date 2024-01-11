use super::{FruitBehavior, Game, GameDisplay, GameError, SnakeBehavior, TileType};

pub struct GameDisplayToString;

impl<S: SnakeBehavior, F: FruitBehavior> GameDisplay<S, F> for GameDisplayToString {
    type Output = String;
    type Error = GameError;

    fn render(&self, game: &Game<S, F>) -> Result<Self::Output, Self::Error> {
        use std::fmt::Write;

        let level = game.level();
        let level_box = level.level_coordinates();
        let dimensions = level.level_dimensions();
        let tiles = level.level();

        let mut output = String::with_capacity(tiles.len());

        // main part of horizontal "wall"
        let v_wall = (level_box.x_min..=level_box.x_max)
            .into_iter()
            .map(|_| '#')
            .collect::<String>();

        // top wall
        write!(output, "#{}#\n\r#", &v_wall).map_err(|_| GameError::RenderingError)?;
        for (index, tile) in tiles.iter().enumerate() {
            let char = match tile.tile_type() {
                TileType::Empty => ' ',
                TileType::Fruit => '@',
                TileType::Snake => '\u{2588}',
            };

            if index > 0 && index % dimensions.width == 0 {
                // wall at end of line and start of new line
                write!(output, "#\n\r#").map_err(|_| GameError::RenderingError)?;
            }

            write!(output, "{}", char).map_err(|_| GameError::RenderingError)?;
        }
        // bottom wall
        write!(output, "#\n\r#{}#", &v_wall).map_err(|_| GameError::RenderingError)?;

        Ok(output)
    }
}
