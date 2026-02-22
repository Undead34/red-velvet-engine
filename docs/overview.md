## Overview — Red Velvet Engine

### Arquitectura actual

- **Crate `rve-core`**: define el dominio (eventos, reglas) y los puertos `RuleRepositoryPort` + `RuleExecutorPort`. Todos los adaptadores consumen estos contratos.
- **Crate `rve`**: expone la API HTTP (Axum), el motor embebido (`RVEngine`) y la implementación actual del repositorio.

```
┌────────────┐    ┌──────────────────────┐    ┌────────────────────┐
│  HTTP API  │ -> │ RuleRepositoryPort   │ -> │ InMemoryRuleRepo   │
│  (Axum)    │    │ (trait en rve-core)  │    │ (swap future Redis)│
└────────────┘    └─────────────┬────────┘    └──────────┬─────────┘
        │                        │                       │
        ▼                        ▼                       │
┌────────────┐    ┌──────────────────────┐                │
│ /decisions │ -> │ RVEngine (ArcSwap)   │ <──────────────┘
└────────────┘    └──────────────────────┘
```

### Flujo de reglas

1. **CRUD** (`/api/v1/rules`) opera sobre el `RuleRepositoryPort`. Hoy la implementación por defecto es in-memory pero cumple el contrato async (`list/get/all/create/replace/delete`).
2. La recarga del motor es explícita vía `POST /api/v1/engine/reload`: `AppState::reload_rules()` vuelve a leer `repo.all()` y llama `RVEngine::publish_rules`.
3. Las mutaciones (`POST/PUT/PATCH/DELETE`) persisten reglas en repositorio, pero no recargan automáticamente el motor.
4. El modelo expuesto por la API es idéntico al de `rve-core`: `Rule`, `RuleMeta`, `RuleSchedule`, etc. No hay DTO divergentes.

### Flujo de decisiones

1. `POST /api/v1/decisions` recibe un `Event` completo (estructura definida en `rve-core`).
2. `RVEngine::evaluate` toma la instantánea actual del `ArcSwap` y, para cada regla compilada:
   - Verifica ventana temporal (`RuleSchedule::is_within_window`).
   - Aplica bucketing (`RolloutPolicy::is_allowed`) calculado determinísticamente a partir del evento.
   - Evalúa `condition` y `logic` usando `datalogic-rs` (el mismo núcleo JSONLogic que emplea dataflow-rs) sobre un contexto con `event`, `payload`, `context`, `signals` y extensiones.
   - Si dispara, suma `RuleEnforcement.score_impact` al `score` total y agrega un `RuleHit` con acción, severidad y tags.
3. La respuesta es un `EngineResult` serializado con `score`, `hits`, cantidad de reglas evaluadas, bucket y la latencia medida en la capa HTTP.

### Hot reload

- El motor mantiene `Arc<Vec<CompiledRule>>`; cada `publish_rules` recompila y se intercambia con `ArcSwap`. No hay locking durante lectura: todas las peticiones de decisión ven la instantánea más reciente.
- Cualquier futuro backend (Redis, Postgres) solo necesita implementar `RuleRepositoryPort` y conservar `repo.all()` para rellenar el motor.

### Qué falta / próximos pasos

- Persistencia real (Redis/Dragonfly) para que el CRUD sobreviva reinicios.
- Validaciones y versionado de reglas (exponer errores de compilación al usuario).
- Integrar un `RuleExecutorPort` alterno (p. ej. dataflow-rs workflows completos) o enriquecer el contexto con señales derivados.
- Observabilidad: métricas por regla, tracing por decisión.

Este documento refleja el estado del motor mínimo viable después de integrar JSONLogic + ArcSwap y el repositorio en memoria.
