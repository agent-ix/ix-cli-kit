---
type: master-requirements
name: ix-cli-kit
org: agent-ix
component_type: rust-lib
tags:
  - cli
  - foundation
implementation_language: rust
depends_on: []
relationships: []
standards_alignment:
  - iso-iec-ieee-29148
  - ieee-828
---
# Master Requirements Specification
## ix-cli-kit

---

## 1. Purpose

This document defines the scope, intent, and governing requirements framework for
`ix-cli-kit`, the shared Rust command-line foundation for the Agent-IX ecosystem.

The crate owns the process **exit taxonomy and caller-owned exit-code pass-through**,
the **stream discipline** that decides which stream a line goes to, **canonical JSON**
encoding, **build-time version provenance**, the **precedence order** by which a
setting's value is chosen, and a shared **OS credential-store contract** for local
secrets.

Requirements FR-001 through FR-013 describe shipped behaviour ported into this
crate. Requirements FR-014 through FR-016 and NFR-004 specify the SWM-12
credential extension. FR-014, FR-015 and NFR-004 are implemented behind the
off-by-default `secrets` feature; FR-016 describes separate downstream consumer
adoption work. FR-017 is a new, unimplemented caller-owned exit-code pass-through
extension that leaves the five-member `Outcome` taxonomy unchanged.

---

## 2. Scope

### 2.1 In Scope

- The five exit statuses `0`–`4` and the reservation of status `5`.
- Opaque caller-owned `u8` exit-code pass-through, excluding reserved status `5`;
  the kit does not define or map the caller's code meaning.
- The result/diagnostic stream split, diagnostic rendering, and colour resolution.
- Recursive-key-sorted canonical JSON and a single encoder.
- Build-time source provenance and the version-agreement assertion.
- The configuration **precedence order** — `flag > env > file > default` — and the
  **union** semantics of an ordered search path.
- The crate-level boundary rules: no argv parsing, no `clap` dependency, and no path
  chosen on a consumer's behalf.
- App-scoped OS credential-store operations, secret-source precedence and reporting,
  non-disclosure, and pinned-revision adoption by `ix-projects` and one Rust CLI.

### 2.2 Out of Scope

This specification does **not** govern:

- **Argv parsing.** v0.1 exports no `Parser`-derived root command type and the crate
  has no `clap` dependency at all. Argument parsing belongs to the consuming binary.
- **Configuration locations and schemas.** The crate decides the ORDER; every
  consumer owns its own locations and its own schema. Each application and module
  knows its own shape, and the crate must never encode any application's shape.
- **Application login screens and command syntax.** Consumers choose their own
  interaction and argument parsing. The shared API supplies store operations and
  source metadata, not an application command.
- **File fallback for credentials.** A locked or unavailable OS credential store is
  an explicit error, with no plaintext or encrypted local-file substitute.
- **The `command_not_found` behaviour** at exit status 5. The status is reserved and
  deliberately unimplemented here; the behaviour is specified for `quoin-cli` in
  `ix://agent-ix/quoin/FR-102`.
- **The meaning of caller-owned exit codes.** The caller supplies that meaning and
  chooses the result and diagnostic streams; the kit passes through the numeric value
  unchanged and does not interpret it as an `Outcome`.
- **Terminal control.** Nothing opens an alternate screen, sets raw mode, asks for
  terminal size, or runs an event loop. That is TUI machinery owned by the consumer.
- **The module-store on-disk layout.** `~/.ix/filament/modules` and
  `~/.ix/filament/registry.json` are owned by `quoin` (producer) and read by
  `quire-rs`. This crate names no path.
- **Publication.** The crate is `publish = false`; consumers pin it by git revision.

---

## 3. System Overview

### 3.1 System Description

`ix-cli-kit` is a `no-argv` Rust library crate. Its five shipped modules were each
moved from an existing implementation in this ecosystem. FR-017 is a new,
unimplemented extension for caller-owned exit-code pass-through:

