use axum::{response::Html, Router};
use axum_macros::debug_handler;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                panic!("Parsing error(s): {}", e);
            }
        };
        tera
    };
}

pub fn router() -> Router {
    Router::new()
        .route("/a", axum::routing::get(index))
        .nest_service("/", ServeDir::new("public"))
}

#[debug_handler]
async fn index() -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Hello, world!");
    let rendered = TEMPLATES.render("index.html", &context).unwrap();
    Html(rendered)
}
