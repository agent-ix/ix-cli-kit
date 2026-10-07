---
id: SR-2722
title: "PLAT-1154 spec review base checklist of the FR-016 consumption wording"
type: SpecReview
analysis: base
scope: "agent-ix/ix-cli-kit@91f7faf5b3fd1bb3d4d1808123e70f08524dc155; spec/functional/FR-016-shared-adoption.md"
review_set: subset
---

## Summary

Ticket: PLAT-1154. ix-cli-kit PR #12 edits only the consumption wording of FR-016: the
Description, the fourth Behavior bullet, and AC-1 and AC-2. The ids, title, relationships,
AC-3, AC-4 and the Dependencies section are unchanged.

`quire validate spec/functional/FR-016-shared-adoption.md` exits 0 under quire 0.36.1
(engine 0.50.1). It prints the catalog notices `semantic.inline-data-schema`, five
`DuplicateArchetype` notices and one `DuplicateInverseEdge` notice. These are notices, not
errors, and come from the installed module catalog rather than this document. The run is
therefore not reported as clean. The matrix rows keep their ids and statuses.

## Verdict

**PASS with one low finding.** AC-1 and AC-2 remain inspectable: a manifest entry with
`branch = "main"`, a committed `Cargo.lock` holding the resolved commit, and `secrets`
enabled are each checkable by reading two files. The edit names no SHA, revision, tag or
local path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-1 says "The `ix-projects` manifest declares `ix-cli-kit` with `branch = "main"`", but `ix-projects` is a Cargo workspace. If the declaration sits in the root `[workspace.dependencies]` and members write `workspace = true`, two inspectors can disagree on whether "the manifest" declares it | spec/functional/FR-016-shared-adoption.md:36 |

### FND-001 detail

ix-projects' root `Cargo.toml` already declares shared dependencies at the workspace level,
and the projects-api crate is a member. Fix: say "the `ix-projects` dependency declaration
(workspace or member manifest)". AC-2 ("A Rust CLI manifest") has the same latent ambiguity
and is listed as related.