| module | moved from |
|---|---|
| `exit` | `quoin-core`'s `protocol::Outcome` |
| `streams` | `quire-cli/src/io.rs` |
| `json` | `quire-corpus`'s `print_json`/`sort_json`, `quoin-core`'s `canonical_json`, `quire-cli`'s `encode_json` |
| `version` | `quire-cli/build.rs` and `quoin/scripts/check-version-agreement.mjs` |
| `config` | `quire-cli/src/commands/validate.rs`'s `scoped_registry_roots` |

SWM-12 adds an off-by-default `secrets` module beside `config`, in this same crate.
This boundary follows the existing division: `config` already owns order and
source reporting, while consumers own settings paths and schemas. A separate
crate would require a second dependency and a second source-reporting contract
for this small local-settings extension. The new module owns secret-specific
source selection and uses the OS store; it does not turn ordinary settings files
into a credential backend. Consumers pin this one crate by git revision.

The 2026-09-12 survey that scoped the crate found five Rust CLIs — `build-chain`,
`quire-cli`, `engineering-assurance`, `quire-corpus`, `quoin-core` — each with its own
exit taxonomy, and all five disagreeing.

### 3.2 Intended Users

- Authors of Agent-IX Rust command-line binaries.
- Authors of Agent-IX terminal user interfaces, `ix-cli` foremost.
- Authors of Rust applications with local credentials, including `ix-projects`.
- Operators and scripts that consume those binaries' exit statuses and streams.

---

## 4. Requirements Architecture

```
spec/
├── spec.md            # This document
├── stakeholder/       # StR-XXX
├── usecase/           # US-XXX
├── functional/        # FR-XXX
└── non-functional/    # NFR-XXX
```

---

## 5. Requirement Classes

### 5.1 Stakeholder Requirements

Authoritative needs. Format `StR-XXX`, location `stakeholder/`, normative for intent.

### 5.2 User Requirements

Usage intent. Format `US-XXX`, location `usecase/`, informational and non-binding.

### 5.3 Functional Requirements

Observable, testable behaviour. Format `FR-XXX`, location `functional/`, normative.

### 5.4 Non-Functional Requirements

Quality constraints. Format `NFR-XXX`, location `non-functional/`, normative.

### 5.5 Acceptance Criteria

Format `{FR-XXX}-AC-N`, inside each functional requirement file, and the verification
anchor that the repository's tests bind to with `Trace:` tags.

### 5.6 Requirements Index

The artifacts are normative; this table is an index.

| ID | Module | Title |
|----|--------|-------|
| [FR-001](./functional/FR-001-exit-taxonomy.md) | `exit` | Report process outcome as one of five defined exit statuses |
| [FR-002](./functional/FR-002-reserved-exit-status.md) | `exit` | Reserve exit status 5 for `command_not_found` without implementing it |
| [FR-003](./functional/FR-003-stream-discipline.md) | `streams` | Send results to stdout and diagnostics to stderr, never colourising a result |
| [FR-004](./functional/FR-004-diagnostic-rendering.md) | `streams` | Render a diagnostic in a human or JSON shape, separately from emitting it |
| [FR-005](./functional/FR-005-colour-resolution.md) | `streams` | Decide colour from an explicit choice, terminal state and `NO_COLOR` |
| [FR-006](./functional/FR-006-canonical-json.md) | `json` | Encode JSON with keys sorted at every depth and no insignificant whitespace |
| [FR-007](./functional/FR-007-source-provenance.md) | `version` | Bake the source revision and working-tree state into the build |
| [FR-008](./functional/FR-008-version-agreement.md) | `version` | Assert that every version surface of a built binary agrees |
| [FR-009](./functional/FR-009-scalar-precedence.md) | `config` | Resolve a scalar setting as flag, then environment, then file, then default |
| [FR-010](./functional/FR-010-search-path-union.md) | `config` | Union search roots in first-seen order, checking existence only for environment-supplied roots |
| [FR-011](./functional/FR-011-configuration-file-loading.md) | `config` | Load a configuration file as absent, present, or malformed at a named position |
| [FR-012](./functional/FR-012-xdg-available-never-imposed.md) | `config` | Offer XDG base directories without imposing them |
| [FR-013](./functional/FR-013-crate-boundary.md) | crate | Ship no argv parser and no command-line framework dependency |
| [FR-014](./functional/FR-014-os-credential-store.md) | `secrets` | Store app-scoped credentials in the OS credential store |
| [FR-015](./functional/FR-015-secret-source-precedence.md) | `secrets` | Resolve secret overrides and report their source without revealing values |
| [FR-016](./functional/FR-016-shared-adoption.md) | crate | Adopt the shared credential contract from an app and a Rust CLI |
| [FR-017](./functional/FR-017-pass-through-caller-exit-code.md) | `exit` | Pass through caller-owned exit codes without interpreting them |
| [NFR-001](./non-functional/NFR-001-canonical-encoding-is-feature-independent.md) | `json` | Canonical encoding holds under dependency feature unification |
| [NFR-002](./non-functional/NFR-002-provenance-and-licence-headers.md) | crate | Every source file declares its licence and the crate stays unpublished |
| [NFR-003](./non-functional/NFR-003-dependency-floor.md) | crate | The dependency surface stays minimal and is declared as caret ranges |
| [NFR-004](./non-functional/NFR-004-secret-nondisclosure.md) | `secrets` | Secret values stay out of formatting and settings serialization |

