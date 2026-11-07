# center-overlay Specification

## Purpose

The graphical "Center" overlay: a layer-shell surface above the niri desktop with a scrim, a search card, and a grouped list of binds — a visual cheat sheet of "look → find → execute" on top of the data and execution of MVP1.

## Requirements

### Requirement: Launch and arguments

`kbinds overlay` SHALL run with the flags `--config PATH`, `--dry-run`, `--mode center` (center is the default mode), and `--font-scale SCALE`; the standalone `kbinds-overlay` binary SHALL accept the same flags. The config SHALL be parsed by the MVP1 library into the model before the UI opens; a parse error or a launch outside a Wayland session SHALL print a human-readable message to stderr and terminate the process with exit code 2, with no panics or backtraces.

#### Scenario: Unreadable config

- **WHEN** running `kbinds overlay --config /нет/такого.kdl`
- **THEN** a human-readable parse error message in stderr, exit code 2, the surface does not open

#### Scenario: Launch outside a Wayland session

- **WHEN** running without an available `WAYLAND_DISPLAY`
- **THEN** a message like «kbinds-overlay: не найдена Wayland-сессия» ("kbinds-overlay: no Wayland session found") in stderr, exit code 2, no panic/backtrace

#### Scenario: Font scale flag

- **WHEN** running `kbinds overlay --font-scale 1.5` (or `kbinds-overlay --font-scale 1.5`)
- **THEN** the overlay opens with all text at 1.5× the base sizes; the rest of the behavior is unchanged

#### Scenario: Malformed font scale flag

- **WHEN** running `kbinds overlay --font-scale abc`
- **THEN** a usage error in stderr, exit code 2, the surface does not open

### Requirement: Layer-shell surface on the focused monitor

The overlay SHALL open as a layer-shell surface of the `overlay` layer with keyboard interactivity `exclusive`, anchored to the focused monitor determined at startup, covering it entirely. The background SHALL be a semi-transparent scrim `rgba(8,6,14,0.55)` through which windows show through. Behavior when the monitor is switched after opening is not regulated.

#### Scenario: Fullscreen scrim on the focused monitor

- **GIVEN** the focused monitor is DP-2
- **WHEN** running `kbinds overlay`
- **THEN** the surface opens on DP-2 fullscreen, the background dims the desktop, windows show through it

### Requirement: The "Center" card

Above the scrim SHALL be displayed a card in the center of the focused monitor with two columns: on the left — a search field with autofocus, a grouped list of binds (group composition and their order — as in MVP1 `groups`), rows of "title + keycaps", a counter of visible binds, and a footer hint; on the right — the important-keys cheat sheet (requirement "Important-keys cheat sheet"). The counter texts (format "N из M" + the word «биндов») and the footer (hints for selection, execution, and closing), as well as the search field placeholder, SHALL be taken from the localization catalog according to the system language (capability `localization`): under `ru` — «N из M биндов» and `↑↓ выбор · Enter выполнить · Esc закрыть`, under `en` — their English equivalents. Card width 1000px at font scale 1.0 (no more than 92% of the monitor width), scaling with the font scale (requirement «Card geometry follows the font scale»), radius 20, dark background, light border, shadow.

#### Scenario: Card composition on the live config

- **WHEN** running on the live config with an empty query
- **THEN** a two-column card: on the left a focused search field, groups in MVP1 order, rows with keycaps, the counter «M из M биндов» (text per the localization catalog; M — the total number of binds parsed by the MVP1 parser) and the footer hint; on the right the important-keys cheat sheet

#### Scenario: No important keys — single-column card

- **GIVEN** a config without a single `hotkey-overlay-title`
- **WHEN** running the overlay
- **THEN** the right column is not displayed, the card is single-column with width 600px at font scale 1.0 (scaling the same way) — the look as before the cheat sheet appeared

#### Scenario: Russian locale

- **WHEN** the overlay is opened under `LANG=ru_RU.UTF-8`
- **THEN** the counter reads «N из M биндов», the footer is `↑↓ выбор · Enter выполнить · Esc закрыть`, the placeholder is «Поиск…»

#### Scenario: English locale

- **WHEN** the overlay is opened under `LC_ALL=en_US.UTF-8`
- **THEN** the counter and footer are shown with the English equivalents from the catalog; the layout, sizes, and behavior of the card do not change

### Requirement: Important-keys cheat sheet

The right column of the card SHALL show a static cheat sheet: binds whose config entry sets `hotkey-overlay-title` (the same set the stock niri PNG overlay showed), as "title + keycaps" rows with keycaps from `keys_display`, in config order. The panel SHALL be read-only: its content is not filtered by the search and does not react to `↑↓`, `Enter`, or clicks; if the rows do not fit in the card, the panel SHALL scroll with the mouse wheel.

#### Scenario: Cheat sheet composition

- **GIVEN** a config with 16 binds marked with `hotkey-overlay-title`
- **WHEN** running the overlay with an empty query
- **THEN** the right column shows exactly these 16 binds as "title + keycaps" rows, Cyrillic keys as characters (e.g., `Mod+В`)

#### Scenario: The cheat sheet is not filtered by search

- **WHEN** typing a query narrows the left list to a single bind
- **THEN** the content of the right column does not change — it still shows all binds with `hotkey-overlay-title`

#### Scenario: View only — clicks and navigation have no effect

