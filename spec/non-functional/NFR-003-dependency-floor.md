---
id: NFR-003
title: "The dependency surface stays minimal and is declared as caret ranges"
type: NFR
quality_attribute: portability
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-013"
    type: "constrains"
  - target: "ix://agent-ix/ix-cli-kit/FR-014"
    type: "constrains"
---
# NFR-003: The dependency surface stays minimal and is declared as caret ranges

## Statement

The crate SHALL declare only the dependencies its enabled modules require, each as a
caret range with no upper bound, and SHALL forbid unsafe code in its own source.

## Scope

- Applies to: `[dependencies]`, `[features]`, target-specific dependency sections,
  and `[lints]` in `Cargo.toml`.
- Operational context: adoption by consumers whose own dependency graphs this crate must
  not disturb.

## Rationale

The crate exists to be adopted. A consumer that wants exit codes must not inherit a
network stack, an argv parser, or an upper version bound that blocks its own upgrades.
The base library stays at its current three direct runtime dependencies. The
prospective [FR-014](../functional/FR-014-os-credential-store.md) OS adapters
need additional dependencies, so `secrets` is an off-by-default feature and a
dependency boundary. Backend dependencies are optional and target-gated; a
consumer enabling `secrets` takes only dependencies needed by its target. The
crate's `unsafe_code = "forbid"` applies to ix-cli-kit source; OS integration
may use audited upstream crates with their own FFI rather than unsafe blocks in
this crate.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Direct runtime dependencies in the default-feature graph | 3 | 3 | `cargo tree` inspection |
| Unneeded OS backend dependencies in the default graph or on a different target | 0 | 0 | Target-matrix `cargo tree` inspection |
| Dependency requirements carrying an upper bound | 0 | 0 | Inspection |
| `unsafe` blocks in the crate | 0 | 0 | Static Analysis |

## Verification

`cargo deny` and the crate-level `unsafe_code = "forbid"` lint run in the repository's
CI target. Compare direct runtime dependencies with `cargo tree --edges normal
--depth 1` for the default feature set and inspect target-specific trees with
`secrets` enabled on macOS, Linux and Windows. Review any new OS adapter
dependency for license, advisory, and target scope on each manifest change.

## Dependencies

- **Upstream**: [FR-013](../functional/FR-013-crate-boundary.md)
- **Downstream**: every consumer's lockfile.
