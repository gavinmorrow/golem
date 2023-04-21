use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use model::AppState;
use std::sync::Arc;

mod logger;
mod model;
mod routes;

type Snowcloud = snowcloud::MultiThread<43, 8, 12>;
const EPOCH: u64 = 1650667342;
const PRIMARY_ID: i64 = 1;

const ROOT_PATH: &str = "127.0.0.1:7878";

#[tokio::main]
async fn main() {
    logger::init();

    info!("Starting golem server at {}", ROOT_PATH);

    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/api/register", post(routes::register))
        .route("/snowflake", get(routes::snowflake))
        .with_state(state);

    axum::Server::bind(&ROOT_PATH.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
