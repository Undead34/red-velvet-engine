use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain::common::Severity;

/// Representa el puntaje numérico preciso de un riesgo de fraude (1.0 - 10.0).
/// Internamente usa un factor de escala de 100 (2 decimales suelen bastar para scoring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Score(u16); // u16 es suficiente para valores hasta 655.35

impl Score {
  const SCALE: u16 = 100;

  pub fn new(val: f32) -> Option<Self> {
    if !(1.0..=10.0).contains(&val) {
      return None;
    }
    let scaled = (val * Self::SCALE as f32).round() as u16;
    Some(Self(scaled))
  }

  pub fn as_f32(&self) -> f32 {
    self.0 as f32 / Self::SCALE as f32
  }
}

/// De Severity a Score: Usamos el valor representativo (el techo del rango).
impl From<Severity> for Score {
  fn from(severity: Severity) -> Self {
    // Safe unwrap porque Severity::value() siempre devuelve 1-10
    Self::new(severity.value() as f32).unwrap()
  }
}

/// De Score a Severity: Mapeo por rangos.
impl From<Score> for Severity {
  fn from(score: Score) -> Self {
    let val = (score.0 / Score::SCALE) as u8;
    // Reutilizamos tu lógica de from_u8 o la expandimos aquí
    Self::from_u8(val).unwrap_or(Severity::None)
  }
}

impl fmt::Display for Score {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let severity: Severity = (*self).into();
    write!(f, "{:.2} [{:?}]", self.as_f32(), severity)
  }
}
