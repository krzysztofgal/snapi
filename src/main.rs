mod snake_game;

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

const LISTEN_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/snake", get(handle_snake_display))
        .route("/snake/:direction", post(handle_snake_direction));

    println!("Game server is running at: {LISTEN_ADDR}");
    axum::Server::bind(&LISTEN_ADDR.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_snake_display() -> impl IntoResponse {}

#[derive(serde::Deserialize)]
enum Direction {
    Left,
    Right,
    Bottom,
    Top,
}

async fn handle_snake_direction(Path(_direction_command): Path<Direction>) -> impl IntoResponse {}
