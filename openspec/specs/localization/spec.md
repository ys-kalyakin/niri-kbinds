# localization Specification

## Purpose

A single localization mechanism for `kbinds`: the language of all user-facing surfaces (the overlay, CLI help, diagnostics, audit, default titles and groups) is selected by the system language, with no new dependencies and no changes to machine contracts.

## Requirements

### Requirement: Detecting the system language

The language of any `kbinds` process (all CLI commands, dmenu, the overlay) SHALL be determined at startup from environment variables in priority order `LC_ALL` → `LC_MESSAGES` → `LANG` (first non-empty wins); the subtag up to the first `_`, `-`, or `.` is used (e.g., `ru_RU.UTF-8` → `ru`). The language is fixed for the process lifetime; live switching is not required. Empty values, `C`, `C.UTF-8`, `POSIX`, and unknown subtags mean English.

#### Scenario: Russian system locale

- **WHEN** a `kbinds` process is run with `LC_ALL=ru_RU.UTF-8`
- **THEN** all user-facing strings (the overlay, diagnostics, audit findings, default titles and groups) are printed in Russian

#### Scenario: English locale and LC_ALL priority

- **WHEN** the process is run with `LC_ALL=en_US.UTF-8` and `LANG=ru_RU.UTF-8`
- **THEN** strings are printed in English: `LC_ALL` takes priority over `LANG`

#### Scenario: Locale without a language

- **WHEN** the locale variables are unset or equal to `C`/`POSIX`
- **THEN** strings are printed in English (fallback)

### Requirement: Catalogs and fallback

The system SHALL support the `ru` and `en` string catalogs. Every message SHALL have a translation into both languages (catalog completeness is guaranteed by the data structure, not by manual checking). A key without a translation in the selected locale SHALL be displayed in English. Strings with substitutions SHALL substitute values according to the grammar of the catalog's language (translation templates are independent).

#### Scenario: Structural catalog completeness

- **WHEN** a new message is added to the catalog
- **THEN** the absence of either of the two translations does not compile (both languages are given in a single message record)

#### Scenario: Value substitution

- **WHEN** the overlay shows the counter under `en_US.UTF-8` with 3 visible binds out of 10
- **THEN** "3 of 10 binds" is displayed, not a mix of languages or a template with unfilled slots

### Requirement: Surface coverage

All user-facing strings SHALL be localized by the system language: the texts of the "Center" overlay (search placeholder, counter, footer hint, cheat-sheet title), the CLI help of commands and flags, human-readable error diagnostics in stderr, `kbinds audit` findings in human-readable form, and the default titles and group names of binds from the built-in dictionary. Code comments and documentation SHALL NOT be localized.

#### Scenario: The overlay follows the locale

- **WHEN** the overlay is opened under `LANG=ru_RU.UTF-8` and then under `LANG=en_US.UTF-8`
- **THEN** in the first case the placeholder is «Поиск…», the counter «N из M биндов», the footer and the title «Важные клавиши»; in the second — their English equivalents

#### Scenario: CLI help follows the locale

- **WHEN** `kbinds --help` is run under `LC_ALL=en_US.UTF-8`
- **THEN** the descriptions of commands and flags are printed in English; under `ru_RU.UTF-8` — in Russian

#### Scenario: Default titles follow the locale

- **WHEN** the config contains `Mod+Q { close-window; }` without `hotkey-overlay-title`, and the process runs under `en_US.UTF-8`
- **THEN** the bind title is the English translation from the dictionary (e.g., "Close window"), the group is the English name of the windows group

### Requirement: Machine contracts stay locale-independent

The locale SHALL NOT affect: bind identifiers (`id`), the set and names of JSON fields (`kbinds list --json`, `kbinds audit --json`), exit codes, the dmenu item format `<keys_display> — <title>` and the fuzzel prompt `binds> `, action names in `niri msg` commands, and the values of `keys_display`.

#### Scenario: JSON schema is stable

- **WHEN** `kbinds list --json` is run under `ru_RU.UTF-8` and under `en_US.UTF-8`
- **THEN** in both cases a valid JSON array with the same set of fields; only the `title`/`group` values of default binds differ, the `id`s are identical

#### Scenario: Prompt and exit codes are stable

- **WHEN** `kbinds dmenu` is run under different locales, and `kbinds audit --strict` is run on a config with a conflict
- **THEN** the fuzzel prompt remains `binds> `, the item format `<keys_display> — <title>` does not change, and the strict-audit exit code is 1 under any locale

### Requirement: User data and the dictionary stay outside localization

Data from the user's config SHALL NEVER be translated: `hotkey-overlay-title`/`title` titles, `spawn` commands, config comments. Whether an action name is in the dictionary (used by audit rule A2) SHALL NOT depend on the locale.

#### Scenario: User data is not translated

- **WHEN** the config sets `hotkey-overlay-title="Случайные обои"` and the system runs in `en_US.UTF-8`
- **THEN** the bind title remains «Случайные обои» in the overlay, dmenu, `list`, and JSON

#### Scenario: A2 is locale-independent

- **WHEN** `kbinds audit` is run under `en_US.UTF-8` on a config with a builtin bind without `hotkey-overlay-title` whose name is in the dictionary
- **THEN** the A2 finding is not emitted, even though the dictionary title is now printed in English
