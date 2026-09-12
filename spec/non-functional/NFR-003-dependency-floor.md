---
id: NFR-003
title: "The dependency surface stays minimal and is declared as caret ranges"
type: NFR
quality_attribute: portability
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-013"
    type: "constrains"
---
# NFR-003: The dependency surface stays minimal and is declared as caret ranges

## Statement

The crate SHALL declare only the dependencies its shipped modules require, each as a
caret range with no upper bound, and SHALL forbid unsafe code.

## Scope

- Applies to: `[dependencies]` and `[lints]` in `Cargo.toml`.
- Operational context: adoption by consumers whose own dependency graphs this crate must
  not disturb.

## Rationale

The crate exists to be adopted. A consumer that wants exit codes must not inherit a
network stack, an argv parser, or an upper version bound that blocks its own upgrades.
Feature-gating a capability is the mechanism for anything heavier; the gate is a
capability boundary, not necessarily a dependency one.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Direct runtime dependencies | 3 | 3 | Inspection |
| Dependency requirements carrying an upper bound | 0 | 0 | Inspection |
| `unsafe` blocks in the crate | 0 | 0 | Static Analysis |

## Verification

`cargo deny` and the crate-level `unsafe_code = "forbid"` lint run in the repository's
CI target; the dependency list is reviewed against this requirement on any manifest
change.

## Dependencies

- **Upstream**: [FR-013](../functional/FR-013-crate-boundary.md)
- **Downstream**: every consumer's lockfile.
