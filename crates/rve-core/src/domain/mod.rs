//! Core domain models and business logic for the fraud detection engine.
//!
//! This crate defines the fundamental building blocks of the risk platform,
//! providing a strongly-typed, validated environment for processing financial
//! events against complex fraud rules.
//!
//! # Core Aggregates
//!
//! The domain is structured around three primary pillars:
//!
//! * [`Event`]: The immutable input representing a transactional or behavioral
//!     occurrence within the system.
//! * [`Rule`]: The programmable logic unit that evaluates events to determine
//!     risk scores and enforcement actions.
//! * [`DomainError`]: The unified error surface ensuring all invariants are
//!     strictly enforced across the system.
//!
//! # Design Principles
//!
//! * **Type Safety**: Leveraging Rust's type system to ensure that invalid
//!     data (e.g., malformed currency codes or inverted schedules) cannot
//!     exist in a running system.
//! * **Deterministic Evaluation**: Rules are designed to be side-effect free,
//!     ensuring predictable outcomes for any given input.
//! * **Explicit Boundaries**: Each sub-domain (Rules, Events, Common) manages
//!     its own internal invariants, which are then rolled up into the top-level
//!     aggregates.

pub mod common;
pub mod error;
pub mod event;
pub mod rule;

pub use error::DomainError;
pub use event::Event;
pub use rule::Rule;
