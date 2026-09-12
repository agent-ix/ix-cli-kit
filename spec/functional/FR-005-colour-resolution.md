---
id: FR-005
title: "Decide colour from an explicit choice, terminal state and NO_COLOR, and keep the facts"
type: FR
relationships:
  - target: "ix://agent-ix/ix-cli-kit/US-004"
    type: "implements"
---
# FR-005: Decide colour from an explicit choice, terminal state and NO_COLOR, and keep the facts

## Description

The crate SHALL decide whether to colourise from a declared [`ColorChoice`], whether the
stream is a terminal, and whether `NO_COLOR` is set, and SHALL expose both the decision
and the facts it was made from.

The decision function is pure so a TUI consumer can ask the question about its own
surface without the crate inspecting the process, and so a test can exercise every
combination without a pty.

## Inputs

- `ColorChoice` — `Auto`, `Always` or `Never`.
- `is_terminal: bool` for the stream in question.
- `no_color_set: bool`, whether `NO_COLOR` is present in the environment.

## Outputs

- `ColorChoice::decide(is_terminal, no_color_set) -> bool`.
- `ColorChoice::resolve() -> ColorDecision { choice, no_color_set, is_terminal, enabled }`.
- `stdout_is_terminal()` and `stderr_is_terminal()`.

## Behavior

- The crate SHALL enable colour for `Always` regardless of terminal state and regardless
  of `NO_COLOR`.
- The crate SHALL disable colour for `Never` regardless of terminal state.
- For `Auto`, the crate SHALL enable colour only when the stream is a terminal and
  `NO_COLOR` is not set.
- The crate SHALL report the terminal state of standard output and standard error
  separately, so a consumer that redirects one and not the other is answered correctly.
- The crate SHALL retain `choice`, `no_color_set` and `is_terminal` alongside `enabled`,
  so a consumer can explain the decision rather than restate it.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-005-CON-1 | `decide` SHALL be a pure, total function of its two boolean inputs, reading nothing from the environment. | Interface | Test |
| FR-005-CON-2 | `Always` overriding `NO_COLOR` is the shipped behaviour and is intentional: an explicit flag outranks an ambient variable. | Behaviour | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | `decide` over all four combinations yields `Always` = true, true; `Never` = false, false; `Auto` = terminal-and-not-NO_COLOR. | Test |
| FR-005-AC-2 | A resolved decision reports the choice, the `NO_COLOR` observation and the terminal observation alongside `enabled`. | Test |
| FR-005-AC-3 | `ColorChoice` parses from `auto`, `always` and `never`. | Test |

## Dependencies

- **Upstream**: [US-004](../usecase/US-004-adopt-from-a-tui-and-a-clapless-cli.md)
- **Downstream**: [FR-004](./FR-004-diagnostic-rendering.md)
