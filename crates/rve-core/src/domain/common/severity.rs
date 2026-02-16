use serde::{Deserialize, Serialize};

/// Nivel de severidad de un evento de fraude bancario.
///
/// Los niveles están ordenados de mayor a menor impacto.
/// Cada variante cubre un rango de valores numéricos (1-10),
/// siendo 10 el más crítico.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
  /// 10 – Catastrófico
  /// Quiebra de la entidad, daño reputacional irreversible,
  /// multas regulatorias extremas, riesgo sistémico.
  Catastrophic,

  /// 9 – 8 Muy alto
  /// Pérdida financiera mayor, fuga masiva de clientes,
  /// intervención del regulador, daño reputacional severo.
  VeryHigh,

  /// 7 – 6 Alto
  /// Pérdida financiera significativa, quejas formales de clientes VIP,
  /// procesos manuales de contingencia, posible multa.
  High,

  /// 5 – 4 Moderado
  /// Pérdida financiera controlable, cliente molesto,
  /// requiere investigación manual, sin multa regulatoria.
  Moderate,

  /// 3 – 2 Bajo
  /// Defecto menor, pérdida irrelevante, cliente apenas lo nota.
  /// Se resuelve con acciones automáticas o en batch.
  Low,

  /// 1 – Nulo o mínimo
  /// Impacto imperceptible para el cliente y el negocio.
  /// Solo registro informativo.
  None,
}

impl Severity {
  /// Retorna el valor numérico **representativo** de la severidad.
  /// Para rangos, se usa el límite superior (el más crítico del rango).
  pub const fn value(self) -> u8 {
    match self {
      Self::Catastrophic => 10,
      Self::VeryHigh => 9, // podría ser 8 también, 9 es el tope
      Self::High => 7,
      Self::Moderate => 5,
      Self::Low => 3,
      Self::None => 1,
    }
  }

  /// Retorna el nivel de severidad a partir de un valor numérico (1-10).
  /// Útil para mapear desde fuentes externas (APIs, bases de datos).
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      10 => Some(Self::Catastrophic),
      9 | 8 => Some(Self::VeryHigh),
      7 | 6 => Some(Self::High),
      5 | 4 => Some(Self::Moderate),
      3 | 2 => Some(Self::Low),
      1 => Some(Self::None),
      _ => None, // fuera de rango
    }
  }

  /// Descripción legible del impacto.
  pub fn description(self) -> &'static str {
    match self {
      Self::Catastrophic => "Quiebra / Daño irreparable / Ilegal",
      Self::VeryHigh => "Pérdida mayor / Fuga clientes / Regulador",
      Self::High => "Pérdida significativa / Clientes VIP afectados",
      Self::Moderate => "Pérdida controlable / Cliente molesto",
      Self::Low => "Defecto menor / Casi imperceptible",
      Self::None => "Sin impacto relevante",
    }
  }
}

/// Conversión infalible desde un entero (paniquea si está fuera de rango).
impl TryFrom<u8> for Severity {
  type Error = &'static str;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Self::from_u8(value).ok_or("Valor de severidad fuera de rango (1-10)")
  }
}
