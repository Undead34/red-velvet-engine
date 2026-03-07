pub mod decisions;
pub mod engine;
pub mod metadata;
pub mod rules;

use axum::{
  Router,
  routing::{get, post},
};

use crate::http::state::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/rules", get(rules::list_rules).post(rules::create_rule))
    .route(
      "/rules/{id}",
      get(rules::get_rule)
        .put(rules::update_rule)
        .patch(rules::patch_rule)
        .delete(rules::delete_rule),
    )
    .route("/metadata/fields", get(metadata::fields))
    .route("/metadata/contract", get(metadata::contract))
    .route("/engine/status", get(engine::status))
    .route("/decisions", post(decisions::create_decision))
    .route("/engine/reload", post(engine::reload))
}
