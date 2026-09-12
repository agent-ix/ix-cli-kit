// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Which stream a line goes to, whether it may be coloured, and who decides.
//!
//! Moved from `quire-cli/src/io.rs`. The split is the contract: **results on
//! stdout, diagnostics on stderr**, and a result is never coloured.
//!
//! [`DiagnosticsFormat`] and [`ColorChoice`] implement [`std::str::FromStr`]
//! rather than deriving a clap `ValueEnum`. That is deliberate and is the
//! reason this crate has **no clap dependency at all**: `FromStr` is what
//! `clap::value_parser!` consumes, what a bare-`std` argv loop consumes, and
//! what a config file consumes. `engineering-assurance` compiles clap with
//! `default-features = false` and no derive feature; a foundation crate that
//! exported clap types could not be adopted there.
//!
//! # Not line-oriented by assumption
//!
//! `ix-cli` is a TUI, and it is the consumer this crate exists to serve. A TUI
//! renders a diagnostic into a pane; it does not `eprintln!` it. So every
//! decision here is split in two:
//!
//! | decide | act |
//! |---|---|
//! | [`ColorChoice`] + [`ColorDecision`] — the operator's intent and the facts | [`Diagnostics::color_enabled`] |
//! | [`Record::render_human`] / [`Record::render_json`] — a `String` | [`Record::emit`] — writes it |
//! | [`stdout_is_terminal`] / [`stderr_is_terminal`] — a fact a consumer may ask | colour resolution |
//!
//! A consumer that wants the string takes the left column and never touches
//! the right. Nothing here opens an alternate screen, sets raw mode, asks for
//! terminal size or runs an event loop — that is TUI machinery and it is not
//! this crate's. The rule observed while writing this module was: where a
//! choice would foreclose one of those, take the other branch and say so.

use std::io::{IsTerminal, Write};

use serde::Serialize;

/// Format selector for stderr diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagnosticsFormat {
    /// One rendered line per diagnostic, for an operator.
    #[default]
    Human,
    /// One JSON object per diagnostic, for a machine.
    Json,
}

impl std::str::FromStr for DiagnosticsFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            other => Err(format!("unknown diagnostics format: '{other}'")),
        }
    }
}

/// When to colorize human-format diagnostics. The operator's **intent**, kept
/// distinct from the resolved decision so a consumer can re-resolve it itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Colorize only when the stream is a terminal and `NO_COLOR` is unset.
    #[default]
    Auto,
    /// Always colorize, even when piped.
    Always,
    /// Never colorize.
    Never,
}

impl std::str::FromStr for ColorChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            other => Err(format!("unknown color choice: '{other}'")),
        }
    }
}

impl ColorChoice {
    /// Resolve against explicitly supplied facts.
    ///
    /// Pure, and separate from [`ColorChoice::resolve`], so a TUI that knows
    /// its own surface is a terminal — or is rendering into a pane that is not
    /// one — can apply the same rule to different facts instead of having the
    /// process's stderr imposed on it.
    #[must_use]
    pub const fn decide(self, is_terminal: bool, no_color_set: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => !no_color_set && is_terminal,
        }
    }

    /// Resolve the choice for *this* process's stderr.
    ///
    /// `Auto` honours the `NO_COLOR` convention and only colorizes a real
    /// terminal — so piped/redirected output (and the test harness) stays
    /// plain, byte-for-byte.
    #[must_use]
    pub fn resolve(self) -> ColorDecision {
        let no_color_set = std::env::var_os("NO_COLOR").is_some();
        let is_terminal = stderr_is_terminal();
        ColorDecision {
            choice: self,
            no_color_set,
            is_terminal,
            enabled: self.decide(is_terminal, no_color_set),
        }
    }
}

/// A resolved colour decision **and the facts it was resolved from**.
///
/// The boolean alone is what the writer needs; the rest is what a consumer
/// needs in order to disagree. A TUI resolves colour itself, and a `doctor`
/// surface has to explain why output came out plain — neither is expressible
/// if the resolution collapses to one `bool` at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorDecision {
    /// What the operator asked for.
    pub choice: ColorChoice,
    /// Whether `NO_COLOR` was set in the environment.
    pub no_color_set: bool,
    /// Whether the stream consulted was a terminal.
    pub is_terminal: bool,
    /// The decision the writer applies.
    pub enabled: bool,
}

