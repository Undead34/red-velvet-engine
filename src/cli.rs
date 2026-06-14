use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};
use rve_core::{ENGINE_EDITION, PKG_DESCRIPTION};

use crate::version::version_metadata;

#[derive(Parser, Debug)]
#[command(
    name = "rve",
    version,
    about = ENGINE_EDITION,
    long_about = PKG_DESCRIPTION
)]
pub struct App {
  #[command(subcommand)]
  pub command: Option<Command>,

  #[arg(long, default_value = "[::]")]
  pub host: String,

  #[arg(long, short, default_value_t = 3439)]
  pub port: u16,

  #[arg(long, short, action = ArgAction::Count)]
  pub verbose: u8,

  #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
  pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// Project and legal information.
  About {
    #[command(subcommand)]
    command: Option<AboutCommand>,
  },
}

#[derive(Subcommand, Debug)]
pub enum AboutCommand {
  /// Show third-party dependencies and their licenses.
  Licenses,
}

impl App {
  pub fn new() -> Self {
    let version = version_metadata();
    let short_version: &'static str = Box::leak(version.cli_short().into_boxed_str());
    let long_version: &'static str = Box::leak(version.cli_long().into_boxed_str());
    let matches = App::command().version(short_version).long_version(long_version).get_matches();

    App::from_arg_matches(&matches).unwrap_or_else(|err: clap::Error| err.exit())
  }
}
