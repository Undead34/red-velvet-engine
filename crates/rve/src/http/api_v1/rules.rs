use axum::{
  Json,
  extract::{Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use rve_core::domain::{common::RuleId, rule::*};
use rve_core::ports::RuleRepositoryError;
use tracing::error;

use crate::http::state::{AppState, EngineSyncError};

#[derive(Deserialize)]
pub struct Pagination {
  pub page: Option<u32>,
  pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct RuleListResponse {
  pub data: Vec<Rule>,
  pub pagination: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
  pub page: u32,
  pub limit: u32,
  pub total: u32,
}

/// Lists all rules with pagination metadata.
pub async fn list_rules(
  State(state): State<AppState>,
  Query(pagination): Query<Pagination>,
) -> Result<Json<RuleListResponse>, StatusCode> {
  let page = pagination.page.filter(|p| *p > 0).unwrap_or(1);
  let limit = pagination.limit.filter(|l| *l > 0).unwrap_or(20).min(100);

  let page_data = state.rule_repo.list(page, limit).await.map_err(map_repository_error)?;

  Ok(Json(RuleListResponse {
    data: page_data.items,
    pagination: PaginationMeta { page, limit, total: page_data.total },
  }))
}

pub async fn create_rule(
  State(state): State<AppState>,
  Json(payload): Json<RuleDocument>,
) -> Result<impl IntoResponse, StatusCode> {
  let rule = payload.into_rule(None);
  let created = state.rule_repo.create(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "create"))?;
  Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Json<Rule>, StatusCode> {
  let id = parse_rule_id(id)?;

  state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
}

pub async fn update_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Json(payload): Json<RuleDocument>,
) -> Result<Json<Rule>, StatusCode> {
  let id = parse_rule_id(id)?;
  let rule = payload.into_rule(Some(id));
  let updated = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "update"))?;
  Ok(Json(updated))
}

pub async fn patch_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Json(payload): Json<Value>,
) -> Result<Json<Rule>, StatusCode> {
  let id = parse_rule_id(id)?;
  let mut rule =
    state.rule_repo.get(&id).await.map_err(map_repository_error)?.ok_or(StatusCode::NOT_FOUND)?;

  apply_patch(&mut rule, payload)?;
  let saved = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "patch"))?;
  Ok(Json(saved))
}

pub async fn delete_rule(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
  let id = match parse_rule_id(id) {
    Ok(id) => id,
    Err(status) => return status,
  };

  match state.rule_repo.delete(&id).await {
    Ok(()) => match state.reload_rules().await {
      Ok(()) => StatusCode::NO_CONTENT,
      Err(err) => map_engine_sync_error(err, "delete"),
    },
    Err(err) => map_repository_error(err),
  }
}

#[derive(Debug, Deserialize)]
pub struct RuleDocument {
  #[serde(default)]
  pub id: Option<RuleId>,
  pub meta: RuleMeta,
  pub state: RuleState,
  pub schedule: RuleSchedule,
  pub rollout: RolloutPolicy,
  pub evaluation: RuleEvaluation,
  pub enforcement: RuleEnforcement,
}

impl RuleDocument {
  fn into_rule(self, override_id: Option<RuleId>) -> Rule {
    Rule {
      id: override_id.or(self.id).unwrap_or_else(generate_rule_id),
      meta: self.meta,
      state: self.state,
      schedule: self.schedule,
      rollout: self.rollout,
      evaluation: self.evaluation,
      enforcement: self.enforcement,
    }
  }
}

fn generate_rule_id() -> RuleId {
  RuleId::new_v7()
}

fn parse_rule_id(id: String) -> Result<RuleId, StatusCode> {
  RuleId::try_from(id).map_err(|_| StatusCode::BAD_REQUEST)
}

fn apply_patch(rule: &mut Rule, patch: Value) -> Result<(), StatusCode> {
  if let Some(state) = patch.get("state") {
    if let Some(mode_value) = state.get("mode") {
      rule.state.mode =
        serde_json::from_value(mode_value.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    if let Some(audit) = state.get("audit") {
      if let Some(updated_by) = audit.get("updated_by") {
        rule.state.audit.updated_by =
          serde_json::from_value(updated_by.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
      }
      if let Some(updated_at) = audit.get("updated_at_ms") {
        rule.state.audit.updated_at_ms =
          serde_json::from_value(updated_at.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
      }
    }
  }

  if let Some(rollout) = patch.get("rollout") {
    if let Some(percent) = rollout.get("percent") {
      rule.rollout.percent =
        serde_json::from_value(percent.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
  }

  if let Some(schedule) = patch.get("schedule") {
    if let Some(active_until_ms) = schedule.get("active_until_ms") {
      rule.schedule.active_until_ms =
        serde_json::from_value(active_until_ms.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    if let Some(active_from_ms) = schedule.get("active_from_ms") {
      rule.schedule.active_from_ms =
        serde_json::from_value(active_from_ms.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
  }

  Ok(())
}

pub(crate) fn map_repository_error(error: RuleRepositoryError) -> StatusCode {
  match error {
    RuleRepositoryError::AlreadyExists(_) => StatusCode::CONFLICT,
    RuleRepositoryError::NotFound(_) => StatusCode::NOT_FOUND,
    RuleRepositoryError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
  }
}

fn map_engine_sync_error(error: EngineSyncError, operation: &str) -> StatusCode {
  error!(target: "BANNER", %error, %operation, "failed to refresh engine ruleset");
  StatusCode::INTERNAL_SERVER_ERROR
}
