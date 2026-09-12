---
id: FR-011
title: "Load a configuration file as absent, present, or malformed at a named position"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-005"
    type: "implements"
---
# FR-011: Load a configuration file as absent, present, or malformed at a named position

## Description

The crate SHALL load a consumer-named configuration file into a consumer-named type,
distinguishing a file that is not there from a file that is there and wrong, and SHALL
name the path, line and column of a malformed file.

The consumer names the path; the crate never derives one.

## Inputs

- A path supplied by the consumer.
- A deserialisation target type, or a caller-supplied parse function.

## Outputs

- `Loaded<T>` — `Absent` or `Present(T)`.
- `load_with(path, parse)`, `load_json(path)`.
- `ConfigError::Unreadable`, `ConfigError::Malformed { path, line, column, message }`.

## Behavior

- When the named path does not exist, the crate SHALL report `Absent`.
- When the named path parses, the crate SHALL report `Present` with the parsed value.
- If the file exists but cannot be read, then the crate SHALL report an `Unreadable`
  error naming the path.
- If the file exists but does not parse, then the crate SHALL report a `Malformed` error
  naming the path, the line, the column and the parser's message.
- The crate SHALL format a malformed error as `path:line:column: message`.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-011-CON-1 | An absent file SHALL NOT be an error: a tool with no configuration file is the normal case. | Behaviour | Test |
| FR-011-CON-2 | A parse failure SHALL NOT be reported without a position; an opaque message hides which file was wrong. | Observability | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-011-AC-1 | Loading a path that does not exist yields `Absent` and no error. | Test |
| FR-011-AC-2 | Loading a malformed file yields an error whose rendering contains the path and the line number of the fault. | Test |
| FR-011-AC-3 | Loading a well-formed file yields `Present` holding the requested type. | Test |

## Dependencies

- **Upstream**: [FR-009](./FR-009-scalar-precedence.md)
- **Downstream**: consumers that have a configuration file.
