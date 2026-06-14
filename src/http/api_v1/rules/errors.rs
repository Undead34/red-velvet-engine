use axum::{
  Json,
  extract::rejection::JsonRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use rve_core::{domain::common::RuleId, ports::rule_repository::RuleRepositoryError};
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
  BadRequest(String),
  Unprocessable(ValidationReport),
  NotFound(String),
  Conflict(String),
  Internal(String),
}

impl ApiError {
  pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
    Self::Unprocessable(ValidationReport {
      errors: vec![ValidationIssue { path: field.into(), message: message.into() }],
      warnings: Vec::new(),
    })
  }
}

#[derive(Serialize)]
struct ErrorBody {
  code: &'static str,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  validation: Option<ValidationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
  pub path: String,
  pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
  pub errors: Vec<ValidationIssue>,
  pub warnings: Vec<ValidationIssue>,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (status, body) = match self {
      ApiError::BadRequest(message) => {
        (StatusCode::BAD_REQUEST, ErrorBody { code: "bad_request", message, validation: None })
      }
      ApiError::Unprocessable(report) => (
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorBody {
          code: "validation_failed",
          message: "request validation failed".to_owned(),
          validation: Some(report),
        },
      ),
      ApiError::NotFound(message) => {
        (StatusCode::NOT_FOUND, ErrorBody { code: "not_found", message, validation: None })
      }
      ApiError::Conflict(message) => {
        (StatusCode::CONFLICT, ErrorBody { code: "conflict", message, validation: None })
      }
      ApiError::Internal(message) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorBody { code: "internal_error", message, validation: None },
      ),
    };

    (status, Json(body)).into_response()
  }
}

pub(super) fn parse_rule_id(id: String) -> ApiResult<RuleId> {
  RuleId::try_from(id).map_err(|_| ApiError::BadRequest("id must be a valid UUID".to_owned()))
}

pub(super) fn map_json_rejection(error: JsonRejection) -> ApiError {
  match error {
    JsonRejection::MissingJsonContentType(_) => {
      ApiError::BadRequest("content-type must be application/json".to_owned())
    }
    JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => {
      ApiError::Unprocessable(ValidationReport {
        errors: vec![ValidationIssue { path: "request".to_owned(), message: error.body_text() }],
        warnings: Vec::new(),
      })
    }
    _ => ApiError::BadRequest(error.body_text()),
  }
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
