---
id: SR-001
title: "base review of the ix-cli-kit founding spec set"
type: SpecReview
analysis: base
scope: "spec/spec.md, spec/stakeholder/**, spec/usecase/**, spec/functional/**, spec/non-functional/**"
review_set: subset
---
## Summary

Checked the founding spec set — one master-requirements document, 2 StR, 5 US, 13 FR and
3 NFR — for ID format, duplication, link integrity, criteria quality and test coverage.
The set validates clean under `quire validate`, IDs are contiguous with no gaps or
duplicates, and every FR carries an acceptance-criteria table bound to at least one
existing test. Four acceptance criteria were authored ahead of their evidence; two were
closed by adding tests over already-shipped functions, and two cannot be tested in
process because the crate forbids unsafe code and `std::env::set_var` is unsafe.

Five acceptance criteria are verified by Inspection rather than Test — `FR-002-AC-4`,
`FR-003-AC-4`, `FR-012-AC-2`, `FR-013-AC-4` and `NFR-002-AC-2`. Each asserts an absence
or a manifest fact that no test can check without becoming a grep. They are not a
coverage hole and are not counted as findings.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-001 | medium | `FR-009-AC-4` (a malformed environment value is an error, never a fall-through) has no test: exercising it requires mutating the process environment, and `#![forbid(unsafe_code)]` forbids `std::env::set_var`. Closing it needs an out-of-process fixture, which is deliberately not built here. | FR-009 | correct-requirement-no-evidence |
| FND-002 | medium | `FR-010-AC-6` (a deprecation report does not change the resolved root set) has no test, for the same reason: a deprecation is only recorded through `push_env`. | FR-010 | correct-requirement-no-evidence |
| FND-003 | low | `FR-007-AC-3` had no test; closed in this change by `a_revision_abbreviates_to_its_first_eight_characters`. | FR-007 | correct-requirement-no-evidence |
| FND-004 | low | `FR-009-AC-3` had no test; closed in this change by `an_unset_variable_parses_to_nothing_rather_than_to_a_default`. | FR-009 | correct-requirement-no-evidence |
