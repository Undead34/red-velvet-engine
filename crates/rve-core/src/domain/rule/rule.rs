use serde::{Deserialize, Serialize};

use crate::domain::{DomainError, common::RuleId};

use super::{RuleDecision, RuleDefinition, RuleIdentity, RuleMode, RulePolicy};

/// A fraud detection rule.
///
/// `Rule` acts as the coordinator for the engine's core components. It ties
/// together business metadata, execution constraints, evaluation logic, and
/// the resulting system actions into a single, cohesive unit.
/// The rule's constructor enforces validation boundaries on policy and
/// definition so invalid rules cannot be instantiated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  /// System-level unique identifier for storage and referencing.
  pub id: RuleId,

  /// Human-readable identity, tracking codes, and organizational tags.
  identity: RuleIdentity,

  /// Deployment controls dictating execution eligibility (state, rollout, schedule).
  policy: RulePolicy,

  /// The logical expression evaluated against the incoming event payload.
  definition: RuleDefinition,

  /// The actions and risk scoring applied upon a positive evaluation.
  outcome: RuleDecision,
}

impl Rule {
  /// Creates a validated rule aggregate.
  ///
  /// This is the rule-level constructor and the last line of defence for
  /// invalid policy or definition payloads.
  pub fn new(
    id: RuleId,
    identity: RuleIdentity,
    policy: RulePolicy,
    definition: RuleDefinition,
    outcome: RuleDecision,
  ) -> Result<Self, DomainError> {
    policy.validate()?;
    definition.validate()?;

    Ok(Self { id, identity, policy, definition, outcome })
  }

  /// Returns `true` when the policy allows execution at `now_ms` and for bucket.
  ///
  /// This method is the aggregate guard used by the engine before evaluating
  /// the rule definition.
  pub fn is_executable(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.policy.can_execute(now_ms, bucket_0_99)
  }

  /// Moves the rule lifecycle mode forward/backward according to `RuleMode` rules.
  ///
  /// Domain errors are returned as `DomainError`; in practice this is backed by
  /// `RulePolicyError` when transition constraints fail.
  pub fn transition_to(&mut self, mode: RuleMode) -> Result<(), DomainError> {
    self.policy.transition_to(mode)?;
    Ok(())
  }

  /// Replaces the current policy after validation.
  ///
  /// This method is intentionally explicit so policy changes are always
  /// validated at the aggregate boundary.
  pub fn set_policy(&mut self, policy: RulePolicy) -> Result<(), DomainError> {
    policy.validate()?;
    self.policy = policy;
    Ok(())
  }

  pub fn identity(&self) -> &RuleIdentity {
    &self.identity
  }

  pub fn policy(&self) -> &RulePolicy {
    &self.policy
  }

  pub fn definition(&self) -> &RuleDefinition {
    &self.definition
  }

  pub fn outcome(&self) -> &RuleDecision {
    &self.outcome
  }

  pub fn state(&self) -> &super::RuleState {
    &self.policy.state
  }

  pub fn schedule(&self) -> &super::RuleSchedule {
    &self.policy.schedule
  }

  pub fn rollout(&self) -> &super::RolloutPolicy {
    &self.policy.rollout
  }

  pub fn evaluation(&self) -> &crate::domain::rule::RuleEvaluation {
    &self.definition.evaluation
  }

  pub fn enforcement(&self) -> &crate::domain::rule::RuleEnforcement {
    &self.outcome.enforcement
  }

  pub fn is_active_mode(&self) -> bool {
    self.policy.is_active_mode()
  }
}

#[cfg(test)]
mod tests {
  use crate::domain::{
    DomainError,
    common::{RuleId, Score, Severity, TimestampMs},
  };

  use crate::domain::rule::audit::RuleAuditError;
  use crate::domain::rule::{
    RolloutPolicy, Rule, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement, RuleEvaluation,
    RuleExpression, RuleIdentity, RuleMode, RulePolicy, RulePolicyError, RuleSchedule, RuleState,
    RuleStateError,
  };

  fn valid_identity() -> RuleIdentity {
    RuleIdentity {
      code: Some("FRAUD-RULE-ID-001".into()),
      name: "Test Rule".into(),
      description: Some("for tests".into()),
      version: semver::Version::new(1, 0, 0),
      autor: "risk-team".into(),
      tags: Some(vec!["test".into()]),
    }
  }

  fn valid_state() -> RuleState {
    RuleState::new(
      RuleMode::Active,
      RuleAudit {
        created_at_ms: TimestampMs::new(1_700_000_000_000).unwrap(),
        updated_at_ms: TimestampMs::new(1_700_000_000_001).unwrap(),
        created_by: Some("qa".into()),
        updated_by: Some("qa".into()),
      },
    )
    .unwrap()
  }

  fn valid_definition() -> RuleDefinition {
    RuleDefinition::new(
      RuleEvaluation::new(
        RuleExpression::new(serde_json::json!(true)).unwrap(),
        RuleExpression::new(serde_json::json!({ "==": [1, 1] })).unwrap(),
      )
      .unwrap(),
    )
    .unwrap()
  }

  fn valid_outcome() -> RuleDecision {
    RuleDecision::new(RuleEnforcement {
      score_impact: Score::new(5.5).unwrap(),
      action: crate::domain::rule::RuleAction::Review,
      severity: Severity::High,
      tags: vec!["fraud".into()],
      cooldown_ms: None,
    })
  }

