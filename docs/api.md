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

- `payload.money.minor_units`
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

### 2.3 Rule scope

- `scope.channels` is optional.
- If omitted, the rule applies to all channels.
- If present, it must contain `1 ..= 16` unique channel identifiers.
- Known channels today: `web`, `mobile`, `api`, `branch`, `call_center`, `pos`, `atm`, `backoffice`, `batch`, `partner`.
- Custom channels are allowed when they use the same identifier format as the rest of the domain.

### 2.4 Metadata input strictness

- `meta.author` is required.
- `meta.autor` is rejected (legacy alias removed).
- Legacy expression operators are rejected (no automatic normalization).

## 3. Runtime behavior notes

- Rules are persisted via `/api/v1/rules*` and remain in the repository until deleted.
- Runtime execution is backed by the `dataflow-rs` engine; use `/api/v1/engine/reload` to compile the latest repository snapshot into the runtime.
- `POST /api/v1/decisions` evaluates incoming events and returns scores/outcomes sourced from the active runtime ruleset.
- `POST /api/v1/decisions/trace` evaluates incoming events and returns both the decision and the execution trace.
- Event routing is inferred from `event.header.channel`; scoped rules only participate when the event channel matches.
- `GET /api/v1/engine/status` reports repository counts, loaded rules, backend mode, and readiness.
- `POST /api/v1/engine/reload` recompiles and loads repository rules into the runtime, returning the new snapshot metadata.

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

Operational note:

- Responses include `X-Request-Id` so logs can be correlated end-to-end.

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
    "scope": {
      "channels": ["web", "mobile"]
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
          { ">": [ { "var": "payload.money.minor_units" }, 100000 ] },
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

### 5.2 Decision endpoint example

```bash
curl -sS -X POST http://localhost:3439/api/v1/decisions \
  -H 'content-type: application/json' \
  -d '{
    "header": {
      "timestamp": "2026-03-08T00:00:00Z",
      "source": "checkout",
      "event_id": "0195d80e-4f96-7a4b-a8e0-3c5a3f0e7b21",
      "channel": "web"
    },
    "context": { "geo": { "country": "US" } },
    "features": { "fin": { "current_hour_count": 2, "current_hour_amount": 1500 } },
    "signals": { "flags": {} },
    "payload": {
      "type": "value_transfer",
      "money": { "minor_units": 150000, "ccy": "USD" },
      "parties": {
        "originator": { "entity_type": "individual", "acct": "acct_001", "country": "US", "watchlist": "no" },
        "beneficiary": { "entity_type": "business", "acct": "acct_002", "country": "US", "watchlist": "unknown" }
      }
    }
  }' | jq
```

Sample response after loading the runtime:

```json
{
  "score": 6.5,
  "outcome": "review",
  "hits": [
    {
      "rule_id": "01952031-1a77-7f0c-9f3c-bfd27d450001",
      "action": "review",
      "severity": "high",
      "score_delta": 6.5,
      "tags": ["financial_fraud", "device_fingerprinting"]
    }
  ],
  "evaluated_rules": 1,
  "executed_rules": 1,
  "rollout_bucket": 21
}
```

### 5.3 Decision trace example

```bash
curl -sS -X POST http://localhost:3439/api/v1/decisions/trace \
  -H 'content-type: application/json' \
  -d @event.json | jq
```

Trace steps expose both:

- `workflow_id`: internal runtime workflow identifier
- `rule_id`: original business rule identifier
- `runtime_channel`: normalized runtime channel (`all` for globally scoped rules)

## 6. Integration checklist

- Build rules using canonical paths first (`payload.*`, `features.*`, `signals.*`).
- Use extension paths (`transaction.*`, `device.*`) only if your producer sends `payload.extensions`.
- Set `event.header.channel` consistently and use `scope.channels` when a rule should only apply to certain entry channels.
- Keep `score_impact` in the documented range.
- Reload the runtime via `/api/v1/engine/reload` after mutating rules so `/api/v1/decisions` evaluates the latest state.
