# Spec Delta — center-overlay

## ADDED Requirements

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

## MODIFIED Requirements

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
