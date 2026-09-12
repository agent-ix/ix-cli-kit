---
id: NFR-001
title: "Canonical encoding holds under dependency feature unification"
type: NFR
quality_attribute: compatibility
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-006"
    type: "constrains"
---
# NFR-001: Canonical encoding holds under dependency feature unification

## Statement

The canonical JSON encoder SHALL produce key-sorted, byte-identical output for a given
value whether or not `serde_json`'s `preserve_order` feature is unified into the build
by another crate in the consumer's dependency graph.

## Scope

- Applies to: `json::canonical`, `json::sort_in_place`, `json::encode_canonical`.
- Operational context: any consumer workspace, whose other dependencies this crate does
  not control.

## Rationale

Cargo unifies features across a build. A consumer that depends on this crate and on
anything enabling `preserve_order` would change `serde_json::Map` from a `BTreeMap` to
an insertion-ordered map. An encoder that relied on the map type's own ordering would
then silently stop being canonical — in the consumer's build only, never in this crate's
own test run. The encoder therefore sorts explicitly rather than relying on the map.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Key order at every depth of an encoded value | sorted | sorted | Test |
| Byte difference between encodings of one value across feature configurations | 0 bytes | 0 bytes | Test |

## Verification

The canonicalisation tests assert sorted key order at every depth, including inside
arrays, against values constructed with deliberately unsorted insertion order — a check
that passes under either map implementation only if sorting is explicit.

## Dependencies

- **Upstream**: [FR-006](../functional/FR-006-canonical-json.md)
- **Downstream**: `quoin-core`'s payload comparison against the retained TypeScript
  implementation.
