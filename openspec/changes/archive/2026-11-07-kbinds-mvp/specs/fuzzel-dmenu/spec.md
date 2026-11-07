# Spec Delta: fuzzel-dmenu

## Purpose

Interactive hotkey search via fuzzel in dmenu mode: the bind list is fed to stdin, the selected item is executed on Enter ("find → execute" instead of "read the PNG → recall the combo → press").

## ADDED Requirements

### Requirement: Menu items

`kbinds dmenu` SHALL launch `fuzzel --dmenu -p "binds> "`, feed one item per bind to stdin in the format `<keys_display> — <title>` (the item SHALL include the action name so fuzzel can search by it), and read the selection from fuzzel's stdout.

#### Scenario: Item format

- **WHEN** dmenu is run on a config with the bind `Mod+Shift+W` («Случайные обои»)
- **THEN** one of the stdin items contains `Mod+Shift+W — Случайные обои` and the action name; line breaks inside an item are sanitized (replaced so that the item remains a single line)

### Requirement: Executing the selected bind

The selected item SHALL be mapped back to the bind and executed exactly like `kbinds exec <id>` (all actions in order). Esc or empty fuzzel output SHALL terminate with code 0 without executing any action.

#### Scenario: Enter executes the action

- **WHEN** the user found a bind by search and pressed Enter
- **THEN** the bind's actions are executed (as with `exec`); searching «воркспейс» finds the binds that move to workspaces

#### Scenario: Esc does nothing

- **WHEN** the user closes fuzzel with Esc
- **THEN** fuzzel's stdout is empty, no action is executed, and the exit code of `kbinds dmenu` is 0

#### Scenario: Second invocation

- **WHEN** dmenu is invoked a second time after a successful execution
- **THEN** the menu opens with the full bind list (the tool keeps no state between runs)

### Requirement: Availability and degradation

If fuzzel is not found in PATH or fails to launch, the command SHALL terminate with code 2 and a diagnostic in stderr, without executing any action.

#### Scenario: fuzzel is missing

- **WHEN** `kbinds dmenu` runs in an environment without fuzzel
- **THEN** exit code 2, an error message in stderr
