---
id: US-002
title: "Redirect a tool's results without losing them to the diagnostic stream"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-001"
    type: "traces_to"
---
# US-002: Redirect a tool's results without losing them to the diagnostic stream

## Story

**As an** operator capturing a tool's output with `>` for later analysis
**I want** the tool's results to land in the file and its commentary to stay on the terminal
**So that** what I capture is the data I asked for, and I can still see what went wrong
while it runs.

The story avoids prescribing how the tool separates the two.

## Context

Measured over `agent-ix/filament-ide-rs`, `quire coverage --scope . > out.txt` produced
a **0-byte file** while 90,462 bytes went to stderr. The same surface rendered
`Coverage: 1238/2390 rows backed (51%)` — a census figure, not a fault — in the same
red as every finding, because one helper wrapped everything in a colour escape and sent
it to stderr.

## Acceptance Examples (Illustrative)

### US-002-EX-1: The redirect captures the data

- **Given** a run that produces a report and reports findings about it
- **When** the operator redirects stdout to a file
- **Then** the file holds the report and none of the findings

### US-002-EX-2: A number does not look like a failure

- **Given** a census figure emitted as a result
- **When** the operator watches the run in a terminal
- **Then** the figure is not styled as an error

### US-002-EX-3: A machine consumer can read the diagnostics

- **Given** a caller that wants findings as data rather than prose
- **When** it selects the machine format
- **Then** each diagnostic arrives as a structured record that names the file and line
  it is about

## Options (Exploratory)

Discussed: colourising by severity only; dropping colour entirely; or splitting the
emit surface in two and making the result half uncolourable. Exploratory only.

## Constraints (Contextual)

Existing callers depend on the byte-for-byte text of today's error messages, so the
error rendering should not gain a prefix. Contextual.

## Dependencies (Contextual)

Upstream: `quire-cli/src/io.rs` and the measured coverage incident. Downstream: likely
requirements on stream discipline, diagnostic rendering, and colour resolution.

## Priority and Risk (Informative)

High value: the failure mode is silent and total. Risk if unmet is an operator
believing a run produced nothing.

## Notes (Informative)

Raised in discovery: a terminal user interface does not print at all, so whatever is
decided must let a consumer obtain the rendered text rather than have it emitted.

## Traceability (Informative)

Traces to the stakeholder need for one contract, and drives the stream and diagnostic
requirements.
