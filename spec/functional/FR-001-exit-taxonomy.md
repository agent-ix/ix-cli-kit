---
id: FR-001
title: "Report process outcome as one of five defined exit statuses"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-001"
    type: "implements"
---
# FR-001: Report process outcome as one of five defined exit statuses

## Description

The crate SHALL define exactly five process outcomes — `Ok`, `Partial`, `Refused`,
`Invalid` and `Internal` — mapped to exit statuses `0`, `1`, `2`, `3` and `4`
respectively, and SHALL expose for each outcome whether its standard output carries a
complete payload.

Adopted verbatim from `quoin-core`'s `protocol::Outcome`. This is a MOVE, not a second
opinion: when `quoin-core` adopts this crate its own file is deleted, so the two must
not diverge.

## Outputs

- `Outcome::code() -> u8`, the process exit status.
- `Outcome::carries_payload() -> bool`, whether stdout holds a complete payload.
- `Outcome::as_str() -> &'static str`, the stable spelling for a diagnostic or a
  `--json` envelope.
- `Outcome::from_code(u8) -> Option<Outcome>`, the recovery of an outcome from an
  observed status.
- `Outcome::ALL`, every status in taxonomy order.
- `impl From<Outcome> for std::process::ExitCode`.

## Behavior

- The crate SHALL map `Ok` to `0`, `Partial` to `1`, `Refused` to `2`, `Invalid` to `3`
  and `Internal` to `4`.
- The crate SHALL report `carries_payload()` as true for `Ok` and `Partial`, and false
  for `Refused`, `Invalid` and `Internal`.
- The crate SHALL report the spellings `ok`, `partial`, `refused`, `invalid` and
  `internal`, one per outcome and all distinct.
- If a status outside the taxonomy is offered to `from_code`, then the crate SHALL
  return `None` rather than a nearest outcome.
- The crate SHALL enumerate every outcome through `Outcome::ALL` so a consumer does not
  discover the surface by trying numbers.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-001-CON-1 | `Partial` SHALL remain a non-zero status whose standard output is a complete payload, learnable from `carries_payload()` rather than from a comparison against zero. | Interface | Test |
| FR-001-CON-2 | The five status numbers SHALL NOT be renumbered while any consumer depends on them. | Compatibility | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | `code()` over `Ok`, `Partial`, `Refused`, `Invalid`, `Internal` yields exactly `[0, 1, 2, 3, 4]`. | Test |
| FR-001-AC-2 | `Partial.carries_payload()` is true while `Partial.code()` is non-zero, and `carries_payload()` is false for `Refused`, `Invalid` and `Internal`. | Test (TC-011) |
| FR-001-AC-3 | For every outcome in `ALL`, `from_code(outcome.code())` returns that same outcome. | Test |
| FR-001-AC-4 | `as_str()` over `ALL` yields `["ok", "partial", "refused", "invalid", "internal"]`. | Test |
| FR-001-AC-5 | A run given an argument that is not a well-formed request exits `Invalid` (3), not `Refused` (2). | Test (TC-013) |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-one-exit-check-for-two-tools.md)
- **Downstream**: [FR-002](./FR-002-reserved-exit-status.md) reserves the adjacent status
