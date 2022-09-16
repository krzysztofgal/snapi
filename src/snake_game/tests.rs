use super::{GameDisplay, GameLevel};

struct GameDisplaySimplePrint;

impl GameDisplay for GameDisplaySimplePrint {
    type Output = String;
    type Error = std::convert::Infallible;

    fn render(&self, level: &GameLevel) -> Result<Self::Output, Self::Error> {
        use super::TileType;
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
    use super::{snake::SnakeUnbounded, GameError, MovementDirection, SnakeBehavior};

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
