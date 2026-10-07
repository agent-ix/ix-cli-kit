---
id: StR-003
title: "Rust applications need one safe local credential contract"
type: StR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/FR-014"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-015"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-cli-kit/FR-016"
    type: "satisfied_by"
---
# StR-003: Rust applications need one safe local credential contract

## Stakeholder Need

The shared Rust credential contract shall keep credentials out of ordinary
settings files and distinguish an absent credential from a locked or
unavailable store.

## Rationale

Applications need to keep local settings and credentials together in their user
flows, while ordinary settings files are unsuitable for secret values. A shared
contract lets applications report the same source and failure states without
duplicating storage policy.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-003-VC-1 | An application and a Rust CLI use the same shared API for credential lookup and source reporting. | Demonstration |
| StR-003-VC-2 | A locked or unavailable OS store produces a distinct failure and no local plaintext credential file. | Demonstration |

## Stakeholders

Maintainers of `ix-projects` and Rust command-line tools; users whose local
credentials those applications access.

## Dependencies

- **Upstream**: [StR-002](./StR-002-adoptable-without-imposing-argv-or-locations.md)
  keeps each consumer in charge of its paths and argument parser.
- **Downstream**: [US-006](../usecase/US-006-use-local-credentials.md).
