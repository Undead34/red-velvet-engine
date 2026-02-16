use crate::domain::{event::Event, rule::Rule};

pub trait RuleExecutorPort {
  fn matches(&self, rule: &Rule, event: &Event) -> Result<bool, String>;
}
