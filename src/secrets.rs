// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Opt-in access to app-scoped operating-system credentials.
//!
//! This module owns secret-store identity and secret-source precedence. It does
//! not choose or write configuration paths, and it never falls back to a file.
//! Enable the crate's `secrets` feature to compile this module and its one
//! target-specific OS credential adapter.

use std::ffi::OsString;
use std::fmt;

use thiserror::Error;

const MAX_IDENTIFIER_BYTES: usize = 255;

/// An application namespace for credentials, validated before backend access.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AppScope(Box<str>);

impl AppScope {
    /// Returns this validated scope's stable identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AppScope {
    type Error = SecretError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_scope(value).map_err(|()| SecretError::InvalidScope)?;
        Ok(Self(value.into()))
    }
}

impl fmt::Debug for AppScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AppScope")
            .field(&self.as_str())
            .finish()
    }
}

/// A credential name within an application scope.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SecretKey(Box<str>);

impl SecretKey {
    /// Returns this validated key's stable identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for SecretKey {
    type Error = SecretError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_identifier(value).map_err(|()| SecretError::InvalidKey)?;
        Ok(Self(value.into()))
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SecretKey")
            .field(&self.as_str())
            .finish()
    }
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    let bytes = value.as_bytes();
    if bytes.len() > MAX_IDENTIFIER_BYTES || !is_identifier_component(bytes) {
        return Err(());
    }
    Ok(())
}

fn validate_scope(value: &str) -> Result<(), ()> {
    if value.len() > MAX_IDENTIFIER_BYTES
        || !value
            .split('/')
            .all(|component| is_identifier_component(component.as_bytes()))
    {
        return Err(());
    }
    Ok(())
}

fn is_identifier_component(bytes: &[u8]) -> bool {
    let Some(first) = bytes.first() else {
        return false;
    };
    (first.is_ascii_lowercase() || first.is_ascii_digit())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(byte))
}

/// A secret value that masks itself in debug output and has no serialization or
/// display implementation.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    /// Creates a secret from an owned or borrowed UTF-8 string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Reveals the value for an explicitly authenticated downstream operation.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([REDACTED])")
    }
}

/// The source that supplied a resolved secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretSource {
    /// An explicit value passed by the consumer.
    Explicit,
    /// A named environment variable.
    Environment,
    /// The operating-system credential store.
    CredentialStore,
}

impl SecretSource {
    /// Returns the stable source label used in diagnostics and metadata.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Environment => "env",
            Self::CredentialStore => "credential_store",
        }
    }
}

/// A secret and its non-sensitive source metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSecret {
    /// The selected secret. Its debug representation is always redacted.
    pub value: SecretValue,
    /// The source selected by secret precedence.
    pub source: SecretSource,
}

/// Whether a credential exists, without returning its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// A credential exists.
    Present,
    /// No credential exists for this identity.
    Absent,
}

/// The result of deleting a credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteStatus {
    /// A credential was removed.
    Deleted,
    /// No credential existed, so there was nothing to remove.
    AlreadyAbsent,
}

/// Typed secret-operation failures. Variants and formatting never contain
/// values supplied by the caller or returned by the credential backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SecretError {
    /// The app scope is empty, malformed, or too long for portable storage.
    #[error("invalid application scope")]
    InvalidScope,
    /// The secret key is empty, malformed, or too long for portable storage.
    #[error("invalid secret key")]
    InvalidKey,
    /// The named environment variable is empty or is not valid UTF-8.
    #[error("invalid secret environment value")]
    InvalidEnvironment,
    /// The operating-system credential store is locked or denied access.
    #[error("operating-system credential store is locked or access was denied")]
    Locked,
    /// No supported operating-system credential store is available.
    #[error("operating-system credential store is unavailable")]
    Unavailable,
    /// The credential backend failed for another reason.
    #[error("operating-system credential operation failed")]
    BackendFailure,
}

