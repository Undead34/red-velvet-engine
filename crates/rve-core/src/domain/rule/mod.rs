pub mod mode;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{
  common::{RuleId, Score, Severity, TimestampMs},
  rule::mode::RuleMode,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  pub id: RuleId,

  /// Identidad humana + versionado
  pub meta: RuleMeta,

  /// Estado + auditoría
  pub state: RuleState,

  /// Ventana de activación (campañas/hotfix)
  pub schedule: RuleSchedule,

  /// Aplicación gradual (gating)
  pub rollout: RolloutPolicy,

  /// Condición + lógica (evaluación)
  pub evaluation: RuleEvaluation,

  /// Qué hacer si la regla dispara (enforcement/outcome)
  pub enforcement: RuleEnforcement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMeta {
  pub name: String,
  pub description: Option<String>,
  pub version: semver::Version,
  pub autor: String,
  pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleState {
  pub mode: RuleMode,
  pub audit: RuleAudit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleAudit {
  pub created_at_ms: TimestampMs,
  pub updated_at_ms: TimestampMs,
  pub created_by: Option<String>,
  pub updated_by: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleSchedule {
  pub active_from_ms: Option<TimestampMs>,
  pub active_until_ms: Option<TimestampMs>,
}

impl RuleSchedule {
  pub fn is_within_window(&self, now_ms: u64) -> bool {
    if let Some(from) = self.active_from_ms {
      if now_ms < from.as_u64() {
        return false;
      }
    }
    if let Some(until) = self.active_until_ms {
      if now_ms >= until.as_u64() {
        return false;
      }
    }
    true
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RolloutPolicy {
  /// 0..=100
  pub percent: u8,
}

impl RolloutPolicy {
  pub fn is_allowed(&self, bucket_0_99: u8) -> bool {
    // bucket_0_99 típico: hash(key) % 100
    bucket_0_99 < self.percent.min(100)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEvaluation {
  /// Condición que define si dispara.
  pub condition: Value,
  /// La regla en sí.
  pub logic: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEnforcement {
  /// Impacto al score total
  pub score_impact: Score,

  /// Qué sugiere hacer la regla al disparar.
  pub action: RuleAction,

  /// Qué tan “grave” es (prioridad/dashboards).
  pub severity: Severity,

  /// Labels para agrupar (network/device/kyc/etc).
  pub tags: Vec<String>,

  /// Evitar spamear hits por la misma key (si implementas throttling).
  pub cooldown_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
  Allow,
  Review,
  Block,
  TagOnly,
}
