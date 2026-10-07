---
id: FR-017
title: "Pass through caller-owned exit codes without interpreting them"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-001"
    type: "supports"
---
# FR-017: Pass through caller-owned exit codes without interpreting them

## Description

The crate SHALL expose `ix_cli_kit::exit::caller_exit_code(code: u8) ->
Result<std::process::ExitCode, CallerExitCodeError>` for a caller-owned process exit
code.

## Inputs

- One caller-owned `u8` process exit code.

## Outputs

- A `std::process::ExitCode` carrying the supplied code unchanged, or a
  `CallerExitCodeError` when the supplied value is `5`.

## Behavior

- When `code` is not `5`, `caller_exit_code` SHALL return
  `Ok(std::process::ExitCode::from(code))`.
- If `code` is `5`, then `caller_exit_code` SHALL return
  `Err(CallerExitCodeError { .. })`; the caller chooses any process status after this
  error.
- `CallerExitCodeError` SHALL expose public `rejected_code` and `reserved_code` fields;
  the `reserved_code` field SHALL equal
  [`RESERVED_COMMAND_NOT_FOUND`](./FR-002-reserved-exit-status.md).
- `CallerExitCodeError` SHALL implement `Debug`, `Display`, and `std::error::Error`.
- `CallerExitCodeError` SHALL NOT be convertible to
  [`Outcome`](./FR-001-exit-taxonomy.md) or `std::process::ExitCode`.
- `caller_exit_code` SHALL NOT write output.
- `caller_exit_code` SHALL NOT select a stream; result and diagnostic stream selection
  remains with the caller.

## Rationale

US-001 and StR-001 seek one meaning for a shared exit status across tools. QSL FR-285
and ADR-029 CB-4 define QSL's process exit-code table; separately, ADR-029 FR-286
defines the `quire-outcome/1` outcome document. This extension preserves the caller's
existing exit-code contract where a consumer owns a protocol; a second mapping in the
kit would change that contract. Values `1` through `4` therefore remain caller-owned
when passed through this API. This requirement supports the shared taxonomy for its
own `Outcome` API, while making this caller-owned exception explicit.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-017-AC-1 | The public API is `ix_cli_kit::exit::caller_exit_code(code: u8) -> Result<std::process::ExitCode, CallerExitCodeError>`; for every value except `5`, including the QSL statuses `0`, `10`, `20`, `21`, `22` and `30`, it returns an `ExitCode` with the same numeric value. | Test |
| FR-017-AC-2 | For input `5`, the function returns `Err(CallerExitCodeError { .. })` carrying rejected value `5` and `RESERVED_COMMAND_NOT_FOUND`; the kit produces no process status for this error. | Test |
| FR-017-AC-3 | `CallerExitCodeError` exposes the rejected value and reserved constant through its public fields, implements `Debug`, `Display`, and `std::error::Error`, and has no conversion to `Outcome` or `std::process::ExitCode`. | Inspection |
| FR-017-AC-4 | Calling `caller_exit_code` writes no output and selects no stream. | Inspection |

## Dependencies

- **Upstream**: [FR-001](./FR-001-exit-taxonomy.md) defines the kit's five `Outcome`
  values; [FR-002](./FR-002-reserved-exit-status.md) publishes reserved value `5`.
- **Downstream**: `ix://agent-ix/quire-driver/FR-002` consumes this pass-through
  contract for QSL-owned CLI exit values under QSL FR-285 / ADR-029 CB-4.
