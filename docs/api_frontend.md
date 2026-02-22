## API Frontend Guide (v1)

Esta guía está pensada para el frontend que construye/edita reglas y consulta decisiones.

### Base
- Health: `GET /health`
- OpenAPI: `GET /api-docs/openapi.json`
- API v1 base: `/api/v1`

### Convenciones de respuesta

#### Éxito
- Listas: `{ "data": [...], "pagination": { ... } }`
- Recursos únicos: JSON del recurso.

#### Error
- Errores de validación de reglas: `422`
- Shape:

```json
{
  "code": "validation_failed",
  "message": "request validation failed",
  "validation": {
    "errors": [
      { "path": "rollout.percent", "message": "range" }
    ],
    "warnings": []
  }
}
```

### 1) Health

`GET /health`

Respuesta:

```json
{ "status": "ok" }
```

### 2) Rules

Nota operativa:
- Las mutaciones de reglas (`POST/PUT/PATCH/DELETE`) actualizan repositorio, pero no recargan el motor automáticamente.
- Para aplicar cambios al motor en runtime, ejecutar `POST /api/v1/engine/reload`.

#### Listar reglas
`GET /api/v1/rules?page=1&limit=20`

Respuesta:

```json
{
  "data": [
    {
      "id": "01952031-1a77-7f0c-9f3c-bfd27d450001",
      "meta": {
        "code": "FRAUD-HV-UNTRUSTED-01",
        "name": "High Value on Untrusted Device",
        "description": "...",
        "version": "1.0.0",
        "autor": "Analista",
        "tags": ["high_value", "device"]
      },
      "state": {
        "mode": "active",
        "audit": {
          "created_at_ms": 1700000000000,
          "updated_at_ms": 1700000100000,
          "created_by": "alice",
          "updated_by": "alice"
        }
      },
      "schedule": {
        "active_from_ms": 1700000000000,
        "active_until_ms": null
      },
      "rollout": { "percent": 100 },
      "evaluation": {
        "condition": true,
        "logic": { ">": [{ "var": "payload.money.value" }, 5000] }
      },
      "enforcement": {
        "score_impact": 8.5,
        "action": "review",
        "severity": "high",
        "tags": ["financial_fraud"],
        "cooldown_ms": 600000
      }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 2
  }
}
```

#### Crear regla
`POST /api/v1/rules`

Body (resource completo):

```json
{
  "meta": {
    "code": "RL01",
    "name": "Rule Name",
    "description": "optional",
    "version": "1.0.0",
    "autor": "RiskOps",
    "tags": ["payments"]
  },
  "state": {
    "mode": "staged",
    "audit": {
      "created_at_ms": 1730000000000,
      "updated_at_ms": 1730000000000,
      "created_by": "alice",
      "updated_by": "alice"
    }
  },
  "schedule": {
    "active_from_ms": 1730000000000,
    "active_until_ms": 1731000000000
  },
  "rollout": { "percent": 50 },
  "evaluation": {
    "condition": true,
    "logic": { ">": [{ "var": "payload.money.value" }, 1000] }
  },
  "enforcement": {
    "score_impact": 6.5,
    "action": "review",
    "severity": "high",
    "tags": ["financial_fraud"],
    "cooldown_ms": 60000
  }
}
```

Validaciones críticas (server-side):
- `meta.name` no vacío.
- `meta.version` semver.
- `state.mode` enum válido (`staged|active|suspended|deactivated`).
- `rollout.percent` `0..100`.
- `enforcement.score_impact` `1.0..10.0`.
- `schedule.active_until_ms > active_from_ms` cuando ambos existen.
- JSONLogic compilable + roots permitidos.

#### Obtener regla
`GET /api/v1/rules/{id}`

#### Reemplazar regla
`PUT /api/v1/rules/{id}`

Mismo contrato que `POST`.

#### Patch de regla
`PATCH /api/v1/rules/{id}`

Notas:
- Solo soporta estos campos: `state.mode`, `state.audit.updated_by`, `state.audit.updated_at_ms`, `rollout.percent`, `schedule.active_from_ms`, `schedule.active_until_ms`.
- Si el body está vacío, no tiene cambios aplicables o incluye campos no patchables, retorna `422`.

Ejemplo:

```json
{
  "state": {
    "mode": "suspended",
    "audit": {
      "updated_by": "reviewer",
      "updated_at_ms": 1730000005000
    }
  },
  "rollout": { "percent": 25 }
}
```

#### Eliminar regla
`DELETE /api/v1/rules/{id}`

Semántica:
- `204` si elimina correctamente.
- `404` si la regla ya no existe.

### 3) Engine

#### Recargar reglas en el motor
`POST /api/v1/engine/reload`

Respuesta:

```json
{
  "status": "ok",
  "message": "engine rules reloaded"
}
```

### 4) Decisions

`POST /api/v1/decisions`

Estado actual: endpoint en esqueleto. Acepta el payload base para mantener contrato, pero responde `501 Not Implemented`.

Request (shape esperado):

```json
{
  "event": {
    "header": {
      "timestamp": "2026-02-20T14:23:11Z",
      "source": "checkout",
      "event_id": "01952031-1a77-7f0c-9f3c-bfd27d451111",
      "instrument": "card",
      "channel": "web"
    },
    "context": { "geo": {}, "net": {}, "env": {}, "fin": {} },
    "signals": { "flags": {} },
    "payload": {
      "money": { "value": 7800.5, "ccy": "EUR" },
      "parties": {
        "originator": {
          "entity_type": "individual",
          "acct": "cust_102938",
          "country": "ES",
          "kyc": "tier_2",
          "watchlist": "no"
        },
        "beneficiary": {
          "entity_type": "business",
          "acct": "merchant_7744",
          "country": "NL",
          "watchlist": "no"
        }
      },
      "extensions": {
        "device": { "trust_score": 0.31 },
        "transaction": { "amount": 7800.5 }
      }
    }
  }
}
```

Response actual:

```json
{
  "code": "NOT_IMPLEMENTED",
  "message": "Decision API is currently a skeleton endpoint",
  "validation": null
}
```

### 5) Metadata para Builder

#### Campos disponibles
`GET /api/v1/metadata/fields`

Uso frontend:
- construir selector de campos.
- limitar operadores según `allowed_operators`.
- si hay `allowed_values`, renderizar select.
- mostrar `description` y `examples` como ayuda contextual.

#### Contrato runtime
`GET /api/v1/metadata/contract`

Uso frontend:
- validar enums locales contra backend.
- detectar cambios de contrato por versión.
- habilitar/deshabilitar features por capacidad real.

### Recomendación de integración
- No hardcodear enums ni paths en frontend.
- Consumir `metadata/*` al iniciar app y cachear.
- Usar `openapi.json` para generar tipos de cliente y evitar drift.
