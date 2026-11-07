# Spec Delta: binds-audit

## Purpose

Static analysis of the parsed bind model: combo conflicts, binds invisible in the stock PNG overlay, and missing binds of common niri actions. An answer to "why does the launcher behave strangely?" without reading 144 config lines.

## ADDED Requirements

### Requirement: A1 — duplicated combo

Rule A1 (level `error`): the same canonical `combo` is assigned to actions with different commands. The finding SHALL list every bind variant with its file and line, without claiming which bind "wins" (per the niri 26.04 sources a config with a duplicate is rejected entirely — the notes of task 5.1). The same action on different keys is NOT a finding.

#### Scenario: A real conflict is detected

- **WHEN** `kbinds audit` runs on a live config where `Mod+D` is assigned twice to different commands (`noctalia msg panel-toggle launcher` and `vicinae toggle`)
- **THEN** an `error`-level finding lists both binds with their files and lines and names no winner

#### Scenario: One action on different keys is not a conflict

- **WHEN** the config contains `Mod+Left` and `Mod+H` with the same action `focus-column-left`
- **THEN** A1 reports no conflict

### Requirement: A2 — a bind invisible in the overlay

Rule A2 SHALL emit a `warn`-level finding for a bind without `hotkey-overlay-title` that has no human-readable dictionary name (a custom `spawn`/`spawn-sh` or an out-of-dictionary builtin) — such a bind is not meaningfully visible in the stock PNG overlay. Builtin binds without an overlay-title whose name is in the dictionary (niri substitutes a default title) are not warnings.

#### Scenario: Spawn without an overlay title

- **WHEN** the config contains a `spawn` bind without `hotkey-overlay-title`
- **THEN** the audit emits a `warn` finding with the bind's file, line, and command

#### Scenario: Builtin with a dictionary name

- **WHEN** the config contains `Mod+Q { close-window; }` without `hotkey-overlay-title`
- **THEN** A2 emits no finding (the name is in the dictionary: «Закрыть окно» / "Close window")

### Requirement: A3 — common actions with no bind

Rule A3 SHALL emit an `info`-level finding for common niri actions (a static list of ~20–30 in code: screenshots, `toggle-window-floating`, `consume-or-expel-window-*`, etc.) that have no bind at all in the config. The rule is a nice-to-have: its absence does not block the rest of the audit.

#### Scenario: An action with no bind

- **WHEN** the config has no bind for an action from the A3 list
- **THEN** the audit emits an `info` finding with the action name

### Requirement: Audit output

`kbinds audit` SHALL print findings in a human-readable form (in Russian), and with `--json` a machine-readable list of findings (rule id, level, message, bind references). With `--strict` and at least one `error` finding the exit code SHALL be 1; without `--strict` — 0 regardless of findings. On a config without findings the audit SHALL yield zero findings and code 0.

#### Scenario: Strict mode

- **WHEN** `kbinds audit --strict` runs on the live config with the `Mod+D` conflict
- **THEN** exit code 1, the `error` finding printed to stdout or stderr per the normal output

#### Scenario: Negative test

- **WHEN** `kbinds audit` runs on a fixture without findings (`clean.kdl`)
- **THEN** zero findings, exit code 0
