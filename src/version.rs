use std::sync::OnceLock;

use chrono::{DateTime, FixedOffset};

shadow_rs::shadow!(build);

pub struct VersionMetadata {
  semver: &'static str,
  calver: String,
  build_time: &'static str,
  commit: &'static str,
  branch: &'static str,
  dirty: bool,
}

impl VersionMetadata {
  fn new() -> Self {
    Self {
      semver: build::PKG_VERSION,
      calver: compute_calver(),
      build_time: build::BUILD_TIME_3339,
      commit: build::SHORT_COMMIT,
      branch: build::BRANCH,
      dirty: !build::GIT_CLEAN,
    }
  }

  pub fn semver(&self) -> &'static str {
    self.semver
  }

  pub fn calver(&self) -> &str {
    &self.calver
  }

  pub fn build_timestamp(&self) -> &'static str {
    self.build_time
  }

  pub fn commit(&self) -> &'static str {
    self.commit
  }

  pub fn branch(&self) -> &'static str {
    self.branch
  }

  pub fn is_dirty(&self) -> bool {
    self.dirty
  }

  pub fn cli_short(&self) -> String {
    let mut short = format!("{} ({})", self.semver, self.calver);
    if self.dirty {
      short.push_str(" dirty");
    }
    short
  }

  pub fn cli_long(&self) -> String {
    let dirty = if self.dirty { " (dirty)" } else { "" };
    format!(
      "SemVer : {semver}\nCalVer : {calver}\nBuild  : {build}\nCommit : {commit} ({branch}){dirty}",
      semver = self.semver,
      calver = self.calver,
      build = self.build_time,
      commit = self.commit,
      branch = self.branch,
      dirty = dirty,
    )
  }
}

static VERSION_METADATA: OnceLock<VersionMetadata> = OnceLock::new();

pub fn version_metadata() -> &'static VersionMetadata {
  VERSION_METADATA.get_or_init(VersionMetadata::new)
}

fn compute_calver() -> String {
  DateTime::parse_from_rfc3339(build::BUILD_TIME_3339)
    .map(|dt: DateTime<FixedOffset>| dt.format("%y.%m.%d").to_string())
    .unwrap_or_else(|_| fallback_calver())
}

fn fallback_calver() -> String {
  build::BUILD_TIME
    .get(2..10)
    .map(|slice| slice.replace('-', "."))
    .unwrap_or_else(|| "00.00.00".to_string())
}
