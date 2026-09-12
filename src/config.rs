// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Where a setting's value comes from, and in what order.
//!
//! **This crate decides the ORDER. The consumer decides the LOCATIONS.**
//!
//! That split is what makes the module safe to adopt. The ecosystem's actual
//! default module root is `~/.ix/filament/modules` — a dotdir that `quoin`
//! materialises into and `quire-rs` reads by default. A shared crate that
//! imposed XDG would silently relocate a path two tools already agree on and
//! break every existing install. So [`xdg`] is *available* to a consumer that
//! wants it and is never applied on the consumer's behalf, and nothing here
//! writes or creates a path the consumer did not name.
//!
//! # This module is a port, not a design
//!
//! The survey that scoped this crate asked "does it have a config FILE" and got
//! 0 of 5. That was the wrong question. "Does it RESOLVE configuration" has a
//! different answer: `quire-cli/src/commands/validate.rs:485-520` already
//! implements the full chain by hand —
//!
//! 1. the `--scope` flag, always the FIRST search root;
//! 2. `IX_FILAMENT_MODULES_PATH`;
//! 3. `IX_SCHEMA_PATH`, a legacy alias;
//! 4. `quire_rs::loader::paths::default_module_root()`, the canonical install
//!    root.
//!
//! Two decisions are encoded there that nobody had written down as a contract,
//! and settling them is why this module exists:
//!
//! - **Path sets UNION; they do not override.** Every path from *both* env vars
//!   is kept, deduplicated in first-seen order by that file's `push_root`. A
//!   plain first-wins chain would drop the second var's roots entirely. See
//!   [`SearchPath`].
//! - **`IX_SCHEMA_PATH` is a legacy alias with no deprecation path and no
//!   warning.** An alias that nothing ever says is deprecated is an alias
//!   forever. [`EnvVar::legacy`] gives one a stated lifecycle and
//!   [`SearchPath::deprecations`] reports each legacy variable that actually
//!   contributed, so a consumer can emit a warning through
//!   [`crate::streams`] instead of the fact being invisible.
//!
//! # Two shapes, because there are two kinds of setting
//!
//! | shape | semantics | for |
//! |---|---|---|
//! | [`SearchPath`] | union, ordered, deduplicated | where to LOOK — module roots, plugin dirs |
//! | [`resolve`] / [`Resolved`] | override: flag > env > file > default | a single VALUE — a format, a timeout |
//!
//! Collapsing them would be the bug: applying override semantics to search
//! roots is precisely the behaviour change that would break `quire-cli`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Where a resolved value came from. Reported so a `--explain`-style surface,
/// or a bug report, can say *why* a tool behaved as it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// An explicit command-line argument.
    Flag,
    /// An environment variable.
    Env,
    /// A configuration file.
    File,
    /// The compiled-in default.
    Default,
}

impl Source {
    /// The stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::Env => "env",
            Self::File => "file",
            Self::Default => "default",
        }
    }
}

/// A value together with where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolved<T> {
    /// The winning value.
    pub value: T,
    /// Which layer supplied it.
    pub source: Source,
    /// The environment variable or file path that supplied it, when one did.
    pub origin: Option<&'static str>,
}

