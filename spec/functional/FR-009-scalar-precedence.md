---
id: FR-009
title: "Resolve a scalar setting as flag, then environment, then file, then default"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-005"
    type: "implements"
---
# FR-009: Resolve a scalar setting as flag, then environment, then file, then default

## Description

The crate SHALL resolve a single-valued setting by taking the first supplied layer in
the order flag, environment, file, default, and SHALL report which layer supplied the
value.

The crate owns the ORDER. It does not own the layers' contents: the consumer supplies
each already-read candidate, so no location or schema is decided here.

## Inputs

- Four optional candidates, one per layer, plus a default value.
- For `resolve_parsed`, an environment variable name and a target `FromStr` type.

## Outputs

- `Resolved<T> { value, source, origin }`.
- `Source` — `Flag`, `Env`, `File`, `Default` — with stable spellings.
- `resolve_parsed::<T>(name) -> Result<Option<T>, ConfigError>`.

## Behavior

- The crate SHALL take the flag layer's value when it is present.
- When no flag value is present, the crate SHALL take the environment layer's value.
- When neither is present, the crate SHALL take the file layer's value.
- When no layer supplied a value, the crate SHALL take the default.
- The crate SHALL record which layer supplied the value on the resolved result.
- When the named variable is unset, `resolve_parsed` SHALL report `Ok(None)`.
- If the named variable's value cannot be parsed into the requested type, then
  `resolve_parsed` SHALL return an error naming the variable and the rejected value,
  and SHALL NOT fall through to the next layer.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-009-CON-1 | A malformed environment value SHALL NOT be silently downgraded to the default; that is the behaviour this requirement exists to prevent. | Reliability | Test |
| FR-009-CON-2 | The `Source` spellings are stable, because a consumer prints them in `--json` output. | Interface | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | With all four layers supplied the flag wins; removing layers one at a time yields env, then file, then default, each reporting its own `Source`. | Test |
| FR-009-AC-2 | `Source::as_str` yields `flag`, `env`, `file`, `default`. | Test |
| FR-009-AC-3 | `resolve_parsed` on an unset variable returns `Ok(None)`. | Test |
| FR-009-AC-4 | `resolve_parsed` on a variable whose value does not parse returns `Err(ConfigError::BadEnvValue)` naming the variable and the value. | Test |

## Dependencies

- **Upstream**: [US-005](../usecase/US-005-resolve-search-roots-predictably.md)
- **Downstream**: [FR-011](./FR-011-configuration-file-loading.md) supplies the file layer.
