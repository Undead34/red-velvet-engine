pub mod api_v1;
pub mod contracts;
pub mod health;
pub mod openapi;
pub mod state;

use std::{collections::HashSet, convert::Infallible, sync::Arc};

use axum::{
  Router,
  body::Body,
  extract::{MatchedPath, State},
  http::{HeaderName, Method, Request},
  middleware::{self, Next},
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
  let request_logger = HttpLogConfig::from_env().map(Arc::new);

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

  let router = if let Some(logger) = request_logger {
    router.layer(middleware::from_fn_with_state(logger, log_filtered_requests))
  } else {
    router
  };

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

#[derive(Clone, Debug)]
struct HttpLogConfig {
  paths: HashSet<String>,
  methods: Option<HashSet<Method>>,
}

impl HttpLogConfig {
  fn from_env() -> Option<Self> {
    let raw = std::env::var("RVE_HTTP_LOG_PATHS").ok()?;
    let paths = raw
      .split(',')
      .filter_map(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) }
      })
      .collect::<HashSet<_>>();
    if paths.is_empty() {
      return None;
    }

    let methods = std::env::var("RVE_HTTP_LOG_METHODS")
      .ok()
      .map(|value| {
        value
          .split(',')
          .filter_map(|raw| Method::from_bytes(raw.trim().as_bytes()).ok())
          .collect::<HashSet<_>>()
      })
      .filter(|set| !set.is_empty());

    Some(Self { paths, methods })
  }

  fn should_log(&self, method: &Method, matched_path: Option<&str>) -> bool {
    let path = matched_path.unwrap_or_default();
    if !self.paths.contains(path) {
      return false;
    }
    if let Some(methods) = &self.methods { methods.contains(method) } else { true }
  }
}

async fn log_filtered_requests(
  State(config): State<Arc<HttpLogConfig>>,
  req: Request<Body>,
  next: Next,
) -> Result<axum::response::Response, Infallible> {
  let matched_path = req.extensions().get::<MatchedPath>().map(|m| m.as_str().to_owned());
  let matched_path_str = matched_path.as_deref();
  let method = req.method().clone();
  let request_id = req.headers().get(request_id_header()).and_then(|value| {
    value.to_str().ok().map(|s| s.to_owned())
  });
  let should_log = config.should_log(&method, matched_path_str);

  if should_log {
    tracing::info!(
      target: "HTTP_FILTER",
      %method,
      path = matched_path_str.unwrap_or("<unknown>"),
      request_id = request_id.as_deref().unwrap_or(""),
      "incoming request"
    );
  }

  let response = next.run(req).await;

  if should_log {
    tracing::info!(
      target: "HTTP_FILTER",
      %method,
      path = matched_path_str.unwrap_or("<unknown>"),
      request_id = request_id.as_deref().unwrap_or(""),
      status = response.status().as_u16(),
      "request completed"
    );
  }

  Ok(response)
}
