// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Canonical JSON on the way out, and one encoder for `--json` / `--pretty`.
//!
//! Moved from three independent implementations the 2026-09-12 survey found:
//! `quire-corpus/src/main.rs` (`print_json` / `sort_json`),
//! `quoin/rust/crates/quoin-core/src/protocol.rs` (`canonical_json`) and
//! `quire-cli/src/io.rs` (`encode_json`).
//!
//! # Why the sort is explicit and recursive
//!
//! `serde_json::Map` is a `BTreeMap` **only while the `preserve_order` feature
//! is off**. quoin-core's canonicaliser leans on that and documents that
//! enabling the feature silently breaks `quoin-difftest`'s byte comparison.
//! quire-corpus enables `preserve_order` deliberately, and therefore had to
//! sort by hand.
//!
//! Cargo unifies features across a dependency graph: the moment ONE crate in a
//! build turns `preserve_order` on, every crate in that build gets an
//! insertion-ordered map, including this one. A canonicaliser that relies on
//! the map type is therefore a canonicaliser that depends on who else is in the
//! build. [`canonical`] sorts explicitly at every depth, so it produces the
//! same bytes under either feature set. That is not belt-and-braces; it is the
//! only form of the function that is true of both consumers this crate already
//! has.

use serde::Serialize;
use serde_json::Value;

/// A JSON encoding failure, with the value's role named.
///
/// The context string is the caller's word for what it was serialising
/// (`"corpus report"`, `"diagnostics"`), because `serde_json`'s own message
/// never says which of a command's several payloads failed.
#[derive(Debug, thiserror::Error)]
#[error("{context}: {source}")]
pub struct JsonError {
    /// What the caller was encoding.
    pub context: String,
    /// The underlying `serde_json` failure.
    #[source]
    pub source: serde_json::Error,
}

impl JsonError {
    fn new(context: &str, source: serde_json::Error) -> Self {
        Self {
            context: context.to_owned(),
            source,
        }
    }
}

/// Route a value through [`Value`] and sort every object key at every depth.
///
/// # Errors
///
/// [`JsonError`] if the value cannot be represented as JSON — a non-string map
/// key or a non-finite float.
pub fn canonical<T: Serialize + ?Sized>(value: &T) -> Result<Value, JsonError> {
    let mut as_value =
        serde_json::to_value(value).map_err(|source| JsonError::new("canonical json", source))?;
    sort_in_place(&mut as_value);
    Ok(as_value)
}

/// Sort every object key in place, recursively, including inside arrays.
pub fn sort_in_place(value: &mut Value) {
    match value {
        Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (_, value) in &mut entries {
                sort_in_place(value);
            }
            object.extend(entries);
        }
        Value::Array(values) => values.iter_mut().for_each(sort_in_place),
        _ => {}
    }
}

/// Encode a value as JSON: compact by default, indented when `pretty`.
///
/// Compact is the default because the compact form is the one a caller pipes
/// into another tool, and `--pretty` is the human affordance layered over it.
/// Object keys are **not** reordered here; call [`canonical`] first when the
/// bytes are going to be compared or digested.
///
/// # Errors
///
/// [`JsonError`] if the value cannot be represented as JSON.
pub fn encode<T: Serialize + ?Sized>(value: &T, pretty: bool) -> Result<String, JsonError> {
    let result = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    result.map_err(|source| JsonError::new("json encode", source))
}

/// Canonicalise and encode in one step — the form a report writer wants.
///
/// # Errors
///
/// [`JsonError`] if the value cannot be represented as JSON.
pub fn encode_canonical<T: Serialize + ?Sized>(
    value: &T,
    pretty: bool,
) -> Result<String, JsonError> {
    encode(&canonical(value)?, pretty)
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
    fn canonical_sorts_keys_at_every_depth() {
        let value = serde_json::json!({ "z": 1, "a": { "y": 2, "b": 3 } });
        assert_eq!(
            encode(&canonical(&value).unwrap(), false).unwrap(),
            r#"{"a":{"b":3,"y":2},"z":1}"#
        );
    }

    // quire-corpus's `sort_json` recursed into arrays; quoin-core got the same
    // behaviour free from `BTreeMap`. Under `preserve_order` only the explicit
    // recursion is doing the work, so it is asserted rather than assumed.
    #[test]
    fn canonical_sorts_objects_nested_inside_arrays() {
        let value = serde_json::json!({ "rows": [{ "b": 1, "a": 2 }, { "d": 3, "c": 4 }] });
        assert_eq!(
            encode(&canonical(&value).unwrap(), false).unwrap(),
            r#"{"rows":[{"a":2,"b":1},{"c":4,"d":3}]}"#
        );
    }

    // Assert the literal bytes, not a re-derivation: re-canonicalising the
    // expectation would agree with the code no matter what the code did.
    #[test]
    fn canonical_output_has_no_insignificant_whitespace() {
        let value = serde_json::json!({ "b": [1, 2], "a": "x" });
        let encoded = encode(&canonical(&value).unwrap(), false).unwrap();
        assert_eq!(encoded, r#"{"a":"x","b":[1,2]}"#);
        assert!(!encoded.contains(' '));
        assert!(!encoded.contains('\n'));
    }

    #[test]
    fn encode_is_compact_by_default_and_indented_when_asked() {
        let value = serde_json::json!({ "a": 1, "b": 2 });
        assert!(!encode(&value, false).unwrap().contains('\n'));
        assert!(encode(&value, true).unwrap().contains('\n'));
    }

    #[test]
    fn encode_canonical_sorts_and_encodes_in_one_step() {
        let value = serde_json::json!({ "z": 1, "a": 2 });
        assert_eq!(encode_canonical(&value, false).unwrap(), r#"{"a":2,"z":1}"#);
    }

    #[test]
    fn a_value_that_cannot_be_json_names_what_was_being_encoded() {
        // A map whose key is not a string cannot be JSON at all — the failure
        // must name what was being encoded, not just what serde_json said.
        let mut bad = std::collections::BTreeMap::new();
        bad.insert((1, 2), 3);
        let error = canonical(&bad).unwrap_err();
        assert_eq!(error.context, "canonical json");
        assert!(error.to_string().starts_with("canonical json: "));
    }
}
