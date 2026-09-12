---
id: FR-013
title: "Ship no argv parser and no command-line framework dependency"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-004"
    type: "implements"
---
# FR-013: Ship no argv parser and no command-line framework dependency

## Description

The crate SHALL leave argv parsing, the root command type, and the choice of
command-line framework entirely to the consumer, depending on no argument parsing crate
itself, so that a consumer's command shape remains the consumer's decision.

`quoin-core` is a subprocess invoked as `quoin-core <domain>.<op>` — one command-shaped
operation per invocation. A TUI consumer has no argv at all. A crate that exported a
`Parser`-derived root type would impose a shape on both.

## Outputs

- Selector types that a consumer's own parser can target: `DiagnosticsFormat` and
  `ColorChoice`, each implementing `FromStr`.

## Behavior

- The crate SHALL NOT read `std::env::args`.
- The crate SHALL NOT declare a dependency on `clap` or any other argv parsing crate.
- The crate SHALL parse `human` and `json` into `DiagnosticsFormat`.
- The crate SHALL parse `auto`, `always` and `never` into `ColorChoice`.
- If a selector value is unrecognised, then the crate SHALL return an error naming the
  value it rejected.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-013-CON-1 | `Cargo.toml` SHALL carry no `clap` dependency, direct or optional. | Architecture | Inspection |
| FR-013-CON-2 | The selector error messages are part of the interface, because a consumer surfaces them to a user. | Interface | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-013-AC-1 | `"human"` and `"json"` parse into the corresponding `DiagnosticsFormat`. | Test |
| FR-013-AC-2 | `"auto"`, `"always"` and `"never"` parse into the corresponding `ColorChoice`. | Test |
| FR-013-AC-3 | An unrecognised selector value yields an error whose message contains the rejected value. | Test |
| FR-013-AC-4 | The manifest declares no argv parsing dependency and the crate exports no root command type. | Inspection |

## Dependencies

- **Upstream**: [StR-002](../stakeholder/StR-002-adoptable-without-imposing-argv-or-locations.md)
- **Downstream**: `quoin-core`'s own dispatch.
