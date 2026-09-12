// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The Unknown install source, observed from outside the process.
//!
//! Provenance: `quire-cli/tests/cli_update.rs` (IT-083, IT-084). The property
//! is the module's load-bearing safety rule — **never guess and clobber a
//! binary we did not place** — and it is the one property a unit test cannot
//! reach, because a unit test supplies the install source instead of deriving
//! it from the running executable.
//!
//! The fixture binary lives under `target/<profile>/`: neither a `node_modules`
//! tree nor `~/.cargo`, so detection resolves to `Unknown`. Both runs below are
//! therefore network-free, and a regression that made them touch the network
//! would show up as a failure here rather than as a surprise on a user's
//! machine.

#![cfg(feature = "self-update")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]

use std::process::Command;

const FIXTURE: &str = env!("CARGO_BIN_EXE_self_update_fixture");

/// `tc_030`: `--check` on an Unknown install source prints the npm recipe, the
/// cargo recipe and the releases URL, exits 0, and installs nothing.
#[test]
fn tc_030_check_on_an_unknown_source_prints_manual_instructions_and_exits_zero() {
    let output = Command::new(FIXTURE).arg("--check").output().unwrap();
    assert!(output.status.success(), "--check should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("could not determine how this binary was installed"),
        "expected Unknown-source guidance, got:\n{stdout}"
    );
    assert!(
        stdout.contains("npm install -g @example/fixture-cli@latest"),
        "expected the npm upgrade recipe, got:\n{stdout}"
    );
    assert!(
        stdout.contains("cargo install --git"),
        "expected the cargo upgrade recipe, got:\n{stdout}"
    );
    assert!(
        stdout.contains("releases"),
        "expected the releases URL, got:\n{stdout}"
    );
}

/// `tc_031`: even *without* `--check`, an Unknown source performs no install —
/// it only emits instructions — so a bare update stays network-free and exits
/// 0. This is the half of the rule a `--check`-only test would miss.
#[test]
fn tc_031_a_bare_update_on_an_unknown_source_is_also_safe() {
    let output = Command::new(FIXTURE).output().unwrap();
    assert!(
        output.status.success(),
        "a bare update should exit 0 on an Unknown source"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("could not determine how this binary was installed"),
        "{stdout}"
    );
}

/// `tc_032`: the engine names the *consumer's* identifiers and never its own
/// provenance. The fixture declares `@example/fixture-cli`; if `quire` appeared
/// in this output, an identifier would have survived the extraction inside the
/// crate.
#[test]
fn tc_032_reports_name_the_consumers_identifiers_only() {
    let output = Command::new(FIXTURE).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !stdout.contains("quire"),
        "a package-specific identifier leaked from the engine:\n{stdout}"
    );
}
