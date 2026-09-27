// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Contract tests for the optional shared secrets API.

#![cfg(feature = "secrets")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test panics provide direct assertion context"
)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::process::Command;
use std::sync::{Arc, Mutex};

use ix_cli_kit::secrets::{
    AppScope, CredentialBackend, DeleteStatus, Presence, SecretError, SecretKey, SecretSource,
    SecretStore, SecretValue,
};

const SENTINEL: &str = "never-print-this-secret-83f2";

#[derive(Clone, Default)]
struct MemoryBackend {
    entries: Arc<Mutex<HashMap<(String, String), SecretValue>>>,
    failure: Option<SecretError>,
}

impl CredentialBackend for MemoryBackend {
    fn get(&self, scope: &AppScope, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        if let Some(error) = &self.failure {
            return Err(*error);
        }
        Ok(self
            .entries
            .lock()
            .map_err(|_| SecretError::BackendFailure)?
            .get(&(scope.as_str().to_owned(), key.as_str().to_owned()))
            .cloned())
    }

    fn set(
        &self,
        scope: &AppScope,
        key: &SecretKey,
        value: &SecretValue,
    ) -> Result<(), SecretError> {
        if let Some(error) = &self.failure {
            return Err(*error);
        }
        self.entries
            .lock()
            .map_err(|_| SecretError::BackendFailure)?
            .insert(
                (scope.as_str().to_owned(), key.as_str().to_owned()),
                value.clone(),
            );
        Ok(())
    }

    fn delete(&self, scope: &AppScope, key: &SecretKey) -> Result<DeleteStatus, SecretError> {
        if let Some(error) = &self.failure {
            return Err(*error);
        }
        Ok(
            if self
                .entries
                .lock()
                .map_err(|_| SecretError::BackendFailure)?
                .remove(&(scope.as_str().to_owned(), key.as_str().to_owned()))
                .is_some()
            {
                DeleteStatus::Deleted
            } else {
                DeleteStatus::AlreadyAbsent
            },
        )
    }

    fn status(&self, scope: &AppScope, key: &SecretKey) -> Result<Presence, SecretError> {
        if let Some(error) = &self.failure {
            return Err(*error);
        }
        Ok(
            if self
                .entries
                .lock()
                .map_err(|_| SecretError::BackendFailure)?
                .contains_key(&(scope.as_str().to_owned(), key.as_str().to_owned()))
            {
                Presence::Present
            } else {
                Presence::Absent
            },
        )
    }
}

fn scope(value: &str) -> AppScope {
    AppScope::try_from(value).expect("valid test scope")
}

fn key(value: &str) -> SecretKey {
    SecretKey::try_from(value).expect("valid test key")
}

fn secret(value: &str) -> SecretValue {
    SecretValue::new(value)
}

/// Trace: FR-014-AC-1, FR-014-AC-2, FR-014-AC-4, NFR-004
/// `tc_030`: operations round-trip and isolate every validated scope/key pair.
#[test]
fn tc_030_store_operations_are_app_scoped_and_secret_safe() {
    let store = SecretStore::new(MemoryBackend::default());
    let first_scope = scope("a-b");
    let first_key = key("c");
    let second_scope = scope("a");
    let second_key = key("b-c");
    let dotted_scope = scope("a.b");
    let dotted_key = key("c");
    let other_dotted_scope = scope("a");
    let other_dotted_key = key("b.c");
    let value = secret(SENTINEL);

    store.set(&first_scope, &first_key, &value).expect("set");
    store
        .set(&second_scope, &second_key, &secret("other"))
        .expect("set other pair");
    store
        .set(&dotted_scope, &dotted_key, &secret("dotted one"))
        .expect("set dotted pair");
    store
        .set(
            &other_dotted_scope,
            &other_dotted_key,
            &secret("dotted two"),
        )
        .expect("set other dotted pair");

    assert_eq!(
        store.get(&first_scope, &first_key).expect("get"),
        Some(value.clone())
    );
    assert_eq!(
        store.status(&first_scope, &first_key).expect("status"),
        Presence::Present
    );
    assert_eq!(
        store
            .get(&second_scope, &second_key)
            .expect("other get")
            .unwrap()
            .expose_secret(),
        "other"
    );
    assert_eq!(
        store
            .get(&dotted_scope, &dotted_key)
            .expect("dotted get")
            .unwrap()
            .expose_secret(),
        "dotted one"
    );
    assert_eq!(
        store
            .get(&other_dotted_scope, &other_dotted_key)
            .expect("other dotted get")
            .unwrap()
            .expose_secret(),
        "dotted two"
    );
    assert_eq!(
        store.delete(&first_scope, &first_key).expect("delete"),
        DeleteStatus::Deleted
    );
    assert_eq!(
        store.get(&first_scope, &first_key).expect("get absent"),
        None
    );
    assert_eq!(
        store
            .delete(&first_scope, &first_key)
            .expect("delete absent"),
        DeleteStatus::AlreadyAbsent
    );

    let rendered = format!("{value:?}");
    assert!(!rendered.contains(SENTINEL));
    assert_eq!(
        store
            .status(&first_scope, &first_key)
            .expect("status absent"),
        Presence::Absent
    );
}

