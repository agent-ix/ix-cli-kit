---
id: FR-006
title: "Encode JSON with keys sorted at every depth and no insignificant whitespace"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-002"
    type: "implements"
---
# FR-006: Encode JSON with keys sorted at every depth and no insignificant whitespace

## Description

The crate SHALL produce a canonical JSON encoding in which object keys are sorted at
every depth, including inside arrays, and the output carries no insignificant
whitespace, so that two runs producing the same value produce the same bytes.

This is the form `quoin-core` speaks. `quoin-core` is a subprocess whose response types
are GENERATED from `schemars`, so field order is whatever the generator emits; a
byte-stable payload cannot come from the type definition and has to come from the
encoder. Byte stability is what makes a payload diffable, hashable and comparable
between the retained TypeScript implementation and the Rust one.

## Inputs

- A `serde_json::Value`, or any `Serialize` value.
- A `pretty: bool` selector for indented output.

## Outputs

- `canonical(value) -> Value`, key-sorted at every depth.
- `sort_in_place(&mut Value)`.
- `encode(value, pretty) -> Result<String, JsonError>`.
- `encode_canonical(value) -> Result<String, JsonError>`.

## Behavior

- The crate SHALL sort the keys of every object, at every depth of nesting.
- The crate SHALL sort the keys of objects nested inside arrays, while preserving array
  element order.
- The crate SHALL emit compact output by default, with no space after `:` or `,`.
- When `pretty` is requested, the crate SHALL emit indented output.
- If a value cannot be encoded as JSON, then the crate SHALL return an error naming what
  was being encoded.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-006-CON-1 | Canonicalisation SHALL hold whether or not `serde_json`'s `preserve_order` feature is unified into the build, because a consumer elsewhere in the dependency graph can turn it on. | Compatibility | Test |
| FR-006-CON-2 | Array order SHALL NOT be changed by canonicalisation. | Data | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | Keys are sorted at every depth of a nested object. | Test |
| FR-006-AC-2 | Keys of an object inside an array are sorted and the array's element order is unchanged. | Test |
| FR-006-AC-3 | Compact output contains no space following `:` or `,`; pretty output contains newlines. | Test |
| FR-006-AC-4 | A value that cannot be JSON yields an error whose message names the subject being encoded. | Test |
| FR-006-AC-5 | A payload captured from the process's standard output has sorted keys at every depth and no insignificant whitespace. | Test (TC-012) |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-redirect-results-without-losing-them.md)
- **Downstream**: `quoin-core`'s `schemars`-generated response payloads.
