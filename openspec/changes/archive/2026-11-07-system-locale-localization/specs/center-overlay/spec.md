# Spec Delta

## MODIFIED Requirements

### Requirement: The "Center" card

Above the scrim SHALL be displayed a card in the center of the focused monitor with two columns: on the left — a search field with autofocus, a grouped list of binds (group composition and their order — as in MVP1 `groups`), rows of "title + keycaps", a counter of visible binds, and a footer control hint; on the right — the important-keys cheat sheet (requirement "Important-keys cheat sheet"). The counter texts (the "N из M" format + the word «биндов») and the footer (hints for selection, execution, and closing), as well as the search field placeholder, SHALL be taken from the localization catalog according to the system language (capability `localization`): under `ru` — «N из M биндов» and `↑↓ выбор · Enter выполнить · Esc закрыть`, under `en` — their English equivalents. Card width 900px (no more than 92% of the monitor width), radius 20, dark background, light border, shadow.

#### Scenario: Card composition on the live config

- **WHEN** running on the live config with an empty query
- **THEN** a two-column card: on the left a focused search field, groups in MVP1 order, rows with keycaps, the counter «M из M биндов» (the text per the localization catalog; M — the total number of binds parsed by the MVP1 parser) and the footer hint; on the right the important-keys cheat sheet

#### Scenario: No important keys — single-column card

- **GIVEN** a config without a single `hotkey-overlay-title`
- **WHEN** running the overlay
- **THEN** the right column is not displayed, the card is single-column with width 600px — the look as before the cheat sheet appeared

#### Scenario: Russian locale

- **WHEN** the overlay is opened under `LANG=ru_RU.UTF-8`
- **THEN** the counter reads «N из M биндов», the footer is `↑↓ выбор · Enter выполнить · Esc закрыть`, the placeholder is «Поиск…»

#### Scenario: English locale

- **WHEN** the overlay is opened under `LC_ALL=en_US.UTF-8`
- **THEN** the counter and footer are shown with the English equivalents from the catalog; the layout, sizes, and behavior of the card do not change
