// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Version agreement, asserted against a compiled artifact.
//!
//! Provenance: the ecosystem's only implementation of this check was
//! `quoin/scripts/check-version-agreement.mjs`, in Node. Its reasoning holds
//! here: a constant read by the same test that bakes it always agrees with
//! itself, so the check is worth nothing unless it runs the built binary.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]

use std::path::Path;

use ix_cli_kit::version::{Agreement, Disagreement};

const FIXTURE: &str = env!("CARGO_BIN_EXE_version_fixture");

/// `tc_001`: every surface of the built binary reports the same version, and a
/// clean tag reports itself.
#[test]
fn tc_001_surfaces_of_the_built_binary_agree() {
    let report = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command("--version", Path::new(FIXTURE), &["--version"])
        .command("--help", Path::new(FIXTURE), &["--help"])
        .against_git_tag(Path::new(env!("CARGO_MANIFEST_DIR")))
        .assert_agreed();

    assert_eq!(report.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(report.surfaces.len(), 3);
}

/// `tc_002`: a surface that cannot be run reports nothing, and reporting nothing
/// is unverifiable rather than a pass. A gate that succeeds when it observed
/// nothing is the decorative gate this helper replaces.
#[test]
fn tc_002_a_binary_that_cannot_run_is_unverifiable_not_a_pass() {
    let error = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command(
            "--version",
            Path::new("/nonexistent/ix-cli-kit-fixture"),
            &["--version"],
        )
        .check()
        .unwrap_err();

    let Disagreement::Unverifiable {
        reported, total, ..
    } = error
    else {
        panic!("expected unverifiable, got {error}");
    };
    assert_eq!((reported, total), (1, 2));
}
