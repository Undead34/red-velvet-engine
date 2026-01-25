use axum::{Router, routing::get};

use crate::http::state::AppState;

mod health;
mod status;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::handler))
        .route("/status", get(status::handler))
        .with_state(state)
}
