use axum::{Json, http::StatusCode};

use rve_core::application::{DecisionServiceError, RuntimeControlError};
use rve_core::ports::rule_engine::RuntimeEngineError;

use crate::interfaces::http::openapi::ErrorResponse;

/// Centralized mapper for [`DecisionServiceError`] used by both
/// `engine` and `decisions` endpoints.
pub fn map_decision_service_error(
  error: DecisionServiceError,
) -> (StatusCode, Json<ErrorResponse>) {
  let (status, code) = match &error {
    DecisionServiceError::Runtime(RuntimeEngineError::Configuration { .. }) => {
      (StatusCode::SERVICE_UNAVAILABLE, "runtime_configuration")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Compilation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_compilation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Evaluation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_evaluation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::NotImplemented { .. }) => {
      (StatusCode::NOT_IMPLEMENTED, "not_implemented")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Internal { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_internal")
    }
    DecisionServiceError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository_error"),
  };

  (
    status,
    Json(ErrorResponse { code: code.to_owned(), message: error.to_string(), validation: None }),
  )
}

/// Centralized mapper for runtime control failures.
pub fn map_runtime_control_error(error: RuntimeControlError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, code) = match &error {
    RuntimeControlError::Runtime(RuntimeEngineError::Configuration { .. }) => {
      (StatusCode::SERVICE_UNAVAILABLE, "runtime_configuration")
    }
    RuntimeControlError::Runtime(RuntimeEngineError::Compilation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_compilation")
    }
    RuntimeControlError::Runtime(RuntimeEngineError::Evaluation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_evaluation")
    }
    RuntimeControlError::Runtime(RuntimeEngineError::NotImplemented { .. }) => {
      (StatusCode::NOT_IMPLEMENTED, "not_implemented")
    }
    RuntimeControlError::Runtime(RuntimeEngineError::Internal { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_internal")
    }
    RuntimeControlError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository_error"),
  };

  (
    status,
    Json(ErrorResponse { code: code.to_owned(), message: error.to_string(), validation: None }),
  )
}
