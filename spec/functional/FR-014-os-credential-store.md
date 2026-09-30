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
  validated types. An application scope is at most 255 bytes and contains one
  or more slash-separated, nonempty lowercase ASCII components, each matching
  `[a-z0-9][a-z0-9._-]*`. A secret key is at most 255 bytes and matches that
  component grammar without slash separators. Other input is rejected before
  backend access.
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
- The crate SHALL keep the credential API behind an off-by-default `secrets`
  feature with target-gated OS backend dependencies, as constrained by
  [NFR-003](../non-functional/NFR-003-dependency-floor.md).
- When a supported OS store is locked, the crate SHALL return a typed locked
  error for operations requiring access.
- When no supported OS store is available, the crate SHALL return a typed
  unavailable error for operations requiring access.
- The crate SHALL map every distinct accepted `(app_scope, key)` pair to a
  distinct OS credential item on each backend.
- The crate SHALL reject identifiers that the selected backend cannot represent
  without truncation or identity normalization.
- The crate SHALL use the OS store as its only persistence backend for secret
  values; it SHALL NOT write a plaintext or encrypted file fallback.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-014-AC-1 | Set, get, status, and delete round-trip a value in one app scope; get after delete is absent and a second delete reports already absent. | Test |
| FR-014-AC-2 | Distinct accepted pairs, including `(a-b, c)` versus `(a, b-c)`, `(a.b, c)` versus `(a, b.c)`, and `(agent-ix/ix-projects, linear-api-key)` versus `(agent-ix/ix-board, linear-api-key)`, do not read, overwrite, or delete each other's values on any backend. | Test |
| FR-014-AC-3 | A locked backend yields the locked variant and an unavailable backend yields the unavailable variant; neither operation creates a fallback file. | Test |
| FR-014-AC-4 | Status reports presence without returning a value; empty, uppercase, Unicode, colon-bearing, empty-component scope (such as `a//b`), slash-bearing key, or unrepresentable scope/key input is rejected before a backend call. | Test |
| FR-014-AC-5 | CI runs a get/set/status/delete integration test against Keychain on macOS, a provisioned Secret Service session on Linux, and Credential Manager on Windows. Each job fails if its expected adapter is unavailable or the round-trip fails. | Test |
| FR-014-AC-6 | A target without a supported backend reports the unavailable variant; the default-feature dependency graph contains no OS credential backend. | Test |

## Dependencies

- **Upstream**: [US-006](../usecase/US-006-use-local-credentials.md).
- **Downstream**: [FR-015](./FR-015-secret-source-precedence.md) resolves an
  environment override against this store.
