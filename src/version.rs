// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Build-time provenance, and the assertion that the surfaces reporting it
//! agree.
//!
//! Two halves, and the second is the one nobody in this ecosystem had in Rust:
//!
//! - [`build`] is called from a consumer's `build.rs` and bakes the source
//!   revision and working-tree state into the binary. Moved from
//!   `quire-cli/build.rs`.
//! - [`Agreement`] is a **reusable test helper** that asserts every surface
//!   reporting a version reports the *same* version, and that a clean tag
//!   reports itself. It is a port of `quoin/scripts/check-version-agreement.mjs`
//!   — which, until now, was the ecosystem's only implementation of this check
//!   and was written in Node, so no Rust CLI could run it.
//!
//! # Why the agreement check exists
//!
//! Reproducing the reasoning from that script's header, because the blast
//! radius is the whole point and a paraphrase loses it:
//!
//! > The defect this exists to catch shipped once already, one repo over:
//! > agent-ix/quire-cli#52, where the 0.24.0–0.28.0 tags all shipped binaries
//! > reporting 0.23.0. Version provenance is load-bearing here — every
//! > SpecReview records the tool version it measured with, and three reviews in
//! > agent-ix/filament-ide-rs cite numbers from a binary whose self-reported
//! > version was wrong.
//! >
//! > Runs against the BUILT binary, not the source: the disagreement was
//! > between a build-time baked value and package.json, so only a built
//! > artifact can show it.
//!
//! That last sentence is a constraint on how [`Agreement`] is used. A unit test
//! comparing two constants inside one crate cannot see this defect; the check
//! belongs in `tests/`, against the compiled binary, which is why the helper
//! takes a path and runs it.
//!
//! # The rule the build half enforces
//!
//! An unresolvable value is reported as `unknown`, never as a plausible
//! substitute. From `quire-cli/build.rs`: "an unresolvable engine is a fact a
//! consumer must be able to see; substituting something plausible is the
//! confident-but-wrong claim this whole surface prevents."

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The string a provenance field carries when it could not be resolved.
///
/// Never a fallback to something plausible: a consumer must be able to see that
/// the build did not know.
pub const UNKNOWN: &str = "unknown";

/// Whether the working tree the binary was built from had uncommitted changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceState {
    /// `git status --porcelain` was empty.
    Clean,
    /// The tree carried changes the revision does not describe.
    Dirty,
    /// Not a git checkout, or git could not be run.
    Unknown,
}

impl SourceState {
    /// The stable spelling baked into the binary.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Dirty => "dirty",
            Self::Unknown => UNKNOWN,
        }
    }

    /// Recover the state from a baked string.
    #[must_use]
    pub fn from_str_or_unknown(text: &str) -> Self {
        match text {
            "clean" => Self::Clean,
            "dirty" => Self::Dirty,
            _ => Self::Unknown,
        }
    }
}

/// What a build knew about the source it was built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIdentity {
    /// The full 40-character revision, or [`UNKNOWN`].
    pub revision: String,
    /// Whether the tree was clean.
    pub state: SourceState,
}

impl SourceIdentity {
    /// The first eight characters of the revision — what a `--version` line
    /// shows.
    #[must_use]
    pub fn short(&self) -> &str {
        short_revision(&self.revision)
    }
}

/// The first eight characters of a revision, or the whole thing if shorter.
#[must_use]
pub fn short_revision(revision: &str) -> &str {
    revision.get(..8).unwrap_or(revision)
}

/// Read the source identity of a git checkout.
///
/// Returns [`UNKNOWN`] / [`SourceState::Unknown`] rather than guessing when git
/// is unavailable, the directory is not a checkout, or `HEAD` is not a
/// 40-character hex revision.
#[must_use]
pub fn source_identity(repo: &Path) -> SourceIdentity {
    let unknown = SourceIdentity {
        revision: UNKNOWN.to_owned(),
        state: SourceState::Unknown,
    };
    let Some(head) = git(repo, &["rev-parse", "HEAD"]) else {
        return unknown;
    };
    let revision = head.trim().to_owned();
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return unknown;
    }
    let state = match git(repo, &["status", "--porcelain", "--untracked-files=normal"]) {
        Some(status) if status.is_empty() => SourceState::Clean,
        Some(_) => SourceState::Dirty,
        None => SourceState::Unknown,
    };
    SourceIdentity { revision, state }
}

