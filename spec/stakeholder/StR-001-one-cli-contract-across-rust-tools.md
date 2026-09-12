---
id: StR-001
title: "Operators need one exit, stream and provenance contract across every Agent-IX Rust CLI"
type: StR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-001"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-003"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-008"
    type: "satisfied_by"
---
# StR-001: Operators need one exit, stream and provenance contract across every Agent-IX Rust CLI

## Stakeholder Need

Every Agent-IX Rust command-line tool shall report its exit status, its stdout content
and its self-reported version under one set of rules, so that a caller driving two of
these binaries interprets both with the same `case $?` and the same redirection.

## Rationale

The 2026-09-12 survey of the five Rust command-line tools in this ecosystem —
`build-chain`, `quire-cli`, `engineering-assurance`, `quire-corpus` and `quoin-core` —
found that all five had an exit taxonomy and all five disagreed. Exit status `2` meant
an argv error in `quire-cli`, a host error in `engineering-assurance`, and a refusal in
`quoin-core`. Three of the five had independently invented a "succeeded but found
problems" status, each with a different number. A caller cannot write one `case $?`
that is correct for two of these binaries.

The same divergence had already caused measured harm on the other two axes.
`runQuireAllowFailure` exists in `quoin`'s `src/quire/exec.ts` because
`quire properties` exits `1` while writing a complete result, and treating that as
total failure silently discarded a whole analysis axis (agent-ix/quoin#103). Measured
over `filament-ide-rs`, `quire coverage --scope . > out.txt` produced a **0-byte file**
while 90,462 bytes went to stderr. And agent-ix/quire-cli#52 shipped tags `0.24.0`
through `0.28.0` as binaries that all reported `0.23.0`, after which three published
SpecReviews cited numbers measured by a binary whose self-reported version was wrong.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-001-VC-1 | A caller can distinguish "complete payload with diagnostics" from "failed, no payload" from the exit status alone, without parsing output. | Demonstration |
| StR-001-VC-2 | A caller redirecting a tool's stdout captures the tool's results and none of its diagnostics. | Demonstration |
| StR-001-VC-3 | A release binary's self-reported version agrees with its package version and, on a clean tag, with the tag. | Demonstration |

## Stakeholders

Primary: operators and automation authors driving Agent-IX command-line tools.
Secondary: the authors of those tools, who are accountable for the contract but do not
consume it. Affected: readers of SpecReview artifacts, whose measurements cite the
tool version that produced them.

## Context and Assumptions

Every tool in scope is a Rust binary built in this ecosystem and distributed by git
revision rather than a registry. It is assumed each tool can adopt a shared library
crate without a lockstep version migration across all of them.

## Dependencies

- **Upstream**: the 2026-09-12 five-CLI survey, and the incidents agent-ix/quoin#103
  and agent-ix/quire-cli#52.
- **Downstream**: [FR-001](../functional/FR-001-exit-taxonomy.md),
  [FR-003](../functional/FR-003-stream-discipline.md) and
  [FR-008](../functional/FR-008-version-agreement.md).

## Priority and Risk (Informative)

High business value: the contract is the interface every script in the ecosystem
depends on. Risk if unmet is silent data loss of the kind already measured twice.

## Traceability

Satisfied by the exit taxonomy, the stream split and the version-agreement assertion.
