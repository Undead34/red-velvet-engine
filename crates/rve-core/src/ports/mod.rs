//! Outbound ports required by the core application layer.
//!
//! These traits are implemented by infrastructure adapters such as Redis-backed
//! repositories or runtime engines. The application layer depends on these
//! abstractions instead of concrete implementations.

pub mod rule_engine;
pub mod rule_repository;

pub use rule_engine::RuleEnginePort;
pub use rule_repository::RuleRepositoryPort;