/// `git describe --tags --dirty`, or `None` when no tag is reachable.
#[must_use]
pub fn git_describe(repo: &Path) -> Option<String> {
    git(repo, &["describe", "--tags", "--dirty"]).map(|out| out.trim().to_owned())
}

/// Whether a `git describe` output names a clean tag and nothing else.
///
/// A build **ahead** of its tag describes as `v1.2.3-4-gabc1234`, and a dirty
/// tree appends `-dirty`. Both are the point of baking `git describe` and
/// neither is a failure — only an exact `v1.2.3` asserts "this build is that
/// release".
#[must_use]
pub fn is_clean_tag(describe: &str) -> bool {
    let core = describe.strip_prefix('v').unwrap_or(describe);
    let mut parts = core.split('.');
    let ok = |part: Option<&str>| {
        part.is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    };
    ok(parts.next()) && ok(parts.next()) && ok(parts.next()) && parts.next().is_none()
}

fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The first `MAJOR.MINOR.PATCH` token in a piece of text, with any
/// pre-release or build suffix attached.
///
/// The suffix travels **verbatim**. Rounding `0.45.0-3-g99e97f0` to its nearest
/// tag would be the same confident-but-wrong claim the provenance surface
/// exists to prevent (`quire-cli/src/lockfile.rs`).
#[must_use]
pub fn semver_in(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    for (start, byte) in bytes.iter().enumerate() {
        if !byte.is_ascii_digit() {
            continue;
        }
        // A digit that continues an earlier number is not a candidate start.
        // A leading `v` is, so the boundary test names digits and dots only.
        let previous = start.checked_sub(1).and_then(|at| bytes.get(at)).copied();
        if previous.is_some_and(|b| b.is_ascii_digit() || b == b'.') {
            continue;
        }
        if let Some(end) = semver_end(bytes, start) {
            return text.get(start..end);
        }
    }
    None
}

fn semver_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut at = start;
    for segment in 0..3 {
        let digits_from = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        if at == digits_from {
            return None;
        }
        if segment < 2 {
            if bytes.get(at) != Some(&b'.') {
                return None;
            }
            at += 1;
        }
    }
    // Pre-release / build / describe suffix, verbatim.
    while bytes
        .get(at)
        .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
    {
        at += 1;
    }
    Some(at)
}

/// One thing that claims to report the tool's version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    /// How the surface is named in a failure message — `--version`, `--help`,
    /// `CARGO_PKG_VERSION`.
    pub name: String,
    /// Everything the surface emitted, kept so a failure can quote it.
    pub raw: String,
    /// The version extracted from `raw`, if there was one.
    pub version: Option<String>,
}

/// Why an agreement check could not pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Disagreement {
    /// Fewer than two surfaces reported a version.
    ///
    /// Deliberately an error and not a pass. A check that silently succeeds
    /// when it observed nothing is the decorative gate this helper replaces.
    #[error(
        "version agreement is unverifiable: {reported} of {total} surface(s) reported a version — {detail}"
    )]
    Unverifiable {
        /// How many surfaces yielded a version.
        reported: usize,
        /// How many surfaces were offered.
        total: usize,
        /// Each surface and what it emitted.
        detail: String,
    },
    /// Two surfaces reported different versions.
    #[error("surfaces disagree about the version: {detail}")]
    Mismatch {
        /// Surface name to reported version, for every surface that reported.
        versions: BTreeMap<String, String>,
        /// The rendered comparison.
        detail: String,
    },
    /// The checkout is on a clean tag and the binary reports something else.
    #[error("a clean tag must report itself: binary reports {reported}, tag is {describe}")]
    TagMismatch {
        /// What every surface agreed on.
        reported: String,
        /// `git describe --tags --dirty`.
        describe: String,
    },
}