impl Default for ColorDecision {
    fn default() -> Self {
        Self {
            choice: ColorChoice::Auto,
            no_color_set: false,
            is_terminal: false,
            enabled: false,
        }
    }
}

/// Whether this process's stdout is a terminal.
///
/// Exported rather than only consumed internally: a TUI has to ask this to
/// decide whether it may take the screen at all, and a tool has to ask it to
/// decide whether a progress surface is worth rendering.
#[must_use]
pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

/// Whether this process's stderr is a terminal.
#[must_use]
pub fn stderr_is_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// Resolved diagnostic settings: the wire format plus the colour decision.
#[derive(Debug, Clone, Copy, Default)]
pub struct Diagnostics {
    /// The stderr encoding.
    pub format: DiagnosticsFormat,
    /// The colour decision, with the facts behind it.
    pub color: ColorDecision,
}

impl Diagnostics {
    /// Pair a format with an already-resolved colour decision.
    #[must_use]
    pub const fn new(format: DiagnosticsFormat, color: ColorDecision) -> Self {
        Self { format, color }
    }

    /// Resolve a [`ColorChoice`] against this process once, at startup.
    #[must_use]
    pub fn resolved(format: DiagnosticsFormat, color: ColorChoice) -> Self {
        Self {
            format,
            color: color.resolve(),
        }
    }

    /// Whether human diagnostics are coloured.
    #[must_use]
    pub const fn color_enabled(self) -> bool {
        self.color.enabled
    }
}

// ANSI SGR codes. Diagnostics are the only colorized surface, so a tiny
// hand-rolled palette beats pulling in a color crate (leaf-binary, deny.toml).
const RED: &str = "\x1b[31m";
const BOLD_YELLOW: &str = "\x1b[1;33m";
const RESET: &str = "\x1b[0m";

/// How severe a diagnostic is. Carried in the JSON shape so machine consumers
/// can separate warnings from errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The run is qualified or degraded but proceeded.
    Warning,
    /// Something the operator must act on.
    Error,
}

impl Severity {
    /// The stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Optional typed fields carried by a machine-readable diagnostic.
///
/// Human diagnostics render from `message`; these fields are the stable
/// interface for consumers that must locate or act on a finding without parsing
/// that prose. `path` and `line` are here because a scanner whose message does
/// not name a file and a line hides the real error behind an opaque sentence —
/// a recurring defect in this ecosystem's tooling.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct DiagnosticFields<'a> {
    /// A stable machine reason code, distinct from the prose message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'a str>,
    /// The file the diagnostic is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<&'a str>,
    /// The 1-based line within `path`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// The identifier of the subject the diagnostic is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<&'a str>,
    /// What the operator should change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<&'a str>,
}

/// One diagnostic, before anything has been decided about where it goes.
///
/// Rendering ([`Record::render_human`], [`Record::render_json`]) and emitting
/// ([`Record::emit`], [`Record::write`]) are separate on purpose: a TUI wants
/// the `String` and will place it itself, and a test wants the `String` so it
/// can assert on bytes instead of capturing a process stream.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Record<'a> {
    /// The diagnostic class — the name a consumer matches on.
    pub kind: &'a str,
    /// A sentence for an operator.
    pub message: &'a str,
    /// How severe it is.
    pub severity: Severity,
    /// Typed machine fields.
    #[serde(flatten)]
    pub fields: DiagnosticFields<'a>,
}

impl<'a> Record<'a> {
    /// An error-severity record with no typed fields.
    #[must_use]
    pub const fn error(kind: &'a str, message: &'a str) -> Self {
        Self {
            kind,
            message,
            severity: Severity::Error,
            fields: DiagnosticFields {
                reason: None,
                path: None,
                line: None,
                subject: None,
                remedy: None,
            },
        }
    }

