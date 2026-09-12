---
id: FR-003
title: "Send results to stdout and diagnostics to stderr, never colourising a result"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-002"
    type: "implements"
---
# FR-003: Send results to stdout and diagnostics to stderr, never colourising a result

## Description

The crate SHALL emit command results on standard output and diagnostics on standard
error, and SHALL NOT apply colour to a result on any stream.

Moved from `quire-cli/src/io.rs`. The split is the contract. Before the split existed,
one helper wrapped everything in a red escape and sent it to stderr; measured over
`agent-ix/filament-ide-rs`, `quire coverage --scope . > out.txt` produced a **0-byte
file** while 90,462 bytes went to stderr, and the census line
`Coverage: 1238/2390 rows backed (51%)` rendered in the same red as every finding.

## Inputs

- A result line, or a block of result bytes.
- A diagnostic record together with its resolved [`Diagnostics`] settings.

## Outputs

- `emit_result(&str)` — one result line on stdout.
- `write_result(&mut W, &str)` — the same line to an arbitrary sink.
- `write_primary_stdout(&[u8])` — primary command output as bytes, written and flushed.
- `Record::emit` / `Record::write` — a diagnostic on stderr, or to an arbitrary sink.

## Behavior

- The crate SHALL write every result through the result surface, which targets stdout.
- The crate SHALL write every diagnostic through the diagnostic surface, which targets
  stderr.
- The crate SHALL NOT apply a colour escape to a result, even when colour is enabled: a
  number is not a severity.
- The crate SHALL flush stdout after writing primary output bytes.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-003-CON-1 | No function that writes a result SHALL accept a colour argument. | Interface | Inspection |
| FR-003-CON-2 | A result and a diagnostic produced by one run SHALL NOT appear on the same stream. | Interface | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | For a run that produces both, the payload appears on stdout and does not appear on stderr. | Test (TC-010) |
| FR-003-AC-2 | For the same run, the diagnostic appears on stderr and does not appear on stdout. | Test (TC-010) |
| FR-003-AC-3 | A result written to a sink contains the message and a newline and no escape byte `0x1b`. | Test |
| FR-003-AC-4 | `write_primary_stdout` flushes after writing. | Inspection |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-redirect-results-without-losing-them.md)
- **Downstream**: [FR-004](./FR-004-diagnostic-rendering.md) renders what this stream
  carries
