use axum::{
  Json,
  extract::{Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use rve_core::domain::rule::Rule;
use serde_json::Value;

use crate::http::state::AppState;

use super::{
  errors::{ApiError, ApiResult, map_engine_sync_error, map_repository_error, parse_rule_id},
  patch::apply_patch,
  types::{Pagination, PaginationMeta, RuleDocumentInput, RuleListResponse, validate_rule},
};

/// Lists all rules with pagination metadata.
pub async fn list_rules(
  State(state): State<AppState>,
  Query(pagination): Query<Pagination>,
) -> ApiResult<Json<RuleListResponse>> {
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
  Json(payload): Json<RuleDocumentInput>,
) -> ApiResult<impl IntoResponse> {
  let rule = payload.into_rule(None)?;
  let created = state.rule_repo.create(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "create"))?;
  Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<Json<Rule>> {
  let id = parse_rule_id(id)?;

  state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .map(Json)
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))
}

pub async fn update_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Json(payload): Json<RuleDocumentInput>,
) -> ApiResult<Json<Rule>> {
  let id = parse_rule_id(id)?;
  let rule = payload.into_rule(Some(id))?;
  let updated = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "update"))?;
  Ok(Json(updated))
}

pub async fn patch_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Json(payload): Json<Value>,
) -> ApiResult<Json<Rule>> {
  let id = parse_rule_id(id)?;
  let mut rule =
    state.rule_repo.get(&id).await.map_err(map_repository_error)?.ok_or_else(|| {
      ApiError::NotFound("rule not found".to_owned())
    })?;

  apply_patch(&mut rule, payload)?;
  validate_rule(&rule)?;
  let saved = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;
  state.reload_rules().await.map_err(|err| map_engine_sync_error(err, "patch"))?;
  Ok(Json(saved))
}

pub async fn delete_rule(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
  let id = parse_rule_id(id)?;

  match state.rule_repo.delete(&id).await {
    Ok(()) => match state.reload_rules().await {
      Ok(()) => Ok(StatusCode::NO_CONTENT),
      Err(err) => Err(map_engine_sync_error(err, "delete")),
    },
    Err(err) => Err(map_repository_error(err)),
  }
}
