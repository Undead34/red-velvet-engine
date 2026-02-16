mod rules;

use axum::{Router, routing::get};

use crate::http::state::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/rules", get(rules::list_rules).post(rules::create_rule))
    .route("/rules/{id}", get(rules::get_rule).put(rules::update_rule).delete(rules::delete_rule))
}
