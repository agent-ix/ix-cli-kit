// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Stream discipline and the exit taxonomy, observed from outside the process.
//!
//! Provenance: `quire-cli/src/io.rs:206-221` records the incident these tests
//! prevent recurring — a run that produced a 0-byte stdout file while 90,462
//! bytes went to stderr.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]

use std::process::Command;

use ix_cli_kit::Outcome;

const FIXTURE: &str = env!("CARGO_BIN_EXE_version_fixture");

/// `tc_010`: the payload lands on stdout, the diagnostic lands on stderr, and
/// neither appears on the other stream.
#[test]
fn tc_010_results_go_to_stdout_and_diagnostics_go_to_stderr() {
    let output = Command::new(FIXTURE).arg("report").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        !stdout.is_empty(),
        "stdout was empty; stderr was {stderr:?}"
    );
    assert!(stdout.starts_with('{'), "{stdout:?}");
    assert!(
        !stdout.contains("finding"),
        "a diagnostic reached stdout: {stdout:?}"
    );
    assert!(stderr.contains("one finding was reported"), "{stderr:?}");
    assert!(
        !stderr.contains("\"alpha\": {"),
        "the payload reached stderr: {stderr:?}"
    );
}

/// `tc_011`: a run with findings exits `Partial` (1) and still carries a complete
/// payload. This is the distinction three of five surveyed CLIs invented
/// independently and none of them could express in its exit code.
#[test]
fn tc_011_findings_present_exits_partial_and_still_carries_a_payload() {
    let output = Command::new(FIXTURE).arg("report").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(i32::from(Outcome::Partial.code()))
    );
    assert!(Outcome::Partial.carries_payload());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["alpha"]["a"], 1);
}

/// `tc_012`: results are emitted canonically — keys sorted at every depth — so a
/// consumer can diff two runs byte for byte.
#[test]
fn tc_012_the_emitted_payload_is_canonical_at_every_depth() {
    let output = Command::new(FIXTURE).arg("report").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim_end(), r#"{"alpha":{"a":1,"b":2},"zeta":1}"#);
}

/// `tc_013`: bad argv exits `Invalid` (3), not `Refused` (2). The two are
/// different questions and the survey found the ecosystem answering them with
/// the same number.
#[test]
fn tc_013_bad_argv_exits_invalid() {
    let output = Command::new(FIXTURE).arg("--nope").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(i32::from(Outcome::Invalid.code()))
    );
    assert!(output.stdout.is_empty(), "a diagnostic reached stdout");
}
