---
id: NFR-002
title: "Every source file declares its licence and the crate stays unpublished"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-013"
    type: "constrains"
---
# NFR-002: Every source file declares its licence and the crate stays unpublished

## Statement

Every Rust source file in the crate SHALL carry an SPDX licence identifier and a
copyright line, and the manifest SHALL declare `publish = false` while the crate is
consumed by path.

## Scope

- Applies to: every `.rs` file under `src/` and `tests/`, and `Cargo.toml`.
- Operational context: the crate is AGPL-3.0-or-later and is consumed from within the
  organisation by path, not from a registry.

## Rationale

`agent-ix` licences its Rust under AGPL-3.0-or-later, and a file without a header
carries no licence when it is copied out of the tree — which is exactly how this crate's
own modules arrived, moved from `quire-cli` and `quoin-core`. `publish = false` is the
standing ecosystem ruling: no crates.io publication.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Rust source files carrying an SPDX identifier | 100% | 100% | Test |
| Rust source files carrying a copyright line | 100% | 100% | Test |
| Manifest `publish` value | `false` | `false` | Inspection |

## Verification

An integration test walks every Rust source file in the tree and asserts both header
lines are present; the manifest value is checked by inspection at review.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-002-AC-1 | Every Rust source file in the tree declares `SPDX-License-Identifier: AGPL-3.0-or-later` and a copyright line. | Test |
| NFR-002-AC-2 | `Cargo.toml` declares `publish = false`. | Inspection |

## Dependencies

- **Upstream**: the organisation's licence policy.
- **Downstream**: any consumer vendoring a module out of this crate.
