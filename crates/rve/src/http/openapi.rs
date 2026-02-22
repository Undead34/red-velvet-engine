use std::collections::BTreeMap;

use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
  paths(
    crate::http::health::handler,
    crate::http::api_v1::rules::handlers::list_rules,
    crate::http::api_v1::rules::handlers::create_rule,
    crate::http::api_v1::rules::handlers::get_rule,
    crate::http::api_v1::rules::handlers::update_rule,
    crate::http::api_v1::rules::handlers::patch_rule,
    crate::http::api_v1::rules::handlers::delete_rule,
    crate::http::api_v1::engine::reload,
    crate::http::api_v1::decisions::create_decision,
    crate::http::api_v1::metadata::fields,
    crate::http::api_v1::metadata::contract
  ),
  components(
    schemas(
      HealthResponse,
      ErrorResponse,
      ValidationReport,
      ValidationIssue,
      RuleDoc,
      RuleMetaDoc,
      RuleStateDoc,
      RuleAuditDoc,
      RuleScheduleDoc,
      RolloutPolicyDoc,
      RuleEvaluationDoc,
      RuleEnforcementDoc,
      ReloadResponseDoc,
      RuleListResponseDoc,
      PaginationMetaDoc,
      RuleDocumentInputDoc,
      DecisionRequestDoc,
      FieldsResponseDoc,
      FieldMetadataDoc,
      ContractResponseDoc,
      JsonLogicContractDoc
    )
  ),
  tags(
    (name = "health", description = "Health endpoints"),
    (name = "rules", description = "Rules CRUD and validation"),
    (name = "engine", description = "Engine lifecycle operations"),
    (name = "decisions", description = "Decision endpoint skeleton"),
    (name = "metadata", description = "Builder metadata and contract")
  )
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
  Json(ApiDoc::openapi())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
  pub status: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
  pub code: String,
  pub message: String,
  pub validation: Option<ValidationReport>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ValidationReport {
  pub errors: Vec<ValidationIssue>,
  pub warnings: Vec<ValidationIssue>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ValidationIssue {
  pub path: String,
  pub message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleListResponseDoc {
  pub data: Vec<RuleDoc>,
  pub pagination: PaginationMetaDoc,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginationMetaDoc {
  pub page: u32,
  pub limit: u32,
  pub total: u32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleDoc {
  pub id: String,
  pub meta: RuleMetaDoc,
  pub state: RuleStateDoc,
  pub schedule: RuleScheduleDoc,
  pub rollout: RolloutPolicyDoc,
  pub evaluation: RuleEvaluationDoc,
  pub enforcement: RuleEnforcementDoc,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleMetaDoc {
  pub code: Option<String>,
  pub name: String,
  pub description: Option<String>,
  pub version: String,
  pub autor: String,
  pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleStateDoc {
  pub mode: String,
  pub audit: RuleAuditDoc,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleAuditDoc {
  pub created_at_ms: u64,
  pub updated_at_ms: u64,
  pub created_by: Option<String>,
  pub updated_by: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleScheduleDoc {
  pub active_from_ms: Option<u64>,
  pub active_until_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RolloutPolicyDoc {
  pub percent: u8,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleEvaluationDoc {
  pub condition: Value,
  pub logic: Value,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleEnforcementDoc {
  pub score_impact: f32,
  pub action: String,
  pub severity: String,
  pub tags: Vec<String>,
  pub cooldown_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReloadResponseDoc {
  pub status: String,
  pub message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RuleDocumentInputDoc {
  pub id: Option<String>,
  pub meta: RuleMetaDoc,
  pub state: RuleStateDoc,
  pub schedule: RuleScheduleDoc,
  pub rollout: RolloutPolicyDoc,
  pub evaluation: RuleEvaluationDoc,
  pub enforcement: RuleEnforcementDoc,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DecisionRequestDoc {
  pub event: Value,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FieldsResponseDoc {
  pub data: Vec<FieldMetadataDoc>,
  pub version: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FieldMetadataDoc {
  pub path: String,
  pub label: String,
  #[serde(rename = "type")]
  pub kind: String,
  pub allowed_operators: Vec<String>,
  pub allowed_values: Option<Vec<String>>,
  pub examples: Vec<Value>,
  pub group: String,
  pub description: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ContractResponseDoc {
  pub contract_version: String,
  pub api_version: String,
  pub rule_schema_version: String,
  pub enums: BTreeMap<String, Vec<String>>,
  pub jsonlogic: JsonLogicContractDoc,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct JsonLogicContractDoc {
  pub root_vars: Vec<String>,
}
