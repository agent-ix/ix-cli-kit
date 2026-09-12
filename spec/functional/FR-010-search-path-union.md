---
id: FR-010
title: "Union search roots in first-seen order, checking existence only for environment-supplied roots"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-005"
    type: "implements"
---
# FR-010: Union search roots in first-seen order, checking existence only for environment-supplied roots

## Description

The crate SHALL accumulate search roots from every contributing layer into one ordered,
deduplicated set, SHALL check existence only for roots supplied by an environment
variable, and SHALL report each legacy environment variable that actually contributed a
root.

**This requirement is 1:1 with `quire-cli`'s `scoped_registry_roots`, at
`/home/peter/dev/quire-cli/src/commands/validate.rs:485-516`, and with nothing else.**
Parity is claimed against that function alone. The other two implementations of the
same on-disk contract were measured and do not agree with it; see *Measured divergence*
below. Where they differ, this crate reproduces `quire-cli`.

## Inputs

- Explicitly supplied roots, typically a `--scope` directory.
- Roots named by environment variables, split with the platform's `PATH` separator.
- A default install root supplied by the consumer.

## Outputs

- `SearchPath` with `push`, `push_if_dir`, `push_env`, `push_envs`, `roots`,
  `into_roots`, `deprecations`.
- `EnvVar::current(name)` and `EnvVar::legacy(name, superseded_by)`.

## The ported layers

`scoped_registry_roots` contributes FIVE layers, in this order:

| # | Layer | Source | Existence checked? |
|---|-------|--------|--------------------|
| 1 | the `--scope` directory | `validate.rs:488` | no |
| 2 | `<scope>/.ix/modules` | `validate.rs:489-492` | yes |
| 3 | `IX_FILAMENT_MODULES_PATH` | `validate.rs:494-501` | yes |
| 4 | `IX_SCHEMA_PATH` (legacy alias) | `validate.rs:494-501` | yes |
| 5 | the default module root | `validate.rs:503-505` | **no** |

## Behavior

- The crate SHALL keep every root any layer contributes, in first-seen order.
- If a root repeats, then the crate SHALL drop the repeat rather than reorder the set.
- The crate SHALL union the roots of every environment variable it is given; a later
  variable SHALL NOT suppress an earlier one.
- The crate SHALL keep an explicitly supplied root whether or not it exists.
- The crate SHALL skip an environment-supplied path that is not an existing directory.
- The crate SHALL keep a default install root whether or not it exists, because the
  default root is where a tool MATERIALISES modules into and must be searched before it
  exists.
- The crate SHALL split an environment variable's value with the platform's `PATH`
  separator.
- The crate SHALL record a legacy environment variable in its deprecation report only
  when that variable actually contributed a root.

## The push / push_if_dir asymmetry, and why

An environment-supplied path that does not exist is probably a user mistake — a stale
entry in a `PATH`-style variable — so it is skipped. The default install root is
different in kind: it is the directory the producer creates when it installs modules.
On a fresh machine it does not exist **yet**. Checking its existence would silently drop
it from the search set on exactly the machines where a first install is about to happen.
`validate.rs:503-505` therefore pushes it unconditionally, and the test at
`validate.rs:786-799` asserts it is present. A consumer reproducing layer 5 therefore
calls `push`, never `push_if_dir`.

This asymmetry was undocumented in the ported code and was mis-stated in this crate's
own module documentation, which claimed existence was checked for "env- and
default-supplied roots". Corrected; filed as `agent-ix/quire-cli#88` and
`agent-ix/ix-cli-kit#1`.

## Measured divergence across the ecosystem

Measured by executing each implementation, not by reading string literals. Under
`HOME=/home/peter` with no `IX_*` variables set, all three resolve the same absolute
modules directory, `/home/peter/.ix/filament/modules`. They diverge as follows:

| Condition | `quire-cli` (ported here) | `quire-rs` | `quoin` |
|-----------|---------------------------|------------|---------|
| `IX_HOME` set | ignored | ignored | **honoured** |
| `IX_FILAMENT_MODULES_PATH` + `IX_SCHEMA_PATH` | both unioned, default root kept | `filament.or(schema)` — second var dropped, **default root dropped** | both ignored |
| `QUOIN_MODULE_PATHS` | ignored | ignored | honoured, prepended |
| default root absent | kept in the set | reported as a `Missing` path diagnostic | **contributes nothing** |
| registry file | none | none | `~/.ix/filament/registry.json` |
| separator | `std::env::split_paths` | colon only | colon only |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-010-CON-1 | Parity is claimed against `quire-cli`'s `scoped_registry_roots` ONLY. No parity is claimed with `quire-rs` or `quoin`, which were measured and differ. | Scope | Inspection |
| FR-010-CON-2 | The crate SHALL NOT supply any default root itself; layer 5's value is named by the consumer. | Architecture | Inspection |
| FR-010-CON-3 | `SearchPath::deprecations` is ADDITIVE — the ported function has no such reporting surface. It reports and never changes the resolved set. | Scope | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | Roots contributed by successive layers appear in first-seen order and a repeated root is dropped, not reordered. | Test |
| FR-010-AC-2 | A root pushed explicitly is retained even though it does not exist on disk, while the same path offered as an environment-supplied root is skipped. | Test |
| FR-010-AC-3 | A default install root that does not exist on disk is retained in the search set, reproducing the fresh-install behaviour of `validate.rs:503-505`. | Test |
| FR-010-AC-4 | An unset environment variable contributes no root and records no deprecation. | Test |
| FR-010-AC-5 | A legacy environment variable names the variable that supersedes it, and a current variable reports itself as not legacy. | Test |
| FR-010-AC-6 | Adding a deprecation report does not change the resolved root set. | Test |

## Dependencies

- **Upstream**: [US-005](../usecase/US-005-resolve-search-roots-predictably.md)
- **Downstream**: `quoin`'s Rust port, which materialises the default module set into
  layer 5's root and would otherwise have hit the fresh-install case.
