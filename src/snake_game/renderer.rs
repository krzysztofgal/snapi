use super::{GameDisplay, GameLevel, TileType};

pub struct GameDisplayToString;

impl GameDisplay for GameDisplayToString {
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
