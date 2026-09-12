---
id: US-005
title: "Resolve where a tool looks, in a stated order, without the foundation choosing the places"
type: US
relationships:
  - target: "ix://agent-ix/ix-cli-kit/StR-002"
    type: "traces_to"
---
# US-005: Resolve where a tool looks, in a stated order, without the foundation choosing the places

## Story

**As a** maintainer wiring a tool's settings together
**I want** the order in which a flag, an environment variable, a file and a default are
consulted to be decided once and shared
**So that** I do not re-implement the subtle part in every tool, while my tool keeps
naming its own paths and its own schema.

## Context

`quire-cli/src/commands/validate.rs` already implements a full resolution chain by hand
for module search roots: the `--scope` flag first, then `IX_FILAMENT_MODULES_PATH`,
then the legacy alias `IX_SCHEMA_PATH`, then the canonical install root. Two decisions
are encoded there that nobody wrote down: the path sets **union** rather than override,
and `IX_SCHEMA_PATH` is a legacy alias with no deprecation path and no warning — an
alias nothing ever calls deprecated is an alias forever.

The locations themselves are not shareable. The ecosystem's real default module root is
the dotdir `~/.ix/filament/modules`, and each application and module knows its own
shape.

## Acceptance Examples (Illustrative)

### US-005-EX-1: One value, one winner

- **Given** the same setting supplied by a flag, an environment variable and a file
- **When** the tool resolves it
- **Then** the flag wins, and the tool can report which layer supplied the value

### US-005-EX-2: Several places, all kept

- **Given** two environment variables that each name directories to search
- **When** the tool resolves its search path
- **Then** every named directory is searched, in first-seen order, with repeats dropped
  rather than reordered

### US-005-EX-3: A legacy name is visible when used

- **Given** an operator still setting a superseded variable
- **When** it actually contributes a directory
- **Then** the tool can tell the operator which variable is legacy and what replaces it

### US-005-EX-4: A typo is not silently ignored

- **Given** an environment variable set to a value the tool cannot parse
- **When** the tool resolves the setting
- **Then** the operator is told, rather than the tool falling through to the default

## Options (Exploratory)

Discussed: one generic resolver covering both scalars and path sets; two shapes, one
per semantics; or leaving each tool to hand-roll it. Collapsing the two semantics would
apply override rules to search roots, which drops the second variable's directories
entirely.

## Constraints (Contextual)

Nothing in the shared layer should name a path, create a directory, or read a variable
of its own. Contextual, and the reason the locations stay with the consumer.

## Dependencies (Contextual)

Upstream: `quire-cli`'s `scoped_registry_roots`. Downstream: requirements on scalar
precedence, search-path union, configuration loading, and XDG availability.

## Priority and Risk (Informative)

Medium-high value: the order is the part every tool gets subtly different.

## Notes (Informative)

Discovery recorded that the three command-line implementations of module-root
resolution in this ecosystem do not agree with one another; which one the shared layer
reproduces must therefore be stated, not assumed.

## Traceability (Informative)

Traces to the stakeholder need for an adoptable foundation that imposes no locations.
