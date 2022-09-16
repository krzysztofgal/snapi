mod snake_game;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use snake_game::MovementDirection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

static LISTEN_ADDR: &str = "0.0.0.0:3000";
static FRAME_TIME: std::time::Duration = std::time::Duration::from_secs(3);

#[derive(Default)]
struct AppState {
    level_display: Arc<Mutex<String>>,
    selected_moves: Arc<Mutex<Vec<MovementDirection>>>,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState::default());

    let thread_app_state = Arc::clone(&app_state);
    std::thread::spawn(move || loop {
        let _ = game_loop(thread_app_state.as_ref());
    });

    let app = Router::new()
        .route("/snake", get(handle_snake_display))
        .route("/snake/:direction", post(handle_snake_direction))
        .layer(Extension(app_state));

    println!("Game server is running at: {LISTEN_ADDR}");
    axum::Server::bind(&LISTEN_ADDR.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_snake_display(Extension(app): Extension<Arc<AppState>>) -> impl IntoResponse {
    let level_display = app.level_display.lock().await;
    level_display.to_owned()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Left,
    Right,
    Bottom,
    Top,
}

async fn handle_snake_direction(
    Extension(app): Extension<Arc<AppState>>,
    Path(direction_command): Path<Direction>,
) -> impl IntoResponse {
    let mov = match direction_command {
        Direction::Left => MovementDirection::Left,
        Direction::Right => MovementDirection::Right,
        Direction::Bottom => MovementDirection::Down,
        Direction::Top => MovementDirection::Up,
    };

    let mut moves = app.selected_moves.lock().await;
    moves.push(mov);

    StatusCode::CREATED
}

fn game_loop(app_state: &AppState) -> Result<(), snake_game::GameError> {
    use snake_game::{
        fruit::FruitRandomLimited, renderer::GameDisplayToString, snake::SnakeUnbounded, Game,
        GameLevel,
    };
    use std::time::Instant;

    let level = GameLevel::new(40, 20);
    let snake = SnakeUnbounded::new(MovementDirection::Right);
    let fruit = FruitRandomLimited::new(5, 0.1);
    let mut game = Game::new(GameDisplayToString, level, snake, fruit);
    game.put_snake(2).unwrap();

    // initial render
    match game.render() {
        Ok(output) => {
            let mut display = app_state.level_display.blocking_lock();
            *display = output;
        }
        Err(_) => (),
    }

    let mut move_timer = Instant::now();

    loop {
        if move_timer.elapsed() > FRAME_TIME {
            move_timer = Instant::now();

            // get available moves (with draining selected moves)
            let available_moves = {
                let mut moves = app_state.selected_moves.blocking_lock();
                let current_direction = game.snake().direction();
                moves
                    .drain(..)
                    .filter(|d| !d.is_opposite_to(&current_direction))
                    .collect::<Vec<_>>()
                // drop lock
            };
            if !available_moves.is_empty() {
                use rand::seq::SliceRandom;
                let mut r = rand::thread_rng();

                // pick random five
                let selected = available_moves.choose_multiple(&mut r, 5).cloned();
                let mut selected_count = HashMap::with_capacity(4);
                for mov in selected {
                    let entry = selected_count.entry(mov).or_insert(0);
                    *entry += 1;
                }
                let (_, most_occurrences) = selected_count
                    .iter()
                    .max_by(|(_, a), (_, b)| a.cmp(b))
                    .unwrap();
                // drop borrow
                let most_occurrences = *most_occurrences;

                // retain moves with most occurrences
                let selected = selected_count
                    .drain()
                    .filter_map(|(mov, count)| {
                        if count == most_occurrences {
                            Some(mov)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                // finally pick movement
                let movement = *selected.choose(&mut r).unwrap();
                game.set_snake_direction(movement).unwrap();
            }

            if game.try_move().is_err() {
                // restart game
                break;
            }

            match game.render() {
                Ok(output) => {
                    let mut display = app_state.level_display.blocking_lock();
                    *display = output;
                }
                Err(_) => break,
            }
        }
    }

    Ok(())
}
