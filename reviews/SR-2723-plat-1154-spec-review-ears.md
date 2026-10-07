---
id: SR-2723
title: "PLAT-1154 EARS conformance of the edited FR-016 statements"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/ix-cli-kit@91f7faf5b3fd1bb3d4d1808123e70f08524dc155; spec/functional/FR-016-shared-adoption.md"
review_set: subset
---

## Summary

Ticket: PLAT-1154. Two edited requirement statements in FR-016 were checked against EARS
grammar.

- **Description, line 13.** "`ix-projects` and one Rust CLI SHALL consume the shared
  credential implementation through `branch = "main"` dependency declarations whose resolved
  commit is recorded in each consumer's `Cargo.lock`, without copying ..."
- **Behavior, line 27.** "Both consumers SHALL declare the authoritative crate using
  `branch = "main"`, retain the resolved commit in `Cargo.lock`, and enable its `secrets`
  feature."

## Verdict

**PASS.** Both are ubiquitous-pattern statements with a named system, a single SHALL and no
trigger or state precondition. The Behavior bullet joins three obligations under one subject
and one SHALL, as the pinned-revision text it replaces joined two. Each of the three is
separately verified in AC-1 and AC-2. No weak verbs ("should", "may", "as appropriate") or
unbounded terms were introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
