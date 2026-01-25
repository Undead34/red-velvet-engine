use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::types::Score;

type RuleId = String;

pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub description: Option<String>,
    pub version: u16,
    pub status: RuleStatus,
    pub lifecycle: RuleLifecycle,
    pub policy: RulePolicy,
    pub when: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    Disabled,
    Enabled,
    Shadow,   // evalúa pero NO aplica policy (solo métricas)
    Archived, // solo auditoría
}

impl Default for RuleStatus {
    fn default() -> Self {
        RuleStatus::Enabled
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleLifecycle {
    // Auditoría básica
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,

    // Ventana activa (útil para campañas / hotfix)
    pub active_from_ms: Option<u64>,
    pub active_until_ms: Option<u64>,
}

impl RuleLifecycle {
    pub fn is_within_window(&self, now_ms: u64) -> bool {
        if let Some(from) = self.active_from_ms {
            if now_ms < from {
                return false;
            }
        }
        if let Some(until) = self.active_until_ms {
            if now_ms >= until {
                return false;
            }
        }
        true
    }
}

pub struct RulePolicy {
    /// Impacto al score total (positivo = más riesgo, negativo = menos).
    pub score: Score,

    /// Qué sugiere hacer la regla al disparar.
    pub action: RuleAction,

    /// Qué tan “grave” es (para priorizar, dashboards, etc.).
    pub severity: Severity,

    /// Labels útiles para agrupar (network/device/kyc/etc).
    pub tags: Vec<String>,

    /// Rollout (canary) para no prender una regla al 100% de una.
    pub rollout: RolloutPolicy,

    /// Evitar spamear hits por la misma key (si implementas throttling).
    pub cooldown_ms: Option<u64>,
}

pub enum RuleAction {
    Allow,
    Review,
    Block,
    TagOnly, // solo etiquetar, no afecta decisión
}

pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Medium
    }
}

pub struct RolloutPolicy {
    /// 0..=100
    pub percent: u8,
}

/// Helpers útiles para runtime (sin depender de ningún engine).
impl Rule {
    /// Activa = status (enabled/shadow) + ventana de tiempo.
    pub fn is_active(&self, now_ms: u64) -> bool {
        matches!(self.status, RuleStatus::Enabled | RuleStatus::Shadow)
            && self.lifecycle.is_within_window(now_ms)
    }

    pub fn is_shadow(&self) -> bool {
        matches!(self.status, RuleStatus::Shadow)
    }
}
