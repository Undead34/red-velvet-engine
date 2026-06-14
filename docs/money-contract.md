# Monetary Handling Contract

This document captures the cross-cutting rules we just approved for dealing with
amount precision, FX conversion, and alert evidence in Red Velvet Engine
(RVE). Nothing here changes the stateless nature of `crates/rve-core`; it
defines what the surrounding services (ingestion, FX, alerting, UI) must honor
to keep the fraud engine deterministic and auditable.

## 1. Precision via `minor_units`

- **Canonical transport**: Every event that hits `/api/v1/decisions` MUST send
  `payload.money.minor_units` (see `crates/rve/src/http/openapi.rs`). No floats
  are accepted by the public API; the compatibility shim only exists for
  legacy `value` fields and re-emits integers before deserialization.
- **Normalization source**: Use the metadata exposed by `rve-money` (currency
  exponent, ISO-4217 code, crypto exponent) to translate human inputs into a
  non-negative integer. Example: `"3500.15" USD → 350015 minor units,
  `"0.00000042" BTC → 42 minor units (satoshis).
- **Internal math**: Rules, features, and services operate purely on integers
  (`Money`, `AssetAmount`). Any cent-level fraud (e.g., shaving pennies) is
  detectable because comparisons, modulo checks, and aggregations work on the
  smallest unit supported by the asset.
- **UI formatting**: Dashboards format integers back into display values using
  the same currency exponent, never by dividing floats arbitrarily. That keeps
  what engineers see (integers) aligned with what Risk Ops sees (localized
  decimals) without losing precision.

## 2. FX Resolution with USD Numeraire

- **Hub model**: The FX Service is the only component allowed to resolve
  currency differences. It uses USD as the numeraire/pivot. Any pair that is
  not directly known is converted by chaining through USD.
- **Request contract**:
  - `base`: currency of the transaction (`payload.money.ccy`).
  - `quote`: currency of the rule threshold (where comparisons must happen).
  - `timestamp`: when the transaction occurred (UTC). Allows historical rates.
- **Response contract**:
  - `rate`: decimal expressed as rational components (`numerator`,
    `denominator`) or a fixed-point integer to avoid floats.
  - `path`: ordered list describing the conversion, e.g., `BTC -> USD -> VES`.
  - `observed_at`, `source`: metadata recorded in the alert snapshot.
- **Responsibilities**:
  - Guarantee deterministic rounding strategy (banker’s/round-half-up) and
    return the converted amount as `minor_units` in the rule currency.
  - Reject unsupported combinations before they reach the engine. The UI and
    rule builder should use `/api/v1/metadata/contract` to limit currency
    choices to assets present in the catalog or known FX routes.
  - Cache or prefetch rates as needed, but never leak partially converted
    values to the Core; the ingestion layer must produce a fully hydrated event
    with the converted integer.

## 3. Immutable Alert Snapshots

When a rule fires, the alerting service persists an immutable snapshot. This is
outside the RVE process but downstream systems must capture the following
fields so that UI and audit tooling can explain every decision:

| Field | Description |
| --- | --- |
| `transaction` | Original amount + currency + payload stored exactly as received. |
| `rule` | Rule ID, version, currency, threshold, schedule info at evaluation time. |
| `evaluation` | Converted amount in rule currency (minor units) and comparison operator result. |
| `fx` | Rate (as rational or fixed-point), base, quote, intermediate path, provider, observed timestamp. |
| `extensions` | Optional derived values used by UI (e.g., analyst’s display currency) but never overwrite originals. |

Rules from `/home/undead34/Projects/rve-project/docs/arch.md` apply directly:

1. Snapshots are never rewritten. If the business changes its “default currency”
   tomorrow, you version rules and keep historical alerts intact.
2. UI surfaces four truths per alert: original transaction, rule threshold,
   evaluated amount, FX rate/path. The user can request display in another
   currency, but that is a new derived field layered on top of the immutable
   snapshot.
3. Every snapshot references the exact rule version (`RuleIdentity.version`) and
   timestamps. Combined with the FX metadata, auditors can replay any
   evaluation even if the system’s defaults change later.

## Operational Flow Summary

1. **Ingress**: Normalize external payloads into integers using `rve-money`.
2. **FX Pass**: If event currency ≠ rule currency, call the FX Service (USD
   hub) and embed the converted amount + metadata before invoking RVE.
3. **Evaluation**: RVE compares `payload.money.minor_units` against rule logic
   (JSONLogic on canonical fields) and returns a `Decision`/`DecisionTrace`.
4. **Snapshot**: Alerting service stores immutable evidence with original
   payload, rule version, converted amount, and FX metadata.
5. **Presentation**: UI renders both raw and user-preferred currencies without
   mutating stored values; analysts can see cent-level discrepancies because all
   calculations were done with integers.

Following this contract keeps cent-fraud detection accurate, avoids the
combinatorial explosion of precalculating FX pairs, and guarantees auditors can
trace every conversion even as business defaults evolve.
