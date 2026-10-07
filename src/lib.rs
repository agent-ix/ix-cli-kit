// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared foundation for the Agent-IX Rust command-line tools.
//!
//! v0.1 owns **streams, statuses and provenance**. It does **not** own argv.
//!
//! | module | what it owns |
//! |---|---|
//! | [`exit`] | the 0/1/2/3/4 `Outcome` taxonomy and caller-owned exit-code pass-through |
//! | [`streams`] | results on stdout, diagnostics on stderr, colour resolution |
//! | [`json`] | canonical (recursively key-sorted) JSON, and one encoder |
//! | [`version`] | build-time source provenance, and the agreement assertion |
//! | [`config`] | the precedence ORDER — flag > env > file > default, and unioned search paths |
//! | [`secrets`] | opt-in OS credential storage and secret-source precedence |
//!
//! [`config`] owns the order and never the locations: the ecosystem's default
//! module root is `~/.ix/filament/modules`, and a shared crate that decided
//! paths on a consumer's behalf would relocate installs that two tools already
//! agree on. See that module's header.
//!
//! # What this crate deliberately does not do
//!
//! **It has no `clap` dependency and exports no `Parser`-derived root command.**
//! Two facts make that load-bearing rather than tidy:
//!
//! - `engineering-assurance` compiles clap with `default-features = false` and
//!   **no derive feature** (`Cargo.toml:56`). A foundation crate exporting a
//!   derived root command could not be adopted there at all.
//! - `ix-cli`'s eventual Rust port is hook-and-plugin dispatch, not a fixed
//!   subcommand enum. A crate that owns the root command cannot host it.
//!
//! The consequence is concrete: the survey's single hardest pin conflict —
//! `engineering-assurance` at `clap =4.5.47` with derive off against
//! `quire-corpus` at `clap =4.6.0` with derive on — is not this crate's
//! problem, because this crate never sees clap. Selector types such as
//! [`streams::DiagnosticsFormat`] implement [`std::str::FromStr`], which is
//! what `clap::value_parser!`, a hand-rolled argv loop and a config file all
//! consume alike.
//!
//! # Dependency versions
//!
//! The three shared dependencies use **caret requirements** with floors
//! compatible with first-party consumer pins: `serde 1.0.228`,
//! `serde_json 1.0.151`, and `thiserror 2.0.20`. The kit has no API requirement
//! for `serde 1.0.229`. Two exact `=` pins on one crate cannot coexist in a
//! graph, so a foundation crate that pinned exactly would force every consumer
//! to change its own pins in the same commit as adoption. Leaf binaries keep
//! their exact pins; this crate states a floor. Its own gates are reproducible
//! from its committed `Cargo.lock`.

#![forbid(unsafe_code)]

pub mod config;
pub mod exit;
pub mod json;
pub mod streams;
pub mod version;

#[cfg(feature = "secrets")]
pub mod secrets;

pub use exit::Outcome;
