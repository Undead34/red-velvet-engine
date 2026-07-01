use rve_core::domain::{
  common::{Channel, RuleId, Severity, TimestampMs},
  rule::{FunctionKind, RuleAction, mode::RuleMode},
};
use serde::Deserialize;
use serde_json::Value;
use utoipa::IntoParams;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

use super::super::errors::{ApiError, ValidationIssue, ValidationReport};

/// Query parameters accepted by the rule listing endpoint.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Pagination {
  #[param(default = 1, minimum = 1)]
  pub page: Option<u32>,
  #[param(default = 20, minimum = 1, maximum = 100)]
  pub limit: Option<u32>,
}

/// Partial update payload accepted by `PATCH /api/v1/rules/{id}`.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RulePatchRequest {
  #[validate(nested)]
  pub state: Option<RuleStatePatch>,
  #[validate(nested)]
  pub rollout: Option<RolloutPolicyPatch>,
  #[validate(nested)]
  pub schedule: Option<RuleSchedulePatch>,
}

/// Lifecycle fields that can be patched independently.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleStatePatch {
  pub mode: Option<RuleMode>,
  #[validate(nested)]
  pub audit: Option<RuleAuditPatch>,
}

/// Audit fields that can be patched independently.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleAuditPatch {
  pub updated_at_ms: Option<TimestampMs>,
  #[validate(length(min = 1, max = 120))]
  pub updated_by: Option<String>,
}

/// Rollout fields that can be patched independently.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RolloutPolicyPatch {
  #[validate(range(max = 100))]
  pub percent: Option<u8>,
}

/// Schedule fields that can be patched independently.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleSchedulePatch {
  pub active_from_ms: Option<TimestampMs>,
  pub active_until_ms: Option<TimestampMs>,
}

/// Complete rule document accepted by create and replace endpoints.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleDocumentRequest {
  #[serde(default)]
  pub id: Option<RuleId>,
  #[validate(nested)]
  pub meta: RuleMetaRequest,
  #[serde(default)]
  #[validate(nested)]
  pub scope: RuleScopeRequest,
  #[validate(nested)]
  pub state: RuleStateRequest,
  #[validate(nested)]
  pub schedule: RuleScheduleRequest,
  #[validate(nested)]
  pub rollout: RolloutPolicyRequest,
  #[validate(nested)]
  pub evaluation: RuleEvaluationRequest,
  #[validate(nested)]
  pub enforcement: RuleEnforcementRequest,
}

/// Channel applicability section of a rule request.
#[derive(Debug, Deserialize, Validate, Default)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_scope_request"))]
pub struct RuleScopeRequest {
  pub channels: Option<Vec<Channel>>,
}

/// Human-readable rule metadata supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleMetaRequest {
  #[validate(length(min = 3, max = 80))]
  pub code: Option<String>,
  #[validate(length(min = 1, max = 120))]
  pub name: String,
  #[validate(length(max = 1000))]
  pub description: Option<String>,
  pub version: semver::Version,
  #[validate(length(min = 1, max = 120))]
  pub author: String,
  #[validate(length(max = 32))]
  pub tags: Option<Vec<String>>,
}

/// Lifecycle state supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleStateRequest {
  pub mode: RuleMode,
  #[validate(nested)]
  pub audit: RuleAuditRequest,
}

/// Audit metadata supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_audit_request"))]
pub struct RuleAuditRequest {
  pub created_at_ms: TimestampMs,
  pub updated_at_ms: TimestampMs,
  #[validate(length(min = 1, max = 120))]
  pub created_by: Option<String>,
  #[validate(length(min = 1, max = 120))]
  pub updated_by: Option<String>,
}

/// Optional activation window supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_schedule_request"))]
pub struct RuleScheduleRequest {
  pub active_from_ms: Option<TimestampMs>,
  pub active_until_ms: Option<TimestampMs>,
}

/// Traffic allocation supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RolloutPolicyRequest {
  #[validate(range(max = 100))]
  pub percent: u8,
}

/// JSONLogic expressions supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationRequest {
  pub condition: Value,
  pub logic: Value,
}

/// Function pipeline item supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleFunctionRequest {
  pub kind: FunctionKind,
  pub config: Value,
}

/// Enforcement settings supplied by API clients.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleEnforcementRequest {
  #[validate(range(min = 1.0, max = 10.0))]
  pub score_impact: f32,
  pub action: RuleAction,
  pub severity: Severity,
  #[validate(length(min = 1, max = 64))]
  pub tags: Vec<String>,
  #[validate(range(min = 1, max = 86_400_000))]
  pub cooldown_ms: Option<u64>,
  #[serde(default)]
  pub functions: Vec<RuleFunctionRequest>,
}

fn validate_schedule_request(schedule: &RuleScheduleRequest) -> Result<(), ValidationError> {
  if let (Some(from), Some(until)) = (schedule.active_from_ms, schedule.active_until_ms)
    && until.as_u64() <= from.as_u64()
  {
    return Err(ValidationError::new("active_until_must_be_greater_than_active_from"));
  }
  Ok(())
}

fn validate_audit_request(audit: &RuleAuditRequest) -> Result<(), ValidationError> {
  if audit.updated_at_ms.as_u64() < audit.created_at_ms.as_u64() {
    return Err(ValidationError::new("updated_at_must_be_greater_or_equal_created_at"));
  }
  Ok(())
}

fn validate_scope_request(scope: &RuleScopeRequest) -> Result<(), ValidationError> {
  let Some(channels) = &scope.channels else {
    return Ok(());
  };

  if channels.is_empty() {
    return Err(ValidationError::new("channels_must_not_be_empty"));
  }

  if channels.len() > 16 {
    return Err(ValidationError::new("channels_limit_exceeded"));
  }

  let unique = channels.iter().collect::<std::collections::HashSet<_>>();
  if unique.len() != channels.len() {
    return Err(ValidationError::new("channels_must_be_unique"));
  }

  Ok(())
}

/// Converts validator output into the API error envelope.
pub(crate) fn map_validation_errors(errors: ValidationErrors) -> ApiError {
  let mut issues = Vec::new();
  collect_validation_messages("", &errors, &mut issues);

  let errors = if issues.is_empty() {
    vec![ValidationIssue {
      path: "request".to_owned(),
      message: "invalid request payload".to_owned(),
    }]
  } else {
    issues
  };

  ApiError::Unprocessable(ValidationReport { errors, warnings: Vec::new() })
}

fn collect_validation_messages(
  prefix: &str,
  errors: &ValidationErrors,
  out: &mut Vec<ValidationIssue>,
) {
  for (field, kind) in errors.errors() {
    let path = if prefix.is_empty() { field.to_string() } else { format!("{prefix}.{field}") };

    match kind {
      ValidationErrorsKind::Field(field_errors) => {
        for field_error in field_errors {
          let message = field_error
            .message
            .clone()
            .unwrap_or_else(|| field_error.code.to_string().into())
            .to_string();
          out.push(ValidationIssue { path: path.clone(), message });
        }
      }
      ValidationErrorsKind::Struct(struct_errors) => {
        collect_validation_messages(&path, struct_errors, out);
      }
      ValidationErrorsKind::List(list_errors) => {
        for (index, nested) in list_errors {
          let indexed = format!("{path}[{index}]");
          collect_validation_messages(&indexed, nested, out);
        }
      }
    }
  }
}
