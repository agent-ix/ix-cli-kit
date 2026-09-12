---
id: SR-003
title: "evidence review of the ix-cli-kit founding spec set"
type: SpecReview
analysis: evidence
scope: "spec/functional/**, spec/non-functional/**, src/**, tests/**"
review_set: subset
---
## Summary

Checked that every acceptance criterion is bound to real evidence and that no criterion
was authored to match a test rather than the other way round. All 47 existing unit and
integration tests were bound with `/// Trace:` tags against the authored criteria; no
test was weakened, renamed or rewritten to fit a criterion. The measured behaviour that
FR-010 asserts was obtained by executing each implementation, not by reading matching
string literals. The `/// Trace:` omission that `agent-ix/quire-research#69` recorded as
deliberate-pending-a-spec is closed by this change.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-002 | medium | FR-010's divergence table is measured, not inferred: each implementation was executed under controlled environments. `IX_HOME` is honoured by `quoin` and silently ignored by both Rust readers, so a user who sets it has `quoin` install into one tree while `quire` reads another. This is a live divergence and is recorded in the spec rather than specified away. | FR-010 | wrong-requirement |
| FND-003 | medium | NFR-001 (canonical encoding under `preserve_order` feature unification) is verified only indirectly: the tests assert sorted order against deliberately unsorted input, which passes under either map implementation only if sorting is explicit. No build actually enables `preserve_order`, so the cross-configuration byte-identity metric is argued, not measured. | NFR-001 | correct-requirement-no-evidence |
