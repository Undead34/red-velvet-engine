use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use axum::http::{HeaderMap, HeaderValue, header};
use rve_core::domain::rule::Rule;

use super::errors::{ApiError, ApiResult};

pub const RULE_VERSION_HEADER: &str = "x-rule-version";

pub(super) fn rule_version(rule: &Rule) -> ApiResult<String> {
  let bytes = serde_json::to_vec(rule)
    .map_err(|_| ApiError::Internal("failed to serialize rule version".to_owned()))?;
  let mut hasher = DefaultHasher::new();
  hasher.write(&bytes);
  Ok(format!("{:016x}", hasher.finish()))
}

pub(super) fn response_version_headers(version: &str) -> ApiResult<HeaderMap> {
  let mut headers = HeaderMap::new();
  let etag = format!("\"{version}\"");

  headers.insert(
    header::ETAG,
    HeaderValue::from_str(&etag)
      .map_err(|_| ApiError::Internal("failed to encode etag header".to_owned()))?,
  );
  headers.insert(
    RULE_VERSION_HEADER,
    HeaderValue::from_str(version)
      .map_err(|_| ApiError::Internal("failed to encode version header".to_owned()))?,
  );

  Ok(headers)
}

pub(super) fn assert_if_match(headers: &HeaderMap, current_version: &str) -> ApiResult<()> {
  let Some(raw) = headers.get(header::IF_MATCH) else {
    return Ok(());
  };

  let raw = raw
    .to_str()
    .map_err(|_| ApiError::BadRequest("If-Match must be a valid ASCII header".to_owned()))?;
  let expected = normalize_etag(raw)?;

  if expected != current_version {
    return Err(ApiError::Conflict(format!(
      "version mismatch: expected {expected}, current {current_version}"
    )));
  }

  Ok(())
}

fn normalize_etag(value: &str) -> ApiResult<String> {
  let trimmed = value.trim();
  if trimmed == "*" {
    return Err(ApiError::BadRequest("If-Match wildcard is not supported".to_owned()));
  }

  let without_weak = trimmed.strip_prefix("W/").unwrap_or(trimmed).trim();
  let unquoted = without_weak
    .strip_prefix('"')
    .and_then(|v| v.strip_suffix('"'))
    .ok_or_else(|| ApiError::BadRequest("If-Match must contain a quoted version".to_owned()))?;

  if unquoted.is_empty() {
    return Err(ApiError::BadRequest("If-Match must not be empty".to_owned()));
  }

  Ok(unquoted.to_owned())
}
