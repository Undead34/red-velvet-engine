pub mod api_v1;
pub mod contracts;
pub mod health;
pub mod openapi;
pub mod state;

use axum::{
  Router,
  extract::MatchedPath,
  http::{HeaderName, Request},
  response::Html,
  routing::get,
};
use tower_http::{
  request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
  trace::{DefaultOnRequest, DefaultOnResponse, OnResponse, TraceLayer},
};
use tracing::{Level, Span, field};

use crate::http::{openapi::openapi_json, state::AppState};

const API_DOCS_HTML: &str = include_str!("pages/api_docs.html");
const HOME_HTML: &str = include_str!("pages/home.html");
const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn build_router(state: AppState) -> Router {
  let request_tracing = TraceLayer::new_for_http()
    .make_span_with(|request: &Request<_>| {
      let matched_path =
        request.extensions().get::<MatchedPath>().map(MatchedPath::as_str).unwrap_or("<unmatched>");
      let request_id = request
        .headers()
        .get(request_id_header())
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

      tracing::span!(
        Level::INFO,
        "http.request",
        method = %request.method(),
        matched_path,
        request_id,
        status_code = field::Empty,
        latency_ms = field::Empty,
      )
    })
    .on_request(DefaultOnRequest::new().level(Level::DEBUG))
    .on_response(
      |response: &axum::http::Response<_>, latency: std::time::Duration, span: &Span| {
        span.record("status_code", response.status().as_u16());
        span.record("latency_ms", latency.as_millis() as u64);
        DefaultOnResponse::new().level(Level::INFO).on_response(response, latency, span);
      },
    );

  let router = Router::new()
    .route("/", get(home))
    .route("/health", get(health::handler))
    .nest("/api/v1", api_v1::router())
    .route("/docs", get(elements_docs))
    .route("/api-docs/openapi.json", get(openapi_json))
    .route("/api-docs", get(elements_docs))
    .layer(request_tracing)
    .layer(PropagateRequestIdLayer::new(request_id_header()))
    .layer(SetRequestIdLayer::new(request_id_header(), MakeRequestUuid))
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

fn request_id_header() -> HeaderName {
  HeaderName::from_static(REQUEST_ID_HEADER)
}
