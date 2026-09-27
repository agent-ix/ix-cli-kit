---
id: SR-012
title: "SWM-12 PR 6 risk-complexity review"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/ix-cli-kit@8c3c32d3583eeb7b137a87f6c48a30b76e7441a7; spec/functional/FR-014-os-credential-store.md, spec/functional/FR-015-secret-source-precedence.md, spec/functional/FR-016-shared-adoption.md, spec/non-functional/NFR-004-secret-nondisclosure.md, spec/stakeholder/StR-003-local-credentials.md"
review_set: subset
---
## Summary

Ticket: SWM-12. Scored the new obligations for technical risk and volatility. FR-014 and NFR-004 are high technical risk because OS credential services and secret redaction are security boundaries; the identified hazards are recorded in focused findings.

## Verdict

**CONDITIONAL — resolve SR-011 identity and SR-010 adapter evidence before tasking the backend.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | spec/functional/FR-014-os-credential-store.md:- |

## Analysis

| Requirement | Technical risk | Volatility | Driver | Mitigation |
| --- | --- | --- | --- | --- |
| StR-003 | medium | low | Shared failure semantics | Verify typed failures in both consumers |
| FR-014 | high | medium | Three OS credential APIs and app isolation | Collision-safe identity and platform contract tests |
| FR-015 | medium | low | Override precedence and secret-safe metadata | Focused precedence and sentinel tests |
| FR-016 | medium | medium | Two consumer adoption boundaries | Name flows; import and integration proof |
| NFR-004 | high | low | Secret disclosure through formatting and serialization | Redaction type/API inspection and sentinel tests |

Top hazards: cross-app identity collision (SR-011), platform adapter drift (SR-010), and dependency-budget pressure (SR-007). Failure-domain cross-check is SR-011.
