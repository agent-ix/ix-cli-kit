---
id: SR-2720
title: "PLAT-1154 code review of the serde floor and kit consumption wording"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-cli-kit@91f7faf5b3fd1bb3d4d1808123e70f08524dc155; Cargo.toml, Cargo.lock, README.md, src/lib.rs, spec/functional/FR-016-shared-adoption.md; context: deny.toml, Makefile, .github/workflows/ci.yml, src/streams.rs, src/json.rs, src/config.rs, spec/spec.md, spec/stakeholder/StR-002-adoptable-without-imposing-argv-or-locations.md"
review_set: subset
---

## Summary

Ticket: PLAT-1154. ix-cli-kit PR #12 against main 8f925d44. This is a code review with the
rust-review lane folded in. It was static only: the reviewer did not run cargo, by brief.

What was measured:

- **Manifest.** Only the serde requirement changes, from `1.0.229` to `1.0.228`. It stays a
  caret requirement with `features = ["derive"]`. `serde_json 1.0.151`, `thiserror 2.0.20`,
  `[features]`, target sections, `rust-version = "1.98.1"` and `deny.toml` are unchanged.
- **Lock.** Only three package entries change: `serde`, `serde_core` and `serde_derive`, each
  from 1.0.229 to 1.0.228. `serde_derive`'s `syn` dependency also moves from `syn 3.0.5` to
  `syn 2.0.119`. That move is required: the cached `serde_derive-1.0.229/Cargo.toml` declares
  `syn = "3"` and `serde_derive-1.0.228` declares `syn = "2.0.81"`. Both syn entries were
  already in the lock, and `syn 3.0.5` keeps four other dependents, so no package is added or
  dropped.
- **Checksums.** The three new checksums equal the sha256 of the cached `.crate` files and the
  entries in quire-spec-language's lock.
- **serde_json.** `serde_json 1.0.151` requires `serde_core >= 1.0.220`, so 1.0.228
  satisfies it.
- **serde API use.** `derive(Serialize)` and `derive(Deserialize)`, `rename_all`,
  `skip_serializing_if`, `flatten` and `de::DeserializeOwned` all long predate 1.0.228. No API
  needs 1.0.229.
- **Consumer pins.** Exact serde pins in the local checkouts are `=1.0.228` (24 sites) and
  `=1.0.229` (3 sites: quoin and the engineering-assurance generated campaign). Neither
  conflicts with `^1.0.228`.
- **Consistency.** The Cargo.toml comment, the README "Dependency versions" section and the
  src/lib.rs header now agree on the floor.

Gates: the author reports macOS `make ci` passed on this head. That claim was not
re-measured. The reviewer ran no cargo. The repository's CI workflow is manual-only
(`workflow_dispatch`), so the only check on the PR is the contributor agreement. The Linux
gate is queued.

## Verdict

**Not merge-ready as is: FND-001 must be fixed, and FND-002 and FND-003 are cheap to fix in
the same round.** The dependency change is correct and minimal. The serde floor is a caret
minimum, not a pin. The lock matches the manifest, MSRV and deny are unaffected, and no SHA,
checksum, local path or compatibility layer is added. The consumption rewrite is incomplete,
though: text outside the five edited hunks still tells consumers to pin by git revision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Cargo.toml still says "Consumers git-rev pin this crate (see README, "Consuming this crate")". The README section it cites now says to declare `branch = "main"`, so the manifest contradicts the instruction it points to | Cargo.toml:10 |
| FND-002 | low | README still describes adoption as pinned: the roadmap row says "Consumer adoption remains a separate pinned-revision change", and the secrets example says "enable it on the same pin". Both contradict the new branch-declaration section | README.md:72, README.md:94 |
| FND-003 | low | README says `--locked` keeps builds on the resolved commit "instead of following later changes to `main`". Cargo uses the locked commit with or without `--locked`. `--locked` only refuses to rewrite the lock, and the commit moves only on `cargo update` or a missing lock entry | README.md:90-92 |

### FND-001 detail

Failure scenario: a consumer reads the manifest comment first, as the comment invites. It
says to git-rev pin and points at the README. The README says to declare `branch = "main"`.
One of the two must be wrong, and the code-side comment is the one the PR missed. Fix:
rewrite the comment to "Consumers declare this crate by git branch (see README, ...)".

### FND-002 detail

README.md:72 sits in the capability table for the `secrets` row, and README.md:94 introduces
the second toml example. Neither line says `rev =`, but both describe the old model. A reader
of line 72 would expect the adoption PR to add a revision pin, which FR-016-AC-1 and
FR-016-AC-2 now reject on inspection.

### FND-003 detail

The sentence implies that a build without `--locked` follows `main`. It does not. Cargo
re-resolves a git branch only when the lock lacks the entry or on `cargo update`. A consumer
could wrongly conclude that `--locked` is what keeps the pinned commit, and treat the
committed lock as optional when `--locked` is set. Suggested wording: "Cargo builds from the
commit recorded in `Cargo.lock`; `cargo update -p ix-cli-kit` moves it, and `--locked` fails
rather than rewriting the lock."

## Dispositions

Round 1 was reviewed at fix head 550c14f0405a9327b8556dca36b2e5055fd922f7 (tree a228fe25). Each
outcome below was checked against the text at that head. Cargo.lock and src/ are unchanged
since the reviewed head.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 550c14f0405a9327b8556dca36b2e5055fd922f7: Cargo.toml:10-13 now reads "Consumers declare the main branch and retain the resolved commit in their Cargo.lock (see README, "Consuming this crate")." |
| FND-002 | fixed | 550c14f0405a9327b8556dca36b2e5055fd922f7: README.md:72 now reads "a separate branch-dependency change, with the resolved commit recorded in each consumer's lockfile"; README.md:95 now reads "enable it on the same branch dependency" |
| FND-003 | fixed | 550c14f0405a9327b8556dca36b2e5055fd922f7: README.md:90-93 now says Cargo uses the resolved commit "with or without `--locked`", and that `--locked` refuses to rewrite the lockfile |