/// What the check observed when it passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The version every reporting surface agreed on.
    pub version: String,
    /// Every surface offered, including those that reported nothing.
    pub surfaces: Vec<Surface>,
    /// `git describe`, when the checkout had a reachable tag.
    pub describe: Option<String>,
    /// Why the clean-tag half did not run, when it did not.
    pub tag_check_skipped: Option<String>,
}

/// Assert that every surface reporting a version reports the same one.
///
/// Build it in a `tests/` integration test against the **compiled binary** —
/// only a built artifact can show a disagreement between a baked value and a
/// manifest.
///
/// ```no_run
/// # use std::path::Path;
/// # use ix_cli_kit::version::Agreement;
/// // In your own integration test this is `env!("CARGO_BIN_EXE_my_tool")`;
/// // that macro only resolves inside the crate that declares the binary.
/// let my_tool = Path::new("/path/to/the/built/binary");
/// Agreement::new()
///     .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
///     .command("--version", my_tool, &["--version"])
///     .command("--help", my_tool, &["--help"])
///     .against_git_tag(Path::new(env!("CARGO_MANIFEST_DIR")))
///     .assert_agreed();
/// ```
#[derive(Debug, Default, Clone)]
pub struct Agreement {
    surfaces: Vec<Surface>,
    repo: Option<PathBuf>,
}

impl Agreement {
    /// An empty check.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a surface from a string already in hand.
    #[must_use]
    pub fn surface(mut self, name: &str, raw: &str) -> Self {
        self.surfaces.push(Surface {
            name: name.to_owned(),
            raw: raw.to_owned(),
            version: semver_in(raw).map(str::to_owned),
        });
        self
    }

    /// Run a command and record what it printed as a surface.
    ///
    /// stdout and stderr are both searched: a CLI that prints its version
    /// banner on stderr is still reporting a version, and a check that looked
    /// at one stream would pass by not looking. A command that fails to run, or
    /// exits non-zero, is recorded as a surface that reported nothing — which
    /// surfaces as [`Disagreement::Unverifiable`] rather than as a pass.
    #[must_use]
    pub fn command(mut self, name: &str, program: &Path, args: &[&str]) -> Self {
        let raw = match Command::new(program).args(args).output() {
            Ok(output) if output.status.success() => format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            Ok(output) => format!("<exited {}>", output.status),
            Err(error) => format!("<could not be run: {error}>"),
        };
        self.surfaces.push(Surface {
            name: name.to_owned(),
            version: semver_in(&raw).map(str::to_owned),
            raw,
        });
        self
    }

    /// Also require that, on a clean tag, the reported version is that tag.
    #[must_use]
    pub fn against_git_tag(mut self, repo: &Path) -> Self {
        self.repo = Some(repo.to_path_buf());
        self
    }

    /// Run the check.
    ///
    /// # Errors
    ///
    /// [`Disagreement`] when fewer than two surfaces reported, when two
    /// surfaces disagree, or when a clean tag is not what the binary reports.
    pub fn check(&self) -> Result<Report, Disagreement> {
        let reporting: BTreeMap<String, String> = self
            .surfaces
            .iter()
            .filter_map(|s| s.version.clone().map(|v| (s.name.clone(), v)))
            .collect();
        let detail = self
            .surfaces
            .iter()
            .map(|s| format!("{} => {}", s.name, s.version.as_deref().unwrap_or("(none)")))
            .collect::<Vec<_>>()
            .join("; ");

        if reporting.len() < 2 {
            return Err(Disagreement::Unverifiable {
                reported: reporting.len(),
                total: self.surfaces.len(),
                detail,
            });
        }
        let mut distinct: Vec<&String> = reporting.values().collect();
        distinct.sort_unstable();
        distinct.dedup();
        let [version] = distinct.as_slice() else {
            return Err(Disagreement::Mismatch {
                versions: reporting,
                detail,
            });
        };
        let version = (*version).clone();

        let (describe, tag_check_skipped) = match self.repo.as_deref().map(git_describe) {
            None => (None, Some("no repository supplied".to_owned())),
            Some(None) => (
                None,
                Some("no git tag reachable — drift check not applicable".to_owned()),
            ),
            Some(Some(describe)) => {
                if is_clean_tag(&describe) {
                    let tag = describe.strip_prefix('v').unwrap_or(&describe);
                    if tag != version {
                        return Err(Disagreement::TagMismatch {
                            reported: version,
                            describe,
                        });
                    }
                    (Some(describe), None)
                } else {
                    let skipped =
                        format!("build is ahead of its tag ({describe}) — drift expected");
                    (Some(describe), Some(skipped))
                }
            }
        };

        Ok(Report {
            version,
            surfaces: self.surfaces.clone(),
            describe,
            tag_check_skipped,
        })
    }

