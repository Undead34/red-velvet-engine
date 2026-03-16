use colored::Colorize;
use rve_core::{
  ENGINE_CODENAME, ENGINE_EDITION, ENGINE_EMOJI, ENGINE_NAME, PKG_DESCRIPTION, PKG_VERSION,
};

const THIRD_PARTY_LICENSES: &str = include_str!("../../../THIRD_PARTY_LICENSES.md");

pub fn show_project_about(quiet: bool) {
  let source = option_env!("CARGO_PKG_REPOSITORY")
    .filter(|value| !value.trim().is_empty())
    .unwrap_or("(not declared)");
  let build_unix_ts = option_env!("RVE_BUILD_UNIX_TS").unwrap_or("unknown");
  let build_profile = option_env!("RVE_BUILD_PROFILE").unwrap_or("unknown");
  let build_target = option_env!("RVE_BUILD_TARGET").unwrap_or("unknown");
  let git_sha = option_env!("RVE_GIT_SHA").unwrap_or("unknown");
  let security_mode =
    if cfg!(debug_assertions) { "debug assertions enabled" } else { "release-hardening profile" };
  let panic_strategy = if cfg!(panic = "abort") { "abort" } else { "unwind" };
  let rule = "-".repeat(72);

  if quiet {
    println!("{}", ENGINE_EDITION);
    println!("{}", rule);
    println!("Project     : {}", ENGINE_NAME);
    println!("Codename    : {}", ENGINE_CODENAME);
    println!("Version     : {}", PKG_VERSION);
    println!("Package     : {}", env!("CARGO_PKG_NAME"));
    println!("Description : {}", PKG_DESCRIPTION);
    println!("License     : {}", env!("CARGO_PKG_LICENSE"));
    println!("Source      : {}", source);
    println!("Build Unix  : {}", build_unix_ts);
    println!("Build       : {} / {}", build_profile, build_target);
    println!("Git Commit  : {}", git_sha);
    println!("Security    : {}", security_mode);
    println!("Panic       : {}", panic_strategy);
    println!("Third-party : rve about licenses");
    return;
  }

  println!("{}", format!("{} {}", ENGINE_EMOJI, ENGINE_EDITION).bold().red());
  println!("{}", rule.bright_black());
  println!("{} {}", "Project".bright_black(), ENGINE_NAME.bold());
  println!("{} {}", "Codename".bright_black(), ENGINE_CODENAME.bright_red());
  println!("{} {}", "Version".bright_black(), PKG_VERSION.bold());
  println!("{} {}", "Package".bright_black(), env!("CARGO_PKG_NAME").cyan());
  println!("{} {}", "Description".bright_black(), PKG_DESCRIPTION.italic());
  println!("{} {}", "License".bright_black(), env!("CARGO_PKG_LICENSE").yellow());
  println!("{} {}", "Source".bright_black(), source.underline());
  println!("{} {}", "Build Unix".bright_black(), build_unix_ts.cyan());
  println!("{} {}", "Build".bright_black(), format!("{} / {}", build_profile, build_target).bold());
  println!("{} {}", "Git Commit".bright_black(), git_sha.green());
  println!("{} {}", "Security".bright_black(), security_mode.yellow());
  println!("{} {}", "Panic".bright_black(), panic_strategy.yellow());
  println!("{} {}", "Third-party".bright_black(), "rve about licenses".green());
}

pub fn show_licenses(quiet: bool) {
  let rule = "-".repeat(72);

  if !quiet {
    println!("{}", "Third-Party Licenses".bold().red());
    println!("{}", rule.bright_black());
  }

  println!("{}", THIRD_PARTY_LICENSES.trim());
}
