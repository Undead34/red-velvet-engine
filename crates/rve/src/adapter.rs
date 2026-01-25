use datalogic_rs::DataLogic;
use rve_core::{
    RuleEvaluator,
    domain::{event::Event, rule::Rule},
};

pub struct RVEngineAdapter {
    engine: DataLogic,
}

impl RVEngineAdapter {
    pub fn new() -> Self {
        let engine = DataLogic::new();
        RVEngineAdapter { engine }
    }
}

impl RuleEvaluator for RVEngineAdapter {
    fn matches(&self, rule: &Rule, event: &Event) -> Result<bool, String> {
        let data = serde_json::to_value(event).map_err(|e| e.to_string())?;

        let compiled = self
            .engine
            .compile(&rule.when)
            .map_err(|_| "Failed to compile rule".to_string())?;

        let result = self
            .engine
            .evaluate_owned(&compiled, data)
            .map_err(|_| "Evaluation error".to_string())?;

        Ok(result.as_bool().unwrap_or(false))
    }
}
