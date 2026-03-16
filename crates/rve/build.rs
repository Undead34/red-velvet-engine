use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
  println!("cargo:rerun-if-changed=../../.git/HEAD");
  println!("cargo:rerun-if-changed=../../.git/refs");

  let build_unix_ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs().to_string())
    .unwrap_or_else(|_| String::from("unknown"));

  println!("cargo:rustc-env=RVE_BUILD_UNIX_TS={build_unix_ts}");

  if let Ok(profile) = env::var("PROFILE") {
    println!("cargo:rustc-env=RVE_BUILD_PROFILE={profile}");
  }

  if let Ok(target) = env::var("TARGET") {
    println!("cargo:rustc-env=RVE_BUILD_TARGET={target}");
  }

  let git_sha = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .ok()
    .filter(|out| out.status.success())
    .and_then(|out| String::from_utf8(out.stdout).ok())
    .map(|s| s.trim().to_owned())
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| String::from("unknown"));

  println!("cargo:rustc-env=RVE_GIT_SHA={git_sha}");

  let source_url = Command::new("git")
    .args(["remote", "get-url", "origin"])
    .output()
    .ok()
    .filter(|out| out.status.success())
    .and_then(|out| String::from_utf8(out.stdout).ok())
    .map(|s| s.trim().to_owned())
    .filter(|s| !s.is_empty())
    .map(normalize_git_url)
    .unwrap_or_else(|| String::from("(not declared)"));

  println!("cargo:rustc-env=RVE_SOURCE_URL={source_url}");
}

fn normalize_git_url(url: String) -> String {
  if let Some(rest) = url.strip_prefix("git@github.com:") {
    return format!("https://github.com/{}", rest.trim_end_matches(".git"));
  }

  if url.starts_with("https://") {
    return url.trim_end_matches(".git").to_owned();
  }

  url
}
