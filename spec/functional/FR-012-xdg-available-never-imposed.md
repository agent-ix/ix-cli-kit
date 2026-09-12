---
id: FR-012
title: "Offer XDG base directories without imposing them"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-005"
    type: "implements"
---
# FR-012: Offer XDG base directories without imposing them

## Description

The crate SHALL resolve XDG base directories only when a consumer calls the [`xdg`]
module, applying no XDG location to a consumer that does not ask for one.

The ecosystem's real default root is the dotdir `~/.ix/filament`, which `quoin`
materialises into and `quire-rs` reads. A shared crate that resolved locations via XDG
on its consumers' behalf would relocate a path two tools already agree on and break
every existing install. Each application and module knows its own shape; this crate
knows none of them.

## Inputs

- The XDG environment variables, and `HOME`.

## Outputs

- `xdg::config_dir()`, `xdg::data_dir()`, `xdg::cache_dir()`.
- `xdg::honour_override(var, fallback)`, `xdg::base()`.

## Behavior

- The crate SHALL return the XDG directory named by the corresponding variable when that
  variable holds an absolute path.
- If the variable holds a relative path, then the crate SHALL ignore it and fall back, as
  the XDG specification requires.
- When the variable is unset, the crate SHALL fall back to the specified default beneath
  the home directory.
- The crate SHALL NOT create any directory it names.
- No other module of this crate SHALL call into `xdg`.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-012-CON-1 | Nothing in the crate SHALL derive a path for a consumer that did not name one. | Architecture | Inspection |
| FR-012-CON-2 | No code path SHALL construct a `.ix` or `filament` path segment; that shape belongs to the consumers, not the kit. Naming it in documentation as the reason for this rule is not encoding it. | Architecture | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | A relative value in an XDG override variable is ignored and the fallback is used. | Test |
| FR-012-AC-2 | No module other than `xdg` references an XDG variable, and no code path in the crate constructs a `.ix` or `filament` path segment. | Inspection |

## Dependencies

- **Upstream**: [StR-002](../stakeholder/StR-002-adoptable-without-imposing-argv-or-locations.md)
- **Downstream**: consumers that choose XDG.
