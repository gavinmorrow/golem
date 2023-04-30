use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use log::info;
use model::AppState;

mod auth;
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

    let state = AppState::new();

    let app = Router::new()
        .route("/api/user/:id", get(routes::get_user))
        .route("/api/logout", post(routes::sessions::logout))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::auth::authenticate,
        ))
        .nest("/api/ws", routes::ws::router())
        .route("/api/login", post(routes::sessions::login))
        .route("/api/register", post(routes::register::register))
        .route("/api/snowflake", get(routes::snowflake))
        .route("/api/snapshot", get(routes::messages::get_snapshot))
        .with_state(state.into());

    axum::Server::bind(&ROOT_PATH.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
