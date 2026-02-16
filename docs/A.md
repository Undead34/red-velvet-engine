Entendido. Lo que te pasé antes quedó **demasiado “enterprise patterns”** para el estilo RVE, y encima no incluía **dataflow-rs** (que es lo que querías aprovechar). Vamos a hacerlo bien:

* **Sin `ClockPort`**: metemos `now_ms` en `metadata` (simple, RVE-ish).
* **Sin “Catalog”**: no hay repos ni nombres raros; el runtime ya “trae” las reglas compiladas.
* **Con dataflow-rs**: cada `Rule` se convierte en un `Task` con `condition` = JSONLogic, y un `function` custom que registra el hit (score/decision/tags).
* **Hexagonal real**: `rve-core` expone el “contrato” (port) y los tipos de salida. `rve` implementa el adapter con dataflow-rs + ArcSwap.

Abajo te dejo una implementación lista para tu repo, con nombres más RVE.

---

## 0) `crates/rve/Cargo.toml` (agregar dataflow-rs)

```toml
[dependencies]
dataflow-rs = "1.0.8"
async-trait = "0.1"
```

Y en `crates/rve-core/Cargo.toml` si querés el port async en core (recomendado):

```toml
[dependencies]
async-trait = "0.1"
```

---

## 1) rve-core: Port + Result types (simple, sin “catalog”, sin clock)

### `crates/rve-core/src/ports/mod.rs` (reemplazar)

```rust
use async_trait::async_trait;

use crate::domain::event::Event;
use crate::services::engine::EngineResult;

/// Inbound port: “esto es lo que hace el motor”.
/// Infra/Adapters lo implementan (dataflow-rs, etc).
#[async_trait]
pub trait EnginePort: Send + Sync {
    async fn inspect(&self, event: &Event) -> Result<EngineResult, String>;
}
```

### `crates/rve-core/src/lib.rs` (export)

```rust
pub mod domain;
pub mod ports;
pub mod services;

pub use ports::EnginePort;
```

### `crates/rve-core/src/services/engine.rs` (arreglos + policy “Block > Review > Allow”)

Tu archivo hoy tiene un bug (`rule.policy.score` no existe). Te dejo una versión alineada a tu domain:

```rust
use crate::domain::rule::{RuleAction, Severity};
use crate::domain::types::Score;

#[derive(Clone, Debug)]
pub struct EngineResult {
    pub score: Score,
    pub decision: Decision,
    pub hits: Vec<RuleHit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Review,
    Block,
}

#[derive(Clone, Debug)]
pub struct RuleHit {
    pub rule_id: String,
    pub action: RuleAction,
    pub severity: Severity,
    pub score_impact: Score,
    pub tags: Vec<String>,
    pub shadow: bool,
}

impl EngineResult {
    pub fn empty() -> Self {
        Self {
            score: 0,
            decision: Decision::Allow,
            hits: Vec::new(),
        }
    }

    pub fn apply_hit(&mut self, hit: RuleHit) {
        if !hit.shadow {
            self.score += hit.score_impact;
        }

        // decisión: Block > Review > Allow (TagOnly no escala)
        match hit.action {
            RuleAction::Block => self.decision = Decision::Block,
            RuleAction::Review => {
                if self.decision != Decision::Block {
                    self.decision = Decision::Review;
                }
            }
            _ => {}
        }

        self.hits.push(hit);
    }
}
```

---

## 2) rve: Adapter “RVE Engine” con dataflow-rs + ArcSwap (swap engine completo)

### `crates/rve/src/engine/mod.rs` (poner esto)

```rust
pub mod rve_flow;

pub use rve_flow::RveEngine;
```

### `crates/rve/src/engine/rve_flow.rs` (NUEVO)

Este es el corazón: arma workflows desde `Vec<Rule>`, registra una función custom `rve_hit`, y swapea el Engine completo.

