---
id: FR-015
title: "Resolve secret overrides and report their source without revealing values"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-006"
    type: "implements"
---
# FR-015: Resolve secret overrides and report their source without revealing values

## Description

The crate SHALL resolve a credential in the order explicit value, named
environment variable, then OS credential store. The crate SHALL report the
selected source without exposing the value in metadata.

## Inputs

- Optional explicit secret value, optional consumer-named environment variable,
  and the app scope and key used by [FR-014](./FR-014-os-credential-store.md).

## Outputs

- An optional secret value with a secret-specific source of `explicit`, `env`,
  or `credential_store`; this does not change ordinary `config::Source` labels.
- An absent result when no source supplies a value.
- A typed invalid-environment error when the named variable is present but empty
  or cannot be represented as a secret string; no error contains its value.

## Behavior

- When an explicit secret is supplied, the crate SHALL select it without
  reading the environment or credential store.
- When no explicit secret is supplied and a valid environment override is
  present, the crate SHALL select it without reading the credential store.
- When neither override is present, the crate SHALL read the OS credential
  store and propagate its locked or unavailable error.
- If an environment override is invalid, then the crate SHALL return the typed
  invalid-environment error without falling through to a stored credential.
- The crate SHALL report source metadata independently of the secret value.
- The crate SHALL NOT serialize that value as settings or diagnostic output.
- The crate SHALL leave ordinary settings resolution under
  [FR-009](./FR-009-scalar-precedence.md).
- The crate SHALL leave consumer file locations under
  [FR-011](./FR-011-configuration-file-loading.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-015-AC-1 | With all sources present, explicit wins; without explicit, env wins; without either, store wins; each result reports the corresponding source. | Test |
| FR-015-AC-2 | An empty or non-text environment value produces the typed invalid-environment error and does not return the stored value. | Test |
| FR-015-AC-3 | With no override, an absent store key yields absent; locked and unavailable store errors retain their distinct variants. | Test |
| FR-015-AC-4 | The source metadata and error rendering contain no sentinel secret value for all three winning sources and each failure path. | Test |
| FR-015-AC-5 | Non-secret settings retain flag > env > file > default and their existing source spellings after secret resolution is added. | Test |

## Dependencies

- **Upstream**: [FR-014](./FR-014-os-credential-store.md),
  [FR-009](./FR-009-scalar-precedence.md).
- **Downstream**: [FR-016](./FR-016-shared-adoption.md).
