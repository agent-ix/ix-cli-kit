---
id: FR-002
title: "Reserve exit status 5 for command_not_found without implementing it"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-001"
    type: "implements"
---
# FR-002: Reserve exit status 5 for command_not_found without implementing it

## Description

The crate SHALL publish exit status `5` as reserved for `command_not_found`, leaving the
behaviour itself unimplemented in v0.1, so that no consumer gives the number a second
meaning while the behaviour is unbuilt.

`command_not_found` is specified for the not-yet-existing `quoin-cli` at
`ix://agent-ix/quoin/FR-102` and has zero Rust implementations today. What this crate
ships is the number, not the behaviour. Publishing the number is the whole point:
unilateral claiming of a status is exactly how the five surveyed taxonomies diverged.

## Outputs

- `RESERVED_COMMAND_NOT_FOUND: u8`, the constant `5`.

## Behavior

- The crate SHALL expose `RESERVED_COMMAND_NOT_FOUND` with the value `5`.
- The crate SHALL NOT include a `command_not_found` outcome in `Outcome::ALL`.
- If `from_code` is offered `RESERVED_COMMAND_NOT_FOUND`, then the crate SHALL return
  `None` — an unclaimed status is an honest absence, not a guess.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-002-CON-1 | No outcome in the taxonomy SHALL take the value `5` while the reservation stands. | Interface | Test |
| FR-002-CON-2 | The behaviour behind the reservation is owned by `ix://agent-ix/quoin/FR-102` and SHALL NOT be implemented in this crate in v0.1. | Scope | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | `RESERVED_COMMAND_NOT_FOUND` equals `5`. | Test |
| FR-002-AC-2 | `Outcome::from_code(5)` returns `None`. | Test |
| FR-002-AC-3 | No member of `Outcome::ALL` has `code() == RESERVED_COMMAND_NOT_FOUND`. | Test |
| FR-002-AC-4 | The crate exports no `command_not_found` outcome, constructor or handler. | Inspection |

## Dependencies

- **Upstream**: [FR-001](./FR-001-exit-taxonomy.md) defines the taxonomy this reserves
  against; `ix://agent-ix/quoin/FR-102` owns the behaviour.
- **Downstream**: `quoin-cli`, when it exists; [FR-017](./FR-017-pass-through-caller-exit-code.md)
  rejects caller-supplied status `5` in its wrapper. This wrapper guard preserves the
  published reservation; it does not enforce the reservation on consumers that bypass
  the wrapper.