    /// [`Agreement::check`], panicking with the full comparison on failure.
    ///
    /// # Panics
    ///
    /// When the check does not pass. In a test, a panic IS the failure report.
    #[must_use]
    #[allow(
        clippy::panic,
        reason = "this IS the test helper; a failed agreement must fail the test"
    )]
    pub fn assert_agreed(&self) -> Report {
        match self.check() {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        }
    }
}

/// Helpers for a consumer's `build.rs`.
///
/// Add this crate to `[build-dependencies]` as well as `[dependencies]`; it
/// pulls in nothing but `std` for this half.
pub mod build {
    use std::path::Path;

    use super::{SourceIdentity, UNKNOWN, short_revision, source_identity};

    /// Bake `<PREFIX>_SOURCE_REVISION`, `_SOURCE_SHORT` and `_SOURCE_STATE`
    /// into the crate, and register the rerun triggers that keep them true.
    ///
    /// Call from `build.rs`:
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    /// ix_cli_kit::version::build::emit_source_provenance("MY_TOOL", &dir);
    /// ```
    ///
    /// and read them back with `env!("MY_TOOL_SOURCE_SHORT")`.
    ///
    /// The rerun triggers are not a nicety. Without them cargo caches the build
    /// script across commits and a release binary ships the revision of
    /// whatever tree happened to be checked out when the script last ran.
    #[must_use]
    pub fn emit_source_provenance(prefix: &str, repo: &Path) -> SourceIdentity {
        println!("cargo:rerun-if-changed=build.rs");
        register_rerun_triggers(repo);
        let identity = source_identity(repo);
        println!(
            "cargo:rustc-env={prefix}_SOURCE_REVISION={}",
            identity.revision
        );
        println!(
            "cargo:rustc-env={prefix}_SOURCE_SHORT={}",
            short_revision(&identity.revision)
        );
        println!(
            "cargo:rustc-env={prefix}_SOURCE_STATE={}",
            identity.state.as_str()
        );
        identity
    }

    /// Bake one already-resolved value, substituting [`UNKNOWN`] for `None`.
    ///
    /// Never a fallback to something plausible — an unresolvable value is a
    /// fact a consumer must be able to see.
    pub fn emit_value(key: &str, value: Option<&str>) {
        println!("cargo:rustc-env={key}={}", value.unwrap_or(UNKNOWN));
    }

    fn register_rerun_triggers(repo: &Path) {
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(repo)
                .args(args)
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        };
        if let Some(files) = git(&["ls-files"]) {
            for file in files.lines() {
                println!("cargo:rerun-if-changed={}", repo.join(file).display());
            }
        }
        if let Some(git_dir) = git(&["rev-parse", "--git-dir"]) {
            let git_dir = git_dir.trim();
            let git_dir = if Path::new(git_dir).is_absolute() {
                std::path::PathBuf::from(git_dir)
            } else {
                repo.join(git_dir)
            };
            println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
            println!("cargo:rerun-if-changed={}", git_dir.join("index").display());
        }
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

    #[test]
    fn a_bare_version_and_a_v_prefixed_one_are_both_found() {
        assert_eq!(semver_in("0.2.0"), Some("0.2.0"));
        assert_eq!(semver_in("v1.20.3"), Some("1.20.3"));
        assert_eq!(semver_in("quire-corpus 0.2.0"), Some("0.2.0"));
    }