- **WHEN** clicking a cheat-sheet row
- **THEN** nothing happens: no action is executed, the overlay does not close, the selection in the left list does not move

#### Scenario: Scrolling a long cheat sheet

- **GIVEN** more binds with `hotkey-overlay-title` than the card can fit by height
- **WHEN** scrolling with the mouse wheel over the right column
- **THEN** the cheat sheet scrolls independently of the left list

### Requirement: Keycaps from keys_display

Row keycaps SHALL be built from the `keys_display` field of the MVP1 model without any new key parsing; Cyrillic keys SHALL be displayed as characters (e.g., `Mod+В`). The secondary text SHALL be readable on both backgrounds of the card.

#### Scenario: Cyrillic bind

- **GIVEN** the bind `Mod+В` in the config
- **THEN** its row's keycaps are displayed as the characters `Mod` and `В`

### Requirement: Live search filter

Typing any characters into the search field SHALL filter the list live by the `search` field of the model (Cyrillic is typed as ordinary text). An empty query SHALL show all binds. Groups without visible rows SHALL be hidden.

#### Scenario: Filtering by «воркспейс»

- **WHEN** the user types «воркспейс» (including in the ru layout)
- **THEN** the list narrows to binds with a match in `search`, the counter shows "N of M", empty groups are not displayed

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

- **GIVEN** the user's noctalia template has been rendered (active scheme — builtin Kanagawa, dark)
- **WHEN** running `kbinds overlay`
- **THEN** the card, text, accent, and scrim are colored with the tokens from the cache; changing the palette in noctalia changes the overlay colors after the template is re-rendered

#### Scenario: No cache — fallback palette

- **WHEN** running with a missing/corrupt `~/.cache/kbinds/theme.json`
- **THEN** the overlay opens with the fixed palette of the task spec §6, without errors or panics

### Requirement: Typography and font scale

Every text element of the overlay SHALL render at `base size × scale`. The scale SHALL be resolved with the priority: the `--font-scale` CLI flag when given, otherwise a valid `font_scale` token in the noctalia theme cache, otherwise `1.0`. The scale SHALL be clamped into the range 0.5–3.0 (a malformed cache token falls back per-token to `1.0`, consistent with the color tokens).

#### Scenario: Base sizes at scale 1.0

- **WHEN** the overlay is opened without `--font-scale` and without a valid `font_scale` token in the theme cache
- **THEN** the text renders at the base sizes: bind row titles and cheat-sheet row titles 18px, keycap labels 14px monospace, search field text 20px, group headers and cheat-sheet header 15px, visible-binds counter and footer hints 13px

#### Scenario: The flag overrides the cache

- **GIVEN** the theme cache carries `font_scale: 1.2`
- **WHEN** running `kbinds overlay --font-scale 1.5`
- **THEN** all text renders at 1.5× the base sizes (e.g., row titles 27px), not 1.2×

#### Scenario: The cache token applies without the flag

- **GIVEN** the theme cache carries `font_scale: 1.2` and no `--font-scale` flag is given
- **WHEN** the overlay is opened
- **THEN** all text renders at 1.2× the base sizes

#### Scenario: Out-of-range values are clamped

- **WHEN** running with `--font-scale 10`
- **THEN** the text renders at 3.0× the base sizes; with `--font-scale 0.1` — at 0.5×, no errors or panics

#### Scenario: Malformed cache token

- **GIVEN** the theme cache carries `font_scale: "big"` and no `--font-scale` flag is given
- **WHEN** the overlay is opened
- **THEN** the text renders at 1.0× the base sizes, without errors or panics

### Requirement: Card geometry follows the font scale

The card width (600px single-column, 1000px with the cheat-sheet panel), the card height (640px), and the cheat-sheet panel width (400px) SHALL scale by the resolved font scale, still capped at 92% of the monitor in each dimension. The card stays centered, and the outside-click hit test uses the same scaled rectangle.

#### Scenario: Card grows with the scale

- **GIVEN** a monitor large enough that the 92% cap does not bite
- **WHEN** running with `--font-scale 1.5`
- **THEN** the two-column card is 1500×960 and the cheat-sheet panel is 600px wide, centered as before, and the text fits the card

#### Scenario: The cap still applies

- **WHEN** running with `--font-scale 3.0` on a small monitor
- **THEN** the card never exceeds 92% of the monitor in either dimension, and the panels scroll

### Requirement: Scrolling without a visible scrollbar

Neither the bind list nor the cheat-sheet panel SHALL draw a visible scrollbar, and the hidden scrollbar SHALL not reserve row width. All existing scrolling behavior SHALL be preserved.

#### Scenario: No scrollbar on either panel

- **GIVEN** more rows than the card fits by height
- **WHEN** the overlay is open
- **THEN** no scrollbar is visible on the edge of either panel, and rows use the full panel width

#### Scenario: Wheel scrolling still works

- **WHEN** the user scrolls the mouse wheel over the bind list or the cheat sheet
- **THEN** the respective panel scrolls as before

#### Scenario: Selection follow still works

- **WHEN** the user moves the selection with `↑↓` past the visible rows
- **THEN** the list scrolls so the selected row stays visible

#### Scenario: Filter change snaps to top

- **WHEN** the user types a new search query
- **THEN** the bind list is scrolled back to the top as before
