pub mod domain;
pub mod ports;
pub mod services;

pub use ports::RuleExecutorPort;

// Engine Edition
pub const ENGINE_NAME: &str = "Red Velvet Engine";
pub const ENGINE_CODENAME: &str = "Black Cherry";
pub const ENGINE_EMOJI: &str = "🍒";

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// "Red Velvet Engine — Black Cherry 🍒"
pub const ENGINE_EDITION: &str = concat!("Red Velvet Engine", " — ", "Black Cherry", " ", "🍒");
