pub mod api_v1;
pub mod health;
pub mod state;
pub mod status;

use axum::{Router, routing::get};

use crate::http::state::AppState;

pub fn build_router(state: AppState) -> Router {
  let router = Router::new()
    .route("/health", get(health::handler))
    .nest("/api/v1", api_v1::router())
    .with_state(state);

  add_dev_cors(router)
}

#[cfg(debug_assertions)]
fn add_dev_cors(router: Router) -> Router {
  use tower_http::cors::CorsLayer;

  router.layer(CorsLayer::permissive())
}

#[cfg(not(debug_assertions))]
fn add_dev_cors(router: Router) -> Router {
  router
}