  fn invalid_audit() -> RuleAudit {
    RuleAudit {
      created_at_ms: TimestampMs::new(2_000).unwrap(),
      updated_at_ms: TimestampMs::new(1_000).unwrap(),
      created_by: Some("qa".into()),
      updated_by: Some("qa".into()),
    }
  }

  #[test]
  fn rejects_invalid_state_in_rule_constructor() {
    let invalid_state = RuleState::new(RuleMode::Active, invalid_audit());
    assert!(invalid_state.is_err());

    let invalid_policy = RulePolicy {
      state: RuleState { mode: RuleMode::Active, audit: invalid_audit() },
      schedule: RuleSchedule::new(None, None).unwrap(),
      rollout: RolloutPolicy::new(100).unwrap(),
    };

    let rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      invalid_policy,
      valid_definition(),
      valid_outcome(),
    );

    assert!(matches!(
      rule,
      Err(DomainError::RulePolicy(RulePolicyError::State(RuleStateError::Audit(
        RuleAuditError::InvalidTimestampOrder { .. }
      ))))
    ));
  }

  #[test]
  fn rejects_invalid_schedule_in_rule_constructor() {
    let invalid_schedule = RuleSchedule::new(
      Some(TimestampMs::new(1_000).unwrap()),
      Some(TimestampMs::new(900).unwrap()),
    );
    assert!(invalid_schedule.is_err());

    let invalid_policy = RulePolicy {
      state: valid_state(),
      schedule: RuleSchedule {
        active_from_ms: Some(TimestampMs::new(1_000).unwrap()),
        active_until_ms: Some(TimestampMs::new(900).unwrap()),
      },
      rollout: RolloutPolicy::new(100).unwrap(),
    };

    let rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      invalid_policy,
      valid_definition(),
      valid_outcome(),
    );

    assert!(matches!(rule, Err(DomainError::RulePolicy(RulePolicyError::Schedule(_)))));
  }

  #[test]
  fn rejects_invalid_rollout_in_rule_constructor() {
    let invalid_policy = RulePolicy {
      state: valid_state(),
      schedule: RuleSchedule::new(None, None).unwrap(),
      rollout: RolloutPolicy { percent: 101 },
    };

    let rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      invalid_policy,
      valid_definition(),
      valid_outcome(),
    );

    assert!(matches!(rule, Err(DomainError::RulePolicy(RulePolicyError::Rollout(_)))));
  }

  #[test]
  fn validates_rule_transitions() {
    let mut rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      RulePolicy::new(
        RuleState::new(
          RuleMode::Staged,
          RuleAudit {
            created_at_ms: TimestampMs::new(1_700_000_000_000).unwrap(),
            updated_at_ms: TimestampMs::new(1_700_000_000_001).unwrap(),
            created_by: Some("qa".into()),
            updated_by: Some("qa".into()),
          },
        )
        .unwrap(),
        RuleSchedule::new(None, None).unwrap(),
        RolloutPolicy::new(100).unwrap(),
      )
      .unwrap(),
      valid_definition(),
      valid_outcome(),
    )
    .unwrap();

    assert!(rule.transition_to(RuleMode::Active).is_ok());
    assert!(rule.transition_to(RuleMode::Suspended).is_ok());
    assert!(rule.transition_to(RuleMode::Active).is_ok());
    assert!(rule.transition_to(RuleMode::Deactivated).is_ok());
    assert!(rule.transition_to(RuleMode::Active).is_err());
  }

  #[test]
  fn accepts_valid_rule_and_checks_executable() {
    let mut rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      RulePolicy::new(
        RuleState::new(
          RuleMode::Active,
          RuleAudit {
            created_at_ms: TimestampMs::new(1_700_000_000_000).unwrap(),
            updated_at_ms: TimestampMs::new(1_700_000_000_001).unwrap(),
            created_by: Some("qa".into()),
            updated_by: Some("qa".into()),
          },
        )
        .unwrap(),
        RuleSchedule::new(None, None).unwrap(),
        RolloutPolicy::new(50).unwrap(),
      )
      .unwrap(),
      valid_definition(),
      valid_outcome(),
    )
    .unwrap();

    let scheduled_rule = Rule::new(
      RuleId::new_v7(),
      valid_identity(),
      RulePolicy::new(
        RuleState::new(
          RuleMode::Active,
          RuleAudit {
            created_at_ms: TimestampMs::new(1_700_000_000_000).unwrap(),
            updated_at_ms: TimestampMs::new(1_700_000_000_001).unwrap(),
            created_by: Some("qa".into()),
            updated_by: Some("qa".into()),
          },
        )
        .unwrap(),
        RuleSchedule::new(
          Some(TimestampMs::new(1_800_000_000_000).unwrap()),
          Some(TimestampMs::new(1_800_000_000_100).unwrap()),
        )
        .unwrap(),
        RolloutPolicy::new(50).unwrap(),
      )
      .unwrap(),
      valid_definition(),
      valid_outcome(),
    )
    .unwrap();

    assert!(rule.is_executable(1_800_000_000_000, 10));
    assert!(!rule.is_executable(1_700_000_000_000, 99));

    assert!(!scheduled_rule.is_executable(1_799_999_999_999, 10));
    assert!(!scheduled_rule.is_executable(1_800_000_000_050, 99));
    assert!(scheduled_rule.is_executable(1_800_000_000_050, 10));

    rule.transition_to(RuleMode::Suspended).unwrap();
    assert!(!rule.is_executable(1_800_000_000_000, 10));
  }
}
