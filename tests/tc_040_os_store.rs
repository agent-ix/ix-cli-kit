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

use std::time::{SystemTime, UNIX_EPOCH};

use ix_cli_kit::secrets::{AppScope, DeleteStatus, Presence, SecretKey, SecretStore, SecretValue};

type NativeIdentity = (AppScope, SecretKey, SecretValue);

fn native_identities(namespace: &str) -> [NativeIdentity; 8] {
    [
        (
            AppScope::try_from(format!("{namespace}-a-b").as_str()).expect("left scope is valid"),
            SecretKey::try_from("c").expect("key is valid"),
            SecretValue::new("hyphenated-left"),
        ),
        (
            AppScope::try_from(format!("{namespace}-a").as_str()).expect("right scope is valid"),
            SecretKey::try_from("b-c").expect("key is valid"),
            SecretValue::new("hyphenated-right"),
        ),
        (
            AppScope::try_from(format!("{namespace}.a").as_str()).expect("dotted scope is valid"),
            SecretKey::try_from("b").expect("key is valid"),
            SecretValue::new("dotted-left"),
        ),
        (
            AppScope::try_from(namespace).expect("base scope is valid"),
            SecretKey::try_from("a.b").expect("key is valid"),
            SecretValue::new("dotted-right"),
        ),
        (
            AppScope::try_from(format!("{namespace}.b").as_str())
                .expect("Windows collision scope is valid"),
            SecretKey::try_from("c").expect("key is valid"),
            SecretValue::new("windows-collision-left"),
        ),
        (
            AppScope::try_from(namespace).expect("Windows collision scope is valid"),
            SecretKey::try_from("b.c").expect("Windows collision key is valid"),
            SecretValue::new("windows-collision-right"),
        ),
        (
            AppScope::try_from(format!("{namespace}/agent-ix/ix-projects").as_str())
                .expect("projects scope is valid"),
            SecretKey::try_from("linear-api-key").expect("key is valid"),
            SecretValue::new("projects-scope"),
        ),
        (
            AppScope::try_from(format!("{namespace}/agent-ix/ix-board").as_str())
                .expect("board scope is valid"),
            SecretKey::try_from("linear-api-key").expect("key is valid"),
            SecretValue::new("board-scope"),
        ),
    ]
}

fn assert_native_identity_isolation(store: &SecretStore, identities: &[NativeIdentity]) {
    for (scope, key, value) in identities {
        store
            .set(scope, key, value)
            .expect("native credential write must succeed");
    }
    for (scope, key, value) in identities {
        assert_eq!(store.get(scope, key), Ok(Some(value.clone())));
    }

    for (deleted_index, (deleted_scope, deleted_key, _)) in identities.iter().enumerate() {
        for (scope, key, value) in identities {
            store
                .set(scope, key, value)
                .expect("native credential write must succeed");
        }
        assert_eq!(
            store.delete(deleted_scope, deleted_key),
            Ok(DeleteStatus::Deleted),
            "selected credential should be removed"
        );
        assert_eq!(
            store.get(deleted_scope, deleted_key),
            Ok(None),
            "deleted credential should be absent"
        );
        for (other_index, (scope, key, value)) in identities.iter().enumerate() {
            if other_index != deleted_index {
                assert_eq!(
                    store.get(scope, key),
                    Ok(Some(value.clone())),
                    "deleting one identity must preserve every other credential"
                );
            }
        }
    }

    for (scope, key, value) in identities {
        store
            .set(scope, key, value)
            .expect("native credential write must succeed before cleanup");
    }
    for (scope, key, _) in identities {
        assert_eq!(store.delete(scope, key), Ok(DeleteStatus::Deleted));
    }
}

/// Trace: FR-014-AC-2, FR-014-AC-5
/// `tc_040`: the provisioned target adapter round-trips a credential through its OS store.
#[test]
#[ignore = "requires the native credential service provisioned by os-secrets-ci"]
fn tc_040_native_os_store_round_trips_and_deletes_a_credential() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let namespace = format!("ix-cli-kit-ci-{}-{nonce}", std::process::id());
    let app_scope = AppScope::try_from(namespace.as_str()).expect("test scope is valid");
    let key = SecretKey::try_from("roundtrip").expect("test key is valid");
    let value = SecretValue::new("os-store-roundtrip-sentinel");
    let store = SecretStore::system();

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

    assert_native_identity_isolation(&store, &native_identities(&namespace));
}
