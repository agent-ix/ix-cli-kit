---
id: SR-004
title: "SWM-12 PR 6 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/functional/FR-016-shared-adoption.md, spec/non-functional/NFR-004-secret-nondisclosure.md, spec/spec.md, spec/stakeholder/StR-003-local-credentials.md, spec/usecase/US-006-use-local-credentials.md, README.md"
review_set: subset
---
## Summary

Ticket: SWM-12. Reviewed the seven changed spec files against the current crate and README. No source, tests, or dependencies changed; the roadmap statement for secrets now contradicts the proposed spec boundary.

## Verdict

**CONDITIONAL — align the public roadmap with the new prospective credential contract.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | README still says secrets require a TUI credential-flow spec, while this PR commits a different trigger and module boundary | spec/spec.md:94-100 |

## Analysis

The README roadmap at line 65 still says the secrets trigger is a written TUI credential-flow specification. This PR instead commits the shared module for `ix-projects` and a Rust CLI. Update the roadmap so readers and future implementers have one active trigger. No vendored source or copied backend appears in this diff.

The `/rust-review` design pass found no Rust source, manifest, CI workflow, or test changes in this PR. The proposed validated scope/key types, typed errors, and non-revealing secret type fit the crate's public Rust API conventions; implementation-level checks and gates belong to the later code PR.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2 |

## Disposition Verdict

**PASS at ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2.** README now states the SWM-12 secrets specification is prospective, off by default, and separate from the TUI login flow; the roadmap row at line 70 agrees.
