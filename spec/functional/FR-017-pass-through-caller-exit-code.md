---
id: FR-017
title: "Pass through caller-owned exit codes without interpreting them"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-001"
    type: "implements"
---
# FR-017: Pass through caller-owned exit codes without interpreting them

## Description

The crate SHALL expose a pass-through wrapper for an opaque caller-owned `u8` process
exit code.

## Inputs

- One caller-owned `u8` process exit code.

## Outputs

- A process exit value carrying the supplied code unchanged, or a typed refusal when
  the supplied value is `5`.

## Behavior

- The pass-through wrapper SHALL accept every `u8` value except `5`.
- When a caller supplies `5`, the crate SHALL return a typed refusal.
- When a caller supplies `5`, the crate SHALL NOT produce a process exit value; `5`
  remains reserved for `command_not_found` under
  [FR-002](./FR-002-reserved-exit-status.md).
- When a caller supplies any value other than `5`, the crate SHALL preserve that value
  unchanged in the process exit value.
- When a caller supplies a value from `1` through `4`, the crate SHALL preserve it as
  an opaque caller-owned value.
- The crate SHALL NOT convert a caller-owned value from `1` through `4` to the
  corresponding [`Outcome`](./FR-001-exit-taxonomy.md).
- The pass-through wrapper SHALL NOT define a meaning for a caller-owned value.
- The pass-through wrapper SHALL NOT map a caller-owned value to an `Outcome`.
- The pass-through wrapper SHALL NOT select an output stream.
- The pass-through wrapper SHALL NOT write output. The caller owns stream selection
  through the existing result and diagnostic stream APIs.
- The crate's `Outcome` taxonomy SHALL remain exactly the five outcomes and codes
  defined by [FR-001](./FR-001-exit-taxonomy.md).
- Caller-owned values SHALL NOT become `Outcome` variants.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-017-AC-1 | For each caller-owned value in `0` and `6` through `255` (including `20`, `21`, `22` and `30`), the pass-through wrapper produces a process exit value with exactly that value. | Test |
| FR-017-AC-2 | For each caller-owned value in `1` through `4`, the wrapper preserves that exact value and does not convert it to an `Outcome`. | Test |
| FR-017-AC-3 | A supplied value of `5` returns a typed refusal and no process exit value; `Outcome::from_code(5)` remains `None` and no member of `Outcome::ALL` has code `5`. | Test |
| FR-017-AC-4 | Calling the pass-through wrapper does not write output or choose a stream; result and diagnostic stream selection remains with the caller. | Inspection |

## Dependencies

- **Upstream**: [FR-001](./FR-001-exit-taxonomy.md) owns the five `Outcome` values;
  [FR-002](./FR-002-reserved-exit-status.md) reserves process status `5`.
- **Downstream**: [quire-driver FR-002](ix://agent-ix/quire-driver/FR-002) consumes
  this pass-through contract for QSL-owned CLI exit values.
