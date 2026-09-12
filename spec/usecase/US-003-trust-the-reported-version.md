---
id: US-003
title: "Trust the version a built binary reports about itself"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-001"
    type: "traces_to"
---
# US-003: Trust the version a built binary reports about itself

## Story

**As a** reviewer reading a report that cites the tool version it was measured with
**I want** that version to be the version of the binary that actually produced the numbers
**So that** I can reproduce the measurement, and so that a stale build cannot quietly
attribute its results to a release it is not.

## Context

agent-ix/quire-cli#52: the `0.24.0` through `0.28.0` tags all shipped binaries
reporting `0.23.0`. Three SpecReviews in `agent-ix/filament-ide-rs` cite numbers from a
binary whose self-reported version was wrong. The only implementation of an agreement
check in this ecosystem was `quoin/scripts/check-version-agreement.mjs`, written in
Node, so no Rust tool could run it.

## Acceptance Examples (Illustrative)

### US-003-EX-1: The surfaces agree

- **Given** a built binary that reports a version from more than one surface
- **When** the check runs
- **Then** it confirms the surfaces report the same version

### US-003-EX-2: A check that saw nothing does not pass

- **Given** a binary that could not be run at all
- **When** the check runs
- **Then** it reports that agreement is unverifiable, rather than passing

### US-003-EX-3: A release tag reports itself

- **Given** a checkout sitting exactly on a release tag
- **When** the check runs
- **Then** it requires the binary to report that tag's version

### US-003-EX-4: A build ahead of its tag is not a failure

- **Given** a checkout with commits after the last tag
- **When** the check runs
- **Then** the tag half is skipped with the reason stated, not failed

## Options (Exploratory)

Discussed: comparing two constants in a unit test; rewriting the Node script per repo;
or shipping a reusable helper that runs against the compiled artifact. The first cannot
see the defect, since a constant read by the test that bakes it always agrees with
itself.

## Constraints (Contextual)

The check must run against a built artifact, which means an integration test rather
than a unit test. Contextual.

## Dependencies (Contextual)

Upstream: `quire-cli/build.rs` and `quoin/scripts/check-version-agreement.mjs`.
Downstream: requirements on build-time provenance and on the agreement assertion.

## Priority and Risk (Informative)

High value: every SpecReview's citation depends on it. The defect has shipped once.

## Notes (Informative)

Noted in discovery: an unresolvable provenance value should be visibly unknown rather
than replaced by something plausible.

## Traceability (Informative)

Traces to the stakeholder need for one contract and drives the provenance requirements.
