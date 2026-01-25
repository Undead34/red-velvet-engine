use crate::domain::{event::Event, rule::Rule, types::Score};
use crate::ports::RuleEvaluator;

pub struct EngineService<T: RuleEvaluator> {
    evaluator: T,
}

impl<T: RuleEvaluator> EngineService<T> {
    pub fn new(evaluator: T) -> Self {
        Self { evaluator }
    }

    pub fn inspect(&self, event: &Event, rules: &[Rule]) -> EngineResult {
        let mut total_score: Score = 0;
        let mut matched_rules = Vec::new();

        for rule in rules {
            // Aquí podrías usar rule.is_active(now_ms)
            if let Ok(true) = self.evaluator.matches(rule, event) {
                if !rule.is_shadow() {
                    total_score += rule.policy.score;
                }
                matched_rules.push(rule.id.clone());
            }
        }

        EngineResult {
            score: total_score,
            hits: matched_rules,
        }
    }
}

pub struct EngineResult {
    pub score: Score,
    pub hits: Vec<String>,
}
