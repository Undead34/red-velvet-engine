//! Runtime crate for Red Velvet Engine.
//!
//! The executable-facing crate wires inbound HTTP adapters and outbound
//! infrastructure adapters around the core `rve-core` application layer.

pub mod about;
pub mod banner;
pub mod bootstrap;
pub mod error;
pub mod infrastructure;
pub mod interfaces;
pub mod logger;
pub mod version;
