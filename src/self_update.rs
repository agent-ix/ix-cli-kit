// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Upgrading a CLI that was installed by somebody else's package manager.
//!
//! **This crate owns the ALGORITHM. The consumer owns the LOCATIONS and the
//! IDENTIFIERS.** Nothing here names a repository, a package, a registry, a
//! releases page or an install path; every one of those arrives through
//! [`SelfUpdateConfig`], which a consuming binary declares as a `const`. The
//! module is therefore adoptable by a second CLI without editing it.
//!
//! # This module is a port, not a design
//!
//! Moved from `quire-cli/src/self_update/mod.rs`, whose own header asked for
//! exactly this: *"when the IX CLIs are ported to Rust behind a shared CLI kit
//! crate, this whole file moves there verbatim."* The behaviour is unchanged —
//! the same three install channels, the same npm flags, the same messages, the
//! same tests. Two things did change and both are recorded below, because a
//! silent behaviour change inside a move is the failure mode this note exists
//! to prevent:
//!
//! - **`anyhow` became [`SelfUpdateError`].** This crate has no `anyhow`
//!   dependency and a library should not impose one on its consumers. Every
//!   message fragment is carried across verbatim, so the rendered text a user
//!   sees is the same; only the type is different.
//! - **The binary's own name became a parameter.** The original formatted
//!   `"run `quire update` to upgrade"` with `quire` written into the engine, in
//!   two places, which is the one genuinely package-specific string the
//!   original's own package-agnosticism audit did not catch. It is now
//!   [`SelfUpdateConfig::update_command`].
//!
//! # Why detection, and why no version comparison
//!
//! A binary can arrive through more than one install channel, so a generic
//! self-update must first work out *how it was installed* and then drive the
//! matching package manager. We never diff the running binary's version against
//! a registry: a CLI's npm wrapper and its Cargo crate version are
//! independently numbered, so a naive compare is meaningless. Idempotency is
//! handed to npm/cargo, which already no-op when up to date.
//!
//! # The feature gate, and what enabling it actually costs
//!
//! This module is behind the **`self-update`** Cargo feature, **default off**,
//! per owner ruling. Be precise about what the gate buys, because this crate's
//! roadmap predicted something else:
//!
//! - **It pulls in no dependency at all.** Not npm-as-a-crate, not `tar`, not
//!   an HTTP client. The module is `std`-only: [`std::process::Command`],
//!   [`std::path`] and [`std::env`]. The default build's dependency graph is
//!   byte-for-byte what it was.
//! - **What it does pull in is a capability: the ability to spawn arbitrary
//!   external programs and let them write to this process's inherited stdio.**
//!   That is the surface a consumer that only wants an exit taxonomy must be
//!   able to decline, and declining it is what the feature is for.
//!
//! No download, no archive extraction, no digest or signature verification, no
//! atomic replacement of the running executable and no rollback happen here,
//! because none of them happened in the original: the package manager does all
//! of it. A kit that grew those on the way across would not be a move.

use std::path::Path;
use std::process::Command;

/// How the running binary was placed on disk. Determines which package manager
/// (if any) can upgrade it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallSource {
    /// Installed via npm (prebuilt-binary wrapper); the executable lives under
    /// a `node_modules` tree.
    Npm,
    /// Installed via `cargo install`; the executable lives under `~/.cargo`.
    Cargo,
    /// Some other placement (prebuilt tarball dropped on `$PATH`, a copied
    /// binary, a dev build). We cannot safely guess an upgrade command.
    Unknown,
}

/// The only CLI-specific data the generic engine needs. A consuming binary
/// supplies these as constants.
///
/// This is the whole of the crate/consumer boundary for this module: every
/// identifier and every location lives here, and nothing else in the file
/// names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelfUpdateConfig {
    /// npm package name, e.g. `@agent-ix/quire-cli`.
    pub npm_package: &'static str,
    /// Git repository for `cargo install --git`, e.g.
    /// `https://github.com/agent-ix/quire-cli`.
    pub cargo_git: &'static str,
    /// Releases page surfaced in the manual/Unknown path.
    pub releases_url: &'static str,
    /// How a user invokes this CLI's own update command, e.g. `quire update`.
    ///
    /// Quoted back in the `--check` reports ("run `<this>` to upgrade"). It is
    /// a parameter rather than a literal because the engine cannot know what
    /// the consuming binary is called — the original wrote `quire` into two
    /// message strings, which is the only package-specific value that survived
    /// its own extraction audit.
    pub update_command: &'static str,
}

