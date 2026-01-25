// 3439

use crate::domain::{event::Event, rule::Rule};

pub trait RuleEvaluator {
    fn matches(&self, rule: &Rule, event: &Event) -> Result<bool, String>;
}