/// The storage operations required by [`SecretStore`].
///
/// This trait is also the deterministic seam for consumers that need to
/// exercise resolution without accessing a user's real credential store.
pub trait CredentialBackend {
    /// Gets a secret, returning `None` when its identity is absent.
    ///
    /// # Errors
    ///
    /// Returns a typed [`SecretError`] when the store cannot be accessed.
    fn get(&self, scope: &AppScope, key: &SecretKey) -> Result<Option<SecretValue>, SecretError>;

    /// Stores or replaces a secret.
    ///
    /// # Errors
    ///
    /// Returns a typed [`SecretError`] when the store cannot be accessed.
    fn set(
        &self,
        scope: &AppScope,
        key: &SecretKey,
        value: &SecretValue,
    ) -> Result<(), SecretError>;

    /// Deletes a secret and reports whether an item existed.
    ///
    /// # Errors
    ///
    /// Returns a typed [`SecretError`] when the store cannot be accessed.
    fn delete(&self, scope: &AppScope, key: &SecretKey) -> Result<DeleteStatus, SecretError>;

    /// Checks whether an item exists without returning its value.
    ///
    /// # Errors
    ///
    /// Returns a typed [`SecretError`] when the store cannot be accessed.
    fn status(&self, scope: &AppScope, key: &SecretKey) -> Result<Presence, SecretError>;
}

/// The supported system credential backend, using the native OS store.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsCredentialBackend;

impl CredentialBackend for OsCredentialBackend {
    fn get(&self, scope: &AppScope, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            let entry = os_entry(scope, key)?;
            match entry.get_password() {
                Ok(value) => Ok(Some(SecretValue::new(value))),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => Err(map_keyring_error(&error)),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let _ = (scope, key);
            Err(SecretError::Unavailable)
        }
    }

