use rve_core::domain::{
  DomainError,
  common::{RuleId, Score, Severity, TimestampMs},
  rule::{
    RolloutPolicy, Rule, RuleAudit, RuleAuditError, RuleDecision, RuleDefinition, RuleEnforcement,
    RuleEvaluation, RuleExpression, RuleIdentity, RuleMode, RulePolicy, RulePolicyError,
    RuleSchedule, RuleState, RuleStateError,
  },
};

fn valid_identity() -> RuleIdentity {
  RuleIdentity {
    code: Some("FRAUD-RULE-ID-001".into()),
    name: "Test Rule".into(),
    description: Some("for tests".into()),
    version: semver::Version::new(1, 0, 0),
    author: "risk-team".into(),
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
    action: rve_core::domain::rule::RuleAction::Review,
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
  let invalid_schedule =
    RuleSchedule::new(Some(TimestampMs::new(1_000).unwrap()), Some(TimestampMs::new(900).unwrap()));
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
