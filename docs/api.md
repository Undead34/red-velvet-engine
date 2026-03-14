# Red Velvet Engine API Contract (v1)

This document is the practical integration contract for `rve` HTTP APIs.
It complements OpenAPI with behavioral notes, invariants, and copy/paste examples.

## 1. Rule variables and field paths

### 1.1 Allowed JSONLogic roots

Rules can only read variables under these roots:

- `event`
- `payload`
- `context`
- `features`
- `signals`
- `extensions`
- `transaction`
- `device`

Any other root is rejected with `422`.

### 1.2 Canonical vs extension-based fields

Use canonical fields by default:

- `payload.money.value`
- `payload.money.ccy`
- `payload.parties.originator.*`
- `payload.parties.beneficiary.*`
- `features.fin.*`
- `signals.flags.*`

Extension-derived fields are optional and only exist when provided in `payload.extensions`:

- `transaction.amount` maps to `payload.extensions.transaction.amount`
- `device.trust_score` maps to `payload.extensions.device.trust_score`

If your rules rely on `transaction.*` or `device.*`, your events must include those extension objects.

## 2. Validation invariants (must-know)

### 2.1 Rule enforcement

- `enforcement.score_impact`: `1.0 ..= 10.0`
- `enforcement.cooldown_ms`: `1 ..= 86_400_000` when present
- `enforcement.tags`: required and non-empty

### 2.2 Rule policy

- `rollout.percent`: `0 ..= 100`
- `schedule.active_until_ms > schedule.active_from_ms` when both are present
- `state.audit.updated_at_ms >= state.audit.created_at_ms`

### 2.3 Metadata input strictness

- `meta.author` is required.
- `meta.autor` is rejected (legacy alias removed).
- Legacy expression operators are rejected (no automatic normalization).

## 3. Runtime behavior notes

- Rules are persisted via `/api/v1/rules*`.
- The decision runtime is currently a placeholder.
- `POST /api/v1/decisions` returns `501 Not Implemented`.
- `GET /api/v1/engine/status` exposes placeholder status only.
- `POST /api/v1/engine/reload` returns `501 Not Implemented`.

## 4. Error model

Validation failures return `422` with path-oriented details:

```json
{
  "code": "unprocessable_entity",
  "message": "validation failed",
  "validation": {
    "errors": [
      { "path": "enforcement.score_impact", "message": "must be between 1.0 and 10.0" }
    ],
    "warnings": []
  }
}
```

## 5. Examples

### 5.1 Create rule

```bash
curl -sS -X POST http://localhost:3439/api/v1/rules \
  -H 'content-type: application/json' \
  -d '{
    "meta": {
      "code": "FRAUD-HV-001",
      "name": "High value transaction",
      "description": "Flags large transaction amounts",
      "version": "1.0.0",
      "author": "RiskOps",
      "tags": ["high_value"]
    },
    "state": {
      "mode": "active",
      "audit": {
        "created_at_ms": 1730000000000,
        "updated_at_ms": 1730000001000,
        "created_by": "riskops",
        "updated_by": "riskops"
      }
    },
    "schedule": { "active_from_ms": 1730000000000 },
    "rollout": { "percent": 100 },
    "evaluation": {
      "condition": true,
      "logic": {
        "and": [
          { ">": [ { "var": "payload.money.value" }, 1000 ] },
          { ">=": [ { "var": "features.fin.current_hour_count" }, 1 ] }
        ]
      }
    },
    "enforcement": {
      "score_impact": 6.5,
      "action": "review",
      "severity": "high",
      "tags": ["financial_fraud"],
      "cooldown_ms": 60000,
      "functions": []
    }
  }' | jq
```

### 5.2 Decision endpoint placeholder

```bash
curl -sS -X POST http://localhost:3439/api/v1/decisions \
  -H 'content-type: application/json' \
  -d '{ \"sample\": \"payload\" }' | jq
```

Current response:

```json
{
  "code": "not_implemented",
  "message": "decision runtime is not implemented yet",
  "validation": null
}
```

## 6. Integration checklist

- Build rules using canonical paths first (`payload.*`, `features.*`, `signals.*`).
- Use extension paths (`transaction.*`, `device.*`) only if your producer sends `payload.extensions`.
- Keep `score_impact` in the documented range.
- Treat `/api/v1/decisions` and `/api/v1/engine/reload` as placeholders until the runtime is reintroduced.
