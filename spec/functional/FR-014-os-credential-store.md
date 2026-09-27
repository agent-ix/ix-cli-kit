---
id: FR-014
title: "Store app-scoped credentials in the OS credential store"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-006"
    type: "implements"
---
# FR-014: Store app-scoped credentials in the OS credential store

## Description

The crate SHALL expose app-scoped get, set, delete and status operations for
secret values through the operating system's credential store.

## Inputs

- A consumer-supplied application scope and secret key, represented as distinct
  validated types.
- A secret value for set.

## Outputs

- Get: a secret value or an absent result.
- Set: success or a typed failure.
- Delete: removed or already absent, without treating absence as a failure.
- Status: present or absent, without returning the value.
- A typed error for locked, unavailable, invalid scope/key, and other backend
  failures. Error text contains no secret value.

## Behavior

- When running on macOS, the crate SHALL use Keychain for persistence.
- When running on Linux with Secret Service available, the crate SHALL use Secret Service
  for persistence.
- When running on Windows, the crate SHALL use Credential Manager for persistence.
- When a supported OS store is locked, the crate SHALL return a typed locked
  error for operations requiring access.
- When no supported OS store is available, the crate SHALL return a typed
  unavailable error for operations requiring access.
- The crate SHALL isolate a key in one application scope from the same key in
  another scope.
- The crate SHALL use the OS store as its only persistence backend for secret
  values; it SHALL NOT write a plaintext or encrypted file fallback.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-014-AC-1 | Set, get, status, and delete round-trip a value in one app scope; get after delete is absent and a second delete reports already absent. | Test |
| FR-014-AC-2 | Two app scopes using the same key do not read, overwrite, or delete each other's values. | Test |
| FR-014-AC-3 | A locked backend yields the locked variant and an unavailable backend yields the unavailable variant; neither operation creates a fallback file. | Test |
| FR-014-AC-4 | Status reports presence without returning a value, and invalid scope or key is rejected before a backend call. | Test |
| FR-014-AC-5 | Platform integration reaches Keychain on macOS, Secret Service on Linux, and Credential Manager on Windows where each service is available. A platform without one reports unavailable. | Demonstration |

## Dependencies

- **Upstream**: [US-006](../usecase/US-006-use-local-credentials.md).
- **Downstream**: [FR-015](./FR-015-secret-source-precedence.md) resolves an
  environment override against this store.
