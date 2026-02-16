use std::str::FromStr;

use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use serde_json::{Value, json};

use rve_core::{ConnectorAck, ConnectorError, ConnectorKind};

use crate::http::state::AppState;

pub async fn ingest_event(
  State(state): State<AppState>,
  Path(connector): Path<String>,
  Json(payload): Json<Value>,
) -> Result<Json<ConnectorAck>, (StatusCode, Json<Value>)> {
  let kind =
    ConnectorKind::from_str(&connector).map_err(|err| build_error(StatusCode::NOT_FOUND, err))?;

  let ack = state.connectors.ingest(kind, payload).await.map_err(|err| {
    build_error(
      match err {
        ConnectorError::InvalidPayload(_) => StatusCode::BAD_REQUEST,
        ConnectorError::UnknownConnector(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
      },
      err,
    )
  })?;

  Ok(Json(ack))
}

fn build_error(status: StatusCode, err: ConnectorError) -> (StatusCode, Json<Value>) {
  let body = json!({ "error": err.to_string() });
  (status, Json(body))
}
