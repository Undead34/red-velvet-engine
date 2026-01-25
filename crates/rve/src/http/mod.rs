pub mod routes;
pub mod state;

use axum::Router;

use crate::http::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new().merge(routes::router(state))
}
