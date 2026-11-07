# Spec Delta

## Purpose

The graphical «Центр» ("Center") overlay: a layer-shell surface above the niri desktop with a scrim, a search card, and a grouped list of binds — a visual cheat sheet of "look → find → execute" on top of the data and execution of MVP1.

## ADDED Requirements

### Requirement: Launch and arguments

`kbinds overlay` SHALL run with the flags `--config PATH`, `--dry-run`, `--mode center` (center is the default mode). The config SHALL be parsed by the MVP1 library into the model before the UI opens; a parse error or a launch outside a Wayland session SHALL print a human-readable message to stderr and terminate the process with exit code 2, with no panics or backtraces.

#### Scenario: Unreadable config

- **WHEN** running `kbinds overlay --config /нет/такого.kdl`
- **THEN** a human-readable parse error message in stderr, exit code 2, the surface does not open

#### Scenario: Launch outside a Wayland session

- **WHEN** running without an available `WAYLAND_DISPLAY`
- **THEN** a message like «kbinds-overlay: не найдена Wayland-сессия» ("kbinds-overlay: no Wayland session found") in stderr, exit code 2, no panic/backtrace

### Requirement: Layer-shell surface on the focused monitor

The overlay SHALL open as a layer-shell surface of the `overlay` layer with keyboard interactivity `exclusive`, anchored to the focused monitor determined at startup, covering it entirely. The background SHALL be a semi-transparent scrim `rgba(8,6,14,0.55)` through which windows show through. Behavior when the monitor is switched after opening is not regulated.

#### Scenario: Fullscreen scrim on the focused monitor

- **GIVEN** the focused monitor is DP-2
- **WHEN** running `kbinds overlay`
- **THEN** the surface opens on DP-2 fullscreen, the background dims the desktop, windows show through it

### Requirement: The «Центр» card

Above the scrim SHALL be displayed a card in the center of the focused monitor: width 600px (no more than 92% of the monitor width), radius 20, dark background, light border, shadow. Inside the card: a search field with autofocus, a grouped list of binds (group composition and their order — as in MVP1 `groups`), "title + keycaps" rows, a «N из M биндов» counter, and the footer hint `↑↓ выбор · Enter выполнить · Esc закрыть`.

#### Scenario: Card composition on the live config

- **WHEN** running on the live config with an empty query
- **THEN** the card contains a focused search field, groups in MVP1 order, rows with keycaps, the counter «M из M биндов» (M — the total number of binds parsed by the MVP1 parser) and the footer hint

### Requirement: Keycaps from keys_display

Row keycaps SHALL be built from the `keys_display` field of the MVP1 model without any new key parsing; Cyrillic keys SHALL be displayed as characters (e.g., `Mod+В`). The secondary text SHALL be readable on both backgrounds of the card.

#### Scenario: Cyrillic bind

- **GIVEN** the bind `Mod+В` in the config
- **THEN** its row's keycaps are displayed as the characters `Mod` and `В`

### Requirement: Live search filter

Typing any characters into the search field SHALL filter the list live by the `search` field of the model (Cyrillic is typed as ordinary text). An empty query SHALL show all binds. Groups without visible rows SHALL be hidden.

#### Scenario: Filtering by «воркспейс»

- **WHEN** the user types «воркспейс» (including in the ru layout)
- **THEN** the list narrows to binds with a match in `search`, the counter reflects «N из M», empty groups are not displayed

#### Scenario: Empty query shows everything

- **WHEN** the user clears the search field
- **THEN** all binds of the config are displayed

### Requirement: Keyboard navigation

`↑` and `↓` SHALL move the selection cyclically across visible rows (the list wraps around); `Enter` SHALL execute the selected bind and close the overlay; `Esc` SHALL close the overlay without executing. `Tab` is not used.

#### Scenario: Selection wrap-around

- **GIVEN** the selection is on the last visible row
- **WHEN** pressing `↓`
- **THEN** the selection moves to the first visible row

#### Scenario: Esc closes without actions

- **WHEN** pressing `Esc` with a bind selected
- **THEN** the overlay closes, no action is executed

### Requirement: Mouse

Clicking a visible row SHALL execute its bind; clicking outside the card SHALL close the overlay; the list SHALL scroll with the mouse wheel.

#### Scenario: Row click

- **WHEN** the user clicks a bind row
- **THEN** the bind is executed the same way as via Enter, the overlay closes

#### Scenario: Click outside the card

- **WHEN** the user clicks the scrim outside the card
- **THEN** the overlay closes without executing actions

### Requirement: Execution and dry-run

Executing the selected bind SHALL go through the MVP1 `exec` routing (`niri msg action …` for all actions of the bind in order), after which the overlay SHALL close. In `--dry-run` mode, instead of executing, the bind's command line SHALL be printed to stderr, the overlay SHALL close with exit code 0, and no actions are executed.

#### Scenario: Dry-run prints the command to stderr

- **WHEN** `kbinds overlay --dry-run` and the user selects a bind
- **THEN** the command line of the selected bind is printed to stderr, the process exits with code 0, the action is not executed

#### Scenario: Enter executes the action

- **GIVEN** a run without `--dry-run`
- **WHEN** the user executes a bind
- **THEN** the bind's actions are executed as `kbinds exec <id>` and the overlay closes

### Requirement: Theme follows the noctalia palette

The overlay colors SHALL be taken from the noctalia palette cache (`~/.cache/kbinds/theme.json` — written by the user's noctalia template from the resolved palette: scrim→scrim, surface_container→card, outline/outline_variant→borders, on_surface/on_surface_variant→text, primary→accent). If the cache is missing or unparseable, the overlay SHALL use the fixed palette of the task spec §6. The alpha levels of §6 (55/94/16%) are preserved.

#### Scenario: Palette cache applied

- **GIVEN** the user's noctalia template has been rendered (the active scheme is builtin Kanagawa, dark)
- **WHEN** running `kbinds overlay`
- **THEN** the card, text, accent, and scrim are colored with the tokens from the cache; changing the palette in noctalia changes the overlay colors after the template is re-rendered

#### Scenario: No cache — fallback palette

- **WHEN** running with a missing/corrupt `~/.cache/kbinds/theme.json`
- **THEN** the overlay opens with the fixed palette of the task spec §6, without errors or panics
