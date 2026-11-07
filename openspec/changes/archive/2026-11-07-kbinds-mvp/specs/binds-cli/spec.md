# Spec Delta: binds-cli

## Purpose

The CLI interface of `kbinds`: the `list` command (including machine-readable JSON) and the `exec` command (executing bind actions via `niri msg action`, with `--dry-run`), uniform exit codes and error handling.

## ADDED Requirements

### Requirement: The list command

`kbinds list` SHALL print the list of binds from the config. With the `--json` flag the output SHALL be a valid JSON array of objects with the fields: `id`, `title`, `title_source`, `actions` (with `kind`, `name`, `cmd`), `keys_raw`, `keys_display`, `combo`, `group`, `repeat`, `overlay_title_present`, `file`, `line`, `search`. Without `--json` the output SHALL be human-readable (id, keys, title, group).

#### Scenario: Valid JSON

- **WHEN** `kbinds list --json` is run on a non-empty config
- **THEN** stdout parses as a JSON array; every object has `combo`, `keys_display`, `group`, `search` filled; entries are filterable with standard `jq`

#### Scenario: Filtering scenario

- **WHEN** `kbinds list --json | jq '.[] | select(.group=="Воркспейсы")'` is run
- **THEN** the output contains only binds of the group «Воркспейсы» (Workspaces)

### Requirement: Bind identifier

Each bind SHALL have a stable `id` — the kebab-case form of the canonical combo (e.g., `mod+shift+w` → `mod-shift-w`), unique across all binds of the model (duplicate combos get a distinguishing suffix).

#### Scenario: id stability across runs

- **WHEN** `list --json` is run twice on the same config
- **THEN** the ids of each bind are the same; `exec` with an id taken from the `list` output finds the same bind

### Requirement: The exec command

`kbinds exec <id>` SHALL execute all actions of the bind in order: `builtin` — via `niri msg action <name> <args…>`; `spawn`/`spawn-sh` — via `niri msg action spawn -- <argv…>` (syntax verified: `niri msg action spawn -- <COMMAND>...`). `--dry-run` SHALL print the commands to stdout and execute nothing. Running exec SHALL NOT require network access.

#### Scenario: Builtin via niri msg

- **WHEN** `kbinds exec <id>` is run for a bind with a `builtin` action
- **THEN** `niri msg action <name> <args…>` is invoked

#### Scenario: Spawn via niri msg spawn

- **WHEN** `kbinds exec <id>` is run for a bind with a `spawn` action
- **THEN** `niri msg action spawn -- <argv…>` is invoked exactly as written in the config

#### Scenario: Multiple actions in order

- **WHEN** a bind contains several actions and `kbinds exec <id>` is run
- **THEN** the actions are executed in the order they appear in the config

#### Scenario: Dry-run executes nothing

- **WHEN** `kbinds exec mod-t --dry-run` is run
- **THEN** the command(s) are printed to stdout exactly as they would be launched, there are no side effects, exit code 0

### Requirement: Config path override

All commands SHALL accept `--config PATH` (default `~/.config/niri/config.kdl`).

#### Scenario: Parsing a fixture

- **WHEN** a command is run with `--config tests/fixtures/edge.kdl`
- **THEN** parsing reads the given file, and `file` in the model points to it

### Requirement: Exit codes and diagnostics

Exit codes: `0` — success; `1` — audit findings under `--strict`; `2` — error (IO, parsing, invalid usage, unknown id in `exec`). Diagnostics SHALL be printed to stderr; stdout SHALL contain only the command's useful output.

#### Scenario: Parse error

- **WHEN** the config does not exist or does not parse
- **THEN** exit code 2, an error message in stderr, stdout empty

#### Scenario: Unknown id

- **WHEN** `kbinds exec no-such-id` is run
- **THEN** exit code 2, the message in stderr states that the bind was not found
