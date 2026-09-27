---
id: SR-007
title: "SWM-12 PR 6 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/functional/FR-016-shared-adoption.md, spec/non-functional/NFR-004-secret-nondisclosure.md, spec/spec.md, spec/stakeholder/StR-003-local-credentials.md, spec/usecase/US-006-use-local-credentials.md, spec/non-functional/NFR-003-dependency-floor.md, Cargo.toml"
review_set: subset
---
## Summary

Ticket: SWM-12. Checked new StR → US → FR/NFR trace, atomicity, and consistency with existing crate constraints. The OS backend requirement has no reconciled dependency budget: NFR-003 caps direct runtime dependencies at three, and the current manifest already declares three.

## Verdict

**CONDITIONAL — reconcile the OS backend contract with NFR-003 before implementing FR-014.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014 adds three OS backends without reconciling NFR-003’s three-dependency ceiling and unsafe-code ban | spec/functional/FR-014-os-credential-store.md:33-36 |

## Analysis

NFR-003 lines 14–15 and 34–36 apply to the whole crate and set a threshold of three direct runtime dependencies with unsafe code forbidden. Cargo.toml currently has serde, serde_json, and thiserror. The new requirement does not identify an implementation path that satisfies those constraints or revise their applicability or metric; this can push implementers into an undocumented workaround.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2 |

## Disposition Verdict

**PASS at ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2.** NFR-003 now keeps the default library at three direct dependencies, permits optional target-gated OS adapters under `secrets`, and retains the crate-local unsafe ban.
