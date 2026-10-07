# ix-cli-kit

## Name and scope

- **It is a shared library of CLI plumbing**: exit-code taxonomy, version provenance, stream discipline, canonical JSON, config precedence, and opt-in OS credential storage.
- **It is NOT a CLI, NOT a framework, and NOT a port of `ix-cli`.** It exports no root command type, no argv parsing, no TUI, and no command dispatch.
- **`ix-cli` remains TypeScript** and is out of scope for this crate and for the Rust burn-down program.

The name is deliberate and should not drift back. An earlier working name, `ix-cli-rs`, reads as "ix-cli, ported to Rust" — the opposite of what this crate is, and it produced exactly that misreading: `ix-cli` is a substantial TUI product with Kubernetes controls and animations, it is explicitly deferred, and nothing here touches it. `ix-cli-kit` is the ecosystem's own term rather than an invented one: `quire-cli/src/self_update/mod.rs:1-8` already says this code should move "behind a shared CLI kit crate."

## What is in v0.1

| module | what it owns |
|---|---|
| `exit` | the 0/1/2/3/4 exit taxonomy, and `carries_payload()` |
| `streams` | results on stdout, diagnostics on stderr, colour resolution |
| `json` | canonical (recursively key-sorted) JSON, and one encoder |
| `version` | build-time source provenance, and the agreement assertion |
| `config` | the precedence ORDER — flag > env > file > default, and unioned search paths |
| `secrets` | opt-in app-scoped OS credential storage and secret-source precedence |

### `exit` — one taxonomy, adopted verbatim

```
0 Ok        1 Partial   2 Refused   3 Invalid   4 Internal
```

Ported from `quoin-core/src/protocol.rs:28-72`, tests and all. Five of five surveyed Rust CLIs had an exit taxonomy and all five differed: exit `2` means `ARGV_ERROR` in `quire-cli` (`src/io.rs:360`), a host error in `engineering-assurance` (`src/main.rs:917`), and `Refused` in `quoin-core`. Three of the five independently invented a "succeeded but found problems" status, which is why `Partial` is the load-bearing member: a non-zero status that still carries a complete payload. Callers ask `carries_payload()`, never `status == 0`.

### `streams` — the measured defect this prevents

Results go to stdout; diagnostics go to stderr; results are never coloured. The rationale comment carried over from `quire-cli/src/io.rs:206-221` records the incident rather than the intention: a run that produced a 0-byte stdout file while 90,462 bytes went to stderr.

Rendering and emission are separable (`Record::render_human`, `Record::render_json`, `Record::write`) so a consumer that places its own output — a TUI — can use the same diagnostics without this crate choosing a sink. `ColorChoice::decide(is_terminal, no_color_set)` is pure, `ColorDecision` carries the facts behind the boolean, and `stdout_is_terminal()` / `stderr_is_terminal()` are exported. Nothing here opens an alternate screen, sets raw mode, asks terminal size or runs an event loop.

### `json` — canonical means recursively key-sorted, explicitly

`serde_json`'s `preserve_order` feature is unified across a Cargo graph. `quire-corpus` enables it and `quoin-core` deliberately does not, so a canonicaliser that relies on `BTreeMap` ordering is correct in one and silently wrong in the other. `json::canonical` sorts at every depth itself and is therefore correct under either feature set.

### `version` — provenance, and the assertion nobody had

Build-time `cargo:rustc-env` provenance ported from `quire-cli/build.rs:19-57`, plus the piece the ecosystem was missing in Rust: a reusable test helper asserting that `--version` and `--help` agree and that a clean tag reports itself. The only existing implementation was `quoin/scripts/check-version-agreement.mjs` — Node, checking the built binary rather than a source constant. `unknown` is never replaced with something plausible.

### `config` — the order, not the locations

**This crate decides the ORDER. The consumer decides the LOCATIONS.** Nothing in `config` writes or resolves a path the consumer did not name. XDG resolution is available as `config::xdg` for consumers that want it and is never the imposed default: the ecosystem's actual default module root is `~/.ix/filament/modules`, a dotdir that `quoin` materialises into and `quire-rs` reads by default, and a shared crate that imposed XDG would silently relocate a path two tools already agree on.

Two shapes, because there are two kinds of setting:

- `SearchPath` — **union**, ordered, deduplicated. For *where to look*.
- `resolve` — **override**: flag > env > file > default. For a single *value*.

Search roots union because that is the ported behaviour, and collapsing them into the override chain would drop one environment variable's roots entirely. Absent and malformed configuration are kept distinct, and a malformed file is reported with its path, line and column.

The `secrets` module is opt-in through the `secrets` Cargo feature. It stores
credentials in Keychain, Secret Service, or Credential Manager and resolves
explicit input > a named environment variable > the OS store. Secret values
have redacted debug output and are not serializable. Settings schemas and file
paths remain consumer-owned, and the module has no file fallback.

## Roadmap: what is excluded, and exactly what promotes it

v0.1 **extracts** code that existing CLIs already wrote independently, where correctness is provable by diffing against what exists. Everything below would be **designed** from scratch against zero or one consumer, and a shared crate designed against a single consumer is that consumer's code in a more expensive location. A capability qualifies for early promotion when it is a **MOVE rather than a DESIGN** — existing code whose correctness is provable by diffing against the original — even at a low consumer count. It also qualifies when every surveyed consumer *lacks* it **and that absence is a known gap rather than a known non-need**. Consumer count is the weakest of the three signals: it is a hint, not the test.

