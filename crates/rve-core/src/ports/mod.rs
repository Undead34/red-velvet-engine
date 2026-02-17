pub mod rule_repository;

use crate::domain::{event::Event, rule::Rule};

pub use rule_repository::{RepositoryResult, RulePage, RuleRepositoryError, RuleRepositoryPort};

pub trait RuleExecutorPort {
  fn matches(&self, rule: &Rule, event: &Event) -> Result<bool, String>;
}
