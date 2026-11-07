# binds-audit Specification

## Purpose

Static analysis of the parsed bind model: combo conflicts, binds invisible in the stock PNG overlay, and missing binds for common niri actions. An answer to the question "why does the launcher behave oddly?" without reading 144 lines of config.

## Requirements

### Requirement: A1 — duplicated combo

Rule A1 (level `error`): the same canonical `combo` is assigned to actions with different commands. A finding SHALL list all bind variants with the file and line of each, without claiming which bind "wins" (per niri 26.04 sources, a config with a duplicate is rejected as a whole — notes of task 5.1). The same action on different keys is NOT a finding.

#### Scenario: Real conflict detected

- **WHEN** `kbinds audit` is run on the live config, where `Mod+D` is assigned twice to different commands (`noctalia msg panel-toggle launcher` and `vicinae toggle`)
- **THEN** an `error`-level finding lists both binds with their files and lines and does not name a winner

#### Scenario: One action on different keys — not a conflict

- **WHEN** the config contains `Mod+Left` and `Mod+H` with the same action `focus-column-left`
- **THEN** A1 reports no conflict

### Requirement: A2 — bind invisible in the overlay

Rule A2 SHALL emit a `warn`-level finding for a bind without `hotkey-overlay-title` that has no human-readable name from the dictionary (a custom `spawn`/`spawn-sh`, or a builtin outside the dictionary) — such a bind is not meaningfully visible in the stock PNG overlay. Builtin binds without an overlay-title whose name is in the dictionary (niri substitutes a default title) are not warnings.

#### Scenario: Spawn without overlay-title

- **WHEN** the config contains a `spawn` bind without `hotkey-overlay-title`
- **THEN** the audit emits a `warn` finding with the file, line, and command of the bind

#### Scenario: Builtin with a dictionary name

- **WHEN** the config contains `Mod+Q { close-window; }` without `hotkey-overlay-title`
- **THEN** A2 emits no finding (the name is in the dictionary: «Закрыть окно» / "Close window")

### Requirement: A3 — common actions without a bind

Rule A3 SHALL emit an `info`-level finding for common niri actions (a static list of ~20–30 in code: screenshots, `toggle-window-floating`, `consume-or-expel-window-*`, etc.) for which the config has no bind at all. The rule is a nice-to-have: its absence does not block the rest of the audit.

#### Scenario: Action without a bind

- **WHEN** the config has no bind for an action from the A3 list
- **THEN** the audit emits an `info` finding with the action name

### Requirement: Audit output

`kbinds audit` SHALL print findings in a human-readable form in the system locale language (capability `localization`): under `ru` — in Russian, under `en` and locales without a translation — in English; the meaning, level, and composition of findings do not depend on the language. With `--json` — a machine-readable list of findings (rule id, level, message, references to binds); field names and rule ids do not depend on the locale, and the message texts in JSON follow the selected locale. With `--strict` and at least one `error` finding the exit code SHALL be 1; without `--strict` — 0 regardless of findings. On a config without findings the audit SHALL produce zero findings and exit code 0.

#### Scenario: Finding language follows the locale

- **WHEN** `kbinds audit` is run on a config with the `Mod+D` conflict under `ru_RU.UTF-8` and then under `en_US.UTF-8`
- **THEN** in both cases the `error` finding is printed, in Russian in the first case and in English in the second; the rule id and level match

#### Scenario: Strict mode

- **WHEN** `kbinds audit --strict` is run on the live config with the `Mod+D` conflict under `en_US.UTF-8`
- **THEN** exit code 1, the English-language `error` finding is printed to stdout or stderr per the regular output rules

#### Scenario: Negative test

- **WHEN** `kbinds audit` is run on a fixture without findings (`clean.kdl`)
- **THEN** zero findings, exit code 0
