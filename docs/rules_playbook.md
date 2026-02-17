## Playbook de Reglas — Red Velvet Engine

Este documento sirve como guía práctica para los equipos que necesitan crear/editar reglas sin explorar el código fuente.

### Campos obligatorios al crear una regla

| Campo | Tipo | Descripción |
| --- | --- | --- |
| `meta.name` | string | Nombre legible. Úsalo como título en dashboards. |
| `meta.description` | string? | Explicación corta para analistas / UI. |
| `meta.version` | semver (ej. `"1.0.0"`) | Versionado manual para auditoría. |
| `meta.autor` | string | Dueño de la regla (persona o equipo). |
| `state.mode` | **Solo** `staged`, `active`, `suspended`, `deactivated` | Estados canónicos descritos abajo. |
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
6. **`evaluation.logic`**: construye JSONLogic usando los campos disponibles (ver sección "Variables disponibles" más abajo). Usa siempre rutas completas (`{"var": "payload.money.value"}`) para evitar ambigüedades.
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

### Variables disponibles en JSONLogic (`var`)

El motor construye un contexto plano con estas claves raíz:

| Variable raíz | Descripción | Ejemplos de rutas |
| --- | --- | --- |
| `event` | Evento completo, incluye `header`, `context`, `signals`, `payload`. | `{"var": "event.header.source"}` |
| `payload` | Alias directo a `event.payload`. | `payload.money.value`, `payload.parties.originator.country` |
| `context` | Alias directo a `event.context`. | `context.fin.current_day_count` |
| `signals` | Alias directo a `event.signals`. | `signals.flags.device_rooted` |
| `extensions` | Alias directo a `event.payload.extensions`. | `extensions.device.trust_score` |
| `transaction` | Atajo a `extensions.transaction` si existe, `null` si no. | `transaction.amount` |
| `device` | Atajo a `extensions.device` si existe. | `device.trust_score` |

Consejos:

- Para evitar `null`, combina con operadores como `missing` o `if` en JSONLogic.
- Si necesitas nuevas señales (por ejemplo `risk_profile`), inclúyelas en `event.payload.extensions` y automáticamente estarán disponibles bajo `extensions.risk_profile`.

### Detalle de `event.header`, `context`, `signals`, `payload`

| Campo | Subcampos | Tipo / Valores | Notas |
| --- | --- | --- | --- |
| `event.header` | `timestamp`, `source`, `event_id?`, `instrument?`, `channel?` | `timestamp`: ISO-8601 (string). Otros campos string libres. | `event_id` habilita idempotencia y bucketing estable. |
| `context.geo` | `address?`, `city?`, `region?`, `country?`, `postal_code?`, `lon?`, `lat?` | strings / coordenadas (`f64`). | Opcional; útil para reglas por país. |
| `context.net` | `source_ip?`, `destination_ip?`, `hop_count?`, `asn?`, `isp?` | IPs (`string`), números (`u8/u32`). | IPs se tratan como strings, el motor no valida formato. |
| `context.env` | `user_agent?`, `locale?`, `timezone?`, `device_id?`, `session_id?` | strings. | `device_id` se refleja en `known_devices`. |
| `context.fin` | ver ejemplo inicial (campos `first_seen_at`, `last_seen_at`, contadores, sets). | números (`u64`), contadores (`u32`), arrays/sets (`[string]`). | Todos los contadores están en unidades naturales (txn count, amount en centavos). |
| `signals.flags` | clave = `signal enum`, valor = `unknown|no|yes`. | Ver lista debajo. | Usa `snake_case` (ej. `vpn`, `email_disposable`). |
| `payload.money` | `value`, `ccy` | `value`: `f64` (importe). `ccy`: string ISO-4217. | Si prefieres centavos, guárdalos en `context.fin`. |
| `payload.parties.originator/beneficiary` | `entity_type`, `acct`, `country?`, `bank?`, `kyc?`, `watchlist`, `sanctions_score?` | strings y `Flag` (`unknown/no/yes`). | `watchlist` se mapea a `Flag`. |
| `payload.extensions` | Diccionario libre `string -> json`. | Recomendado: `device`, `transaction`, `risk_profile`, etc. | El motor crea atajos `device` y `transaction`. |

**Enumeraciones admitidas en `signals.flags`:**

```
vpn, proxy, tor, relay, public_vpn, hosting, timezone_mismatch,
rooted, jailbroken, emulator, virtual_machine, tampering, cloned_app, frida_detected,
incognito, devtools_open, remote_control_suspected,
email_disposable, email_breached, phone_voip, phone_recent_port, has_social_profiles
```

*(Todos se escriben en `snake_case`. Usa `"flags": { "vpn": "yes" }` por ejemplo.)*
#### Significado de cada `state.mode`

- `staged`: reglas en borrador. El motor no las evalúa, pero puedes versionarlas y mostrarlas en UI.
- `active`: regla en producción. Se evalúa siempre que `schedule`/`rollout` lo permitan.
- `suspended`: regla pausada temporalmente. Se mantiene el historial pero no se evalúa.
- `deactivated`: regla retirada permanentemente. Úsalo cuando quieras ocultarla de dashboards o marcarla para borrado definitivo.
