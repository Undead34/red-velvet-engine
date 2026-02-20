use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use rve_core::{domain::common::RuleId, ports::RuleRepositoryError};
use serde::Serialize;
use tracing::error;

use crate::http::state::EngineSyncError;

pub(crate) type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub(crate) enum ApiError {
  Validation { field: String, message: String },
  NotFound(String),
  Conflict(String),
  Internal(String),
}

impl ApiError {
  pub(super) fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
    Self::Validation { field: field.into(), message: message.into() }
  }
}

#[derive(Serialize)]
struct ErrorBody {
  code: &'static str,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  field: Option<String>,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (status, body) = match self {
      ApiError::Validation { field, message } => (
        StatusCode::BAD_REQUEST,
        ErrorBody { code: "validation_error", message, field: Some(field) },
      ),
      ApiError::NotFound(message) => {
        (StatusCode::NOT_FOUND, ErrorBody { code: "not_found", message, field: None })
      }
      ApiError::Conflict(message) => {
        (StatusCode::CONFLICT, ErrorBody { code: "conflict", message, field: None })
      }
      ApiError::Internal(message) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorBody { code: "internal_error", message, field: None },
      ),
    };

    (status, Json(body)).into_response()
  }
}

pub(super) fn parse_rule_id(id: String) -> ApiResult<RuleId> {
  RuleId::try_from(id).map_err(|_| ApiError::validation("id", "must be a valid UUID"))
}

pub(super) fn map_repository_error(error: RuleRepositoryError) -> ApiError {
  match error {
    RuleRepositoryError::AlreadyExists(id) => {
      ApiError::Conflict(format!("rule already exists: {id}"))
    }
    RuleRepositoryError::NotFound(id) => ApiError::NotFound(format!("rule not found: {id}")),
    RuleRepositoryError::Storage(message) => ApiError::Internal(message),
  }
}

pub(super) fn map_engine_sync_error(error: EngineSyncError, operation: &str) -> ApiError {
  error!(target: "BANNER", %error, %operation, "failed to refresh engine ruleset");
  ApiError::Internal("failed to refresh engine ruleset".to_owned())
}
