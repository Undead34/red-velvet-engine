//! Mapping between rule HTTP DTOs and domain aggregates.
//!
//! Handlers call these functions at the port boundary. The mapper owns all
//! conversion details so DTO modules remain transport-focused and the domain
//! model stays independent from HTTP concerns.

use rve_core::domain::{
  DomainError,
  common::{RuleId, Score},
  rule::{
    RolloutPolicy, Rule, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement, RuleEvaluation,
    RuleExpression, RuleFunctionSpec, RuleIdentity, RulePolicy, RuleSchedule, RuleScope, RuleState,
  },
};
use validator::Validate;

use super::{
  dto::request::{
    RolloutPolicyRequest, RuleAuditRequest, RuleDocumentRequest, RuleEnforcementRequest,
    RuleEvaluationRequest, RuleFunctionRequest, RuleMetaRequest, RuleScheduleRequest,
    RuleScopeRequest, RuleStateRequest, map_validation_errors,
  },
  errors::{ApiError, ApiResult},
  validation::validate_rule,
};

/// Builds a validated domain rule from an HTTP rule document.
pub(crate) fn rule_from_document(
  request: RuleDocumentRequest,
  override_id: Option<RuleId>,
) -> ApiResult<Rule> {
  request.validate().map_err(map_validation_errors)?;

  let id = override_id.or(request.id).unwrap_or_else(RuleId::new_v7);
  let identity = map_identity(request.meta);
  let scope = map_scope(request.scope)?;
  let policy = RulePolicy::new(
    map_state(request.state)?,
    map_schedule(request.schedule)?,
    map_rollout(request.rollout)?,
  )
  .map_err(|error| map_domain_error("policy", error.into()))?;
  let definition = RuleDefinition::new(map_evaluation(request.evaluation)?)
    .map_err(|error| map_domain_error("definition", error))?;
  let outcome = RuleDecision::new(map_enforcement(request.enforcement)?);

  let rule = Rule::new(id, identity, scope, policy, definition, outcome)
    .map_err(|error| map_domain_error("rule", error))?;

  validate_rule(&rule)?;
  Ok(rule)
}

fn map_identity(request: RuleMetaRequest) -> RuleIdentity {
  RuleIdentity {
    code: request.code,
    name: request.name,
    description: request.description,
    version: request.version,
    author: request.author,
    tags: request.tags,
  }
}

fn map_scope(request: RuleScopeRequest) -> ApiResult<RuleScope> {
  RuleScope::try_from(request.channels)
    .map_err(|error| map_domain_error("scope", DomainError::RuleScope(error)))
}

fn map_state(request: RuleStateRequest) -> ApiResult<RuleState> {
  RuleState::new(request.mode, map_audit(request.audit))
    .map_err(|error| map_domain_error("state", error.into()))
}

fn map_audit(request: RuleAuditRequest) -> RuleAudit {
  RuleAudit {
    created_at_ms: request.created_at_ms,
    updated_at_ms: request.updated_at_ms,
    created_by: request.created_by,
    updated_by: request.updated_by,
  }
}

fn map_schedule(request: RuleScheduleRequest) -> ApiResult<RuleSchedule> {
  RuleSchedule::new(request.active_from_ms, request.active_until_ms)
    .map_err(|error| map_domain_error("schedule", error.into()))
}

fn map_rollout(request: RolloutPolicyRequest) -> ApiResult<RolloutPolicy> {
  RolloutPolicy::new(request.percent).map_err(|error| map_domain_error("rollout", error.into()))
}

fn map_evaluation(request: RuleEvaluationRequest) -> ApiResult<RuleEvaluation> {
  let condition = RuleExpression::new(request.condition)
    .map_err(|error| map_domain_error("evaluation.condition", error))?;
  let logic = RuleExpression::new(request.logic)
    .map_err(|error| map_domain_error("evaluation.logic", error))?;

  RuleEvaluation::new(condition, logic).map_err(|error| map_domain_error("evaluation", error))
}

fn map_enforcement(request: RuleEnforcementRequest) -> ApiResult<RuleEnforcement> {
  let score_impact = Score::new(request.score_impact).map_err(|_| {
    ApiError::validation("enforcement.score_impact", "must be between 1.0 and 10.0")
  })?;
  let mut functions = Vec::with_capacity(request.functions.len());

  for (index, function) in request.functions.into_iter().enumerate() {
    let field = format!("enforcement.functions[{index}]");
    let function = map_function(function).map_err(|error| map_domain_error(&field, error))?;
    functions.push(function);
  }

  Ok(RuleEnforcement {
    score_impact,
    action: request.action,
    severity: request.severity,
    tags: request.tags,
    cooldown_ms: request.cooldown_ms,
    functions,
  })
}

fn map_function(request: RuleFunctionRequest) -> Result<RuleFunctionSpec, DomainError> {
  RuleFunctionSpec::new(request.kind, request.config)
}

fn map_domain_error(field: &str, error: DomainError) -> ApiError {
  ApiError::validation(field, error.to_string())
}
