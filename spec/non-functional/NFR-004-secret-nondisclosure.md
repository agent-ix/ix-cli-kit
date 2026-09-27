---
id: NFR-004
title: "Secret values stay out of formatting and settings serialization"
type: NFR
quality_attribute: security
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-014"
    type: "constrains"
  - target: "ix://agent-ix/ix-cli-kit/FR-015"
    type: "constrains"
---
# NFR-004: Secret values stay out of formatting and settings serialization

## Statement

The shared API SHALL keep secret values out of its Debug and Display output,
error messages, source reports, diagnostic rendering, and ordinary settings
serialization.

## Scope

Applies to values handled by the new credential API. A consumer that explicitly
reveals a value to call an authenticated service remains responsible for its own
downstream use of that value.

## Rationale

The existing generic `Resolved<T>` derives Debug, and the ordinary config
loader accepts serializable settings. A secret-specific value type without
Serialize or Display, and without a revealing Debug representation, is needed
so those ordinary code paths cannot reveal credentials.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Secret-value occurrences in shared API formatting, errors, diagnostics, and settings serialization | 0 | 0 | Sentinel-value tests and API inspection |
| Shared API filesystem fallback writes for secret values | 0 | 0 | Backend-failure test and source inspection |

## Verification

Use a sentinel credential in get, set, resolve, status, and failure tests; inspect
every rendered or serialized shared API output for the sentinel. Inspect the
public secret type for Debug, Display, and Serialize behavior and verify that
backend failure does not write any secret file.

## Dependencies

- **Upstream**: [FR-014](../functional/FR-014-os-credential-store.md) and
  [FR-015](../functional/FR-015-secret-source-precedence.md).