```rust
use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use dataflow_rs::{AsyncFunctionHandler, Engine as FlowEngine, FunctionConfig, Message, Workflow};
use dataflow_rs::engine::message::{Change}; // si el path cambia, ajustalo al que exporte tu versión

use rve_core::{
    domain::{event::Event, rule::{Rule, RuleMode, RuleAction, Severity}},
    services::engine::{EngineResult, RuleHit},
    ports::EnginePort,
};

/// RVE-style: un solo handle que “trae todo” adentro.
/// Swap completo del engine (inmutable) con ArcSwap.
pub struct RveEngine {
    live: ArcSwap<FlowEngine>,
}

impl RveEngine {
    pub fn new(rules: Vec<Rule>) -> Result<Self, String> {
        let engine = build_flow_engine(rules)?;
        Ok(Self { live: ArcSwap::from_pointee(engine) })
    }

    /// Recarga atómica: compila TODO y luego swap.
    pub fn reload(&self, rules: Vec<Rule>) -> Result<(), String> {
        let engine = build_flow_engine(rules)?;
        self.live.store(Arc::new(engine));
        Ok(())
    }
}

#[async_trait]
impl EnginePort for RveEngine {
    async fn inspect(&self, event: &Event) -> Result<EngineResult, String> {
        let engine = self.live.load_full();

        // Armamos un Message con un context root igual al “shape” de tus reglas:
        // header/context/signals/payload + metadata (now_ms, rollout_key)
        let now_ms = Utc::now().timestamp_millis() as u64;

        let ctx = json!({
            "header": event.header,
            "context": event.context,
            "signals": event.signals,
            "payload": event.payload,
            "metadata": {
                "now_ms": now_ms,
                // key estable para rollout (event_id si existe, sino acct)
                "rollout_key": event.header.event_id
                    .as_deref()
                    .unwrap_or(&event.payload.parties.originator.acct),
            }
        });

        // Ojo: Message API puede variar según versión; ajustá si tu crate expone otro ctor.
        let mut msg = Message::from_value(&ctx);

        engine.process_message(&mut msg).await.map_err(|e| format!("{e:?}"))?;

        // El custom function deja el resultado en context.fraud
        let fraud = msg.context["fraud"].clone();
        Ok(parse_engine_result(fraud))
    }
}

/// ---- build: Rule -> dataflow Workflow/Tasks ----

fn build_flow_engine(rules: Vec<Rule>) -> Result<FlowEngine, String> {
    // 1) Convertimos rules -> workflow JSON
    let workflow_json = rules_to_workflow_json(&rules);

    // 2) Parse Workflow
    let wf = Workflow::from_json(&workflow_json).map_err(|e| format!("workflow json invalid: {e:?}"))?;

    // 3) Registramos custom function: rve_hit
    let mut custom: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> = HashMap::new();
    custom.insert("rve_hit".to_string(), Box::new(RveHitFn));

    // 4) Engine inmutable (dataflow-rs compila en new)
    Ok(FlowEngine::new(vec![wf], Some(custom)))
}

fn rules_to_workflow_json(rules: &[Rule]) -> String {
    // Un workflow con una task por regla.
    // Nota: filtramos Archived/Disabled acá (porque el engine es inmutable y lo vas a swappear).
    let tasks: Vec<Value> = rules
        .iter()
        .filter(|r| matches!(r.state.mode, RuleMode::Enabled | RuleMode::Shadow))
        .map(rule_to_task_json)
        .collect();

    // priority fijo (si lo usás), id fijo “rve”
    let wf = json!({
        "id": "rve_rules",
        "name": "RVE Rules",
        "priority": 0,
        "tasks": tasks
    });

    wf.to_string()
}

fn rule_to_task_json(rule: &Rule) -> Value {
    // condition = AND(condition, logic, window checks)
    let mut ands = vec![rule.evaluation.condition.clone(), rule.evaluation.logic.clone()];

    if let Some(from) = rule.schedule.active_from_ms {
        ands.push(json!({ ">=": [ { "var": "metadata.now_ms" }, from ] }));
    }
    if let Some(until) = rule.schedule.active_until_ms {
        ands.push(json!({ "<": [ { "var": "metadata.now_ms" }, until ] }));
    }

    let cond = json!({ "and": ands });

    // function = rve_hit con input de enforcement + rollout + shadow
    json!({
        "id": rule.id,
        "name": rule.meta.name,
        "description": rule.meta.description,
        "condition": cond,
        "function": {
            "name": "rve_hit",
            "input": {
                "rule_id": rule.id,
                "shadow": rule.is_shadow(),
                "rollout_percent": rule.rollout.percent,
                "score_impact": rule.enforcement.score_impact,
                "action": rule.enforcement.action,
                "severity": rule.enforcement.severity,
                "tags": rule.enforcement.tags
            }
        }
    })
}

/// ---- Custom Function: rve_hit ----
/// Se ejecuta cuando condition == true.
/// Acá aplicamos rollout determinista y acumulamos hits/score/decision.
struct RveHitFn;

#[async_trait]
impl AsyncFunctionHandler for RveHitFn {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<datalogic_rs::DataLogic>,
    ) -> dataflow_rs::engine::error::Result<(usize, Vec<Change>)> {
        // input viene de rule_to_task_json
        let input = &config.input;

        let rule_id = input["rule_id"].as_str().unwrap_or("").to_string();
        let shadow = input["shadow"].as_bool().unwrap_or(false);
        let rollout_percent = input["rollout_percent"].as_u64().unwrap_or(100) as u8;

        // rollout_key viene en metadata.rollout_key
        let rollout_key = message.context["metadata"]["rollout_key"]
            .as_str()
            .unwrap_or("");

        if !rollout_allowed(&rule_id, rollout_key, rollout_percent) {
            return Ok((200, vec![]));
        }

        // Build hit
        let score_impact = input["score_impact"].as_i64().unwrap_or(0) as i16;
        let action: RuleAction = serde_json::from_value(input["action"].clone()).unwrap_or(RuleAction::TagOnly);
        let severity: Severity = serde_json::from_value(input["severity"].clone()).unwrap_or(Severity::Medium);
        let tags: Vec<String> = serde_json::from_value(input["tags"].clone()).unwrap_or_default();

        // Ensure fraud object exists
        if message.context["fraud"].is_null() {
            message.context["fraud"] = json!({
                "score": 0,
                "decision": "allow",
                "hits": []
            });
        }

        // Push hit
        let hit = json!({
            "rule_id": rule_id,
            "action": action,
            "severity": severity,
            "score_impact": score_impact,
            "tags": tags,
            "shadow": shadow
        });

        message.context["fraud"]["hits"]
            .as_array_mut()
            .map(|arr| arr.push(hit))
            .unwrap_or_else(|| {
                message.context["fraud"]["hits"] = json!([hit]);
            });

        // Score (only if not shadow)
        if !shadow {
            let current = message.context["fraud"]["score"].as_i64().unwrap_or(0);
            message.context["fraud"]["score"] = json!(current + score_impact as i64);
        }

        // Decision policy
        let cur = message.context["fraud"]["decision"].as_str().unwrap_or("allow");
        let next = match action {
            RuleAction::Block => "block",
            RuleAction::Review if cur != "block" => "review",
            _ => cur,
        };
        message.context["fraud"]["decision"] = json!(next);

        Ok((200, vec![]))
    }
}

fn rollout_allowed(rule_id: &str, key: &str, percent: u8) -> bool {
    if percent >= 100 {
        return true;
    }
    if percent == 0 {
        return false;
    }

    // FNV-1a simple (determinista)
    let mut h: u64 = 14695981039346656037;
    for b in rule_id.as_bytes().iter().chain([0xFF].iter()).chain(key.as_bytes()) {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    let bucket = (h % 100) as u8;
    bucket < percent
}

/// ---- Parse output (Message.context["fraud"]) -> EngineResult ----
fn parse_engine_result(fraud: Value) -> EngineResult {
    let mut out = EngineResult::empty();

    // score
    out.score = fraud["score"].as_i64().unwrap_or(0) as i16;

    // decision
    out.decision = match fraud["decision"].as_str().unwrap_or("allow") {
        "block" => rve_core::services::engine::Decision::Block,
        "review" => rve_core::services::engine::Decision::Review,
        _ => rve_core::services::engine::Decision::Allow,
    };

    // hits
    if let Some(arr) = fraud["hits"].as_array() {
        for h in arr {
            let rule_id = h["rule_id"].as_str().unwrap_or("").to_string();
            let action: RuleAction = serde_json::from_value(h["action"].clone()).unwrap_or(RuleAction::TagOnly);
            let severity: Severity = serde_json::from_value(h["severity"].clone()).unwrap_or(Severity::Medium);
            let score_impact = h["score_impact"].as_i64().unwrap_or(0) as i16;
            let tags: Vec<String> = serde_json::from_value(h["tags"].clone()).unwrap_or_default();
            let shadow = h["shadow"].as_bool().unwrap_or(false);

            out.hits.push(RuleHit {
                rule_id,
                action,
                severity,
                score_impact,
                tags,
                shadow,
            });
        }
    }

    out
}
```