/// Runtime options parsed from the command line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelfUpdateOpts {
    /// Report availability without installing.
    pub check: bool,
    /// Override the npm registry (npm channel only). When `None`, npm resolves
    /// the package via the ambient config — i.e. however it was installed.
    pub registry: Option<String>,
}

/// What [`run_self_update`] decided/did. Reporting is the caller's job — this
/// keeps the engine free of any I/O-format policy, so a consumer renders the
/// outcome through [`crate::streams`], through a TUI, or not at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `--check` only: no changes made.
    Checked {
        /// The npm-reported version when the source is npm; `None` for the
        /// cargo channel, which tracks a git branch and has no single
        /// "latest" to report.
        latest: Option<String>,
    },
    /// An upgrade command ran to completion.
    Installed,
    /// The install source was unknown; the report carries manual instructions.
    Manual,
}

/// The outcome of a self-update attempt: the detected source, the action
/// taken, and human-readable summary lines for the caller to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfUpdateReport {
    /// How the running binary was judged to have been installed.
    pub source: InstallSource,
    /// What was decided or done.
    pub action: Action,
    /// Summary lines, in order, for the caller to emit.
    pub messages: Vec<String>,
}

/// A self-update failure. Every variant names the program it is about.
///
/// This replaces the original's `anyhow` chain — see the module header. Each
/// variant's `Display` carries the original's context string and its underlying
/// message in one line, so the text an operator reads is unchanged.
#[derive(Debug, thiserror::Error)]
pub enum SelfUpdateError {
    /// The running executable's own path could not be resolved, so the install
    /// source cannot be classified.
    #[error("resolving the running executable path: {source}")]
    CurrentExe {
        /// The underlying `std::io` failure.
        #[source]
        source: std::io::Error,
    },
    /// A package manager could not be started.
    #[error("{action}: spawning `{program}` (is it installed and on PATH?): {source}")]
    Spawn {
        /// What was being attempted, for the operator.
        action: &'static str,
        /// The program that could not be started.
        program: &'static str,
        /// The underlying `std::io` failure.
        #[source]
        source: std::io::Error,
    },
    /// A package manager ran and exited non-zero.
    #[error("{action}: `{program}` exited with {status}")]
    Failed {
        /// What was being attempted, for the operator.
        action: &'static str,
        /// The program that failed.
        program: &'static str,
        /// The rendered exit status.
        status: String,
    },
    /// A package manager whose output we were capturing exited non-zero. Its
    /// stderr is the message, because it is more useful than the status.
    #[error("{action}: `{program}` failed: {message}")]
    FailedWithStderr {
        /// What was being attempted, for the operator.
        action: &'static str,
        /// The program that failed.
        program: &'static str,
        /// The program's trimmed stderr.
        message: String,
    },
}

/// npm flags that force `registry` for a scoped/unscoped package.
///
/// A plain `--registry` is silently ignored for a scoped package when the
/// user's npmrc pins a `@scope:registry`; the scope-specific override is the
/// one npm honors. Returns an empty vec when no override is requested (ambient
/// config resolves the package).
fn registry_args(npm_package: &str, registry: Option<&str>) -> Vec<String> {
    let Some(registry) = registry else {
        return Vec::new();
    };
    if let Some((scope, _)) = npm_package
        .split_once('/')
        .filter(|_| npm_package.starts_with('@'))
    {
        vec![format!("--{scope}:registry={registry}")]
    } else {
        vec!["--registry".to_owned(), registry.to_owned()]
    }
}

