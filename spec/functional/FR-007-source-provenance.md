---
id: FR-007
title: "Bake the source revision and working-tree state into the build"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-003"
    type: "implements"
---
# FR-007: Bake the source revision and working-tree state into the build

## Description

The crate SHALL make a checkout's revision and working-tree state, read at build time,
available to the built binary, reporting `unknown` rather than a guess when the
directory is not a checkout.

Provenance is load-bearing because measurements are cited by version: reviews in
`agent-ix/filament-ide-rs` cite numbers from a binary whose self-reported version was
wrong.

## Inputs

- A repository directory.
- A key prefix for the emitted values.

## Outputs

- `SourceIdentity { revision, state }` with `short()` and `short_revision()`.
- `source_identity(repo) -> SourceIdentity`.
- `git_describe(repo)`, `is_clean_tag(describe)`.
- `build::emit_source_provenance(prefix, repo)`, `build::emit_value(key, value)`,
  `build::register_rerun_triggers(repo)`.

## Behavior

- The crate SHALL take the revision from `git rev-parse HEAD`, accepting it only when it
  is 40 hexadecimal characters.
- The crate SHALL take the working-tree state from `git status --porcelain
  --untracked-files=normal`, reporting `clean` for empty output and `dirty` otherwise.
- If the directory is not a checkout, or git cannot be run, then the crate SHALL report
  the state `unknown` and SHALL NOT substitute a plausible value.
- The crate SHALL abbreviate a revision to its first 8 characters for display.
- The crate SHALL register rerun triggers so a build's baked provenance is invalidated
  when the checkout moves.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-007-CON-1 | The spellings `clean`, `dirty` and `unknown` SHALL round-trip, because they are baked into a binary by one build and parsed by another. | Interface | Test |
| FR-007-CON-2 | A failure to determine provenance SHALL NOT fail the build. | Reliability | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | Every `SourceState` round-trips through its baked spelling, and an unrecognised spelling becomes `Unknown`. | Test |
| FR-007-AC-2 | A directory that is not a git checkout yields state `Unknown` and no revision. | Test |
| FR-007-AC-3 | `short_revision` yields the first 8 characters of a 40-character revision. | Test |

## Dependencies

- **Upstream**: [US-003](../usecase/US-003-trust-the-reported-version.md)
- **Downstream**: [FR-008](./FR-008-version-agreement.md)