    /// A warning-severity record with no typed fields.
    #[must_use]
    pub const fn warning(kind: &'a str, message: &'a str) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(kind, message)
        }
    }

    /// Attach typed machine fields.
    #[must_use]
    pub const fn with_fields(mut self, fields: DiagnosticFields<'a>) -> Self {
        self.fields = fields;
        self
    }

    /// Render the human form, without emitting it.
    ///
    /// A warning is prefixed `warning:` so it is distinguishable from an error
    /// at a glance; an error carries no prefix, which preserves the byte-for-byte
    /// message every existing caller emits today.
    #[must_use]
    pub fn render_human(&self, color: bool) -> String {
        match (self.severity, color) {
            (Severity::Error, true) => format!("{RED}{}{RESET}", self.message),
            (Severity::Error, false) => self.message.to_owned(),
            (Severity::Warning, true) => format!("{BOLD_YELLOW}warning:{RESET} {}", self.message),
            (Severity::Warning, false) => format!("warning: {}", self.message),
        }
    }

    /// Render the machine form, without emitting it.
    ///
    /// # Errors
    ///
    /// [`crate::json::JsonError`] if the typed fields cannot be serialised.
    pub fn render_json(&self) -> Result<String, crate::json::JsonError> {
        crate::json::encode(self, false)
    }

    /// Render according to `out`'s format, without emitting.
    ///
    /// A JSON-encoding failure degrades to the human rendering rather than
    /// returning an error, because the diagnostic must still reach the
    /// operator: swallowing it would hide the original problem behind an
    /// encoding problem nobody ever sees. The fallback is marked so the
    /// degradation is visible rather than silent.
    #[must_use]
    pub fn render(&self, out: Diagnostics) -> String {
        match out.format {
            DiagnosticsFormat::Human => self.render_human(out.color_enabled()),
            DiagnosticsFormat::Json => self.render_json().unwrap_or_else(|error| {
                format!(
                    "{{\"kind\":\"DiagnosticEncodingFailure\",\"severity\":\"error\",\"message\":{},\"reason\":{}}}",
                    escaped(self.message),
                    escaped(&error.to_string())
                )
            }),
        }
    }

    /// Render and write to an arbitrary sink.
    ///
    /// # Errors
    ///
    /// Any `std::io` failure from the sink.
    pub fn write<W: Write>(&self, out: Diagnostics, sink: &mut W) -> std::io::Result<()> {
        writeln!(sink, "{}", self.render(out))
    }

    /// Render and write to this process's stderr — the convenience over
    /// [`Record::write`], for a tool that is not placing its own output.
    pub fn emit(&self, out: Diagnostics) {
        eprintln!("{}", self.render(out));
    }
}

fn escaped(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_owned())
}

