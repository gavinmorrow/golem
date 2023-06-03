use axum::{extract::State, response::Html, Router};
use axum_macros::debug_handler;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

#[derive(Clone, Debug)]
struct TemplateState {
    templates: Tera,
}

pub fn router() -> Router {
    let templates = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    let state = TemplateState { templates };

    Router::new()
        .route("/", axum::routing::get(index))
        .nest_service("/static", ServeDir::new("public"))
        .with_state(state)
}

#[debug_handler]
async fn index(State(state): State<TemplateState>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Hello, world!");
    let rendered = state.templates.render("index.html", &context).unwrap();
    Html(rendered)
}
