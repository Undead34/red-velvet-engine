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
    crate::http::api_v1::engine::status,
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
      DecisionResponseDoc,
      DecisionHitDoc,
      EngineStatusResponseDoc,
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
    (name = "decisions", description = "Decision endpoint"),
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
#[schema(
  example = json!({
    "code": "unprocessable_entity",
    "message": "validation failed",
    "validation": {
      "errors": [
        {"path": "enforcement.score_impact", "message": "must be between 1.0 and 10.0"}
      ],
      "warnings": []
    }
  })
)]
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
  pub author: String,
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
  /// Numeric risk impact applied when the rule matches. Range: `1.0..=10.0`.
  pub score_impact: f32,
  /// Action applied when the rule matches. Enum: `allow|review|block|tag_only`.
  pub action: String,
  /// Severity classification for observability/escalation.
  pub severity: String,
  pub tags: Vec<String>,
  /// Optional cooldown window in milliseconds. Range: `1..=86_400_000` when present.
  pub cooldown_ms: Option<u64>,
  /// Ordered runtime function pipeline (domain-level abstraction).
  pub functions: Vec<Value>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReloadResponseDoc {
  pub status: String,
  pub message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EngineStatusResponseDoc {
  pub ruleset_version: u64,
  pub loaded_rules: u32,
  pub repository_rules: u32,
  pub last_reload_at_ms: Option<u64>,
  pub last_reload_error: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(
  example = json!({
    "meta": {
      "code": "FRAUD-HV-001",
      "name": "High value transaction",
      "description": "Flags large transaction amounts",
      "version": "1.0.0",
      "author": "RiskOps",
      "tags": ["high_value"]
    },
    "state": {
      "mode": "active",
      "audit": {
        "created_at_ms": 1730000000000u64,
        "updated_at_ms": 1730000001000u64,
        "created_by": "riskops",
        "updated_by": "riskops"
      }
    },
    "schedule": {
      "active_from_ms": 1730000000000u64
    },
    "rollout": { "percent": 100 },
    "evaluation": {
      "condition": true,
      "logic": {
        "and": [
          { ">": [ { "var": "payload.money.value" }, 1000 ] },
          { ">=": [ { "var": "features.fin.current_hour_count" }, 1 ] }
        ]
      }
    },
    "enforcement": {
      "score_impact": 6.5,
      "action": "review",
      "severity": "high",
      "tags": ["financial_fraud"],
      "cooldown_ms": 60000,
      "functions": [
        {
          "kind": "validate",
          "config": {
            "rules": [
              { "logic": true, "message": "ok" }
            ]
          }
        }
      ]
    }
  })
)]
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
#[schema(
  example = json!({
    "header": {
      "timestamp": "2026-03-08T00:00:00Z",
      "source": "checkout",
      "event_id": "123e4567-e89b-12d3-a456-426614174000",
      "instrument": "card",
      "channel": "web"
    },
    "context": {
      "geo": { "country": "US", "lat": 40.71, "lon": -74.01 },
      "net": { "source_ip": "203.0.113.10" },
      "env": { "device_id": "dev_1", "session_id": "sess_1" }
    },
    "features": {
      "fin": {
        "first_seen_at": 1730000000000u64,
        "last_seen_at": 1730000005000u64,
        "last_declined_at": Value::Null,
        "total_successful_txns": 12u64,
        "total_declined_txns": 1u64,
        "total_amount_spent": 150000u64,
        "max_ticket_ever": 45000u64,
        "consecutive_failed_logins": 0,
        "consecutive_declines": 0,
        "current_hour_count": 2,
        "current_hour_amount": 1500u64,
        "current_day_count": 3,
        "current_day_amount": 1500u64,
        "known_ips": ["203.0.113.10"],
        "known_devices": ["dev_1"]
      }
    },
    "signals": { "flags": {} },
    "payload": {
      "money": { "value": 1500.0, "ccy": "USD" },
      "parties": {
        "originator": {
          "entity_type": "individual",
          "acct": "acc_1",
          "country": "US",
          "bank": "bank_1",
          "kyc": "tier_2",
          "watchlist": "no",
          "sanctions_score": 0.01
        },
        "beneficiary": {
          "entity_type": "business",
          "acct": "acc_2",
          "country": "US",
          "bank": "bank_2",
          "kyc": "tier_3",
          "watchlist": "no",
          "sanctions_score": 0.0
        }
      },
      "extensions": {
        "transaction": { "amount": 1500 },
        "device": { "trust_score": 0.7 }
      }
    }
  })
)]
pub struct DecisionRequestDoc {
  /// Header context.
  /// `event_id` is optional, but when present it must be a valid UUID string.
  pub header: Value,
  /// Snapshot context (`geo`, `net`, `env`).
  pub context: Value,
  /// Historical anti-fraud features.
  /// Current contract expects a complete `features.fin` object, not a partial patch.
  pub features: Value,
  /// Flags map (`signals.flags`).
  pub signals: Value,
  /// Transaction payload.
  /// `payload.parties.originator` and `payload.parties.beneficiary` are required.
  pub payload: Value,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(
  example = json!({
    "score": 6.5,
    "outcome": "review",
    "hits": [
      {
        "rule_id": "01952031-1a77-7f0c-9f3c-bfd27d450001",
        "action": "review",
        "severity": "high",
        "score_delta": 6.5,
        "explanation": "High value transaction",
        "tags": ["financial_fraud"]
      }
    ],
    "evaluated_rules": 2,
    "executed_rules": 1,
    "rollout_bucket": 42
  })
)]
pub struct DecisionResponseDoc {
  /// Aggregated risk score from matched rules.
  pub score: f32,
  /// Final outcome.
  /// When no rules match, outcome is `allow`.
  pub outcome: String,
  /// Rule hits that matched for this event.
  pub hits: Vec<DecisionHitDoc>,
  pub evaluated_rules: u32,
  pub executed_rules: u32,
  pub rollout_bucket: u8,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DecisionHitDoc {
  pub rule_id: String,
  pub action: String,
  pub severity: String,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FieldsResponseDoc {
  pub data: Vec<FieldMetadataDoc>,
  pub version: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(
  example = json!({
    "path": "payload.money.value",
    "label": "Money Value",
    "type": "number",
    "allowed_operators": [">", ">=", "<", "<=", "==", "!=", "!==", "==="],
    "allowed_values": Value::Null,
    "examples": [100, 5000],
    "group": "payload.money",
    "description": "Monto de la transaccion."
  })
)]
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
#[schema(
  example = json!({
    "contract_version": "0.1.0",
    "api_version": "v1",
    "rule_schema_version": "2026-02-01",
    "enums": {
      "state.mode": ["staged", "active", "suspended", "deactivated"],
      "enforcement.action": ["allow", "review", "block", "tag_only"]
    },
    "jsonlogic": {
      "root_vars": ["event", "payload", "context", "features", "signals", "extensions", "transaction", "device"]
    }
  })
)]
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