/// Classify an install from the binary's path.
///
/// Pure and I/O-free so it is unit testable without touching the real
/// executable. Symlinked global bins (npm, pnpm) resolve through
/// [`std::env::current_exe`] into the store, so a `node_modules` component is
/// still present after resolution. Matching whole path *components* (not
/// substrings) avoids false positives like `.cargo-backup` or a project
/// literally named `node_modules-tools`. npm wins if both appear.
#[must_use]
pub fn detect_source(exe_path: &Path) -> InstallSource {
    let mut npm = false;
    let mut cargo = false;
    for component in exe_path.components() {
        if let std::path::Component::Normal(c) = component {
            if c == "node_modules" {
                npm = true;
            } else if c == ".cargo" {
                cargo = true;
            }
        }
    }
    if npm {
        InstallSource::Npm
    } else if cargo {
        InstallSource::Cargo
    } else {
        InstallSource::Unknown
    }
}

/// Detect the install source from the running executable and dispatch the
/// matching upgrade path.
///
/// Shelling out to npm/cargo (with inherited stdio so their progress is
/// visible) is the one side effect, and it is gated behind `opts.check`.
///
/// # Errors
///
/// [`SelfUpdateError::CurrentExe`] if the running executable's path cannot be
/// resolved, or whatever [`run_for_source`] returns.
pub fn run_self_update(
    cfg: &SelfUpdateConfig,
    opts: &SelfUpdateOpts,
) -> Result<SelfUpdateReport, SelfUpdateError> {
    let exe =
        std::env::current_exe().map_err(|source| SelfUpdateError::CurrentExe { source })?;
    run_for_source(cfg, opts, detect_source(&exe))
}

/// Source-injected core, split out so tests can exercise every channel without
/// manipulating the real [`std::env::current_exe`].
///
/// # Errors
///
/// The npm and cargo channels fail if their package manager cannot be spawned
/// or exits non-zero. [`InstallSource::Unknown`] never fails: it installs
/// nothing.
pub fn run_for_source(
    cfg: &SelfUpdateConfig,
    opts: &SelfUpdateOpts,
    source: InstallSource,
) -> Result<SelfUpdateReport, SelfUpdateError> {
    match source {
        InstallSource::Npm => npm_update(cfg, opts),
        InstallSource::Cargo => cargo_update(cfg, opts),
        InstallSource::Unknown => Ok(manual_report(cfg)),
    }
}

fn npm_update(
    cfg: &SelfUpdateConfig,
    opts: &SelfUpdateOpts,
) -> Result<SelfUpdateReport, SelfUpdateError> {
    let reg_args = registry_args(cfg.npm_package, opts.registry.as_deref());

    if opts.check {
        let mut args = vec!["view", cfg.npm_package, "version"];
        args.extend(reg_args.iter().map(String::as_str));
        let latest = run_capture(
            "npm",
            &args,
            "querying the npm registry for the latest version",
        )?;
        let messages = vec![
            format!("installed via npm ({})", cfg.npm_package),
            format!("latest published version: {latest}"),
            format!("run `{}` to upgrade", cfg.update_command),
        ];
        return Ok(SelfUpdateReport {
            source: InstallSource::Npm,
            action: Action::Checked {
                latest: Some(latest),
            },
            messages,
        });
    }

    let spec = format!("{}@latest", cfg.npm_package);
    let mut args = vec!["install", "-g", spec.as_str()];
    args.extend(reg_args.iter().map(String::as_str));
    run_inherited("npm", &args, "running npm install -g to upgrade")?;
    Ok(SelfUpdateReport {
        source: InstallSource::Npm,
        action: Action::Installed,
        messages: vec![format!("upgraded {} via npm", cfg.npm_package)],
    })
}