/// Resolve one scalar setting: **flag > env > file > default**.
///
/// The order is the contract and it is implemented once, here, because it is
/// the part every CLI gets subtly different. An env layer that failed to parse
/// is **not** skipped in favour of the next layer — see [`resolve_parsed`];
/// falling through would make a typo in a variable name's *value* silently
/// change behaviour.
pub fn resolve<T>(
    flag: Option<T>,
    env: Option<(&'static str, T)>,
    file: Option<T>,
    default: T,
) -> Resolved<T> {
    if let Some(value) = flag {
        return Resolved {
            value,
            source: Source::Flag,
            origin: None,
        };
    }
    if let Some((name, value)) = env {
        return Resolved {
            value,
            source: Source::Env,
            origin: Some(name),
        };
    }
    if let Some(value) = file {
        return Resolved {
            value,
            source: Source::File,
            origin: None,
        };
    }
    Resolved {
        value: default,
        source: Source::Default,
        origin: None,
    }
}

/// Read and parse one environment variable, keeping a parse failure visible.
///
/// `Ok(None)` means unset. A variable that is set but unparseable is an
/// `Err`, never a fall-through to the next layer: a tool that quietly ignores
/// `IX_DIAGNOSTICS_FORMAT=jsonn` and uses the default has told the operator
/// nothing and done the wrong thing.
///
/// # Errors
///
/// [`ConfigError::BadEnvValue`] when the variable is set and does not parse.
pub fn resolve_parsed<T: std::str::FromStr>(name: &'static str) -> Result<Option<T>, ConfigError>
where
    T::Err: std::fmt::Display,
{
    let Some(raw) = std::env::var_os(name) else {
        return Ok(None);
    };
    let raw = raw.to_string_lossy().into_owned();
    raw.parse::<T>()
        .map(Some)
        .map_err(|error| ConfigError::BadEnvValue {
            name,
            value: raw,
            message: error.to_string(),
        })
}

/// An environment variable this tool reads, and its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvVar {
    /// The variable name.
    pub name: &'static str,
    /// The variable that supersedes this one, when it is a legacy alias.
    pub superseded_by: Option<&'static str>,
}

impl EnvVar {
    /// A variable with no scheduled removal.
    #[must_use]
    pub const fn current(name: &'static str) -> Self {
        Self {
            name,
            superseded_by: None,
        }
    }

    /// A legacy alias, naming what replaces it.
    ///
    /// The naming is the whole feature. `IX_SCHEMA_PATH` has been a legacy
    /// alias in `quire-cli` with no deprecation path and no warning, which
    /// means nothing has ever told an operator to stop using it — and so
    /// nothing can ever remove it.
    #[must_use]
    pub const fn legacy(name: &'static str, superseded_by: &'static str) -> Self {
        Self {
            name,
            superseded_by: Some(superseded_by),
        }
    }

    /// Whether this variable is a legacy alias.
    #[must_use]
    pub const fn is_legacy(&self) -> bool {
        self.superseded_by.is_some()
    }
}

/// An ordered, deduplicated set of directories to search.
///
/// **Union semantics**, ported from `quire-cli`'s `scoped_registry_roots`: a
/// root contributed by any layer is kept, in first-seen order, and a repeat is
/// dropped rather than reordered. This is deliberately *not* the override chain
/// [`resolve`] implements — applying override semantics to search roots would
/// drop the second environment variable's paths entirely, which is a behaviour
/// change, not a cleanup.
///
/// Existence is checked (`is_dir`) for env- and default-supplied roots, matching
/// the ported behaviour: a stale entry in a `PATH`-style variable should not
/// become a search root. An explicitly supplied root is kept regardless, because
/// an operator who names a directory that does not exist has made an error the
/// consumer should be able to report rather than one this crate should hide.
#[derive(Debug, Default, Clone)]
pub struct SearchPath {
    roots: Vec<PathBuf>,
    seen: HashSet<PathBuf>,
    deprecations: Vec<EnvVar>,
}

impl SearchPath {
    /// An empty search path.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an explicitly supplied root — a `--scope` flag, typically.
    ///
    /// Kept whether or not it exists, and first if pushed first: in the ported
    /// code the `--scope` directory is always the first search root.
    #[must_use]
    pub fn push(mut self, root: impl Into<PathBuf>) -> Self {
        self.insert(root.into());
        self
    }

    /// Push a root only when it is an existing directory.
    #[must_use]
    pub fn push_if_dir(mut self, root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        if root.is_dir() {
            self.insert(root);
        }
        self
    }

    /// Union in every existing directory named by one environment variable,
    /// split with the platform's `PATH` separator.
    ///
    /// Records the variable in [`SearchPath::deprecations`] when it is a legacy
    /// alias **and it actually contributed a root** — a deprecation warning for
    /// a variable that changed nothing is noise.
    #[must_use]
    pub fn push_env(mut self, var: EnvVar) -> Self {
        let Some(raw) = std::env::var_os(var.name) else {
            return self;
        };
        let mut contributed = false;
        for path in std::env::split_paths(&raw) {
            if path.is_dir() && self.insert(path) {
                contributed = true;
            }
        }
        if contributed && var.is_legacy() {
            self.deprecations.push(var);
        }
        self
    }

    /// Union in every variable in order. The first named variable's roots come
    /// first; none of them suppresses another.
    #[must_use]
    pub fn push_envs(mut self, vars: &[EnvVar]) -> Self {
        for var in vars {
            self = self.push_env(*var);
        }
        self
    }

    /// The resolved roots, in search order.
    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Consume and return the roots.
    #[must_use]
    pub fn into_roots(self) -> Vec<PathBuf> {
        self.roots
    }

    /// Every legacy alias that actually contributed a root.
    ///
    /// A consumer turns these into warnings through [`crate::streams`]. This
    /// module reports the fact and does not emit: a TUI places its own output.
    #[must_use]
    pub fn deprecations(&self) -> &[EnvVar] {
        &self.deprecations
    }

    fn insert(&mut self, root: PathBuf) -> bool {
        if self.seen.insert(root.clone()) {
            self.roots.push(root);
            true
        } else {
            false
        }
    }
}

/// What was found at a configuration path.
///
/// Absent and malformed are kept apart deliberately. "No config file" is a
/// normal state for every tool in this ecosystem; "a config file that does not
/// parse" is an error an operator must see, with the file and line named.
/// Collapsing them into `Option` makes a syntax error indistinguishable from a
/// fresh install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loaded<T> {
    /// No file at that path.
    Absent,
    /// A parsed configuration.
    Present(T),
}

