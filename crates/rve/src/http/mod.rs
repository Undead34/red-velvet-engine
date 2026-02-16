pub mod api_v1;
pub mod health;
pub mod state;

use axum::{Router, routing::get};

use crate::http::state::AppState;

pub fn build_router(state: AppState) -> Router {
  Router::<AppState>::new()
    .route("/health", get(health::handler))
    .route("/status", get(status::handler))
    .nest("/api/v1", api_v1::router())
    .with_state(state)
}
