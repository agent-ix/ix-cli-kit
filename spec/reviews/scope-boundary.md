---
id: SR-002
title: "scope-boundary review of the ix-cli-kit founding spec set"
type: SpecReview
analysis: scope-boundary
scope: "spec/spec.md §2.2, FR-010, FR-012, FR-013"
review_set: subset
---
## Summary

Checked whether the spec set keeps to the crate's stated boundary — the crate owns the
precedence ALGORITHM and each consumer owns its own locations and schemas — and whether
anything specified exceeds what the shipped v0.1 code does. The boundary holds: no
module outside `config::xdg` touches an XDG variable, no code path constructs a `.ix` or
`filament` segment, and the manifest declares no argv parsing dependency. Two deviations
between the shipped crate and its source are recorded rather than specified away.

Out of scope and deliberately unspecified here: a `self_update` module exists on the
branch `feat/self-update-module`. v0.1 excludes it by owner directive, so specifying it
belongs to the next spec, not this one.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-001 | medium | `SearchPath::deprecations` is an ADDITIVE reporting surface: `quire-cli`'s `scoped_registry_roots` has no equivalent. It is within the spirit of the port — `IX_SCHEMA_PATH` is an alias nothing ever declared deprecated — but it is not 1:1, and until this spec set nothing said so. Recorded in FR-010-CON-3 and constrained to report only, never to change the resolved set. | FR-010 | missing-requirement |
| FND-002 | medium | `DiagnosticFields` omits `change_target`, which `quire-cli` populates at `validate.rs:656`, `:696` and `:716`. Adopting `streams` unchanged would drop it from every `--diagnostics-format=json` line on the validate paths. Deliberately NOT added here; recorded as a stated incompleteness in FR-004-CON-1 and filed as `agent-ix/ix-cli-kit#1`. | FR-004 | missing-requirement |
| FND-003 | high | The crate's `config` module documentation claimed existence was checked for "env- and default-supplied roots". The ported code checks only the env-supplied ones and pushes the default root unconditionally (`validate.rs:503-505`). An adopter following the doc would have used `push_if_dir` and silently dropped the default module root on every fresh machine — the exact case `quoin`'s Rust port hits, because `quoin` materialises the default module set into that root. Corrected in this change, specified in FR-010 with its reason, and guarded by a new regression test. | FR-010 | wrong-requirement |
