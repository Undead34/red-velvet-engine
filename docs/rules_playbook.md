## Playbook de Reglas — Red Velvet Engine

Este documento sirve como guía práctica para los equipos que necesitan crear/editar reglas sin explorar el código fuente.

### Campos obligatorios al crear una regla

| Campo | Tipo | Descripción |
| --- | --- | --- |
| `meta.name` | string | Nombre legible. Úsalo como título en dashboards. |
| `meta.description` | string? | Explicación corta para analistas / UI. |
| `meta.version` | semver (ej. `"1.0.0"`) | Versionado manual para auditoría. |
| `meta.autor` | string | Dueño de la regla (persona o equipo). |
| `state.mode` | `active | paused | draft` | Controla si la regla participa en el motor. |
| `state.audit.created_by/updated_by` | string? | Pista de quién tocó la regla. Debe actualizarse manualmente. |
| `schedule.active_from_ms` | u64? | Timestamp UTC en milisegundos. Permite diferir activaciones. |
| `rollout.percent` | u8 (0-100) | Garganta de gradual release. 100 = tráfico completo. |
| `evaluation.condition` | JSONLogic | Guard clause rápida (puede ser `true`). |
| `evaluation.logic` | JSONLogic | Expresión completa que define la regla. |
| `enforcement.score_impact` | f32 (1.0 - 10.0) | Delta que aporta al score total. |
| `enforcement.action` | `allow | review | block | tag_only` | Recomendación al sistema aguas arriba. |
| `enforcement.severity` | `none/low/moderate/high/very_high/catastrophic` | Semáforo para priorizar hits. |
| `enforcement.tags` | [string] | Labels para agrupar métricas/alertas. |

> Nota: Todos los campos opcionales (`description`, `audit`, `schedule`) pueden omitirse pero se recomienda llenarlos para facilitar soporte.

### Ejemplo completo (listo para `POST /api/v1/rules`)

```json
{
  "meta": {
    "name": "High Value on Untrusted Device",
    "description": "Monto > 5000 y trust score < 0.4",
    "version": "1.0.0",
    "autor": "FraudOps",
    "tags": ["high_value", "device"]
  },
  "state": {
    "mode": "active",
    "audit": {
      "created_at_ms": 1700000000000,
      "updated_at_ms": 1700000000000,
      "created_by": "FraudOps",
      "updated_by": "FraudOps"
    }
  },
  "schedule": {
    "active_from_ms": 1700000000000,
    "active_until_ms": null
  },
  "rollout": { "percent": 100 },
  "evaluation": {
    "condition": true,
    "logic": {
      "and": [
        { ">": [{ "var": "transaction.amount" }, 5000] },
        { "<": [{ "var": "device.trust_score" }, 0.4] }
      ]
    }
  },
  "enforcement": {
    "score_impact": 8.5,
    "action": "review",
    "severity": "high",
    "tags": ["financial_fraud", "device_fingerprinting"],
    "cooldown_ms": 600000
  }
}
```

### Cómo seleccionar cada campo

1. **`meta.*`**: acordar convención interna (prefijos por tenant, etc.). Usa `version` para cambios incompatible.
2. **`state.mode`**: trabaja en `draft` durante QA, cambia a `active` para ponerla en producción, usa `paused` para toggles temporales.
3. **`schedule`**: define ventanas automáticas (campañas, soft-launch). Si ambos campos son `null`, la regla está siempre disponible.
4. **`rollout.percent`**: arranca en 10-20% para validar, luego sube a 100%. El motor calcula `bucket` automáticamente.
5. **`evaluation.condition`**: guarda expresiones rápidas/costosas; por ejemplo, `{"==": [{"var": "payload.money.ccy"}, "USD"]}` para evitar evaluar el resto si no aplica.
6. **`evaluation.logic`**: construye JSONLogic usando las señales disponibles (`event`, `payload`, `context`, `signals`, `extensions`). Es compatible con la misma sintaxis que `dataflow-rs`.
7. **`enforcement.score_impact`**: respeta el rango 1.0-10.0 (usa `Score::new`). Si no estás seguro, alinea con la severidad (`Severity::value()` ya sugiere un nivel).
8. **`enforcement.action`**: define qué hará el workflow aguas arriba (auto-block, review manual, etc.).
9. **`enforcement.tags`**: agrupa en dashboards (ej. `card_testing`, `kyc_low`).

### Operaciones recomendadas en la UI / flujo humano

1. **Crear**: usa el ejemplo anterior, guarda `mode = draft`, `percent = 10`. Despliega para QA.
2. **Promover**: `PATCH` (o `PUT`) para subir `percent` y pasar `mode = active` una vez aprobada.
3. **Revisión**: actualiza `state.audit.updated_by` y `updated_at_ms` cada vez que edites para mantener rastreabilidad.
4. **Desactivar temporalmente**: `PATCH` -> `{ "state": { "mode": "paused" } }`.
5. **Eliminar**: `DELETE /api/v1/rules/{id}` solo después de dejarla en `draft` (para evitar golpes accidentales).

### Validaciones manuales

Aunque el backend todavía no valida cada campo, se recomienda:

- Verificar que `score_impact` esté dentro de 1.0-10.0 y que `rollout.percent` <= 100.
- Probar la expresión JSONLogic con datos reales antes de `PUT` final (usa herramientas como dataflow-rs CLI o `jq` + unit tests).
- Mantener convenciones de `tags` y `autor` para facilitar filtros en dashboards.

### Recursos adicionales

- `docs/rules_decisions_contract.md`: detalle campo por campo con ejemplos.
- `docs/api_frontend.md`: endpoints y payloads completos para UI o integraciones.
- `docs/overview.md`: arquitectura general para entender cómo impactan los cambios en Runtime.
