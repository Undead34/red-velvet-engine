use std::collections::BTreeMap;

use axum::Json;
use rve_core::domain::{
  common::Channel,
  rule::{JSONLOGIC_ROOT_VARS, RuleExpression},
};
use serde::Serialize;

use crate::interfaces::http::contracts::RULE_SCHEMA_VERSION;

#[derive(Serialize)]
pub struct BuilderConfigResponse {
  /// Operator palette grouped by semantic category.
  pub operator_groups: BTreeMap<&'static str, Vec<&'static str>>,
  /// Allowed root namespaces for `var` paths in expressions.
  pub root_vars: Vec<&'static str>,
  /// Available enums for dropdown rendering.
  pub enums: BTreeMap<&'static str, Vec<&'static str>>,
  /// Rule field definitions for dynamic form generation.
  pub rule_fields: Vec<FieldDef>,
  /// Current rule schema version.
  pub rule_schema_version: &'static str,
}

#[derive(Serialize)]
pub struct FieldDef {
  pub path: &'static str,
  #[serde(rename = "type")]
  pub kind: &'static str,
  pub required: bool,
  pub description: &'static str,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allowed_values: Option<Vec<&'static str>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub items: Option<Box<FieldDef>>,
}

#[utoipa::path(
  get,
  path = "/api/v1/ui/builder-config",
  tag = "metadata",
  responses(
    (
      status = 200,
      description = "UI builder configuration — operator palette, root vars, enums, and rule field definitions for dynamic form generation.",
      body = crate::interfaces::http::openapi::BuilderConfigResponseDoc
    )
  )
)]
pub async fn builder_config() -> Json<BuilderConfigResponse> {
  let mut operator_groups = BTreeMap::new();
  for (label, operators) in RuleExpression::operator_groups() {
    operator_groups.insert(label, operators.to_vec());
  }

  let mut enums: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
  enums.insert("state.mode", vec!["staged", "active", "suspended", "deactivated"]);
  enums.insert("enforcement.action", vec!["allow", "review", "block", "tag_only"]);
  enums.insert(
    "enforcement.severity",
    vec!["none", "low", "moderate", "high", "very_high", "catastrophic"],
  );
  enums.insert("scope.channels", Channel::known_values().to_vec());
  enums.insert("signals.flags.*", vec!["unknown", "no", "yes"]);

  Json(BuilderConfigResponse {
    operator_groups,
    root_vars: JSONLOGIC_ROOT_VARS.to_vec(),
    enums,
    rule_schema_version: RULE_SCHEMA_VERSION,
    rule_fields: vec![
      FieldDef {
        path: "identity.name",
        kind: "string",
        required: true,
        description: "Human-readable display name for the rule",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "identity.code",
        kind: "string",
        required: false,
        description: "Environment-agnostic business identifier (e.g. FRAUD-HV-001)",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "identity.description",
        kind: "string",
        required: false,
        description: "Detailed explanation of the rule's criteria and intent",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "identity.author",
        kind: "string",
        required: true,
        description: "Entity or team responsible for maintaining the rule",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "identity.version",
        kind: "semver",
        required: true,
        description: "Semantic version tracking the rule's iterations",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "identity.tags",
        kind: "array",
        required: false,
        description: "Categorical labels for filtering and aggregation",
        allowed_values: None,
        items: Some(Box::new(FieldDef {
          path: "identity.tags[]",
          kind: "string",
          required: false,
          description: "Tag value",
          allowed_values: None,
          items: None,
        })),
      },
      FieldDef {
        path: "state.mode",
        kind: "enum",
        required: true,
        description: "Operational lifecycle mode of the rule",
        allowed_values: Some(vec!["staged", "active", "suspended", "deactivated"]),
        items: None,
      },
      FieldDef {
        path: "schedule.active_from_ms",
        kind: "timestamp_ms",
        required: false,
        description: "Inclusive start timestamp for rule execution eligibility",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "schedule.active_until_ms",
        kind: "timestamp_ms",
        required: false,
        description: "Exclusive end timestamp for rule execution eligibility",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "rollout.percent",
        kind: "number",
        required: true,
        description: "Percentage of traffic (0-100) subjected to this rule",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "scope.channels",
        kind: "array",
        required: false,
        description: "Event channels this rule applies to (empty = all channels)",
        allowed_values: None,
        items: Some(Box::new(FieldDef {
          path: "scope.channels[]",
          kind: "string",
          required: false,
          description: "Channel name",
          allowed_values: None,
          items: None,
        })),
      },
      FieldDef {
        path: "evaluation.condition",
        kind: "expression",
        required: true,
        description: "Guard expression evaluated before the main logic",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "evaluation.logic",
        kind: "expression",
        required: true,
        description: "Main logic expression evaluated against the event",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "enforcement.action",
        kind: "enum",
        required: true,
        description: "Operational directive when the rule triggers",
        allowed_values: Some(vec!["allow", "review", "block", "tag_only"]),
        items: None,
      },
      FieldDef {
        path: "enforcement.severity",
        kind: "enum",
        required: true,
        description: "Criticality classification for monitoring and escalation",
        allowed_values: Some(vec!["none", "low", "moderate", "high", "very_high", "catastrophic"]),
        items: None,
      },
      FieldDef {
        path: "enforcement.score_impact",
        kind: "number",
        required: true,
        description: "Numerical weight added to the aggregate risk score",
        allowed_values: None,
        items: None,
      },
      FieldDef {
        path: "enforcement.tags",
        kind: "array",
        required: false,
        description: "Categorical metadata for downstream telemetry and grouping",
        allowed_values: None,
        items: Some(Box::new(FieldDef {
          path: "enforcement.tags[]",
          kind: "string",
          required: false,
          description: "Tag value",
          allowed_values: None,
          items: None,
        })),
      },
      FieldDef {
        path: "enforcement.cooldown_ms",
        kind: "number",
        required: false,
        description: "Suppression window in milliseconds to reduce repeated hits",
        allowed_values: None,
        items: None,
      },
    ],
  })
}
