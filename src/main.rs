use axum::{routing::{get, post, delete}, Router};

mod routes;
mod model;

const ROOT_PATH: &str = "127.0.0.1:7878";

#[tokio::main]
async fn main() {
    println!("Starting golem server at {}", ROOT_PATH);

    let app = Router::new();

    axum::Server::bind(&ROOT_PATH.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
