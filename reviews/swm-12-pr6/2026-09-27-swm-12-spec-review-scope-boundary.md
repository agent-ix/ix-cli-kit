---
id: SR-009
title: "SWM-12 PR 6 scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/functional/FR-016-shared-adoption.md, spec/non-functional/NFR-004-secret-nondisclosure.md, spec/spec.md, spec/stakeholder/StR-003-local-credentials.md, spec/usecase/US-006-use-local-credentials.md, ix-projects/crates/projects-api/src/lib.rs, spec/functional/FR-011-configuration-file-loading.md"
review_set: subset
---
## Summary

Ticket: SWM-12. Checked shared API ownership versus consumer paths, schemas, argv, and adoption proof. The shared crate boundary is clear, but FR-016 AC-1 refers to an unspecified existing `ix-projects` settings path.

## Verdict

**CONDITIONAL — name the app-local behavior and path that adoption must preserve.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-016-AC-1 cannot identify which existing ix-projects settings path its adoption must retain | spec/functional/FR-016-shared-adoption.md:31 |

## Analysis

Measured `ix-projects` at `crates/projects-api/src/lib.rs:293-326`: Linear and write-token credentials come from environment variables, relay settings come from `relay_core::config::load_core`, and a separate `PROJECTS_DRIFT_CONFIG_PATH` controls a drift JSON file. The criterion never says which of these paths is the preservation target or which credential flow must use the new API. The consumer should select its own path, but the acceptance criterion needs an observable referent.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2 |

## Disposition Verdict

**PASS at ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2.** FR-016 names the LINEAR_API_KEY env-over-store flow and says no credential settings file is added while relay, cache, and drift paths remain app-owned.