---

## 6. Requirement Identification

| Artifact | Format | Example |
|---|---|---|
| Stakeholder Requirement | `StR-XXX` | `StR-001` |
| User Story | `US-XXX` | `US-003` |
| Functional Requirement | `FR-XXX` | `FR-010` |
| Non-Functional Requirement | `NFR-XXX` | `NFR-002` |
| Acceptance Criteria | `{FR}-AC-N` | `FR-010-AC-4` |
| Test Case | `TC-XXX` | `TC-001` |

Identifiers are immutable once assigned. Test-case identifiers correspond to the
`tc_NNN`-prefixed test function names already present under `tests/`.

---

## 7. Requirement Quality Policy

Functional requirements SHALL define observable behaviour, be atomic, and be testable
through explicit criteria. They SHALL NOT encode a consuming application's policy.
FR-001 through FR-013 remain a 1:1 port. FR-017 is new and unimplemented: it preserves
the supplied caller-owned number without interpreting consumer policy. The SWM-12
requirements also describe new, unimplemented extensions and must not be read as
shipped behaviour.

---

## 8. Error and Failure Model

### 8.1 Error Classification

- **Refused** (exit 2) — understood and refused by a stated rule.
- **Invalid** (exit 3) — not a well-formed request for a known operation.
- **Internal** (exit 4) — the tool itself failed.
- **Partial** (exit 1) — a complete payload accompanied by diagnostics.

### 8.2 Failure Handling Guarantees

A non-zero status does not imply an absent payload. Callers SHALL ask
`Outcome::carries_payload()` rather than comparing the status to zero.

For a binary using FR-017's caller-owned pass-through, `Outcome::from_code()` and
`Outcome::carries_payload()` describe only the kit's `Outcome` taxonomy; they do not
describe the meaning or payload status of the binary's caller-owned exit codes.

An unresolvable provenance value is reported as `unknown`, never as a plausible
substitute.

---

## 9. Traceability

Bidirectional traceability is maintained between StR → US → FR → AC → test, with
tests binding to acceptance criteria via `Trace:` tags in the test source.

---

## 10. Verification Strategy

Requirements are verified by automated tests in this repository, by inspection of the
source where the requirement is a boundary rule (a dependency that must be absent
cannot be demonstrated by running it), and by analysis where the requirement concerns
behaviour of a dependency's feature unification. FR-016 additionally requires
focused integration evidence in `ix-projects` and one Rust CLI. Those adoption
changes follow implementation of the shared contract; this spec change makes no
consumer code changes.

---

## 11. Change Management

Requirements artifacts are configuration-controlled. A behaviour change to a ported
module is a change to both this specification and the implementation it was ported
from; the two must not diverge silently. The shared API implementation must
satisfy FR-014, FR-015 and NFR-004 before either consumer adopts it; FR-016 is
verified by the separate consumer changes.

---

## 12. References

- ISO/IEC/IEEE 29148 — Requirements Engineering
- IEEE 828 — Configuration Management
- `ix://agent-ix/quoin/FR-102` — `command_not_found`, which reserves exit status 5