**Qué te llevás con esto**

* El engine “real” es **dataflow-rs**.
* Cada reload recompila todo (como querías) y vos sólo hacés `engine.reload(new_rules)`.
* Nada de “catalog”, nada de “clock port”; `now_ms` vive en `metadata` y listo.
* Hexagonal: `rve-core::EnginePort` es el contrato; `rve::engine::RveEngine` es el adapter.

---

## 3) Wiring en `AppState` (para usarlo desde HTTP luego)

### `crates/rve/src/http/state.rs`

```rust
use std::sync::Arc;

use rve_core::EnginePort;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<dyn EnginePort>,
}

impl AppState {
    pub fn new(engine: Arc<dyn EnginePort>) -> Self {
        Self { engine }
    }
}
```

Y en `main.rs` (crear engine):

```rust
use crate::engine::RveEngine;
// ...

let engine = Arc::new(RveEngine::new(vec![]).unwrap()); // después: cargar reglas reales
let state = AppState::new(engine);
```

---

## Notas rápidas (por si algo no compila por “API drift”)

Dataflow-rs a veces cambia paths (`Message::from_value`, `Change`, `FlowEngine::new` args). Si tu versión difiere:

* lo único que hay que ajustar son imports/constructores,
* el diseño queda igual.

---

Si me pegás el `cargo tree | rg dataflow` o el `preadd crates/rve/src` luego de agregar la dep, te lo dejo 100% compilando con **la API exacta** que te instaló Cargo (sin inventar paths).
