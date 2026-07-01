# Red Velvet Engine API

## API Overview

The Red Velvet Engine (RVE) API is a RESTful interface that provides programmatic access to real-time fraud decisioning and policy management. It is designed to evaluate high-risk digital flows deterministically, strictly separating your fraud policies from your application infrastructure.

## Authentication

The RVE API uses API keys to authenticate requests. Keep your API key secure and never expose it in client-side code (browsers or mobile apps). 

API keys must be provided via HTTP Bearer authentication in the request header:
`Authorization: Bearer RVE_API_KEY`

## Available APIs

The RVE API is grouped into three core operational domains:

* **Decisions API** (`/v1/decisions`): The high-throughput evaluation endpoint. Submit events (like value transfers or login attempts) to receive synchronous fraud decisions (Approve, Review, Decline).
* **Rules API** (`/v1/rules`): Programmatically manage your fraud policies. Create, read, update, delete, and change the state of your JSONLogic-based rulesets.
* **Metadata & Engine API** (`/v1/metadata`, `/v1/engine`): Introspect available domain fields for rule creation, fetch contract schemas, and manage runtime states like explicit engine reloads.

## Request and Response Format

All API requests and responses are encoded in JSON (`application/json`). 

**Strict Validation:** RVE enforces strict domain boundaries at the edge. The API does not guess your intent. If a payload violates a domain invariant (e.g., an invalid ISO-4217 currency code, a negative transaction amount, or a chronological impossibility), the request will fail immediately with a `400 Bad Request` or `422 Unprocessable Entity` error.

## Debugging and Headers

In addition to standard HTTP status codes, RVE includes custom headers in its responses to help you debug and trace evaluations.

| Header | Description |
| :--- | :--- |
| `x-request-id` | A globally unique identifier for the request. We recommend logging this value to cross-reference with RVE audit logs. |
| `x-rve-processing-ms` | The time (in milliseconds) the engine took to evaluate the rules and generate the decision. |
| `x-rve-ruleset-version` | The active version of the ruleset that evaluated the event. |

You may also supply your own unique identifier via the `x-client-request-id` header in your request, and RVE will include it in the internal audit logs for tracing.

## Basic Example

Here is a minimal request using the Decisions API to evaluate a value transfer:

```bash
curl [https://api.rve.yourdomain.com/v1/decisions](https://api.rve.yourdomain.com/v1/decisions) \
  -H "Authorization: Bearer $RVE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "header": {
      "timestamp": "2026-03-15T10:00:00Z",
      "source": "web_checkout"
    },
    "payload": {
      "type": "value_transfer",
      "money": { "minor_units": 15000, "ccy": "USD" },
      "parties": { /* ... */ }
    },
    "context": { /* ... */ },
    "features": { /* ... */ },
    "signals": { /* ... */ }
  }'

```

Response:

```json
{
  "id": "dec_01HQ8XV9Z2W4",
  "status": "success",
  "decision": {
    "action": "review",
    "severity": "high",
    "score": 8.5,
    "matched_rules": ["rule_velocity_spike_usd"]
  }
}

```

## Next Steps

Explore the interactive API reference below to view the complete endpoint specifications, strict data schemas, and granular error codes.