/// Trace: FR-014-AC-4
/// `tc_031`: identifiers reject malformed and normalization-prone input.
#[test]
fn tc_031_identifiers_are_validated_before_backend_access() {
    for invalid in ["", "Upper", "with:colon", "unicode-ø", " leading"] {
        assert_eq!(
            AppScope::try_from(invalid),
            Err(SecretError::InvalidScope),
            "scope {invalid:?}"
        );
        assert_eq!(
            SecretKey::try_from(invalid),
            Err(SecretError::InvalidKey),
            "key {invalid:?}"
        );
    }
    assert!(AppScope::try_from("a").is_ok());
    assert!(SecretKey::try_from("a0._-").is_ok());
    let oversized = format!("a{}", "b".repeat(255));
    assert_eq!(
        AppScope::try_from(oversized.as_str()),
        Err(SecretError::InvalidScope)
    );
    assert_eq!(
        SecretKey::try_from(oversized.as_str()),
        Err(SecretError::InvalidKey)
    );
}

/// Trace: FR-014-AC-3, FR-015-AC-3, NFR-004
/// `tc_032`: locked and unavailable backend failures stay typed and redacted.
#[test]
fn tc_032_backend_failures_are_typed_and_redacted() {
    for error in [SecretError::Locked, SecretError::Unavailable] {
        let store = SecretStore::new(MemoryBackend {
            entries: Arc::default(),
            failure: Some(error),
        });
        let result = store.set(&scope("sample"), &key("token"), &secret(SENTINEL));
        assert_eq!(result, Err(error));
        assert_eq!(store.get(&scope("sample"), &key("token")), Err(error));
        assert_eq!(store.status(&scope("sample"), &key("token")), Err(error));
        assert_eq!(store.delete(&scope("sample"), &key("token")), Err(error));
        assert!(!format!("{error}").contains(SENTINEL));
        assert!(!format!("{error:?}").contains(SENTINEL));
    }
}

/// Trace: FR-015-AC-1, FR-015-AC-2, FR-015-AC-3, FR-015-AC-4, FR-015-AC-5
/// `tc_033`: secret overrides have their own precedence and source labels.
#[test]
fn tc_033_secret_precedence_reports_only_the_source() {
    let store = SecretStore::new(MemoryBackend::default());
    let scope = scope("sample");
    let token_key = key("token");
    store
        .set(&scope, &token_key, &secret("stored"))
        .expect("seed store");

    let explicit = store
        .resolve_from(
            Some(secret(SENTINEL)),
            Some(("IX_TEST_SECRET", OsString::from("environment"))),
            &scope,
            &token_key,
        )
        .expect("explicit wins")
        .expect("value present");
    assert_eq!(explicit.source, SecretSource::Explicit);
    assert_eq!(explicit.value.expose_secret(), SENTINEL);

    let environment = store
        .resolve_from(
            None,
            Some(("IX_TEST_SECRET", OsString::from(SENTINEL))),
            &scope,
            &token_key,
        )
        .expect("environment wins")
        .expect("value present");
    assert_eq!(environment.source, SecretSource::Environment);
    assert_eq!(environment.value.expose_secret(), SENTINEL);

    let stored = store
        .resolve_from(None, None, &scope, &token_key)
        .expect("stored wins")
        .expect("stored value present");
    assert_eq!(stored.source, SecretSource::CredentialStore);
    assert_eq!(stored.value.expose_secret(), "stored");
    for source in [explicit.source, environment.source, stored.source] {
        assert!(!format!("{source:?}").contains(SENTINEL));
        assert!(!source.as_str().contains(SENTINEL));
    }

    let absent = store
        .resolve_from(None, None, &scope, &key("missing"))
        .expect("missing store value is absent");
    assert!(absent.is_none());

    for error in [SecretError::Locked, SecretError::Unavailable] {
        let failing_store = SecretStore::new(MemoryBackend {
            entries: Arc::default(),
            failure: Some(error),
        });
        assert_eq!(
            failing_store.resolve_from(None, None, &scope, &token_key),
            Err(error)
        );
    }

    assert!(!format!("{explicit:?}").contains(SENTINEL));
    assert!(!format!("{environment:?}").contains(SENTINEL));
    assert!(!format!("{stored:?}").contains("stored"));

    let invalid_values = [OsString::new()];
    for invalid in invalid_values {
        let result =
            store.resolve_from(None, Some(("IX_TEST_SECRET", invalid)), &scope, &token_key);
        assert_eq!(result, Err(SecretError::InvalidEnvironment));
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        let result = store.resolve_from(
            None,
            Some(("IX_TEST_SECRET", OsString::from_vec(vec![0xff]))),
            &scope,
            &token_key,
        );
        assert_eq!(result, Err(SecretError::InvalidEnvironment));
    }
}

/// Trace: FR-015-AC-1, FR-015-AC-4
/// `tc_034`: the named environment variable is read without process-global mutation.
#[test]
fn tc_034_named_environment_variable_is_resolved() {
    const CHILD_MARKER: &str = "IX_CLI_KIT_TC_034_CHILD";
    const ENV_NAME: &str = "IX_CLI_KIT_TC_034_SECRET";

    if std::env::var_os(CHILD_MARKER).is_some() {
        let store = SecretStore::new(MemoryBackend::default());
        let resolved = store
            .resolve(None, Some(ENV_NAME), &scope("sample"), &key("token"))
            .expect("named environment value is valid")
            .expect("environment value is present");
        assert_eq!(resolved.source, SecretSource::Environment);
        assert!(
            resolved.value.expose_secret() == SENTINEL,
            "environment value should be selected"
        );
    } else {
        let output = Command::new(std::env::current_exe().expect("test executable path"))
            .args(["--exact", "tc_034_named_environment_variable_is_resolved"])
            .env(CHILD_MARKER, "1")
            .env(ENV_NAME, SENTINEL)
            .output()
            .expect("spawn isolated environment test");
        assert!(output.status.success(), "isolated environment test failed");
    }
}