fn cargo_update(
    cfg: &SelfUpdateConfig,
    opts: &SelfUpdateOpts,
) -> Result<SelfUpdateReport, SelfUpdateError> {
    if opts.check {
        let messages = vec![
            "installed via cargo".to_owned(),
            format!(
                "cargo installs track the git default branch ({}); there is no single \
                 published version to compare against",
                cfg.cargo_git
            ),
            format!(
                "run `{}` to rebuild from the latest source",
                cfg.update_command
            ),
        ];
        return Ok(SelfUpdateReport {
            source: InstallSource::Cargo,
            action: Action::Checked { latest: None },
            messages,
        });
    }

    run_inherited(
        "cargo",
        &["install", "--git", cfg.cargo_git, "--force"],
        "running cargo install --git to upgrade",
    )?;
    Ok(SelfUpdateReport {
        source: InstallSource::Cargo,
        action: Action::Installed,
        messages: vec![format!("upgraded from {} via cargo", cfg.cargo_git)],
    })
}

/// Unknown install source: never guess and clobber a binary we did not place.
/// Report how to upgrade and succeed.
fn manual_report(cfg: &SelfUpdateConfig) -> SelfUpdateReport {
    let messages = vec![
        "could not determine how this binary was installed".to_owned(),
        format!(
            "if installed via npm:   npm install -g {}@latest",
            cfg.npm_package
        ),
        format!(
            "if installed via cargo: cargo install --git {} --force",
            cfg.cargo_git
        ),
        format!("or download the latest release: {}", cfg.releases_url),
    ];
    SelfUpdateReport {
        source: InstallSource::Unknown,
        action: Action::Manual,
        messages,
    }
}

/// Run a command with inherited stdio; non-zero exit is an error.
fn run_inherited(
    program: &'static str,
    args: &[&str],
    action: &'static str,
) -> Result<(), SelfUpdateError> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|source| SelfUpdateError::Spawn {
            action,
            program,
            source,
        })?;
    if !status.success() {
        return Err(SelfUpdateError::Failed {
            action,
            program,
            status: status.to_string(),
        });
    }
    Ok(())
}

