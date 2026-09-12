---
id: US-001
title: "Script two Agent-IX tools with one exit-status check"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-001"
    type: "traces_to"
---
# US-001: Script two Agent-IX tools with one exit-status check

## Story

**As an** automation author driving several Agent-IX command-line tools from one script
**I want** every tool's exit status to mean the same thing
**So that** I can write one branch on the status and trust it for all of them, instead of
learning a separate numbering per binary and discovering the differences in production.

This story expresses the automation author's perspective informally and does not
prescribe which numbers carry which meaning.

## Context

The five surveyed Rust tools each invented their own taxonomy. The case that keeps
biting is the middle one: a run that found problems, reported them, and still produced
a complete, valid result. Three tools invented a status for it independently and none
of the three agreed. `runQuireAllowFailure` in `quoin`'s `src/quire/exec.ts` is a
workaround written because a caller could not tell that case apart from a crash.

## Acceptance Examples (Illustrative)

These clarify the author's expectations. They are illustrative only.

### US-001-EX-1: A qualified result is still a result

- **Given** a run that completes and reports findings
- **When** the script inspects the exit status
- **Then** the script learns that the output is worth reading, without parsing it

### US-001-EX-2: A refusal is distinguishable from a malformed request

- **Given** a request the tool understood and declined, and a second request that was
  not well-formed
- **When** the script inspects each status
- **Then** the two are different numbers

### US-001-EX-3: An unclaimed status is honestly unknown

- **Given** a status the taxonomy does not define
- **When** the script asks what it means
- **Then** it is told the status is not one of the known outcomes, rather than being
  given a plausible guess

## Options (Exploratory)

Approaches raised in discovery: adopting the widest existing taxonomy wholesale;
negotiating a new numbering across all five tools; or moving one tool's taxonomy into
a shared crate and deleting the others. None implied commitment at the time.

## Constraints (Contextual)

Some of the surveyed tools already have released statuses that callers depend on, so a
renumbering has a migration cost. Contextual only.

## Dependencies (Contextual)

Upstream: the existing taxonomies in the five surveyed tools. Downstream: a likely
functional requirement defining the taxonomy and one reserving an unimplemented status.

## Priority and Risk (Informative)

High value, high urgency: the ambiguity has already caused silent data loss once.

## Notes (Informative)

Open question captured in discovery: whether a "command not found" status should be
part of the taxonomy now or reserved until something implements it.

## Traceability (Informative)

Traces to the stakeholder need for one contract across the Rust tools, and is expected
to drive requirements on the exit taxonomy and its reserved status.
