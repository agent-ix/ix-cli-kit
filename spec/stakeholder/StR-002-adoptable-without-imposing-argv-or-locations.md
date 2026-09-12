---
id: StR-002
title: "Consumers need a foundation crate that imposes neither an argument parser nor a configuration location"
type: StR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-013"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-010"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-012"
    type: "satisfied_by"
---
# StR-002: Consumers need a foundation crate that imposes neither an argument parser nor a configuration location

## Stakeholder Need

The shared foundation shall require no change to a consumer's argument parser, its
dependency pins, or the paths it already reads — for a clap-less binary, a clap binary
with derive disabled, and a terminal user interface alike — so that adoption is a
single additive commit rather than a migration.

## Rationale

Two facts observed in the surveyed repositories make this load-bearing rather than
tidy. `engineering-assurance` compiles `clap` with `default-features = false` and no
derive feature, so a foundation crate exporting a derive-generated root command could
not be adopted there at all. And the survey's hardest pin conflict —
`engineering-assurance` at `clap =4.5.47` with derive off against `quire-corpus` at
`clap =4.6.0` with derive on — is unresolvable inside one dependency graph; a
foundation crate that never sees `clap` is simply not party to it.

The location half has the same shape. The ecosystem's real default module root is the
dotdir `~/.ix/filament/modules`, which `quoin` materialises into and `quire-rs` reads
by default. A shared crate that imposed an XDG path would silently relocate a path two
tools already agree on and break every existing install. Each application and module
knows its own shape; the crate must never encode any application's shape.

`ix-cli`, a terminal user interface, is the consumer this crate exists to serve. A TUI
renders a diagnostic into a pane and does not print it, so a foundation whose only
affordance is `eprintln!` forecloses it.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-002-VC-1 | A consumer whose `clap` is compiled without the derive feature can depend on the crate unchanged. | Inspection |
| StR-002-VC-2 | Adopting the crate changes no path any consumer already reads or writes. | Inspection |
| StR-002-VC-3 | A terminal user interface can obtain every rendered string without the crate writing to a process stream. | Demonstration |

## Stakeholders

Primary: the maintainers of `engineering-assurance`, `quire-corpus`, `quire-cli`,
`quoin-core` and `ix-cli`. Affected: operators whose installed module trees must keep
working across the adoption.

## Context and Assumptions

It is assumed consumers pin this crate by git revision, and that this crate declares
its own dependencies as caret ranges rather than exact pins, because two exact pins on
one crate cannot coexist in a dependency graph.

## Stakeholder Constraints (Contextual)

`ix-cli`'s eventual Rust port is hook-and-plugin dispatch rather than a fixed
subcommand enum, so a crate that owned the root command could not host it. This is a
stakeholder-level expectation informing the boundary requirement.

## Dependencies

- **Upstream**: the surveyed `clap` configurations and pin conflict.
- **Downstream**: [FR-013](../functional/FR-013-crate-boundary.md),
  [FR-010](../functional/FR-010-search-path-union.md) and
  [FR-012](../functional/FR-012-xdg-available-not-imposed.md).

## Priority and Risk (Informative)

High urgency: an un-adoptable foundation is an un-adopted foundation, and the
divergence StR-001 describes continues in its absence.

## Traceability

Satisfied by the crate boundary rules, the search-path ownership split, and the
availability-without-imposition of XDG resolution.
