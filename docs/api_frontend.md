## Guía rápida para Frontend / Integraciones

Este documento resume los endpoints disponibles y los payloads mínimos necesarios desde una app web o herramienta interna.

### Healthcheck
- `GET /health` → `{ "status": "ok" }`
- Use para comprobar que el binario está vivo antes de mostrar UI.

### Listar reglas
- `GET /api/v1/rules?page=1&limit=20`
- Respuesta JSON **exacta**:
  ```json
  {
    "data": [
      {
        "id": "FRAUD-HV-UNTRUSTED-01",
        "meta": {
          "name": "High Value on Untrusted Device",
          "description": "Texto opcional",
          "version": "1.0.0",
          "autor": "Analyst",
          "tags": ["device", "high_value"]
        },
        "state": {
          "mode": "active",
          "audit": {
            "created_at_ms": 1700000000000,
            "updated_at_ms": 1701000000000,
            "created_by": "User",
            "updated_by": "Reviewer"
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

### Crear / editar regla
- `POST /api/v1/rules` con un `Rule` completo (sin `id` → el backend lo genera `FRAUD-AUTO-*`).
- `PUT /api/v1/rules/{id}` reemplaza toda la regla.
- `PATCH /api/v1/rules/{id}` se usa para toggles rápidos:
  ```json
  {
    "state": { "mode": "paused" },
    "rollout": { "percent": 25 }
  }
  ```
- Después de cualquier modificación, la UI puede volver a `GET /api/v1/rules/{id}` para confirmar que el motor recargó la regla (los endpoints responden con el cuerpo actualizado).

### Eliminar regla
- `DELETE /api/v1/rules/{id}` remueve la regla definitivamente.
- Si se quiere un “soft delete”, primero enviar `PATCH` con `state.mode = "draft"` y luego decidir si se elimina.

- ### Solicitar decisión
- `POST /api/v1/decisions`
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
      "context": {
        "geo": { "country": "US" },
        "net": { "source_ip": "1.2.3.4" },
        "env": {},
        "fin": {
          "first_seen_at": 1699999999000,
          "last_seen_at": 1700001111000,
          "last_declined_at": null,
          "total_successful_txns": 42,
          "total_declined_txns": 1,
          "total_amount_spent": 123456,
          "max_ticket_ever": 100000,
          "consecutive_failed_logins": 0,
          "consecutive_declines": 0,
          "current_hour_count": 5,
          "current_hour_amount": 70000,
          "current_day_count": 12,
          "current_day_amount": 300000,
          "known_ips": ["1.2.3.4"],
          "known_devices": ["device-123"]
        }
      },
      "signals": { "flags": {} },
      "payload": {
        "money": { "value": 7500, "ccy": "USD" },
        "parties": {
          "originator": {
            "entity_type": "individual",
            "acct": "cust_01",
            "country": "US",
            "bank": "bank_123",
            "kyc": "tier_2",
            "watchlist": "no",
            "sanctions_score": null
          },
          "beneficiary": {
            "entity_type": "business",
            "acct": "vendor_01",
            "country": "MX"
          }
        },
        "extensions": {
          "device": { "trust_score": 0.33 },
          "transaction": { "amount": 7500 }
        }
      }
    }
  }
  ```
- Respuesta **literal**:
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
          "tags": ["financial_fraud"]
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
- Tips UI: mostrar `score` total y los hits como tarjetas; `rollout_bucket` sirve para debugging, no es necesario en la UI final.

### Estados importantes
- `state.mode` controla si la regla participa (`active`) o queda pausada (`paused`).
- `rollout.percent` permite lanzar gradualmente (p. ej. 10% de tráfico) sin duplicar reglas.
- `schedule.active_from_ms/active_until_ms` (en milisegundos UTC) habilitan ventanas automáticas.
- `schedule.active_from_ms/active_until_ms` (en milisegundos UTC) habilitan ventanas automáticas.

### Referencia completa de JSON
- Este archivo contiene ejemplos **completos** para que frontend pueda copiar/pegar estructuras válidas.
- Si se requiere descripción campo por campo, entregar también `docs/rules_decisions_contract.md` (describe cada propiedad y sus rangos permitidos) como parte del paquete de documentación para equipos externos.

### Errores comunes
- `400` → JSON mal formado / campos faltantes.
- `404` → `rule_id` inexistente.
- `409` → creación de regla con `id` duplicado.
- `500` → error interno; revisar logs (`target: BANNER`) y notificar al equipo backend.

Mantén esta guía cerca del equipo de diseño/frontend para asegurar que las pantallas reflejen el estado real del motor.
