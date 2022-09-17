mod helper;
mod snake_game;

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use snake_game::MovementDirection;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

static LISTEN_ADDR: &str = "0.0.0.0:3000";
static FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(200);
const SNAKE_TAIL_SIZE: usize = 2; // snake len = head + tail size
const MAX_FRUITS: usize = 5;
const NEW_FRUIT_CHANCE: f64 = 0.1; // 10% on each move
const LEVEL_WIDTH: usize = 40;
const LEVEL_HEIGHT: usize = 20;

#[derive(Default)]
struct AppState {
    level_display: Arc<Mutex<String>>,
    selected_moves: Arc<Mutex<Vec<MovementDirection>>>,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState::default());
    let (shutdown_sig, shutdown_recv) = oneshot::channel::<()>();
    let (game_exit_sig, game_exit_recv) = mpsc::channel::<()>();
    let (preview_send, preview_recv) = mpsc::channel::<String>();

    // game thread
    let thread_app_state = Arc::clone(&app_state);
    std::thread::spawn(move || loop {
        use snake_game::GameError;

        println!("New Game");
        if let Err(err) = game_loop(
            thread_app_state.as_ref(),
            &game_exit_recv,
            preview_send.clone(),
        ) {
            match err {
                GameError::RenderingError | GameError::InvalidInternalState => {
                    eprintln!("{err}");
                    // shutdown server
                    shutdown_sig.send(()).unwrap();
                    break;
                }
                _ => println!("{err}"),
            }
        } else {
            println!("Game thread shutdown.");
            break;
        }
    });

    // terminal renderer thread
    std::thread::spawn(|| {
        if let Err(err) = render_game_in_terminal(preview_recv) {
            eprintln!("Preview rendering: {err}");
        }
        println!("Game preview thread shutdown.");
    });

    let app = Router::new()
        .route("/snake", get(handle_snake_display))
        .route("/snake/:direction", post(handle_snake_direction))
        .layer(Extension(app_state));

    println!("Game server is running at: {LISTEN_ADDR}");
    axum::Server::bind(&LISTEN_ADDR.parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            use tokio::signal;

            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("failed to install signal handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
                _ = shutdown_recv => {}
            }

            println!("Game server shutdown...");
            game_exit_sig.send(()).ok(); // shutdown game thread
        })
        .await
        .unwrap();
}

static LEVEL_TEMPLATE: &str = include_str!("../level.html");

async fn handle_snake_display(Extension(app): Extension<Arc<AppState>>) -> impl IntoResponse {
    let level_display = app.level_display.lock().await;
    let output_html = LEVEL_TEMPLATE.replace("{{ level }}", &level_display);
    ([(header::CONTENT_TYPE, "text/html")], output_html)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Left,
    Right,
    #[serde(alias = "down")]
    Bottom,
    #[serde(alias = "up")]
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

fn game_loop<T>(
    app_state: &AppState,
    end_sig: &mpsc::Receiver<T>,
    preview_send: mpsc::Sender<String>,
) -> Result<(), snake_game::GameError> {
    use snake_game::{
        fruit::FruitRandomLimited, renderer::GameDisplayToString, snake::SnakeUnbounded, Game,
        GameLevel,
    };
    use std::time::Instant;

    let level = GameLevel::new(LEVEL_WIDTH, LEVEL_HEIGHT);
    let snake = SnakeUnbounded::new(MovementDirection::Right);
    let fruit = FruitRandomLimited::new(MAX_FRUITS, NEW_FRUIT_CHANCE);
    let mut game = Game::new(level, snake, fruit);
    game.put_snake(SNAKE_TAIL_SIZE)?;

    let renderer = GameDisplayToString;
    // initial render
    let output = game.render(&renderer)?;
    preview_send.send(output.to_owned()).ok();
    {
        let mut display = app_state.level_display.blocking_lock();
        *display = output;
    }

    let mut move_timer = Instant::now();

    loop {
        if end_sig.try_recv().is_ok() {
            return Ok(());
        }
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
                let (most_occurrences, mut selected_count) =
                    helper::get_most_move_occurrences_in(selected);

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

                // finally pick movement, unwrap is ok - selected vec cannot be empty
                let movement = *selected.choose(&mut r).unwrap();
                game.set_snake_direction(movement)?;
            }

            game.try_move()?;

            let output = game.render(&renderer)?;
            preview_send.send(output.to_owned()).ok();
            let mut display = app_state.level_display.blocking_lock();
            *display = output;
        }
        // slowdown
        std::thread::sleep(std::time::Duration::from_micros(10));
    }
}

fn render_game_in_terminal(output_recv: mpsc::Receiver<String>) -> std::io::Result<()> {
    use crossterm::{cursor, execute, style, terminal};
    use std::io::{stdout, Write};
    let mut stdout = stdout();

    // block thread until new output is available
    // err on recv means channel is closed - game exit
    while let Ok(output) = output_recv.recv() {
        execute!(
            stdout,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            style::Print(&output),
            cursor::RestorePosition,
        )?;

        stdout.flush()?;
    }

    // clear display
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
    stdout.flush()?;

    Ok(())
}
