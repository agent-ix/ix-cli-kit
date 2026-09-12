---
id: US-004
title: "Adopt the foundation from a TUI and from a clap-less command-line tool"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-002"
    type: "traces_to"
---
# US-004: Adopt the foundation from a TUI and from a clap-less command-line tool

## Story

**As a** maintainer of an Agent-IX tool whose argument handling is already decided
**I want** to take the shared exit, stream, JSON and provenance behaviour without also
taking an argument parser
**So that** adopting the foundation is one additive commit, and neither my `clap`
configuration nor my terminal rendering has to change to accommodate it.

## Context

`engineering-assurance` compiles `clap` with `default-features = false` and no derive
feature. `quire-corpus` pins `clap =4.6.0` with derive on; `engineering-assurance` pins
`=4.5.47`. Two exact pins on one crate cannot coexist in a dependency graph. `ix-cli`
is a terminal user interface whose eventual Rust port dispatches through hooks and
plugins rather than a fixed subcommand enum, and which renders diagnostics into panes
rather than printing them.

## Acceptance Examples (Illustrative)

### US-004-EX-1: Adoption does not touch argv

- **Given** a tool with a hand-rolled argument loop
- **When** it adds the foundation
- **Then** its argument handling is unchanged and nothing new parses argv

### US-004-EX-2: A selector is usable from three callers

- **Given** a setting whose value names a format or a colour policy
- **When** it arrives from a command-line flag, from a config file, or from a hand-rolled
  loop
- **Then** the same conversion accepts all three

### US-004-EX-3: A TUI gets the string, not the print

- **Given** an interface that places its own output
- **When** it renders a diagnostic
- **Then** it obtains the rendered text and decides where it goes

### US-004-EX-4: A TUI decides colour from its own facts

- **Given** an interface drawing into a pane that is not the process's stderr
- **When** it resolves whether to colourise
- **Then** it applies the same rule to its own facts rather than inheriting the
  process's

## Options (Exploratory)

Discussed: exporting a derive-generated root command; exporting `clap::ValueEnum`
implementations behind a feature; or exporting only standard-library conversions.
Exploratory only.

## Constraints (Contextual)

The foundation's own dependency declarations should not force a consumer to change its
pins in the same commit as adoption. Contextual.

## Dependencies (Contextual)

Upstream: the surveyed `clap` configurations. Downstream: requirements on the crate
boundary and on the separation of rendering from emitting.

## Priority and Risk (Informative)

High urgency: a foundation that cannot be adopted by its two hardest consumers is not a
foundation.

## Notes (Informative)

Noted: whether a `clap` integration should later ship behind an off-by-default feature
was left open.

## Traceability (Informative)

Traces to the stakeholder need for an adoptable foundation.
