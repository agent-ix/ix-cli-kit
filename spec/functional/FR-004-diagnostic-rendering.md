---
id: FR-004
title: "Render a diagnostic in a human or JSON shape, separately from emitting it"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-004"
    type: "implements"
---
# FR-004: Render a diagnostic in a human or JSON shape, separately from emitting it

## Description

The crate SHALL turn a diagnostic record into a line of text in either a human or a
JSON shape, and SHALL offer that rendering independently of writing it to a stream, so
that a consumer which owns its own output surface can use the same vocabulary.

The separation exists for a named consumer class: a TUI cannot let a library write to
its terminal, and a `quoin-core` subprocess may want the JSON shape on its own writer.
A crate that only offered `emit` would be unusable to both.

## Inputs

- A `Record` — kind, message, [`Severity`], and typed [`DiagnosticFields`].
- A [`Diagnostics`] carrying the chosen format and the resolved colour decision.

## Outputs

- `Record::render_human(color) -> String`.
- `Record::render_json() -> String`.
- `Record::render(&Diagnostics) -> String`, selecting by format.
- `Record::write(&mut W, &Diagnostics)` and `Record::emit(&Diagnostics)`.

## Behavior

- The crate SHALL render an error in the human shape as the message text alone, with no
  severity prefix.
- The crate SHALL render a warning in the human shape with a `warning: ` prefix, so a
  warning is not read as an error.
- When colour is enabled, the crate SHALL wrap an error in the red escape and a warning
  in the bold-yellow escape, and SHALL reset at the end of the line.
- The crate SHALL render the JSON shape carrying the record's kind, severity spelling,
  message, and each populated field of [`DiagnosticFields`].
- The crate SHALL omit an unpopulated diagnostic field from the JSON shape rather than
  emit it as null.
- If the JSON shape cannot be encoded, then the crate SHALL emit a marked
  `DiagnosticEncodingFailure` object rather than lose the diagnostic.
- The crate SHALL accept an arbitrary `io::Write` sink for a record, so a consumer that
  owns its terminal never has the crate write to it.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-004-CON-1 | `DiagnosticFields` is the set the ported code populates and is NOT claimed to be complete for every consumer. `quire-cli` also populates `change_target` (`validate.rs:656`, `:696`, `:716`), which v0.1 does not carry; adopting `streams` unchanged would drop it from JSON diagnostics on the validate paths. Tracked as `agent-ix/ix-cli-kit#1`. | Scope | Inspection |
| FR-004-CON-2 | Rendering SHALL NOT read a global, so the same record renders identically under test and under a terminal. | Interface | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | An error rendered human-readably without colour is byte-for-byte the message. | Test |
| FR-004-AC-2 | A warning rendered human-readably carries the `warning: ` prefix. | Test |
| FR-004-AC-3 | The JSON shape of a record carries its kind, its severity spelling, and every populated typed field. | Test |
| FR-004-AC-4 | A record written to an arbitrary sink appears on that sink and not on the process's streams. | Test |

## Dependencies

- **Upstream**: [FR-003](./FR-003-stream-discipline.md) owns which stream receives this;
  [FR-005](./FR-005-colour-resolution.md) supplies the colour decision.
- **Downstream**: `quoin-core`'s diagnostic envelope.