    fn set(
        &self,
        scope: &AppScope,
        key: &SecretKey,
        value: &SecretValue,
    ) -> Result<(), SecretError> {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            os_entry(scope, key)?
                .set_password(value.expose_secret())
                .map_err(|error| map_keyring_error(&error))
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let _ = (scope, key, value);
            Err(SecretError::Unavailable)
        }
    }

    fn delete(&self, scope: &AppScope, key: &SecretKey) -> Result<DeleteStatus, SecretError> {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            match os_entry(scope, key)?.delete_credential() {
                Ok(()) => Ok(DeleteStatus::Deleted),
                Err(keyring::Error::NoEntry) => Ok(DeleteStatus::AlreadyAbsent),
                Err(error) => Err(map_keyring_error(&error)),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let _ = (scope, key);
            Err(SecretError::Unavailable)
        }
    }

    fn status(&self, scope: &AppScope, key: &SecretKey) -> Result<Presence, SecretError> {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            match os_entry(scope, key)?.get_password() {
                Ok(_) => Ok(Presence::Present),
                Err(keyring::Error::NoEntry) => Ok(Presence::Absent),
                Err(error) => Err(map_keyring_error(&error)),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let _ = (scope, key);
            Err(SecretError::Unavailable)
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn os_entry(scope: &AppScope, key: &SecretKey) -> Result<keyring::Entry, SecretError> {
    #[cfg(target_os = "windows")]
    {
        let target = format!(
            "{}:{}:{}",
            scope.as_str().len(),
            scope.as_str(),
            key.as_str()
        );
        return keyring::Entry::new_with_target(&target, scope.as_str(), key.as_str())
            .map_err(|error| map_keyring_error(&error));
    }
    #[cfg(not(target_os = "windows"))]
    keyring::Entry::new(scope.as_str(), key.as_str()).map_err(|error| map_keyring_error(&error))
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn map_keyring_error(error: &keyring::Error) -> SecretError {
    match error {
        keyring::Error::NoStorageAccess(_) => SecretError::Locked,
        keyring::Error::PlatformFailure(_) => SecretError::Unavailable,
        _ => SecretError::BackendFailure,
    }
}

/// App-scoped credential operations and source resolution.
pub struct SecretStore<B = OsCredentialBackend> {
    backend: B,
}

impl SecretStore<OsCredentialBackend> {
    /// Creates a store backed by this platform's native credential service.
    #[must_use]
    pub const fn system() -> Self {
        Self {
            backend: OsCredentialBackend,
        }
    }
}

impl<B: CredentialBackend> SecretStore<B> {
    /// Creates a store with the supplied credential backend.
    #[must_use]
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Retrieves the value for a validated app/key identity.
    ///
    /// # Errors
    ///
    /// Returns the backend's typed access failure.
    pub fn get(
        &self,
        scope: &AppScope,
        key: &SecretKey,
    ) -> Result<Option<SecretValue>, SecretError> {
        self.backend.get(scope, key)
    }

    /// Stores or replaces a value for a validated app/key identity.
    ///
    /// # Errors
    ///
    /// Returns the backend's typed access failure.
    pub fn set(
        &self,
        scope: &AppScope,
        key: &SecretKey,
        value: &SecretValue,
    ) -> Result<(), SecretError> {
        self.backend.set(scope, key, value)
    }

    /// Deletes a value, treating absence as a successful outcome.
    ///
    /// # Errors
    ///
    /// Returns the backend's typed access failure.
    pub fn delete(&self, scope: &AppScope, key: &SecretKey) -> Result<DeleteStatus, SecretError> {
        self.backend.delete(scope, key)
    }

    /// Checks whether a value exists without returning it.
    ///
    /// # Errors
    ///
    /// Returns the backend's typed access failure.
    pub fn status(&self, scope: &AppScope, key: &SecretKey) -> Result<Presence, SecretError> {
        self.backend.status(scope, key)
    }

    /// Resolves explicit input, an environment override, then the OS store.
    ///
    /// If `explicit` is present the environment is not read. An empty or
    /// non-UTF-8 environment value is an error and never falls through to the
    /// store. Passing `None` for `environment_variable` skips that layer.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError::InvalidEnvironment`] for an empty or non-UTF-8
    /// environment value and propagates typed store failures.
    pub fn resolve(
        &self,
        explicit: Option<SecretValue>,
        environment_variable: Option<&str>,
        scope: &AppScope,
        key: &SecretKey,
    ) -> Result<Option<ResolvedSecret>, SecretError> {
        if let Some(value) = explicit {
            return Ok(Some(ResolvedSecret {
                value,
                source: SecretSource::Explicit,
            }));
        }
        if let Some(name) = environment_variable {
            validate_environment_name(name)?;
        }
        let environment =
            environment_variable.and_then(|name| std::env::var_os(name).map(|value| (name, value)));
        self.resolve_from(None, environment, scope, key)
    }

    /// Resolves using an already-read environment value.
    ///
    /// This variant lets consumers and tests provide a controlled environment
    /// snapshot. Its behavior and precedence are identical to [`Self::resolve`].
    ///
    /// # Errors
    ///
    /// Returns [`SecretError::InvalidEnvironment`] for an empty or non-UTF-8
    /// value and propagates typed store failures.
    pub fn resolve_from(
        &self,
        explicit: Option<SecretValue>,
        environment: Option<(&str, OsString)>,
        scope: &AppScope,
        key: &SecretKey,
    ) -> Result<Option<ResolvedSecret>, SecretError> {
        if let Some(value) = explicit {
            return Ok(Some(ResolvedSecret {
                value,
                source: SecretSource::Explicit,
            }));
        }
        if let Some((name, raw)) = environment {
            validate_environment_name(name)?;
            let value = raw
                .into_string()
                .map_err(|_| SecretError::InvalidEnvironment)?;
            if value.is_empty() {
                return Err(SecretError::InvalidEnvironment);
            }
            return Ok(Some(ResolvedSecret {
                value: SecretValue::new(value),
                source: SecretSource::Environment,
            }));
        }
        self.get(scope, key).map(|value| {
            value.map(|value| ResolvedSecret {
                value,
                source: SecretSource::CredentialStore,
            })
        })
    }
}

fn validate_environment_name(name: &str) -> Result<(), SecretError> {
    if name.is_empty()
        || name
            .chars()
            .any(|character| character == '=' || character == '\0')
    {
        return Err(SecretError::InvalidEnvironment);
    }
    Ok(())
}
