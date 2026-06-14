use axum::{
  Json,
  extract::{Path, Query, State},
  response::IntoResponse,
};
use tracing::instrument;

use crate::http::{
  openapi::{ErrorResponse, RuleDoc, RuleListResponseDoc},
  state::AppState,
};

use super::{
  errors::{ApiError, ApiResult, map_repository_error, parse_rule_id},
  requests::Pagination,
  responses::{PaginationMeta, RuleListResponse},
  versioning::{response_version_headers, rule_version},
};

/// List rules
///
/// Returns a paginated list of existing rules in the system.
#[utoipa::path(
  get,
  path = "/api/v1/rules",
  tag = "rules",
  params(Pagination),
  responses(
    (status = 200, description = "Paginated rules", body = RuleListResponseDoc),
    (status = 500, description = "Repository error", body = ErrorResponse)
  )
)]
#[instrument(name = "http.rules.list", skip(state, pagination))]
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

/// Get a rule by ID
///
/// Retrieves the details of a specific rule using its unique identifier.
/// If the provided ID format is invalid or the rule does not exist, an appropriate error is returned.
#[utoipa::path(
  get,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "019c7d7b-dc31-7fa9-8b1b-10e4fe820cb8")
  ),
  responses(
    (status = 200, description = "Rule successfully retrieved", body = RuleDoc),
    (status = 400, description = "Invalid rule ID format provided", body = ErrorResponse),
    (status = 404, description = "Rule not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
#[instrument(name = "http.rules.get", skip(state), fields(rule_id = %id))]
pub async fn get_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<impl IntoResponse> {
  let id = parse_rule_id(id)?;

  let rule = state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))?;

  let version = rule_version(&rule)?;
  let headers = response_version_headers(&version)?;

  Ok((headers, Json(rule)))
}
