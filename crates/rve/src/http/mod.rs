pub mod api_v1;
pub mod health;
pub mod openapi;
pub mod state;
pub mod status;

use axum::{Router, response::Html, routing::get};

use crate::http::{openapi::openapi_json, state::AppState};

const API_DOCS_HTML: &str = include_str!("pages/api_docs.html");
const HOME_HTML: &str = include_str!("pages/home.html");

pub fn build_router(state: AppState) -> Router {
  let router = Router::new()
    .route("/", get(home))
    .route("/health", get(health::handler))
    .nest("/api/v1", api_v1::router())
    .route("/api-docs/openapi.json", get(openapi_json))
    .route("/api-docs", get(elements_docs))
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

async fn elements_docs() -> Html<&'static str> {
  Html(API_DOCS_HTML)
}

async fn home() -> Html<&'static str> {
  Html(HOME_HTML)
}
