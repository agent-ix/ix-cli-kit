// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The exit taxonomy every Agent-IX Rust CLI reports.
//!
//! Adopted **verbatim** from `quoin-core`'s `protocol::Outcome`
//! (`agent-ix/quoin`, `rust/crates/quoin-core/src/protocol.rs`). This module is
//! a MOVE, not a second opinion: when quoin-core adopts this crate its own file
//! is deleted, so the two must never diverge.
//!
//! # Why this is shared
//!
//! The 2026-09-12 survey of the five Rust CLIs in this ecosystem — `build-chain`,
//! `quire-cli`, `engineering-assurance`, `quire-corpus`, `quoin-core` — found
//! that **all five had an exit taxonomy and all five disagreed**. Exit status 2
//! means `ARGV_ERROR` in quire-cli (`src/io.rs`), a host error in
//! engineering-assurance (`src/main.rs`), and [`Outcome::Refused`] here. Three
//! of the five had independently invented a "succeeded but found problems"
//! status, each with a different number. A caller scripting two of these
//! binaries cannot write one `case $?` that is correct for both.
//!
//! # The load-bearing member
//!
//! [`Outcome::Partial`] is a non-zero status whose stdout is nevertheless a
//! complete, valid payload. `runQuireAllowFailure` in quoin's
//! `src/quire/exec.ts` exists because `quire properties` exits 1 while writing a
//! full result for every document that did resolve, and treating that as total
//! failure silently discarded the whole property axis over two untyped files
//! (agent-ix/quoin#103). Ask [`Outcome::carries_payload`], never `status == 0`.

/// The exit status reserved for "the named command does not exist".
///
/// **Reserved, not implemented.** `command_not_found` is specified for the
/// not-yet-existing `quoin-cli` (agent-ix/quoin, FR-102) and has zero Rust
/// implementations today, so v0.1 of this crate does not ship the behaviour.
/// What it does ship is the number: no consumer may give status 5 a second
/// meaning in the meantime, which is exactly how the five taxonomies above
/// diverged. [`Outcome::from_code`] returns `None` for 5 — an unclaimed status
/// is an honest `None`, not a guess.
pub const RESERVED_COMMAND_NOT_FOUND: u8 = 5;

/// An attempted caller-owned exit code that conflicts with the reserved status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("caller exit code {rejected_code} is reserved (reserved code {reserved_code})")]
pub struct CallerExitCodeError {
    /// The caller-supplied value rejected by [`caller_exit_code`].
    pub rejected_code: u8,
    /// The reserved value that caused the rejection.
    pub reserved_code: u8,
}

/// Convert a caller-owned status into a process exit code without interpreting it.
///
/// Every value except [`RESERVED_COMMAND_NOT_FOUND`] passes through unchanged.
/// The reserved value returns [`CallerExitCodeError`] so the caller can choose a
/// status according to its own policy.
///
/// # Errors
///
/// Returns [`CallerExitCodeError`] when `code` is
/// [`RESERVED_COMMAND_NOT_FOUND`].
pub fn caller_exit_code(code: u8) -> Result<std::process::ExitCode, CallerExitCodeError> {
    if code == RESERVED_COMMAND_NOT_FOUND {
        return Err(CallerExitCodeError {
            rejected_code: code,
            reserved_code: RESERVED_COMMAND_NOT_FOUND,
        });
    }

    Ok(std::process::ExitCode::from(code))
}

/// How the process terminated, and therefore whether stdout is worth reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome {
    /// Complete payload, no diagnostics that matter. Exit 0.
    Ok,
    /// Complete, valid payload AND diagnostics. Exit 1. The caller decides
    /// what a qualified result is worth; this status only stops the decision
    /// being made by an exception.
    Partial,
    /// Understood and refused by a stated rule. No payload. Exit 2.
    Refused,
    /// Not a well-formed request for a known operation. No payload. Exit 3.
    Invalid,
    /// The tool itself failed. No payload. Exit 4.
    Internal,
}

impl Outcome {
    /// Every status, in taxonomy order. Lets a consumer enumerate the surface
    /// instead of discovering it by trying numbers.
    pub const ALL: &'static [Self] = &[
        Self::Ok,
        Self::Partial,
        Self::Refused,
        Self::Invalid,
        Self::Internal,
    ];

    /// The process exit status.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Partial => 1,
            Self::Refused => 2,
            Self::Invalid => 3,
            Self::Internal => 4,
        }
    }

    /// Whether stdout holds a complete payload.
    ///
    /// A method on the taxonomy rather than a comparison at each call site,
    /// because "non-zero but valid" is exactly the judgement call sites get
    /// wrong.
    #[must_use]
    pub const fn carries_payload(self) -> bool {
        matches!(self, Self::Ok | Self::Partial)
    }

    /// Recover the outcome from an observed exit status.
    ///
    /// `None` for anything outside the taxonomy — including
    /// [`RESERVED_COMMAND_NOT_FOUND`], which is reserved and not yet meaningful.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Ok),
            1 => Some(Self::Partial),
            2 => Some(Self::Refused),
            3 => Some(Self::Invalid),
            4 => Some(Self::Internal),
            _ => None,
        }
    }

    /// The stable spelling, for a diagnostic or a `--json` envelope.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Partial => "partial",
            Self::Refused => "refused",
            Self::Invalid => "invalid",
            Self::Internal => "internal",
        }
    }
}

impl From<Outcome> for std::process::ExitCode {
    fn from(outcome: Outcome) -> Self {
        Self::from(outcome.code())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]
mod tests {
    use super::*;

    /// Trace: FR-001-AC-1
    /// Provenance: agent-ix/quoin#103
    #[test]
    fn exit_statuses_are_the_documented_taxonomy() {
        assert_eq!(
            [
                Outcome::Ok.code(),
                Outcome::Partial.code(),
                Outcome::Refused.code(),
                Outcome::Invalid.code(),
                Outcome::Internal.code(),
            ],
            [0, 1, 2, 3, 4]
        );
    }

    /// Trace: FR-001-AC-3
    #[test]
    fn every_status_round_trips() {
        for outcome in Outcome::ALL {
            assert_eq!(Outcome::from_code(outcome.code()), Some(*outcome));
        }
        assert_eq!(Outcome::from_code(5), None);
    }

    /// Trace: FR-001-AC-2
    /// Provenance: agent-ix/quoin#103
    #[test]
    fn non_zero_but_valid_is_distinguishable_from_failed() {
        assert!(Outcome::Partial.carries_payload());
        assert_ne!(Outcome::Partial.code(), 0);
        for dead in [Outcome::Refused, Outcome::Invalid, Outcome::Internal] {
            assert!(!dead.carries_payload());
        }
    }

    /// Trace: FR-002-AC-1
    /// Trace: FR-002-AC-2
    /// Trace: FR-002-AC-3
    // The reservation is only worth something if nothing quietly claims it.
    #[test]
    fn the_reserved_command_not_found_status_is_claimed_by_nobody() {
        assert_eq!(RESERVED_COMMAND_NOT_FOUND, 5);
        assert_eq!(Outcome::from_code(RESERVED_COMMAND_NOT_FOUND), None);
        assert!(
            Outcome::ALL
                .iter()
                .all(|o| o.code() != RESERVED_COMMAND_NOT_FOUND)
        );
    }

    /// Trace: FR-001-AC-4
    #[test]
    fn spellings_are_distinct_and_stable() {
        let spellings: Vec<&str> = Outcome::ALL.iter().map(|o| o.as_str()).collect();
        assert_eq!(
            spellings,
            ["ok", "partial", "refused", "invalid", "internal"]
        );
    }
}
