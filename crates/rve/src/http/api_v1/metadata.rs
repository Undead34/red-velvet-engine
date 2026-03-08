use std::collections::BTreeMap;

use axum::Json;
use rve_core::domain::rule::JSONLOGIC_ROOT_VARS;
use serde::Serialize;
use serde_json::{Value, json};

use rve_core::PKG_VERSION;

const FIELDS_VERSION: &str = "2026-03-08";
const RULE_SCHEMA_VERSION: &str = "2026-02-01";

#[derive(Serialize)]
pub struct FieldsResponse {
  pub data: Vec<FieldMetadata>,
  pub version: &'static str,
}

#[derive(Serialize)]
pub struct FieldMetadata {
  pub path: &'static str,
  pub label: &'static str,
  #[serde(rename = "type")]
  pub kind: &'static str,
  pub allowed_operators: Vec<&'static str>,
  pub allowed_values: Option<Vec<&'static str>>,
  pub examples: Vec<Value>,
  pub group: &'static str,
  pub description: &'static str,
}

#[utoipa::path(
  get,
  path = "/api/v1/metadata/fields",
  tag = "metadata",
  responses(
    (
      status = 200,
      description = "Supported fields for the rule builder. Prefer canonical paths (`payload.*`, `features.*`, `signals.*`). `transaction.*` and `device.*` are extension-derived aliases from `payload.extensions`.",
      body = crate::http::openapi::FieldsResponseDoc
    )
  )
)]
pub async fn fields() -> Json<FieldsResponse> {
  Json(FieldsResponse { data: supported_fields(), version: FIELDS_VERSION })
}

#[derive(Serialize)]
pub struct ContractResponse {
  pub contract_version: &'static str,
  pub api_version: &'static str,
  pub rule_schema_version: &'static str,
  pub enums: BTreeMap<String, Vec<&'static str>>,
  pub jsonlogic: JsonLogicContract,
}

#[derive(Serialize)]
pub struct JsonLogicContract {
  pub root_vars: Vec<&'static str>,
}

#[utoipa::path(
  get,
  path = "/api/v1/metadata/contract",
  tag = "metadata",
  responses(
    (
      status = 200,
      description = "Runtime contract metadata, including allowed enums and JSONLogic root variables.",
      body = crate::http::openapi::ContractResponseDoc
    )
  )
)]
pub async fn contract() -> Json<ContractResponse> {
  let mut enums = BTreeMap::new();
  enums.insert("state.mode".to_owned(), vec!["staged", "active", "suspended", "deactivated"]);
  enums.insert("enforcement.action".to_owned(), vec!["allow", "review", "block", "tag_only"]);
  enums.insert(
    "enforcement.severity".to_owned(),
    vec!["none", "low", "moderate", "high", "very_high", "catastrophic"],
  );
  enums.insert("signals.flags.*".to_owned(), vec!["unknown", "no", "yes"]);

  Json(ContractResponse {
    contract_version: PKG_VERSION,
    api_version: "v1",
    rule_schema_version: RULE_SCHEMA_VERSION,
    enums,
    jsonlogic: JsonLogicContract { root_vars: JSONLOGIC_ROOT_VARS.to_vec() },
  })
}

fn supported_fields() -> Vec<FieldMetadata> {
  vec![
    FieldMetadata {
      path: "payload.money.value",
      label: "Money Value",
      kind: "number",
      allowed_operators: vec![">", ">=", "<", "<=", "==", "!=", "!==", "==="],
      allowed_values: None,
      examples: vec![json!(100), json!(5000)],
      group: "payload.money",
      description: "Monto de la transaccion.",
    },
    FieldMetadata {
      path: "payload.money.ccy",
      label: "Currency",
      kind: "string",
      allowed_operators: vec!["==", "===", "!=", "!==", "in"],
      allowed_values: None,
      examples: vec![json!("USD"), json!("EUR")],
      group: "payload.money",
      description: "Codigo de moneda ISO-4217.",
    },
    FieldMetadata {
      path: "payload.parties.originator.country",
      label: "Originator Country",
      kind: "string",
      allowed_operators: vec!["==", "===", "!=", "!==", "in"],
      allowed_values: None,
      examples: vec![json!("US"), json!("MX")],
      group: "payload.parties.originator",
      description: "Pais ISO-3166 alpha-2 del originador.",
    },
    FieldMetadata {
      path: "features.fin.current_hour_count",
      label: "Current Hour Count",
      kind: "number",
      allowed_operators: vec![">", ">=", "<", "<=", "==", "!=", "!==", "==="],
      allowed_values: None,
      examples: vec![json!(1), json!(5), json!(12)],
      group: "features.fin",
      description: "Cantidad de transacciones en la hora actual.",
    },
    FieldMetadata {
      path: "features.fin.last_seen_at",
      label: "Last Seen At",
      kind: "timestamp_ms",
      allowed_operators: vec![">", ">=", "<", "<=", "==", "!=", "!==", "==="],
      allowed_values: None,
      examples: vec![json!(1730000000000u64)],
      group: "features.fin",
      description: "Ultimo timestamp conocido del cliente en epoch ms.",
    },
    FieldMetadata {
      path: "signals.flags.vpn",
      label: "VPN Flag",
      kind: "enum",
      allowed_operators: vec!["==", "===", "!=", "!=="],
      allowed_values: Some(vec!["unknown", "no", "yes"]),
      examples: vec![json!("yes")],
      group: "signals.flags",
      description: "Flag de riesgo asociado al uso de VPN.",
    },
    FieldMetadata {
      path: "signals.flags.proxy",
      label: "Proxy Flag",
      kind: "enum",
      allowed_operators: vec!["==", "!==", "!=", "==="],
      allowed_values: Some(vec!["unknown", "no", "yes"]),
      examples: vec![json!("no")],
      group: "signals.flags",
      description: "Flag de riesgo asociado al uso de proxy.",
    },
    FieldMetadata {
      path: "device.trust_score",
      label: "Device Trust Score",
      kind: "number",
      allowed_operators: vec![">", ">=", "<", "<=", "==", "!=", "!==", "==="],
      allowed_values: None,
      examples: vec![json!(0.2), json!(0.8)],
      group: "extensions.device",
      description: "Score de confianza del dispositivo en `payload.extensions.device.trust_score` (opcional).",
    },
    FieldMetadata {
      path: "transaction.amount",
      label: "Transaction Amount",
      kind: "number",
      allowed_operators: vec![">", ">=", "<", "<=", "==", "!=", "!==", "==="],
      allowed_values: None,
      examples: vec![json!(1500), json!(7500)],
      group: "extensions.transaction",
      description: "Monto en `payload.extensions.transaction.amount` (opcional, no sustituye `payload.money.value`).",
    },
  ]
}