impl<T> Loaded<T> {
    /// The parsed value, or a supplied default for an absent file.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Absent => default,
            Self::Present(value) => value,
        }
    }

    /// Whether a file was found.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }
}

/// A configuration failure. Every variant names the thing it is about.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file exists but could not be read.
    #[error("{path}: cannot be read: {source}")]
    Unreadable {
        /// The path that failed.
        path: String,
        /// The underlying `std::io` failure.
        #[source]
        source: std::io::Error,
    },
    /// The file exists and does not parse. Names file, line and column.
    ///
    /// A scanner whose message does not name a file and a line hides the real
    /// error; this variant's `Display` is `path:line:column: message`.
    #[error("{path}:{line}:{column}: {message}")]
    Malformed {
        /// The path that failed.
        path: String,
        /// 1-based line.
        line: usize,
        /// 1-based column.
        column: usize,
        /// What the parser said.
        message: String,
    },
    /// An environment variable is set and does not parse.
    #[error("{name}={value}: {message}")]
    BadEnvValue {
        /// The variable name.
        name: &'static str,
        /// What it was set to.
        value: String,
        /// What the parser said.
        message: String,
    },
}

/// Where a parse failed, for a format whose error type this crate cannot know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLocation {
    /// 1-based line.
    pub line: usize,
    /// 1-based column.
    pub column: usize,
}

impl Default for ParseLocation {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

/// Load a configuration file with a caller-supplied parser.
///
/// **No format lock-in**: the contract is "text in, `Result` out". A consumer
/// that wants TOML, YAML or a bespoke format supplies its own parser and its
/// own [`ParseLocation`]; this crate does not grow a dependency for each.
///
/// # Errors
///
/// [`ConfigError::Unreadable`] if the file exists and cannot be read, or
/// [`ConfigError::Malformed`] if the parser rejects it. A missing file is
/// [`Loaded::Absent`], not an error.
pub fn load_with<T, E, F>(path: &Path, parse: F) -> Result<Loaded<T>, ConfigError>
where
    F: FnOnce(&str) -> Result<T, (ParseLocation, E)>,
    E: std::fmt::Display,
{
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::Absent),
        Err(source) => {
            return Err(ConfigError::Unreadable {
                path: path.display().to_string(),
                source,
            });
        }
    };
    parse(&text)
        .map(Loaded::Present)
        .map_err(|(at, error)| ConfigError::Malformed {
            path: path.display().to_string(),
            line: at.line,
            column: at.column,
            message: error.to_string(),
        })
}

