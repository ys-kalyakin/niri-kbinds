# Spec Delta

## MODIFIED Requirements

### Requirement: The "Center" card

Above the scrim SHALL be displayed a card in the center of the focused monitor with two columns: on the left — a search field with autofocus, a grouped list of binds (group composition and their order — as in MVP1 `groups`), rows of "title + keycaps", the counter «N из M биндов» and the footer hint `↑↓ выбор · Enter выполнить · Esc закрыть`; on the right — the important-keys cheat sheet (requirement "Important-keys cheat sheet"). Card width 900px (no more than 92% of the monitor width), radius 20, dark background, light border, shadow.

#### Scenario: Card composition on the live config

- **WHEN** running on the live config with an empty query
- **THEN** a two-column card: on the left a focused search field, groups in MVP1 order, rows with keycaps, the counter «M из M биндов» (M — the total number of binds parsed by the MVP1 parser) and the footer hint; on the right the important-keys cheat sheet

#### Scenario: No important keys — single-column card

- **GIVEN** a config without a single `hotkey-overlay-title`
- **WHEN** running the overlay
- **THEN** the right column is not displayed, the card is single-column with width 600px — the look as before the cheat sheet appeared

## ADDED Requirements

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
