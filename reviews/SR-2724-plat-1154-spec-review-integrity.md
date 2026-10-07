---
id: SR-2724
title: "PLAT-1154 integrity of the consumption model across the spec set"
type: SpecReview
analysis: integrity
scope: "agent-ix/ix-cli-kit@91f7faf5b3fd1bb3d4d1808123e70f08524dc155; spec/functional/FR-016-shared-adoption.md, spec/spec.md, spec/stakeholder/StR-002-adoptable-without-imposing-argv-or-locations.md, spec/non-functional/NFR-003-dependency-floor.md"
review_set: subset
---

## Summary

Ticket: PLAT-1154. FR-016 now requires consumers to declare `branch = "main"` and keep the
resolved commit in `Cargo.lock`. This review checked that change against every other spec
statement about how consumers depend on the crate. The search covered `spec/` for "rev =",
"rev=", "revision", "pin" and 40-hex strings.

No `rev =` string or SHA remains in `spec/` outside the historical `reviews/` folder. The
FR-016 ids, cross-references and matrix rows are intact.

## Verdict

**FAIL: two findings.** The spec set now holds two consumption models. FR-016 says to declare
the branch, while the SRS and StR-002 still say consumers pin by git revision. The PR edited
only FR-016.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec/spec.md still states the old model in three places: §2.1 lists "pinned-revision adoption by `ix-projects` and one Rust CLI", §2.2 says "consumers pin it by git revision", and line 110 says "Consumers pin this one crate by git revision". Each contradicts FR-016's `branch = "main"` declaration | spec/spec.md:56, spec/spec.md:83, spec/spec.md:110 |
| FND-002 | medium | StR-002's assumptions still say "It is assumed consumers pin this crate by git revision". That stakeholder assumption now contradicts FR-016's Description and AC-1 and AC-2 | spec/stakeholder/StR-002-adoptable-without-imposing-argv-or-locations.md:58 |

### FND-001 detail

Failure scenario: an adoption author reads the SRS scope (§2.1) first, sees "pinned-revision
adoption", and writes `rev = "<sha>"` into ix-projects. FR-016-AC-1 then fails on inspection,
or the inspector follows the SRS and passes it. Fix: rewrite the three sentences to "consumers
declare it by git branch; the resolved commit lives in each consumer's `Cargo.lock`".

### FND-002 detail

StR-002 is the stakeholder root for adoptability. Its Context and Assumptions section still
grounds the caret-range rationale on revision pinning. Replace "pin this crate by git
revision" with "declare this crate by git branch and record the commit in their lockfile".
The caret-range reasoning that follows is unaffected.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | StR-003-VC-1, the stakeholder parent of FR-016 (`satisfied_by`), still says "use the same pinned shared API". This is residual pin wording that the review pass's "pin" search missed because it matched only the whole word. A lockfile does fix the commit, so it is not strictly false, but it reads as the old revision-pin model that FR-016 and the SRS now replace | spec/stakeholder/StR-003-local-credentials.md:32 |

### FND-003 detail

Suggested fix: "use the same shared API, declared as one `branch = "main"` dependency with
the resolved commit in each consumer's `Cargo.lock`", or simply "the same shared API". The
validation method is unaffected.

## Dispositions

Round 1 was reviewed at fix head 550c14f0405a9327b8556dca36b2e5055fd922f7 (tree a228fe25).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 550c14f0405a9327b8556dca36b2e5055fd922f7: spec/spec.md:56-57, 84-85 and 112-113 now describe `branch = "main"` declarations with the resolved commit in each consumer's `Cargo.lock` |
| FND-002 | fixed | 550c14f0405a9327b8556dca36b2e5055fd922f7: StR-002:58-61 now reads "It is assumed consumers declare this crate with `branch = "main"` and retain the resolved commit in their own `Cargo.lock`." |
| FND-003 | still-open | Found in this disposition pass. No fix has landed yet |
