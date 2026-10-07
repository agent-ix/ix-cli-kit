// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public API contract for caller-owned process exit codes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]

use std::process::ExitCode;

use ix_cli_kit::exit::{CallerExitCodeError, RESERVED_COMMAND_NOT_FOUND, caller_exit_code};

/// Trace: FR-017-AC-1, FR-017-AC-2
#[test]
fn tc_050_caller_exit_codes_pass_through_except_the_reserved_status() {
    for code in 0..=u8::MAX {
        if code == RESERVED_COMMAND_NOT_FOUND {
            continue;
        }

        assert_eq!(
            caller_exit_code(code),
            Ok(ExitCode::from(code)),
            "caller code {code} must pass through unchanged"
        );
    }

    assert_eq!(
        caller_exit_code(RESERVED_COMMAND_NOT_FOUND),
        Err(CallerExitCodeError {
            rejected_code: RESERVED_COMMAND_NOT_FOUND,
            reserved_code: RESERVED_COMMAND_NOT_FOUND,
        })
    );
}