/// Load a JSON configuration file into any `Deserialize` type.
///
/// A convenience over [`load_with`] for the one format this crate already
/// depends on — and the one whose parser reports a line and a column, so the
/// error can name them.
///
/// # Errors
///
/// As [`load_with`].
pub fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Loaded<T>, ConfigError> {
    load_with(path, |text| {
        serde_json::from_str::<T>(text).map_err(|error| {
            (
                ParseLocation {
                    line: error.line(),
                    column: error.column(),
                },
                error,
            )
        })
    })
}

/// XDG base-directory resolution — **available, never imposed**.
///
/// Offered for a consumer that wants it. It is not this crate's default and no
/// function outside this module calls it: the ecosystem's existing default root
/// is `~/.ix/filament/modules`, and relocating it would break every install.
///
/// The rules implemented are the XDG Base Directory Specification's, stated
/// rather than assumed:
///
/// - `$XDG_CONFIG_HOME`, else `$HOME/.config`.
/// - `$XDG_DATA_HOME`, else `$HOME/.local/share`.
/// - `$XDG_CACHE_HOME`, else `$HOME/.cache`.
/// - **A relative path in any of those variables is ignored**, per the
///   specification, and the default is used instead. A silently-honoured
///   relative path resolves against the process's working directory, which
///   makes a tool's config location depend on where it was run from.
///
/// The spec is Unix-shaped. On Windows `%APPDATA%` and `%LOCALAPPDATA%` are used
/// instead, because there is no `$HOME` to fall back to; the function returns
/// `None` when neither a variable nor its fallback is available, rather than
/// inventing a path.
pub mod xdg {
    use std::path::PathBuf;

    /// `$XDG_CONFIG_HOME/<app>`, else `$HOME/.config/<app>`.
    #[must_use]
    pub fn config_dir(app: &str) -> Option<PathBuf> {
        base("XDG_CONFIG_HOME", &[".config"], "APPDATA").map(|base| base.join(app))
    }

    /// `$XDG_DATA_HOME/<app>`, else `$HOME/.local/share/<app>`.
    #[must_use]
    pub fn data_dir(app: &str) -> Option<PathBuf> {
        base("XDG_DATA_HOME", &[".local", "share"], "LOCALAPPDATA").map(|base| base.join(app))
    }

    /// `$XDG_CACHE_HOME/<app>`, else `$HOME/.cache/<app>`.
    #[must_use]
    pub fn cache_dir(app: &str) -> Option<PathBuf> {
        base("XDG_CACHE_HOME", &[".cache"], "LOCALAPPDATA").map(|base| base.join(app))
    }

    /// Apply the specification's rule for an override variable's value.
    ///
    /// A relative path is **ignored**, per the specification. Honouring one
    /// silently resolves a tool's configuration location against the process's
    /// working directory, so where a tool was run from would change where it
    /// read its settings.
    #[must_use]
    pub fn honour_override(value: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
        let path = PathBuf::from(value?);
        path.is_absolute().then_some(path)
    }

