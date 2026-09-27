---
id: US-006
title: "Use a local credential from an application or CLI"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-003"
    type: "traces_to"
---
# US-006: Use a local credential from an application or CLI

## Story

**As a** maintainer of a Rust application or CLI
**I want** to save and retrieve credentials through one shared local API
**So that** users can configure access without placing secrets in settings files
and can understand when their system credential store cannot be used.

## Context

The application decides its settings schema and file path. The same user may
set a temporary environment override or save a credential for later runs.

## Acceptance Examples (Illustrative)

### US-006-EX-1: Save and reuse

- **Given** a working system credential store
- **When** the user saves a credential for one application
- **Then** that application can retrieve it on a later run

### US-006-EX-2: Temporary override

- **Given** a saved credential and a configured environment override
- **When** the application resolves that credential
- **Then** it uses the override and reports its source without printing its value

### US-006-EX-3: Store unavailable

- **Given** an unavailable system credential store
- **When** the user tries to save a credential
- **Then** the application reports the failure and writes no credential file

## Dependencies (Contextual)

Upstream: [StR-003](../stakeholder/StR-003-local-credentials.md). Downstream:
requirements for store operations, source selection, and adoption.
