use colored::Colorize;
use rve_core::{ENGINE_CODENAME, ENGINE_EDITION, ENGINE_EMOJI, ENGINE_NAME, PKG_DESCRIPTION};

// Genera todas las constantes build::* en tiempo de compilación
shadow_rs::shadow!(build);

const THIRD_PARTY_LICENSES: &str = include_str!("../../../THIRD_PARTY_LICENSES.md");

/// Punto de entrada para mostrar la información del proyecto.
pub fn show_project_about(quiet: bool) {
  if quiet {
    print_quiet_metadata();
  } else {
    print_colored_metadata();
  }
}

/// Imprime los metadatos del proyecto en formato texto plano (ideal para logs o scripts).
fn print_quiet_metadata() {
  let separator = "-".repeat(72);

  // Variables de entorno de Cargo y banderas de compilación locales
  let source_url = option_env!("CARGO_PKG_REPOSITORY").unwrap_or("(not declared)");
  let license = option_env!("CARGO_PKG_LICENSE").unwrap_or("(not declared)");
  let security_mode =
    if cfg!(debug_assertions) { "debug assertions enabled" } else { "release-hardening profile" };
  let panic_strategy = if cfg!(panic = "abort") { "abort" } else { "unwind" };

  println!("{}", ENGINE_EDITION);
  println!("{}", separator);
  println!("Project     : {}", ENGINE_NAME);
  println!("Codename    : {}", ENGINE_CODENAME);

  if !build::GIT_CLEAN {
    println!("Version     : {} (DIRTY)", build::PKG_VERSION);
  } else {
    println!("Version     : {}", build::PKG_VERSION);
  }

  println!("Package     : {}", build::PROJECT_NAME);
  println!("Description : {}", PKG_DESCRIPTION);
  println!("License     : {}", license);
  println!("Source      : {}", source_url);
  println!("Build Time  : {}", build::BUILD_TIME);
  println!("Build Env   : {} / {}", build::BUILD_RUST_CHANNEL, build::BUILD_TARGET);
  println!("Git Commit  : {} ({})", build::SHORT_COMMIT, build::BRANCH);
  println!("Commit Date : {}", build::COMMIT_DATE);
  println!("Rustc       : {}", build::RUST_VERSION);
  println!("Cargo       : {}", build::CARGO_VERSION);
  println!("Security    : {}", security_mode);
  println!("Panic       : {}", panic_strategy);
  println!("Third-party : rve about licenses");
}

/// Imprime los metadatos del proyecto utilizando colores y estilos (para terminales interactivas).
fn print_colored_metadata() {
  let separator = "-".repeat(72);

  let source_url = option_env!("CARGO_PKG_REPOSITORY").unwrap_or("(not declared)");
  let license = option_env!("CARGO_PKG_LICENSE").unwrap_or("(not declared)");
  let security_mode =
    if cfg!(debug_assertions) { "debug assertions enabled" } else { "release-hardening profile" };
  let panic_strategy = if cfg!(panic = "abort") { "abort" } else { "unwind" };

  println!("{}", format!("{} {}", ENGINE_EMOJI, ENGINE_EDITION).bold().red());
  println!("{}", separator.bright_black());

  println!("{} {}", "Project    ".bright_black(), ENGINE_NAME.bold());
  println!("{} {}", "Codename   ".bright_black(), ENGINE_CODENAME.bright_red());

  let dirty_status = if !build::GIT_CLEAN { " (DIRTY)".red() } else { "".normal() };
  println!("{} {}{}", "Version    ".bright_black(), build::PKG_VERSION.bold(), dirty_status);

  println!("{} {}", "Package    ".bright_black(), build::PROJECT_NAME.cyan());
  println!("{} {}", "Description".bright_black(), PKG_DESCRIPTION.italic());
  println!("{} {}", "License    ".bright_black(), license.yellow());
  println!("{} {}", "Source     ".bright_black(), source_url.underline());
  println!("{} {}", "Build Time ".bright_black(), build::BUILD_TIME.cyan());
  println!(
    "{} {}",
    "Build Env  ".bright_black(),
    format!("{} / {}", build::BUILD_RUST_CHANNEL, build::BUILD_TARGET).bold()
  );
  println!(
    "{} {}",
    "Git Commit ".bright_black(),
    format!("{} ({})", build::SHORT_COMMIT, build::BRANCH).green()
  );
  println!("{} {}", "Commit Date".bright_black(), build::COMMIT_DATE.cyan());
  println!("{} {}", "Rustc      ".bright_black(), build::RUST_VERSION.blue());
  println!("{} {}", "Cargo      ".bright_black(), build::CARGO_VERSION.blue());
  println!("{} {}", "Security   ".bright_black(), security_mode.yellow());
  println!("{} {}", "Panic      ".bright_black(), panic_strategy.yellow());
  println!("{} {}", "Third-party".bright_black(), "rve about licenses".green());
}

/// Imprime el contenido de las licencias de terceros.
pub fn show_licenses(quiet: bool) {
  if !quiet {
    let separator = "-".repeat(72);
    println!("{}", "Third-Party Licenses".bold().red());
    println!("{}", separator.bright_black());
  }

  println!("{}", THIRD_PARTY_LICENSES.trim());
}
