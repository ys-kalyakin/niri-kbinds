# fuzzel-dmenu Specification

## Purpose

Interactive hotkey search via fuzzel in dmenu mode: the list of binds is fed on stdin, the selected item is executed on Enter ("found → executed" instead of "read the PNG → recall the combo → press the keys").

## Requirements

### Requirement: Menu items

`kbinds dmenu` SHALL run `fuzzel --dmenu -p "binds> "`, feed one item per bind on stdin in the format `<keys_display> — <title>` (an item SHALL include the action name so that fuzzel can search by it), and read the selection from fuzzel's stdout.

#### Scenario: Item format

- **WHEN** dmenu is run on a config with the bind `Mod+Shift+W` («Случайные обои»)
- **THEN** one of the stdin items contains `Mod+Shift+W — Случайные обои` and the action name; line breaks inside an item are sanitized (replaced so that the item remains a single line)

### Requirement: Executing the selected bind

The selected item SHALL be mapped back to the bind and executed the same way as `kbinds exec <id>` (all actions in order). Esc or empty fuzzel output SHALL end the run with exit code 0 without executing any actions.

#### Scenario: Enter executes the action

- **WHEN** the user finds a bind by search and presses Enter
- **THEN** the bind's actions are executed (as with `exec`); searching «воркспейс» finds the binds that move to workspaces

#### Scenario: Esc does nothing

- **WHEN** the user closes fuzzel with Esc
- **THEN** fuzzel's stdout is empty, no action is executed, the exit code of `kbinds dmenu` is 0

#### Scenario: Repeated invocation

- **WHEN** dmenu is invoked a second time after a successful execution
- **THEN** the menu opens with the full list of binds (the tool keeps no state between runs)

### Requirement: Availability and degradation

If fuzzel is not found in PATH or launching it fails, the command SHALL exit with code 2 and diagnostics in stderr without executing any actions.

#### Scenario: fuzzel missing

- **WHEN** `kbinds dmenu` is run in an environment without fuzzel
- **THEN** exit code 2, an error message in stderr
