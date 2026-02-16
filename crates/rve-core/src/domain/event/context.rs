use std::{collections::HashSet, net::IpAddr};

use serde::{Deserialize, Serialize};

use crate::domain::common::CountryCode;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Context {
  pub geo: GeoContext,
  pub net: NetworkContext,
  pub env: EnvironmentContext,
  pub fin: FinancialContext,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GeoContext {
  pub address: Option<String>,
  pub city: Option<String>,
  pub region: Option<String>,
  pub country: Option<CountryCode>,
  pub postal_code: Option<String>,
  pub lon: Option<f64>,
  pub lat: Option<f64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
  pub source_ip: Option<IpAddr>,
  pub destination_ip: Option<IpAddr>,
  pub hop_count: Option<u8>,
  pub asn: Option<u32>,
  pub isp: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
  pub user_agent: Option<String>,
  pub locale: Option<String>,
  pub timezone: Option<String>,
  pub device_id: Option<String>,
  pub session_id: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FinancialContext {
  // --- 1. Marcadores de Tiempo (Timestamps) ---
  // Indispensable para calcular: "Días desde primera aparición" y "Tiempo desde última txn"
  pub first_seen_at: u64,            // Unix timestamp
  pub last_seen_at: u64,             // Unix timestamp de la última actividad exitosa
  pub last_declined_at: Option<u64>, // Para detectar reintentos rápidos tras fallo

  // --- 2. Acumuladores de Ciclo de Vida (Lifetime Totals) ---
  // Indispensables para perfilar: "¿Es este un cliente VIP o nuevo?"
  pub total_successful_txns: u64,
  pub total_declined_txns: u64,
  pub total_amount_spent: u64, // Entero (centavos) para evitar errores de punto flotante
  pub max_ticket_ever: u64,    // El monto más alto visto históricamente

  // --- 3. Contadores de Seguridad (Security Counters) ---
  // Indispensables para: Fuerza bruta y Account Takeover
  pub consecutive_failed_logins: u32, // Se reinicia a 0 con un login exitoso
  pub consecutive_declines: u32,      // Se reinicia a 0 con una compra exitosa

  // --- 4. Ventanas Temporales (Sliding Window Buckets) ---
  // Aquí es donde ocurre la magia. En lugar de un solo número, guardas "cubos".
  // Ejemplo simplificado: Un mapa de { "minuto_epoch": count } o estructuras optimizadas.
  // Para simplificar este struct, asumiremos que usamos contadores atómicos rotativos:
  pub current_hour_count: u32,
  pub current_hour_amount: u64,

  pub current_day_count: u32,
  pub current_day_amount: u64,

  // --- 5. Diversidad (Sets para Cardinalidad) ---
  // Indispensables para: "Distinct Merchants", "Distinct IPs"
  // Nota: En producción, usar HyperLogLog para ahorrar memoria si hay muchos datos.
  pub known_ips: HashSet<String>,     // IPs previas
  pub known_devices: HashSet<String>, // DeviceIDs previos
}
