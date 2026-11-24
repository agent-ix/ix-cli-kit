---
id: SR-2732
title: "PLAT-1730 crate-doc secrets link code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-cli-kit; src/lib.rs; context: Cargo.toml, src/self_update.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1730. Reviewed agent-ix/ix-cli-kit#16 at head 6ca473bc92b3f7b8219a5f2e54eefa99dcb9da16, diff `origin/main...HEAD` only, with the Rust review lane folded in. The change turns the crate-level module-table cell `[`secrets`]` into the code span `` `secrets` `` in `src/lib.rs:16`. It is a doc comment change only: no item, cfg, feature or dependency changed. `secrets` is declared under `#[cfg(feature = "secrets")]` (`src/lib.rs:63`) and `default = []` (`Cargo.toml:37`), so the old link did not resolve in a default build. The other gated module, `self_update` (`#[cfg(feature = "self-update")]`, `src/lib.rs:58`), was already a plain span. The only other intra-doc link in a gated module, `[`run_self_update`]` at `src/self_update.rs:110`, is inside that module and is documented only when the module is compiled. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked -j 2` passed on the head.

## Verdict

**PASS.** Doc-only and correct; the default-feature rustdoc build succeeds with warnings denied. One low layout nit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The two feature-gated rows mark opt-in differently: `self_update` carries "(opt-in)" in the module column, `secrets` only says "opt-in" in the description column | src/lib.rs:16 |