/// Run a command and capture trimmed stdout; non-zero exit is an error.
fn run_capture(
    program: &'static str,
    args: &[&str],
    action: &'static str,
) -> Result<String, SelfUpdateError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|source| SelfUpdateError::Spawn {
            action,
            program,
            source,
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SelfUpdateError::FailedWithStderr {
            action,
            program,
            message: stderr.trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
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

    // A fixture consumer, standing in for whichever CLI adopts this module.
    // The values are quire-cli's because that is the implementation these tests
    // were ported from; nothing in `super` knows them.
    const CFG: SelfUpdateConfig = SelfUpdateConfig {
        npm_package: "@agent-ix/quire-cli",
        cargo_git: "https://github.com/agent-ix/quire-cli",
        releases_url: "https://github.com/agent-ix/quire-cli/releases",
        update_command: "quire update",
    };

    /// Provenance: quire-cli TC-086 — no override yields an empty arg vec.
    #[test]
    fn registry_args_omitted_when_no_override() {
        assert!(registry_args("@agent-ix/quire-cli", None).is_empty());
    }

    /// Provenance: quire-cli TC-086 — a scoped package yields the
    /// `--@scope:registry=` form.
    #[test]
    fn registry_args_uses_scope_form_for_scoped_package() {
        // A plain --registry is ignored for scoped packages when an npmrc pins
        // a scope registry; the @scope:registry form is what npm honors.
        assert_eq!(
            registry_args("@agent-ix/quire-cli", Some("http://npm.ix/")),
            vec!["--@agent-ix:registry=http://npm.ix/".to_owned()]
        );
    }

    /// Provenance: quire-cli TC-086 — an unscoped package yields a plain
    /// `--registry <url>`.
    #[test]
    fn registry_args_uses_plain_registry_for_unscoped_package() {
        assert_eq!(
            registry_args("some-cli", Some("https://registry.npmjs.org/")),
            vec![
                "--registry".to_owned(),
                "https://registry.npmjs.org/".to_owned()
            ]
        );
    }

    /// Provenance: quire-cli TC-085 — a `node_modules` path classifies as Npm.
    #[test]
    fn detect_npm_from_node_modules_path() {
        let p =
            Path::new("/home/u/.npm-global/lib/node_modules/@agent-ix/quire-cli-linux-x64/quire");
        assert_eq!(detect_source(p), InstallSource::Npm);
    }

    /// Provenance: quire-cli TC-085 — a `.cargo` path classifies as Cargo.
    #[test]
    fn detect_cargo_from_cargo_bin_path() {
        let p = Path::new("/home/u/.cargo/bin/quire");
        assert_eq!(detect_source(p), InstallSource::Cargo);
    }

    /// Provenance: quire-cli TC-085 — a bare path classifies as Unknown.
    #[test]
    fn detect_unknown_from_bare_path() {
        let p = Path::new("/usr/local/bin/quire");
        assert_eq!(detect_source(p), InstallSource::Unknown);
    }

    #[test]
    fn detect_ignores_lookalike_path_components() {
        // Substring matching would misclassify these; component matching does not.
        assert_eq!(
            detect_source(Path::new("/home/u/.cargo-backup/bin/quire")),
            InstallSource::Unknown
        );
        assert_eq!(
            detect_source(Path::new("/opt/node_modules-tools/quire")),
            InstallSource::Unknown
        );
    }

    /// Provenance: quire-cli TC-087 — `run_for_source(Unknown)` reports Manual
    /// and installs nothing.
    #[test]
    fn unknown_source_emits_manual_instructions_without_installing() {
        let opts = SelfUpdateOpts {
            check: false,
            registry: None,
        };
        let report = run_for_source(&CFG, &opts, InstallSource::Unknown).unwrap();
        assert_eq!(report.action, Action::Manual);
        assert_eq!(report.source, InstallSource::Unknown);
        // Carries both upgrade recipes and the releases URL.
        let joined = report.messages.join("\n");
        assert!(joined.contains("npm install -g @agent-ix/quire-cli@latest"));
        assert!(joined.contains("cargo install --git"));
        assert!(joined.contains("releases"));
    }

    /// Provenance: quire-cli TC-087 — `run_for_source(Cargo, --check)` reports
    /// git-branch tracking with `latest: None`, i.e. no cross-scheme version.
    #[test]
    fn cargo_check_reports_branch_tracking_without_a_version() {
        let opts = SelfUpdateOpts {
            check: true,
            registry: None,
        };
        let report = run_for_source(&CFG, &opts, InstallSource::Cargo).unwrap();
        assert_eq!(report.action, Action::Checked { latest: None });
        // `latest: None` alone never checked that the report actually SAYS why
        // there is no version to compare (quire-cli SR-006 FND-002).
        let joined = report.messages.join("\n");
        assert!(
            joined.contains("track the git default branch"),
            "the check report must explain branch tracking: {joined}"
        );
        assert!(
            joined.contains(CFG.cargo_git),
            "the report names the tracked repository: {joined}"
        );
    }

    // The boundary rule, asserted rather than asserted-about: every identifier
    // in a report comes from the config, so a second consumer's report names
    // that consumer and never the one these tests were ported from.
    #[test]
    fn every_identifier_in_a_report_comes_from_the_config() {
        const OTHER: SelfUpdateConfig = SelfUpdateConfig {
            npm_package: "@example/other-cli",
            cargo_git: "https://github.com/example/other-cli",
            releases_url: "https://github.com/example/other-cli/releases",
            update_command: "other update",
        };
        let check = SelfUpdateOpts {
            check: true,
            registry: None,
        };
        let manual = run_for_source(&OTHER, &SelfUpdateOpts::default(), InstallSource::Unknown)
            .unwrap()
            .messages
            .join("\n");
        let cargo = run_for_source(&OTHER, &check, InstallSource::Cargo)
            .unwrap()
            .messages
            .join("\n");
        for rendered in [&manual, &cargo] {
            assert!(
                !rendered.contains("quire"),
                "a package-specific identifier leaked from the engine: {rendered}"
            );
        }
        assert!(manual.contains("@example/other-cli"), "{manual}");
        assert!(cargo.contains("run `other update`"), "{cargo}");
    }
}
