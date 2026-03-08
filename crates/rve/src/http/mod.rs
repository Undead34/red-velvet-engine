pub mod api_v1;
pub mod health;
pub mod openapi;
pub mod state;
pub mod status;

use axum::{Router, response::Html, routing::get};

use crate::http::{openapi::openapi_json, state::AppState};

pub fn build_router(state: AppState) -> Router {
  let router = Router::new()
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
  Html(
    r#"
        <!doctype html>
        <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
            <title>API Reference</title>
            <script src="https://unpkg.com/@stoplight/elements/web-components.min.js"></script>
            <link rel="stylesheet" href="https://unpkg.com/@stoplight/elements/styles.min.css">
          </head>
          <body>
            <elements-api
              apiDescriptionUrl="/api-docs/openapi.json"
              router="hash"
              layout="sidebar"
            ></elements-api>
          </body>
        </html>
        "#,
  )
}
