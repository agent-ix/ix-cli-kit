// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Native credential-store round-trip, run only in the provisioned OS lane.

#![cfg(feature = "secrets")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test panics provide direct assertion context"
)]

use ix_cli_kit::secrets::{AppScope, DeleteStatus, Presence, SecretKey, SecretStore, SecretValue};

/// Trace: FR-014-AC-2, FR-014-AC-5
/// `tc_040`: the provisioned target adapter round-trips a credential through its OS store.
#[test]
#[ignore = "requires the native credential service provisioned by os-secrets-ci"]
fn tc_040_native_os_store_round_trips_and_deletes_a_credential() {
    let namespace = format!("ix-cli-kit-ci-{}", std::process::id());
    let app_scope = AppScope::try_from(namespace.as_str()).expect("test scope is valid");
    let key = SecretKey::try_from("roundtrip").expect("test key is valid");
    let value = SecretValue::new("os-store-roundtrip-sentinel");
    let store = SecretStore::system();

    store
        .delete(&app_scope, &key)
        .expect("native credential service must be available");
    store
        .set(&app_scope, &key, &value)
        .expect("native credential write must succeed");
    assert_eq!(
        store.status(&app_scope, &key),
        Ok(Presence::Present),
        "native credential should be present"
    );
    assert_eq!(
        store.get(&app_scope, &key),
        Ok(Some(value)),
        "native credential should round-trip"
    );
    assert_eq!(
        store.delete(&app_scope, &key),
        Ok(DeleteStatus::Deleted),
        "native credential should be removed"
    );
    assert_eq!(store.get(&app_scope, &key), Ok(None));

    let hyphenated_left =
        AppScope::try_from(format!("{namespace}-a-b").as_str()).expect("left scope is valid");
    let hyphenated_right =
        AppScope::try_from(format!("{namespace}-a").as_str()).expect("right scope is valid");
    let dotted_left =
        AppScope::try_from(format!("{namespace}.a").as_str()).expect("dotted scope is valid");
    let dotted_right = AppScope::try_from(namespace.as_str()).expect("base scope is valid");
    let windows_left = AppScope::try_from(format!("{namespace}.b").as_str())
        .expect("Windows collision scope is valid");
    let windows_right = AppScope::try_from("b").expect("Windows collision scope is valid");
    let identities = [
        (
            hyphenated_left,
            SecretKey::try_from("c").expect("key is valid"),
            SecretValue::new("hyphenated-left"),
        ),
        (
            hyphenated_right,
            SecretKey::try_from("b-c").expect("key is valid"),
            SecretValue::new("hyphenated-right"),
        ),
        (
            dotted_left,
            SecretKey::try_from("b").expect("key is valid"),
            SecretValue::new("dotted-left"),
        ),
        (
            dotted_right,
            SecretKey::try_from("a.b").expect("key is valid"),
            SecretValue::new("dotted-right"),
        ),
        (
            windows_left,
            SecretKey::try_from("c").expect("key is valid"),
            SecretValue::new("windows-collision-left"),
        ),
        (
            windows_right,
            SecretKey::try_from(format!("c.{namespace}").as_str())
                .expect("Windows collision key is valid"),
            SecretValue::new("windows-collision-right"),
        ),
        (
            AppScope::try_from("agent-ix/ix-projects").expect("projects scope is valid"),
            SecretKey::try_from("linear-api-key").expect("key is valid"),
            SecretValue::new("projects-scope"),
        ),
        (
            AppScope::try_from("agent-ix/ix-board").expect("board scope is valid"),
            SecretKey::try_from("linear-api-key").expect("key is valid"),
            SecretValue::new("board-scope"),
        ),
    ];
    for (scope, key, value) in &identities {
        store
            .delete(scope, key)
            .expect("native credential service must be available");
        store
            .set(scope, key, value)
            .expect("native credential write must succeed");
    }
    for (scope, key, value) in &identities {
        assert_eq!(store.get(scope, key), Ok(Some(value.clone())));
    }
    for (scope, key, _) in &identities {
        assert_eq!(store.delete(scope, key), Ok(DeleteStatus::Deleted));
    }
}
