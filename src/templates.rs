use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Router,
};
use axum_macros::debug_handler;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

use crate::model::AppState;

#[derive(Clone)]
struct TemplateState {
    templates: Tera,
    appstate: AppState,
}

pub fn router(appstate: AppState) -> Router {
    let templates = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    let state = TemplateState {
        templates,
        appstate,
    };

    Router::new()
        .route("/", axum::routing::get(index))
        .nest_service("/static", ServeDir::new("public"))
        .route("/room/:room_name", axum::routing::get(room))
        .with_state(state)
}

#[debug_handler]
async fn index(State(state): State<TemplateState>) -> Html<String> {
    let context = Context::new();
    let rendered = state.templates.render("base.html", &context).unwrap();
    Html(rendered)
}

#[debug_handler]
async fn room(
    State(state): State<TemplateState>,
    Path(room_name): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // Get the room from the database
    let database = state.appstate.database.lock().await;
    let Some(room) = database.get_room_by_name(&room_name).unwrap() else {
        return Err(StatusCode::NOT_FOUND);
    };

    let mut context = Context::new();
    context.insert("room_id", &room.id);
    context.insert("room_name", &room_name);

    let rendered = state.templates.render("room.html", &context).unwrap();
    Ok(Html(rendered))
}
