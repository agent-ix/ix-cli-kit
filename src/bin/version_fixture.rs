// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A test fixture, not a product.
//!
//! `version::Agreement` and the stream-discipline rules can only be proven
//! against a **compiled artifact**: a baked-in constant that has drifted from
//! its manifest is invisible to a unit test that reads the same constant. This
//! binary exists so the integration tests have an artifact to run.
//!
//! It deliberately parses argv by hand. This crate has no `clap` dependency and
//! must keep none — see the crate header.

use std::io::Write as _;

use ix_cli_kit::streams::{ColorChoice, DiagnosticFields, Diagnostics, DiagnosticsFormat, Record};
use ix_cli_kit::{Outcome, json, streams};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = Diagnostics::resolved(DiagnosticsFormat::Human, ColorChoice::Never);

    match args.first().map(String::as_str) {
        Some("--version") => {
            streams::emit_result(&format!("ix-cli-kit-fixture {VERSION}"));
            Outcome::Ok.into()
        }
        Some("--help") => {
            streams::emit_result(&format!(
                "ix-cli-kit-fixture {VERSION}\n\nUsage: ix-cli-kit-fixture <--version|--help|report>"
            ));
            Outcome::Ok.into()
        }
        Some("report") => report(out),
        other => {
            let message = other.map_or_else(
                || "no subcommand given".to_owned(),
                |arg| format!("unknown argument: {arg}"),
            );
            Record::error("argv", &message).emit(out);
            Outcome::Invalid.into()
        }
    }
}

/// Emit a complete payload on stdout and a diagnostic on stderr, then exit
/// `Partial`: the exact shape the taxonomy exists for — findings present, and
/// the run itself intact.
fn report(out: Diagnostics) -> std::process::ExitCode {
    let payload = serde_json::json!({ "zeta": 1, "alpha": { "b": 2, "a": 1 } });
    let Ok(encoded) = json::encode_canonical(&payload, false) else {
        Record::error("json", "the payload could not be encoded").emit(out);
        return Outcome::Internal.into();
    };
    streams::emit_result(&encoded);
    let _ = std::io::stdout().flush();
    Record::warning("finding", "one finding was reported")
        .with_fields(DiagnosticFields {
            subject: Some("alpha"),
            ..DiagnosticFields::default()
        })
        .emit(out);
    Outcome::Partial.into()
}
