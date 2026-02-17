## Contrato de Reglas y Decisiones

Este documento describe el payload oficial que expone la API `rve` para administrar reglas de fraude (`/api/v1/rules`) y para solicitar decisiones en línea (`/api/v1/decisions`). Los campos y tipos referencian las estructuras del crate `rve-core`.

### Objeto `Rule`

```json
{
  "id": "FRAUD-HV-UNTRUSTED-01",
  "meta": {
    "name": "High Value on Untrusted Device",
    "description": "Dispara si el monto es > $5000 y el fingerprint del dispositivo es nuevo.",
    "version": "1.0.0",
    "autor": "Analista",
    "tags": ["high_value", "device"]
  },
  "state": {
    "mode": "active",
    "audit": {
      "created_at_ms": 1706790000000,
      "updated_at_ms": 1707830000000,
      "created_by": "Super User",
      "updated_by": "Analyst Jane"
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

#### Campos clave
- `state.mode`: `active | paused | draft` (según `RuleMode`). Controla si el motor puede evaluar la regla.
- `schedule`: ventana opcional para activar/desactivar automáticamente. `is_within_window(now_ms)` debe ser true para correr.
- `rollout.percent`: aplica gating gradual (0-100). El backend usa bucket hash `% 100`.
- `evaluation.condition`: guard extra rápido (bool, número o DSL). Se evalúa antes del `logic` completo.
- `evaluation.logic`: JSON Logic / `dataflow-rs` serializado. Debe ser determinístico.
- `enforcement.score_impact`: `Score` en el rango permitido por `rve-core`. Impacta al total del evento.
- `enforcement.action`: `allow | review | block | tag_only`.
- `enforcement.tags`: labels para dashboards/alerting.
- `enforcement.cooldown_ms`: evita hits repetidos para la misma key (definida por la integración).

### Endpoints de Reglas

1. `GET /api/v1/rules?page=1&limit=20`
   - Respuesta: `{ "data": [Rule], "pagination": { "page": 1, "limit": 20, "total": 125 } }`.
   - Orden default: `meta.name` ASC. Futuro soporte `?sort=updated_at_ms`.

2. `POST /api/v1/rules`
   - Body: `Rule` sin `id` (el backend asigna ID) o con `id` proporcionado si no existe.
   - Validaciones: `rollout.percent <= 100`, `score_impact` válido, `logic` JSON válido.
   - Respuesta: `201` con `Rule` completo.

3. `GET /api/v1/rules/{id}`
   - Respuesta: `Rule`.
   - Errores: `404` si no existe.

4. `PUT /api/v1/rules/{id}`
   - Body: `Rule` completo. Reemplaza todo el recurso; cualquier campo omitido se resetea a default.
   - Uso recomendado para flujos declarativos (“source of truth” en Git, Terraform, etc.).
   - Concurrency: usar `If-Match`/`meta.version`/`state.audit.updated_at_ms` para evitar pisar ediciones.

5. `PATCH /api/v1/rules/{id}`
   - Body: JSON parcial siguiendo `application/merge-patch+json`.
   - Ideal para toggles rápidos (`state.mode`, `rollout.percent`, `schedule.active_until_ms`).
   - Validaciones se restringen a los campos enviados; `score_impact` y `logic` solo se revalidan si vienen en el payload.
   - Respuesta: `200` con el `Rule` completo resultante.

6. `DELETE /api/v1/rules/{id}`
   - Elimina la regla del repositorio activo. Al no existir soft delete todavía, cualquier bandera (`state.mode`) debe gestionarse vía `PATCH`.
   - Tras eliminar, el motor recompila automáticamente la lista de reglas para que la decisión refleje el estado más reciente.

### Contrato de Decisiones (`/api/v1/decisions`)

El endpoint ya se encuentra disponible y utiliza el motor interno (`RVEngine`) con JSONLogic compilado. Acepta el `Event` completo y produce un `EngineResult` enriquecido.

#### Request

```json
{
  "event": {
    "header": {
      "timestamp": "2025-02-01T03:04:05Z",
      "source": "checkout",
      "event_id": "evt_123",
      "instrument": "card",
      "channel": "web"
    },
    "context": { /* signals compartidos */ },
    "signals": { /* features preprocesadas */ },
    "payload": {
      "money": {
        "amount": 7500,
        "currency": "USD"
      },
      "parties": {
        "originator": {
          "entity_type": "individual",
          "acct": "cust_01",
          "country": "US",
          "kyc": "tier_2",
          "watchlist": "clear"
        },
        "beneficiary": { /* mismos campos */ }
      },
      "extensions": {
        "device": { "trust_score": 0.33 },
        "transaction": { "amount": 7500 }
      }
    }
  }
}
```

#### Response

```json
{
  "decision": {
    "score": 42.5,
    "hits": [
      {
        "rule_id": "FRAUD-HV-UNTRUSTED-01",
        "action": "review",
        "severity": "high",
        "score_delta": 8.5,
        "explanation": "Monto > 5000 y device nuevo",
        "tags": ["financial_fraud", "device_fingerprinting"]
      }
    ],
    "metadata": {
      "evaluated_rules": 128,
      "latency_ms": 12,
      "rollout_bucket": 73
    }
  }
}
```

- `score`: suma de `RuleEnforcement.score_impact` (f32) de todas las reglas activas que dispararon.
- `hits`: mantiene orden determinístico, incluye la delta aportada por la regla y los tags declarados para debugging.
- `metadata.evaluated_rules`: cuántas reglas se procesaron en la ventana y bucket actual.
- `metadata.rollout_bucket`: entero 0-99 calculado a partir del evento (se comparte con `RolloutPolicy`).
- `metadata.latency_ms`: tiempo total de la solicitud (medido en la capa HTTP).

#### Errores
- `400`: payload inválido (schema, campos obligatorios).
- `409`: reservado para idempotencia (aún no implementado).
- `500`: error interno en la compilación/evaluación de reglas (se loguea y se debe revisar la regla involucrada).

### Auditoría y Versionado
- Cada mutación debería actualizar `state.audit.updated_at_ms`/`updated_by`. Aún no se aplican validaciones server-side, por lo que el cliente debe enviar estos campos.
- Creación inicial define `created_at_ms`/`created_by`.
- Se recomienda incluir `X-RVE-Actor` en la cabecera HTTP y reflejarlo en `audit`; una vez que exista persistencia externa, servirá para tracking.

### Convenciones
- Timestamps siempre en milisegundos UTC o ISO-8601 si es string.
- Campos numéricos usan punto decimal (JSON).
- Los identificadores deben ser únicos por tenant (cuando multi-tenant esté disponible) con prefijos recomendados `SEGMENT-CASO-N`.

Este contrato se debe mantener sincronizado con `rve-core`. Cambios incompatibles requieren versionar la API (`/api/v2`).
