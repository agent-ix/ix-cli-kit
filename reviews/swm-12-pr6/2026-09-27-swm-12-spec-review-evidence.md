---
id: SR-010
title: "SWM-12 PR 6 evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/functional/FR-016-shared-adoption.md, spec/non-functional/NFR-004-secret-nondisclosure.md, spec/stakeholder/StR-003-local-credentials.md"
review_set: subset
---
## Summary

Ticket: SWM-12. Ran `quoin advise` over the new obligations and compared its recommendations to authored methods. The focused store and precedence criteria use Test; the only substantive evidence gap is that real platform adapter use has Demonstration as its sole method.

## Verdict

**CONDITIONAL — require repeatable integration evidence for each available OS adapter.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014-AC-5 relies on Demonstration for three real OS adapters, leaving no repeatable platform integration evidence | spec/functional/FR-014-os-credential-store.md:54 |

## Analysis

The advisor marks FR-014-AC-5 as a method mismatch and recommends Test methods based on its example and security characteristics. Demonstration can document a manual observation, but cannot make backend regressions visible in CI. FR-016 AC-1/2/4 also appear as advisor mismatches because its broad word matching treats static import and manifest inspections as security tests; inspection remains a sound method for those structural facts.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2 |

## Disposition Verdict

**PASS at ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2.** FR-014-AC-5 now requires CI integration tests for Keychain, provisioned Secret Service, and Credential Manager, with failure when an expected adapter is unavailable.
