# binds-parsing Specification

## Purpose

Parsing of the `binds` section of the niri config (`config.kdl`) into a full machine-readable model of binds: a KDL dialect compatible with the niri parser, `include` support, combo canonicalization, human-readable key display (including Cyrillic), and deterministic grouping. The model is the single source of truth for all `kbinds` commands and future integrations (v2 overlay).

## Requirements

### Requirement: Extract all binds from the config

The tool SHALL extract all bind nodes from the `binds { … }` block of the file `~/.config/niri/config.kdl` (default path, overridden by `--config`), including binds spliced in from `include` files. No bind SHALL be skipped or duplicated.

#### Scenario: Full coverage of the live config

- **WHEN** the tool parses the user's live config (`~/.config/niri/config.kdl` with the line `include "./noctalia.kdl"`)
- **THEN** the number of parsed binds matches the grep baseline over all bind lines of the `binds` section (159 as of 2026-11-07; the literal baseline of the task spec, `grep -cE '^\s*Mod'` = 144, counts only Mod-binds, the remaining 15 are XF86/Alt/Ctrl/Print); any mismatch is a parser defect

#### Scenario: All bind forms parse

- **WHEN** the config contains a block form with a body (`Mod+Q repeat=false { close-window; }`), multiple attributes (`Mod+Space repeat=false hotkey-overlay-title="Switch layout" { switch-layout "next"; }`), spawn with arguments, `spawn-sh`, and `//` and `/* */` comments
- **THEN** each case is parsed: actions and attributes (`repeat`, `hotkey-overlay-title`) are reflected in the model

### Requirement: Include resolution

The tool SHALL resolve `include "path"` relative to the directory of the including file and splice its top-level nodes at the point of inclusion. A cyclic include SHALL result in a parse error (not infinite recursion).

#### Scenario: Relative include

- **WHEN** `config.kdl` contains `include "./noctalia.kdl"`
- **THEN** the file is read relative to the directory of `config.kdl`, its top-level nodes participate in parsing; binds from the include enter the model with their own file paths and lines

#### Scenario: Cyclic include

- **WHEN** file A includes B, and B includes A
- **THEN** parsing fails with a parse error (exit code 2), the message names the file

### Requirement: Bind model

Each bind SHALL contain: all action nodes of the body in order (`kind`: `builtin` | `spawn` | `spawn-sh`; for `builtin` — the action name, for `spawn`/`spawn-sh` — argv), the `repeat` flag, the `hotkey-overlay-title` attribute, the source file and line, the original key entry `keys_raw`, the canonical `combo`, the `keys_display` mapping, the group, and the `search` field.

#### Scenario: Bind with multiple actions

- **WHEN** a bind contains two actions in its body
- **THEN** the model stores both actions in order; execution within `binds-cli` runs both

### Requirement: Tolerance of unknown constructs

Unknown properties and attributes of a bind SHALL be semantically ignored — parsing SHALL NOT fail on them; `XF86*` key names and `condition` constructs SHALL be tolerated. The verbatim representation of such constructs SHALL be preserved in the model's `raw` field as diagnostics data for humans and future tooling; the audit rules (A1–A3) SHALL NOT depend on it.

#### Scenario: Unknown property does not break parsing

- **WHEN** a bind contains an attribute or property unknown to the tool (e.g., `condition { … }`, `cooldown-ms=100`)
- **THEN** parsing succeeds and the property is preserved verbatim in `raw`; the key `XF86AudioMute` parses as a regular key

#### Scenario: Audit output does not depend on unknown constructs

- **WHEN** a bind carries arbitrary unknown attributes
- **THEN** the audit findings for the config are identical to the findings for the same config without those attributes (rules A1–A3 never read `raw`)

### Requirement: Canonical combo

`combo` SHALL be the canonical combo key: modifiers in the order `mod, ctrl, shift, alt`, the key in lowercase, separator `+`. Entries with the same meaning SHALL produce the same `combo`.

#### Scenario: Canonicalization

- **WHEN** a bind is written as `Mod+Shift+W`
- **THEN** `combo` equals `mod+shift+w`

### Requirement: Human-readable key display

`keys_display` SHALL be built from `keys_raw` by the rules: `Left/Right/Up/Down` → `←/→/↑/↓`, `Return` → `Enter`, `Slash` → `/`, `Cyrillic_*` → Unicode character per the reference table captured from the local reference (`/usr/include/X11/keysymdef.h` or `xkbcli how-to-type`; reproducing from memory is forbidden). An unknown key name SHALL be displayed as-is without an error. `keys_raw` SHALL preserve the original.

#### Scenario: Special keys

- **WHEN** a bind contains `Return`, `Left`, `Slash`
- **THEN** `keys_display` contains `Enter`, `←`, `/`

#### Scenario: Cyrillic key

- **WHEN** a bind contains `Mod+Cyrillic_ve`
- **THEN** `keys_display` shows the character `в` (U+0432, lowercase — per keysymdef.h), `keys_raw` preserves `Mod+Cyrillic_ve`

#### Scenario: Unknown key name

- **WHEN** a bind contains a key name missing from the table
- **THEN** the name is displayed as-is, parsing and the other commands do not fail

### Requirement: Bind title

The bind title SHALL be selected by priority: the `hotkey-overlay-title` attribute → the human-readable action name from the dictionary (`close-window` → «Закрыть окно» ("Close window" in the ru catalog); unknown actions — the raw entry) → for `spawn`, the command as a single string. The title source SHALL be reflected (`title_source`).

#### Scenario: Title priority

- **WHEN** a bind has `hotkey-overlay-title="Случайные обои"` and the action `spawn "noctalia" "msg" "wallpaper-random"`
- **THEN** the title is «Случайные обои», `title_source` = `overlay-title`

### Requirement: Deterministic grouping

Each bind SHALL be assigned to a deterministic group by action type (e.g., `close-window` → «Окна» (Windows), workspace actions → «Воркспейсы» (Workspaces), spawn with a launcher → «Система» (System)); the action title dictionary SHALL be shared between grouping and titles. Grouping of the same bind SHALL produce the same result between runs.

#### Scenario: Stable group

- **WHEN** `list` is run twice on the same config
- **THEN** the groups of all binds match one-to-one; the `search` field contains a lowercase concatenation of the title, keys, actions, and group
