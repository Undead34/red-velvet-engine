use colored::*;
use indoc::indoc;

use rve_core::{ENGINE_CODENAME, ENGINE_EMOJI, PKG_DESCRIPTION, PKG_VERSION};

pub struct ReadyMsg(String);

impl Drop for ReadyMsg {
  fn drop(&mut self) {
    println!("\n{}", self.0);
  }
}

pub fn show_banner(quiet: bool) -> Option<ReadyMsg> {
  if quiet {
    return None;
  }

  let banner = indoc! {r"
          ____          _  __     __     _            _
         |  _ \ ___  __| | \ \   / /__  | |_   _____ | |_
         | |_) / _ \/ _` |  \ \ / / _ \ | \ \ / / _ \| __|
         |  _ <  __/ (_| |   \ V /  __/ | |\ V /  __/| |_
         |_| \_\___|\__,_|    \_/ \___| |_| \_/ \___| \__|
    "};

  for line in banner.lines() {
    // iterate, to prevent color errors
    println!("{}", line.red().bold());
  }

  println!();

  // Subtítulo 100% dinámico
  let edition_info = format!("{} Edition", ENGINE_CODENAME);
  println!(
    "{:>48}",
    format!("{} {} v{}", edition_info.black().on_red(), ENGINE_EMOJI, PKG_VERSION).bold()
  );

  println!("\n{}", PKG_DESCRIPTION.bright_black().italic());
  println!("{}\n", "─".repeat(60).bright_black());

  let ready = format!("🚀 {} is hot and ready.", "RVE").green().bold();

  Some(ReadyMsg(ready.to_string()))
}
