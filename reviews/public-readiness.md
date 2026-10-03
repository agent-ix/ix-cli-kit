---
id: SR-014
title: "Public-readiness and contribution review"
type: SpecReview
analysis: code-review
scope: "ix-cli-kit@b10a81fca113949e108a2774825309de6674e210; README.md, CLA.md, CONTRIBUTING.md, CONTENT_RIGHTS.md, LICENSE, Cargo.toml, .github/workflows/cla.yml, .github/workflows/ci.yml, spec/functional/FR-010-search-path-union.md, reviews/2026-09-27-shared-secrets-code-review.md; all fetched Git history"
review_set: subset
---

## Summary

Reviewed publication readiness and all six changed files. No Rust source,
dependency, license, or runtime contract changes. The new agreement workflow
delegates to the authoritative organization workflow without checking out or
executing contributor code. Ordinary issue comments are excluded. Existing
manual build and native credential lanes are unchanged.

## Verdict

**PASS for document cleanup; BLOCKED for public visibility.** Current documents
are sanitized. Publication requires sanitized history, coordinated consumer
revision updates, and resolution of retained GitHub commit and PR pages.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Inherited contribution instructions linked to a missing local agreement, preventing contributors from reaching the promised agreement. | README.md at cc69a934f887966f955440bff8c356e3da449bee; inherited organization CONTRIBUTING.md |
| FND-002 | high | Publishing existing history exposes local workstation paths prohibited by the content-rights policy. Cleaning the current tree alone leaves historical documents exposed. | CONTENT_RIGHTS.md; historical review and FR-010 documents |

## Dispositions

- FND-001: fixed 4e196af7ee7f8e26d0658f06d72619262221bd27. Added a canonical agreement link, accurate per-repository signing instructions, reusable agreement enforcement, and the verified community invite.
- FND-002: current documents fixed b10a81fca113949e108a2774825309de6674e210. Sanitized history prepared and verified locally; remote publication remains blocked pending the authorized revision transition and GitHub retention resolution.

## Evidence

- AGPL license text, Cargo license declaration, and source SPDX headers agree.
- Gitleaks 8.30.1 scanned all 18 fetched commits: no leaks detected. Separately scanned GitHub issue and comment metadata: no leaks detected. This is scanner evidence, not a guarantee of absence.
- Sanitized candidate history: 19 commits, zero owner workstation-path matches. Replacement for cc69a934f887966f955440bff8c356e3da449bee is 8e6781eb39072bbdb43c8e7d19ea3ed216959897. Runtime source, tests, Cargo manifests/lock, Makefile, and deny configuration have identical Git blob identities.
- The canonical CLA is publicly accessible in agent-ix/.github. Discord API confirmed invite k8DVhuYBR2 resolves to Agent IX with no expiration.
- First complete `make ci`: passed default and secrets Clippy, default tests, injected secrets acceptance tests, license checks, formatting, and unsafe audit. Native OS tests were not run; no real credentials were accessed or changed.
