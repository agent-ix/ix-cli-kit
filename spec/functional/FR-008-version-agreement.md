---
id: FR-008
title: "Assert that every version surface of a built binary agrees"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-003"
    type: "implements"
---
# FR-008: Assert that every version surface of a built binary agrees

## Description

The crate SHALL compare the version reported by each surface of a built binary against
the manifest version, and SHALL report a surface that reports nothing as unverifiable
rather than as agreement.

## Inputs

- A manifest version string.
- Named surfaces: a literal string, or a command whose output is searched.
- Optionally, a `git describe` output to check a clean tag against.

## Outputs

- `Agreement` builder: `surface(name, raw)`, `command(name, ...)`,
  `against_git_tag(describe)`, `check() -> Report`, `assert_agreed()`.
- `Report { version, surfaces, describe, tag_check_skipped }` and
  `Disagreement { Unverifiable, Mismatch, TagMismatch }`.
- `semver_in(text) -> Option<Surface-version>`.

## Behavior

- The crate SHALL extract the first `MAJOR.MINOR.PATCH` triple found in a surface's
  text, accepting a bare or a `v`-prefixed form.
- The crate SHALL carry any suffix following the triple verbatim, so a `git describe`
  suffix is not silently dropped.
- The crate SHALL search both standard output and standard error of a surface command,
  because a tool may print its version to either.
- If a surface's text contains no version triple, then the crate SHALL record that
  surface as reporting nothing.
- If a surface command cannot be run, then the crate SHALL record it as reporting
  nothing rather than as passing.
- If fewer than two surfaces reported a version, then the crate SHALL report
  `Unverifiable` rather than agreement.
- If a surface's version differs from the manifest version, then the crate SHALL report
  a `Mismatch` naming the surface and both values.
- When a `git describe` output names a clean tag, the crate SHALL compare it to the
  manifest version and SHALL report a `TagMismatch` when they differ.
- When the describe output is not a clean tag, the crate SHALL record the tag check as
  skipped instead of performing it.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-008-CON-1 | Silence SHALL NOT be treated as agreement anywhere in this requirement. | Reliability | Test |
| FR-008-CON-2 | A skipped tag check SHALL be visible in the report rather than indistinguishable from a passed one. | Observability | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | A bare version and a `v`-prefixed version are both found; on a line with two versions the first wins. | Test |
| FR-008-AC-2 | A describe suffix travels verbatim with the extracted triple. | Test |
| FR-008-AC-3 | Text with no triple reports nothing rather than a fabricated pair. | Test |
| FR-008-AC-4 | Two surfaces reporting the manifest version pass; a baked value lagging the manifest is a `Mismatch`. | Test |
| FR-008-AC-5 | A single reporting surface yields `Unverifiable`, not a pass. | Test |
| FR-008-AC-6 | A command that cannot be run yields `Unverifiable`, not a pass. | Test (TC-002) |
| FR-008-AC-7 | The surfaces of an actually-built binary agree. | Test (TC-001) |
| FR-008-AC-8 | A clean tag and a drifted describe output are distinguished. | Test |

## Dependencies

- **Upstream**: [FR-007](./FR-007-source-provenance.md)
- **Downstream**: a consumer's `make check-version`.