    fn base(var: &str, home_relative: &[&str], windows_var: &str) -> Option<PathBuf> {
        if let Some(path) = honour_override(std::env::var_os(var).as_deref()) {
            return Some(path);
        }
        if cfg!(windows)
            && let Some(value) = std::env::var_os(windows_var)
        {
            return Some(PathBuf::from(value));
        }
        let mut path = PathBuf::from(std::env::var_os("HOME")?);
        for segment in home_relative {
            path.push(segment);
        }
        Some(path)
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
    fn the_override_chain_is_flag_then_env_then_file_then_default() {
        assert_eq!(
            resolve(Some(1), Some(("E", 2)), Some(3), 4),
            Resolved {
                value: 1,
                source: Source::Flag,
                origin: None
            }
        );
        assert_eq!(
            resolve(None, Some(("E", 2)), Some(3), 4),
            Resolved {
                value: 2,
                source: Source::Env,
                origin: Some("E")
            }
        );
        assert_eq!(
            resolve(None::<i32>, None, Some(3), 4),
            Resolved {
                value: 3,
                source: Source::File,
                origin: None
            }
        );
        assert_eq!(
            resolve(None::<i32>, None, None, 4),
            Resolved {
                value: 4,
                source: Source::Default,
                origin: None
            }
        );
    }

    // The ported behaviour: the explicit root is FIRST, both env vars are
    // UNIONED, and a repeat is dropped rather than reordered. An override chain
    // would have kept only one of the two variables.
    #[test]
    fn search_roots_union_and_deduplicate_in_first_seen_order() {
        let path = SearchPath::new()
            .push("/scope")
            .push("/a")
            .push("/b")
            .push("/a");
        assert_eq!(
            path.roots(),
            [
                PathBuf::from("/scope"),
                PathBuf::from("/a"),
                PathBuf::from("/b")
            ]
        );
    }

    #[test]
    fn an_explicit_root_is_kept_even_when_it_does_not_exist() {
        let path = SearchPath::new().push("/definitely/not/here");
        assert_eq!(path.roots().len(), 1);
        assert!(
            SearchPath::new()
                .push_if_dir("/definitely/not/here")
                .roots()
                .is_empty()
        );
    }

    #[test]
    fn a_legacy_alias_names_what_supersedes_it() {
        let legacy = EnvVar::legacy("IX_SCHEMA_PATH", "IX_FILAMENT_MODULES_PATH");
        assert!(legacy.is_legacy());
        assert_eq!(legacy.superseded_by, Some("IX_FILAMENT_MODULES_PATH"));
        assert!(!EnvVar::current("IX_FILAMENT_MODULES_PATH").is_legacy());
    }

    #[test]
    fn an_unset_variable_contributes_nothing_and_deprecates_nothing() {
        let path =
            SearchPath::new().push_envs(&[EnvVar::legacy("IX_CLI_RS_NO_SUCH_VAR", "IX_OTHER")]);
        assert!(path.roots().is_empty());
        assert!(path.deprecations().is_empty());
    }

    #[test]
    fn an_absent_file_is_absent_not_an_error() {
        let loaded: Loaded<serde_json::Value> =
            load_json(Path::new("/definitely/not/here/config.json")).unwrap();
        assert_eq!(loaded, Loaded::Absent);
        assert!(!loaded.is_present());
    }

    // The distinction this type exists for: malformed must name file AND line.
    #[test]
    fn a_malformed_file_names_the_path_and_the_line() {
        let dir = std::env::temp_dir().join("ix-cli-kit-config-malformed");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        std::fs::write(&path, "{\n  \"a\": 1,\n  oops\n}\n").unwrap();

        let error = load_json::<serde_json::Value>(&path).unwrap_err();
        let ConfigError::Malformed { line, .. } = &error else {
            panic!("expected malformed, got {error}");
        };
        assert_eq!(*line, 3);
        let rendered = error.to_string();
        assert!(
            rendered.starts_with(&format!("{}:3:", path.display())),
            "{rendered}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
    struct Settings {
        format: String,
    }

    #[test]
    fn a_present_file_parses_into_the_requested_type() {
        let dir = std::env::temp_dir().join("ix-cli-kit-config-present");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        std::fs::write(&path, r#"{"format":"json"}"#).unwrap();

        let loaded: Loaded<Settings> = load_json(&path).unwrap();
        assert_eq!(
            loaded,
            Loaded::Present(Settings {
                format: "json".to_owned()
            })
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn source_spellings_are_stable() {
        assert_eq!(
            [
                Source::Flag.as_str(),
                Source::Env.as_str(),
                Source::File.as_str(),
                Source::Default.as_str()
            ],
            ["flag", "env", "file", "default"]
        );
    }

    // XDG is available, not imposed — and a relative override is ignored per
    // the specification rather than resolved against the working directory.
    #[test]
    fn a_relative_xdg_override_is_ignored() {
        use std::ffi::OsStr;
        assert_eq!(
            xdg::honour_override(Some(OsStr::new("relative/path"))),
            None
        );
        assert_eq!(xdg::honour_override(Some(OsStr::new(""))), None);
        assert_eq!(
            xdg::honour_override(Some(OsStr::new("/absolute/path"))),
            Some(PathBuf::from("/absolute/path"))
        );
        assert_eq!(xdg::honour_override(None), None);
    }
}
