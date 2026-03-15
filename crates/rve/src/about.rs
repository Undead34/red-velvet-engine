use colored::Colorize;
use rve_core::{
  ENGINE_CODENAME, ENGINE_EDITION, ENGINE_EMOJI, ENGINE_NAME, PKG_DESCRIPTION, PKG_VERSION,
};

const THIRD_PARTY_LICENSES: &str = include_str!("../../../THIRD_PARTY_LICENSES.md");

pub fn show_project_about(quiet: bool) {
  let source = option_env!("CARGO_PKG_REPOSITORY")
    .filter(|value| !value.trim().is_empty())
    .unwrap_or("(not declared)");
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