    // quire-cli's VERSION_LINE names two versions on one line, CLI first. The
    // helper must take the tool's, not the engine's.
    #[test]
    fn the_first_version_on_a_two_version_line_wins() {
        assert_eq!(
            semver_in("0.28.0 (cli 99e97f01, engine 0.45.0@99e97f01)"),
            Some("0.28.0")
        );
    }

    #[test]
    fn a_describe_suffix_travels_verbatim() {
        assert_eq!(semver_in("v0.45.0-3-g99e97f0"), Some("0.45.0-3-g99e97f0"));
    }

    #[test]
    fn text_without_a_triple_reports_nothing_rather_than_a_pair() {
        assert_eq!(semver_in("Usage: quire-corpus --root <ROOT>"), None);
        assert_eq!(semver_in("version 1.2"), None);
    }

    #[test]
    fn clean_tags_and_drifted_ones_are_distinguished() {
        assert!(is_clean_tag("v1.2.3"));
        assert!(is_clean_tag("1.2.3"));
        assert!(!is_clean_tag("v1.2.3-4-gabc1234"));
        assert!(!is_clean_tag("v1.2.3-dirty"));
        assert!(!is_clean_tag("v1.2"));
    }

    #[test]
    fn agreeing_surfaces_pass() {
        let report = Agreement::new()
            .surface("CARGO_PKG_VERSION", "0.2.0")
            .surface("--version", "quire-corpus 0.2.0")
            .check()
            .unwrap();
        assert_eq!(report.version, "0.2.0");
        assert_eq!(
            report.tag_check_skipped.as_deref(),
            Some("no repository supplied")
        );
    }

    // agent-ix/quire-cli#52: the 0.24.0–0.28.0 tags all shipped binaries
    // reporting 0.23.0. This is that defect, in a fixture.
    #[test]
    fn a_baked_value_lagging_its_manifest_is_a_mismatch() {
        let error = Agreement::new()
            .surface("CARGO_PKG_VERSION", "0.28.0")
            .surface("--version", "quire 0.23.0")
            .check()
            .unwrap_err();
        let Disagreement::Mismatch { versions, .. } = &error else {
            panic!("expected a mismatch, got {error}");
        };
        assert_eq!(versions["CARGO_PKG_VERSION"], "0.28.0");
        assert_eq!(versions["--version"], "0.23.0");
    }

    // A check that observed nothing must not report success. One surface is
    // one opinion, and an agreement of one is not an agreement.
    #[test]
    fn a_single_reporting_surface_is_unverifiable_not_a_pass() {
        let error = Agreement::new()
            .surface("--version", "0.2.0")
            .surface("--help", "Usage: tool [OPTIONS]")
            .check()
            .unwrap_err();
        let Disagreement::Unverifiable {
            reported, total, ..
        } = error
        else {
            panic!("expected unverifiable");
        };
        assert_eq!((reported, total), (1, 2));
    }

    #[test]
    fn a_command_that_cannot_be_run_reports_nothing_rather_than_passing() {
        let error = Agreement::new()
            .surface("CARGO_PKG_VERSION", "0.1.0")
            .command(
                "--version",
                Path::new("/nonexistent/ix-cli-kit-no-such-binary"),
                &["--version"],
            )
            .check()
            .unwrap_err();
        assert!(matches!(error, Disagreement::Unverifiable { .. }));
    }

    #[test]
    fn source_state_round_trips_through_its_baked_spelling() {
        for state in [SourceState::Clean, SourceState::Dirty, SourceState::Unknown] {
            assert_eq!(SourceState::from_str_or_unknown(state.as_str()), state);
        }
        assert_eq!(
            SourceState::from_str_or_unknown("something else"),
            SourceState::Unknown
        );
    }

    #[test]
    fn a_directory_that_is_not_a_checkout_is_unknown_not_guessed() {
        let identity = source_identity(Path::new("/"));
        if identity.state == SourceState::Unknown {
            assert_eq!(identity.revision, UNKNOWN);
            assert_eq!(identity.short(), UNKNOWN);
        } else {
            // `/` is inside a checkout on some machines; the assertion that
            // matters is that an unknown revision is spelled, not invented.
            assert_eq!(identity.revision.len(), 40);
        }
    }
}
