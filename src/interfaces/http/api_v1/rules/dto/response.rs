use rve_core::domain::rule::{Rule, RuleFunctionSpec};
use serde::Serialize;
use serde_json::Value;

/// Rule document returned by the rules API.
#[derive(Debug, Clone, Serialize)]
pub struct RuleResponse {
  pub id: String,
  pub meta: RuleMetaResponse,
  pub scope: RuleScopeResponse,
  pub state: RuleStateResponse,
  pub schedule: RuleScheduleResponse,
  pub rollout: RolloutPolicyResponse,
  pub evaluation: RuleEvaluationResponse,
  pub enforcement: RuleEnforcementResponse,
}

/// Metadata section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleMetaResponse {
  pub code: Option<String>,
  pub name: String,
  pub description: Option<String>,
  pub version: String,
  pub author: String,
  pub tags: Option<Vec<String>>,
}

/// Channel applicability section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleScopeResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub channels: Option<Vec<String>>,
}

/// Lifecycle state section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleStateResponse {
  pub mode: String,
  pub audit: RuleAuditResponse,
}

/// Audit section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleAuditResponse {
  pub created_at_ms: u64,
  pub updated_at_ms: u64,
  pub created_by: Option<String>,
  pub updated_by: Option<String>,
}

/// Schedule section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleScheduleResponse {
  pub active_from_ms: Option<u64>,
  pub active_until_ms: Option<u64>,
}

/// Rollout section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RolloutPolicyResponse {
  pub percent: u8,
}

/// JSONLogic evaluation section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleEvaluationResponse {
  pub condition: Value,
  pub logic: Value,
}

/// Enforcement section returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleEnforcementResponse {
  pub score_impact: f32,
  pub action: String,
  pub severity: String,
  pub tags: Vec<String>,
  pub cooldown_ms: Option<u64>,
  pub functions: Vec<RuleFunctionResponse>,
}

/// Function pipeline item returned for a rule.
#[derive(Debug, Clone, Serialize)]
pub struct RuleFunctionResponse {
  pub kind: String,
  pub config: Value,
}

/// Paginated rule collection returned by the rules API.
#[derive(Debug, Clone, Serialize)]
pub struct RuleListResponse {
  pub data: Vec<RuleResponse>,
  pub pagination: PaginationMeta,
}

impl RuleListResponse {
  /// Builds a paginated response from domain rules.
  pub fn from_rules(items: Vec<Rule>, page: u32, limit: u32, total: u32) -> Self {
    Self {
      data: items.iter().map(RuleResponse::from).collect(),
      pagination: PaginationMeta { page, limit, total },
    }
  }
}

/// Pagination metadata returned by collection endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
  pub page: u32,
  pub limit: u32,
  pub total: u32,
}

impl From<&Rule> for RuleResponse {
  fn from(rule: &Rule) -> Self {
    let identity = rule.identity();
    let state = rule.state();
    let schedule = rule.schedule();
    let rollout = rule.rollout();
    let evaluation = rule.evaluation();
    let enforcement = rule.enforcement();

    Self {
      id: rule.id.to_string(),
      meta: RuleMetaResponse {
        code: identity.code.clone(),
        name: identity.name.clone(),
        description: identity.description.clone(),
        version: identity.version.to_string(),
        author: identity.author.clone(),
        tags: identity.tags.clone(),
      },
      scope: RuleScopeResponse {
        channels: rule
          .scope()
          .channels()
          .map(|channels| channels.iter().map(|channel| channel.as_str().to_owned()).collect()),
      },
      state: RuleStateResponse {
        mode: serialize_as_string(&state.mode),
        audit: RuleAuditResponse {
          created_at_ms: state.audit.created_at_ms.as_u64(),
          updated_at_ms: state.audit.updated_at_ms.as_u64(),
          created_by: state.audit.created_by.clone(),
          updated_by: state.audit.updated_by.clone(),
        },
      },
      schedule: RuleScheduleResponse {
        active_from_ms: schedule.active_from_ms.map(|timestamp| timestamp.as_u64()),
        active_until_ms: schedule.active_until_ms.map(|timestamp| timestamp.as_u64()),
      },
      rollout: RolloutPolicyResponse { percent: rollout.percent },
      evaluation: RuleEvaluationResponse {
        condition: evaluation.condition.as_value().clone(),
        logic: evaluation.logic.as_value().clone(),
      },
      enforcement: RuleEnforcementResponse {
        score_impact: enforcement.score_impact.as_f32(),
        action: serialize_as_string(&enforcement.action),
        severity: serialize_as_string(&enforcement.severity),
        tags: enforcement.tags.clone(),
        cooldown_ms: enforcement.cooldown_ms,
        functions: enforcement.functions.iter().map(RuleFunctionResponse::from).collect(),
      },
    }
  }
}

impl From<&RuleFunctionSpec> for RuleFunctionResponse {
  fn from(function: &RuleFunctionSpec) -> Self {
    Self { kind: function.kind.as_str().to_owned(), config: function.config.clone() }
  }
}

fn serialize_as_string<T>(value: &T) -> String
where
  T: Serialize + std::fmt::Debug,
{
  match serde_json::to_value(value) {
    Ok(Value::String(value)) => value,
    Ok(value) => value.to_string(),
    Err(_) => format!("{value:?}"),
  }
}
