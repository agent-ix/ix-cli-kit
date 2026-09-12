// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A test fixture, not a product.
//!
//! The Unknown-source path can only be proven against a **compiled artifact**:
//! a unit test calls `run_for_source` with a source it chose itself and so
//! never exercises [`std::env::current_exe`], which is the step that decides
//! what a real install is. This binary lives under `target/<profile>/` —
//! neither a `node_modules` tree nor `~/.cargo` — so the detector resolves to
//! `Unknown`, and the integration test can assert that the engine then installs
//! nothing and exits 0 without touching the network.
//!
//! It deliberately parses argv by hand. This crate has no `clap` dependency and
//! must keep none — see the crate header.

use ix_cli_kit::Outcome;
use ix_cli_kit::self_update::{self, SelfUpdateConfig, SelfUpdateOpts};
use ix_cli_kit::streams;

/// Deliberately not a real package: the fixture must never resolve to
/// something that exists, so a defect that reached the npm path would fail
/// loudly rather than quietly install.
const CONFIG: SelfUpdateConfig = SelfUpdateConfig {
    npm_package: "@example/fixture-cli",
    cargo_git: "https://example.invalid/fixture-cli",
    releases_url: "https://example.invalid/fixture-cli/releases",
    update_command: "fixture update",
};

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|arg| arg == "--check");

    match self_update::run_self_update(
        &CONFIG,
        &SelfUpdateOpts {
            check,
            registry: None,
        },
    ) {
        Ok(report) => {
            for line in &report.messages {
                streams::emit_result(line);
            }
            Outcome::Ok.into()
        }
        Err(error) => {
            streams::Record::error("update", &error.to_string())
                .emit(streams::Diagnostics::resolved(
                    streams::DiagnosticsFormat::Human,
                    streams::ColorChoice::Never,
                ));
            Outcome::Internal.into()
        }
    }
}
