// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public API contract for caller-owned process exit codes.

use std::process::ExitCode;

use ix_cli_kit::exit::{CallerExitCodeError, caller_exit_code};

fn assert_error_and_display<T: std::error::Error + std::fmt::Display>() {}

const _: fn() = assert_error_and_display::<CallerExitCodeError>;

/// Trace: FR-017-AC-1, FR-017-AC-2
#[test]
fn tc_050_caller_exit_codes_pass_through_except_the_reserved_status() {
    for code in 0..=u8::MAX {
        if code == 5 {
            continue;
        }

        assert_eq!(
            caller_exit_code(code),
            Ok(ExitCode::from(code)),
            "caller code {code} must pass through unchanged"
        );
    }

    assert_eq!(
        caller_exit_code(5),
        Err(CallerExitCodeError {
            rejected_code: 5,
            reserved_code: 5,
        })
    );
}
