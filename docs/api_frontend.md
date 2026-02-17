## Guía rápida para Frontend / Integraciones

Este documento resume los endpoints disponibles y los payloads mínimos necesarios desde una app web o herramienta interna.

### Healthcheck
- `GET /health` → `{ "status": "ok" }`
- Use para comprobar que el binario está vivo antes de mostrar UI.

### Listar reglas
- `GET /api/v1/rules?page=1&limit=20`
- Respuesta: `{ data: [Rule], pagination: { page, limit, total } }`
- Cada `Rule` ya viene con nombre, descripción, tags y el estado actual (active/draft/etc.).

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

### Solicitar decisión
- `POST /api/v1/decisions`
  ```json
  {
    "event": {
      "header": { "timestamp": "2025-02-01T03:04:05Z", "source": "checkout", "event_id": "evt_123" },
      "context": { ... },
      "signals": { "flags": {} },
      "payload": { "money": { "value": 7500, "ccy": "USD" }, "parties": { ... }, "extensions": { "device": { "trust_score": 0.33 } } }
    }
  }
  ```
- Respuesta:
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
          "explanation": "Monto > 5000 y device nuevo"
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

### Errores comunes
- `400` → JSON mal formado / campos faltantes.
- `404` → `rule_id` inexistente.
- `409` → creación de regla con `id` duplicado.
- `500` → error interno; revisar logs (`target: BANNER`) y notificar al equipo backend.

Mantén esta guía cerca del equipo de diseño/frontend para asegurar que las pantallas reflejen el estado real del motor.