/// Write the primary command output to stdout, as bytes.
///
/// # Errors
///
/// Any `std::io` failure writing or flushing stdout.
pub fn write_primary_stdout(bytes: &[u8]) -> std::io::Result<()> {
    let mut out = std::io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

/// Write one **result** line to an arbitrary sink.
///
/// # Errors
///
/// Any `std::io` failure from the sink.
pub fn write_result<W: Write>(sink: &mut W, msg: &str) -> std::io::Result<()> {
    writeln!(sink, "{msg}")
}

/// Emit one **result** line — a census figure, a per-row record, anything a
/// caller redirecting with `>` came for.
///
/// Goes to **stdout**, never colorized. This is the other half of the
/// diagnostic surface, and the split is the contract: results on stdout,
/// diagnostics on stderr.
///
/// Before this existed, `write_diagnostic_human` was `eprintln!` wrapped in
/// `RED` and every human surface used it for everything. Measured over
/// `agent-ix/filament-ide-rs`, `quire coverage --scope . > out.txt` produced a
/// **0-byte file** while 90,462 bytes went to stderr — and
/// `Coverage: 1238/2390 rows backed (51%)`, a census, rendered in the same red
/// as every finding.
///
/// Never colorized even when color is on: a number is not a severity, and the
/// whole defect was a census that looked like a failure.
pub fn emit_result(msg: &str) {
    println!("{msg}");
}

/// Emit an error diagnostic according to the configured format.
pub fn emit_diagnostic(out: Diagnostics, kind: &str, message: &str) {
    Record::error(kind, message).emit(out);
}

/// Emit an error diagnostic with typed machine fields.
pub fn emit_diagnostic_with_fields(
    out: Diagnostics,
    kind: &str,
    message: &str,
    fields: DiagnosticFields<'_>,
) {
    Record::error(kind, message).with_fields(fields).emit(out);
}

/// Emit an advisory **warning** according to the configured format.
pub fn emit_warning(out: Diagnostics, kind: &str, message: &str) {
    Record::warning(kind, message).emit(out);
}

/// Emit an advisory warning with typed machine fields.
pub fn emit_warning_with_fields(
    out: Diagnostics,
    kind: &str,
    message: &str,
    fields: DiagnosticFields<'_>,
) {
    Record::warning(kind, message).with_fields(fields).emit(out);
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
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn diagnostics_format_parses() {
        assert_eq!(
            DiagnosticsFormat::from_str("human").unwrap(),
            DiagnosticsFormat::Human
        );
        assert_eq!(
            DiagnosticsFormat::from_str("json").unwrap(),
            DiagnosticsFormat::Json
        );
        assert!(DiagnosticsFormat::from_str("yaml").is_err());
    }

    #[test]
    fn color_choice_parses() {
        assert_eq!(ColorChoice::from_str("auto").unwrap(), ColorChoice::Auto);
        assert_eq!(
            ColorChoice::from_str("always").unwrap(),
            ColorChoice::Always
        );
        assert_eq!(ColorChoice::from_str("never").unwrap(), ColorChoice::Never);
        assert!(ColorChoice::from_str("rainbow").is_err());
    }

    #[test]
    fn an_unparseable_selector_names_the_value_it_rejected() {
        assert_eq!(
            DiagnosticsFormat::from_str("yaml").unwrap_err(),
            "unknown diagnostics format: 'yaml'"
        );
        assert_eq!(
            ColorChoice::from_str("rainbow").unwrap_err(),
            "unknown color choice: 'rainbow'"
        );
    }

    // The pure form is the one a TUI calls with its own facts, so it is pinned
    // against every combination rather than against this process's stderr.
    #[test]
    fn the_colour_rule_is_pure_and_total() {
        for terminal in [true, false] {
            for no_color in [true, false] {
                assert!(ColorChoice::Always.decide(terminal, no_color));
                assert!(!ColorChoice::Never.decide(terminal, no_color));
                assert_eq!(
                    ColorChoice::Auto.decide(terminal, no_color),
                    terminal && !no_color
                );
            }
        }
    }

    #[test]
    fn resolving_keeps_the_facts_the_decision_was_made_from() {
        let decision = ColorChoice::Never.resolve();
        assert!(!decision.enabled);
        assert_eq!(decision.choice, ColorChoice::Never);
        // Under the test harness stderr is not a terminal.
        assert!(!decision.is_terminal);
        assert!(!ColorChoice::Auto.resolve().enabled);
        assert!(ColorChoice::Always.resolve().enabled);
    }

    #[test]
    fn a_human_error_renders_byte_for_byte_the_message() {
        let record = Record::error("Broken", "spec/FR-001.md:12: missing criteria");
        assert_eq!(
            record.render_human(false),
            "spec/FR-001.md:12: missing criteria"
        );
        assert_eq!(
            record.render_human(true),
            "\x1b[31mspec/FR-001.md:12: missing criteria\x1b[0m"
        );
    }

    #[test]
    fn a_human_warning_is_prefixed_so_it_is_not_read_as_an_error() {
        assert_eq!(
            Record::warning("Advisory", "two matrices disagree").render_human(false),
            "warning: two matrices disagree"
        );
    }

    #[test]
    fn the_json_shape_carries_severity_kind_and_typed_fields() {
        let record = Record::error("Unbacked", "row has no test").with_fields(DiagnosticFields {
            path: Some("spec/matrix.md"),
            line: Some(41),
            ..DiagnosticFields::default()
        });
        assert_eq!(
            record.render_json().unwrap(),
            r#"{"kind":"Unbacked","message":"row has no test","severity":"error","path":"spec/matrix.md","line":41}"#
        );
    }

    // Rendering and emitting are separable, and this is the assertion that
    // keeps them so: a sink that is not a process stream gets the same bytes.
    #[test]
    fn a_record_writes_to_an_arbitrary_sink() {
        let out = Diagnostics::new(DiagnosticsFormat::Human, ColorDecision::default());
        let mut sink = Vec::new();
        Record::error("Broken", "one line")
            .write(out, &mut sink)
            .unwrap();
        assert_eq!(String::from_utf8(sink).unwrap(), "one line\n");
    }

    #[test]
    fn a_result_line_writes_to_an_arbitrary_sink_uncoloured() {
        let mut sink = Vec::new();
        write_result(&mut sink, "Coverage: 1238/2390 rows backed (51%)").unwrap();
        let written = String::from_utf8(sink).unwrap();
        assert_eq!(written, "Coverage: 1238/2390 rows backed (51%)\n");
        assert!(!written.contains('\x1b'));
    }
}
