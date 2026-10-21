---
id: FR-018
title: "Detect a CLI install channel and dispatch a requested self-update"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-004"
    type: "supports"
---
# FR-018: Detect a CLI install channel and dispatch a requested self-update

## Description

When a consumer enables the default-off `self-update` feature, the crate SHALL
classify the running executable's install channel and return a report or dispatch
the matching package manager. The consumer supplies package names, repository
and release URLs, and the spelling of its own update command.

## Inputs

- `SelfUpdateConfig`: npm package, Cargo git repository, releases page, and
  update-command spelling, each supplied by the consumer.
- `SelfUpdateOpts`: check-only selection and optional npm registry override.
- The running executable path, or an explicitly supplied path/source for pure
  classification and tests.

## Outputs

- `SelfUpdateReport`: detected `InstallSource`, `Action`, and ordered summary
  messages for the consumer to render.
- `SelfUpdateError`: typed current-executable, spawn, or package-manager failure.

## Behavior

- The crate SHALL classify whole path components: `node_modules` indicates npm,
  `.cargo` indicates Cargo, and otherwise the source is Unknown. Npm wins if both
  components occur.
- For an Unknown source, the crate SHALL return manual npm, Cargo and release
  instructions without starting either package manager, including in a normal
  update run.
- For npm, a check SHALL query the published version.
- For npm, an update SHALL invoke `npm install -g <package>@latest`.
- A scoped npm registry override SHALL use the scope-specific npm flag.
- For Cargo, a check SHALL report branch tracking without inventing a latest
  version.
- For Cargo, an update SHALL invoke `cargo install --git <repo> --force`.
- The crate SHALL report package-manager launch and nonzero exit failures as
  typed errors.
- The crate SHALL return messages rather than write them to a stream.
- The crate SHALL use the consumer-supplied identities in reports.
- The crate SHALL omit another CLI's identity from a consumer's report.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-018-CON-1 | `self-update` SHALL be off by default and add no runtime dependency. | Feature boundary | Inspection |
| FR-018-CON-2 | The crate SHALL leave output rendering and CLI argument parsing to the consumer. | Responsibility | Inspection |
| FR-018-CON-3 | Local and manual CI gates SHALL exercise all features. | Verification | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-018-AC-1 | Component-based detection distinguishes npm, Cargo, and Unknown paths, ignores lookalike names, and selects npm when both components occur. | Test |
| FR-018-AC-2 | A compiled consumer in an Unknown location exits successfully in both check and normal mode, prints manual instructions, and invokes no package manager. | Test |
| FR-018-AC-3 | A scoped npm package receives a scope-specific registry argument, an unscoped package receives the plain registry pair, and no override adds no registry argument. | Test |
| FR-018-AC-4 | A Cargo check reports branch tracking with no latest version and names the configured repository and update command. | Test |
| FR-018-AC-5 | Reports created from a second consumer's config contain that consumer's identities and no first-consumer identity. | Test |
| FR-018-AC-6 | A missing package-manager executable produces a typed spawn error naming the attempted action and program. | Test |

## Dependencies

- **Upstream**: [FR-003](./FR-003-stream-discipline.md) keeps result rendering
  in the consumer.
- **Downstream**: the consuming CLI's own update command and distribution
  coordinates.
