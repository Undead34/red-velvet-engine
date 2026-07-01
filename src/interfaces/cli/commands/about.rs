use crate::about;

use super::AboutCommand;

/// Handles `rve about` commands.
pub fn run(command: Option<AboutCommand>, quiet: bool) {
  match command {
    Some(AboutCommand::Licenses) => about::show_licenses(quiet),
    None => about::show_project_about(quiet),
  }
}
