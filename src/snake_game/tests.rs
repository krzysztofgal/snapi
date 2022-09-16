use super::{renderer::GameDisplayToString, Game, GameLevel, NullFruit, NullSnake};

#[test]
fn level_render() {
    let level = GameLevel::new(20, 10);
    let game = Game::new(level, NullSnake, NullFruit);
    let renderer = GameDisplayToString;
    let output = game.render(&renderer).unwrap();
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

    let mut game = Game::new(level, snake, NullFruit);
    let renderer = GameDisplayToString;

    let output = game.render(&renderer).unwrap();
    println!("{output}");

    for s in 0..63 {
        game.try_move().unwrap();
        let output = game.render(&renderer).unwrap();
        println!("Step: {s}");
        println!("{output}");

        match s {
            // go through entire level
            15 => game.set_snake_direction(MovementDirection::Up).unwrap(),
            17 => {
                // test grow
                assert_eq!(game.snake().len(), start_len + 1);
            }
            30 => game.set_snake_direction(MovementDirection::Left).unwrap(),
            45 => game.set_snake_direction(MovementDirection::Down).unwrap(),
            // go to collision
            60 => game.set_snake_direction(MovementDirection::Left).unwrap(),
            61 => game.set_snake_direction(MovementDirection::Up).unwrap(),
            // make collision
            62 => {
                game.set_snake_direction(MovementDirection::Right).unwrap();
                let result = game.try_move();
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
