---
id: SR-011
title: "SWM-12 PR 6 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/non-functional/NFR-004-secret-nondisclosure.md"
review_set: subset
---
## Summary

Ticket: SWM-12. Checked backend failure modes, identity mapping, and secret disclosure boundaries. Typed locked/unavailable errors and failure without file fallback are stated; the identity rule does not cover collisions between different scope/key pairs.

## Verdict

**FAIL — require a collision-free credential identity before approving the security boundary.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-014 permits two different app-scope/key pairs to map to one OS item; AC-2 tests only identical keys across scopes | spec/functional/FR-014-os-credential-store.md:41-42 |

## Analysis

For example, a backend identity assembled as `scope:key` aliases `(scope="a:b", key="c")` and `(scope="a", key="b:c")`, while the current AC-2 still passes. The spec needs an injective mapping (or validation that makes aliasing impossible) and an AC covering distinct pairs with delimiter and normalization edge cases. Store failures are otherwise typed and non-fallback.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2 |

## Disposition Verdict

**PASS at ac879dfd9e0f9b3ada0768350e93e9ebfd485bd2.** FR-014 now requires distinct OS items for every accepted scope/key pair and rejects backend identity normalization; AC-2 adds adversarial pair examples.