The census that produced these counts asked "does it have a config FILE" and got 0 of 5. It never asked "does it RESOLVE configuration", which is a different question with a different answer — `quire-cli` already implements the full precedence chain by hand at `src/commands/validate.rs:485-520`. That is the reusable lesson: **count the behaviour, not the artefact.**

| capability | Rust consumers today | what it would cost | named trigger |
|---|---|---|---|
| `self_update` | 1 (`quire-cli`) | channel detection, release-channel policy | **COMMITTED — the next work item after v0.1 lands.** Owner ruling: "self-update will have many [consumers]." It will be **feature-gated, default OFF** — but as a CAPABILITY gate, not a dependency gate. Measured: `quire-cli/src/self_update/` is `std`-only and adds **zero** dependencies (it never downloads, extracts, verifies a digest, replaces the running executable, or rolls back; it detects the install channel and shells out to `npm`/`cargo`). What the gate keeps out of a default build is code that can spawn external programs with inherited stdio. Detail: agent-ix/ix-cli-kit#2. |
| `config` | — | — | **PROMOTED INTO v0.1.** 1 of 5 already implements the precedence chain by hand, unioned, with an undocumented legacy alias (`IX_SCHEMA_PATH`). Promoted because it exists and is unowned. |
| plugin / command dispatch | 0 | a command-resolution model, a `command_not_found` disposition, a plugin manifest contract | quoin `spec/functional/FR-102-command-surface-and-oclif-retirement.md` (AC-3 `command_not_found`, AC-4 plugin command resolution) reaches a decided disposition **and** a second Rust CLI needs it. Exit code `5` is reserved for `command_not_found` and deliberately not implemented: `exit::from_code(5)` returns `None`. |
| secrets | `ix-projects` and one Rust CLI are planned adopters | OS backend, redaction, source reporting | **IMPLEMENTED by SWM-12.** The app and CLI flow is defined in FR-014 through FR-016 and NFR-004. The capability is off by default and has target-gated OS dependencies. Consumer adoption remains a separate branch-dependency change, with the resolved commit recorded in each consumer's lockfile. This does not specify a TUI login flow. |
| device-auth | 0 | a polling flow, token storage, refresh, revocation | A separate written device-auth flow covering polling, refresh, and revocation is required; SWM-12 covers local credential storage only. |
| marketplace | 0 | a registry protocol and a trust model | A registry protocol exists in specification and a Rust consumer needs to read it. |
| stable error-code envelope | 0 | a numbering authority, a stability promise per code | Two consumers need machine-stable error identity beyond the five-member exit taxonomy. Until then `streams::DiagnosticFields` carries `reason` as free text. |

## Consuming this crate

Branch declaration; lockfile revision. Consumers declare the `main`
branch, and Cargo records the resolved commit in each consumer's `Cargo.lock`.
Keep the lockfile committed and update it deliberately when moving to a newer
kit commit. No vendoring, no tag pins, no crates.io — `publish = false` is
declared explicitly in `Cargo.toml`.

```toml
[dependencies]
ix-cli-kit = { git = "https://github.com/agent-ix/ix-cli-kit", branch = "main" }
```

The manifest names the branch; `Cargo.lock` records the commit actually used.
Cargo uses that resolved commit when the lockfile entry exists, with or without
`--locked`. A `cargo update` or missing lockfile entry can move the resolution;
`--locked` refuses to rewrite the lockfile.

Consumers that need the optional credential API enable it on the same branch dependency:

```toml
ix-cli-kit = { git = "https://github.com/agent-ix/ix-cli-kit", branch = "main", features = ["secrets"] }
```

The application supplies its own scope and key and decides when to reveal a
value to an authenticated operation. The crate does not add a settings file or
choose a path.

### Dependency versions

Caret requirements are floored at versions compatible with first-party consumer pins (`serde 1.0.228`, `serde_json 1.0.151`, `thiserror 2.0.20`), not `=` pins. The kit does not require serde 1.0.229 for an API. Two exact pins on one crate cannot coexist in a graph, so an `=`-pinned foundation would make adoption a lockstep version migration for every consumer. Leaf binaries keep their exact pins; this crate states a compatible floor, and its own gates are reproducible from the committed `Cargo.lock`.

There is **no `clap` dependency** and no exported `Parser`-derived root command. `engineering-assurance` compiles clap with `default-features = false` and no derive feature, so a foundation exporting a derived command could not be adopted there at all. Selector types implement `FromStr`, which `clap::value_parser!`, a hand-rolled argv loop and a config file all consume alike. The consequence is that the survey's hardest pin conflict — `clap =4.5.47` derive-off against `=4.6.0` derive-on — never reaches this crate.

## Build

```bash
make ci     # fmt-check + lint + test + deny + audit-unsafe
```

## License

AGPL-3.0-or-later

Contributions follow the [canonical Agent IX CLA](CLA.md) and
[content rights policy](CONTENT_RIGHTS.md). See [Contributing](CONTRIBUTING.md)
for development and signing instructions. Community discussion is on
[Agent IX Discord](https://discord.gg/k8DVhuYBR2).
