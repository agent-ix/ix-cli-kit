---
id: FR-016
title: "Adopt the shared credential contract from an app and a Rust CLI"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-006"
    type: "implements"
---
# FR-016: Adopt the shared credential contract from an app and a Rust CLI

## Description

`ix-projects` and one Rust CLI SHALL consume the shared credential implementation
through pinned-revision dependency imports, without copying its source or backend
artifacts into either consumer.

## Behavior

- `ix-projects` SHALL use the shared app-scoped credential and source-reporting
  API for its `LINEAR_API_KEY` flow, with the existing environment variable as
  an override of the OS store.
- `ix-projects` SHALL add no credential settings file; its unrelated relay,
  cache, and drift configuration paths remain consumer-owned.
- One Rust CLI SHALL use the same API for a real credential flow and retain its
  own argument parser and settings path.
- Both consumers SHALL pin the authoritative crate by git revision and enable
  its `secrets` feature.
- Both consumers SHALL import its public API instead of maintaining another
  backend implementation.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-016-AC-1 | The `ix-projects` manifest pins `ix-cli-kit` by revision with `secrets` enabled; its `LINEAR_API_KEY` flow calls the shared API with env override, and adds no credential file or change to relay, cache, or drift paths. | Inspection |
| FR-016-AC-2 | A Rust CLI manifest pins the same authoritative crate by revision with `secrets` enabled, and a credential flow calls its public API without changing argv parsing or settings paths. | Inspection |
| FR-016-AC-3 | A focused integration test in each consumer demonstrates source reporting and locked/unavailable handling without printing or persisting a secret value. | Test |
| FR-016-AC-4 | Neither consumer contains a copied credential backend, binary, schema, or fixture from another repository. | Inspection |

## Dependencies

- **Upstream**: [FR-014](./FR-014-os-credential-store.md) and
  [FR-015](./FR-015-secret-source-precedence.md).
- **Downstream**: `ix-projects` and one Rust CLI adoption PR after this shared
  contract is implemented.
