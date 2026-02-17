## Roadmap — Red Velvet Engine

### 1. Corto plazo (0-2 sprints)
- Configuración redistribuible: flags/env para Redis/Dragonfly, scripts `docker compose`, generación de datos demo.
- API de reglas CRUD real: persistencia en Redis (hashes, índices), validaciones, paginación y contratos JSON documentados.
- Motor mínimo viable: wiring `RVEngine` + `RuleExecutorPort`, evaluación de `RuleSchedule`, `RolloutPolicy` y `RuleEnforcement`.
- Testing base y CI: suites unitarias/integración (Axum + Redis), `cargo fmt/clippy/test` automatizados.
- Documentación inicial: `docs/overview.md`, ejemplos cURL, diagramas simples enlazados desde README.

### 2. Mediano plazo (3-6 sprints)
- DSL para lógica: serializador/compilador JSON Logic o `dataflow-rs`, caché de evaluaciones.
- Versionado/auditoría de reglas: endpoints de publish/unpublish, historial basado en `RuleAudit`.
- Endpoint de decisiones: ingestión `/api/v1/decisions` aceptando `Event`, enriquecimientos y respuesta `EngineResult`.
- Observabilidad: métricas `tracing` + `opentelemetry`, logs por request, dashboards básicos.
- Harness de validación: datasets sintéticos, fuzzing de reglas, pruebas de carga (k6) para throughput/latencia.

### 3. Largo plazo (6+ sprints)
- Multi-tenant: espacios de nombres por cliente, cuotas, rate limits y aislamiento en Redis.
- Persistencia híbrida: Redis + Postgres para histórico, migraciones automatizadas y replay.
- Workflows/acciones asincrónicas: webhooks, colas para `RuleAction::Review/Block`, integraciones externas.
- Herramientas de ruleops: UI/CLI para inspección de hits, simulador, despliegues canary y approvals.
- Hardening producción: autenticación/autorización API, límites de payload, guías de despliegue cloud.
