# Proposal: system-locale-localization

## Why

All user-facing strings of `kbinds` are hardcoded: the overlay and diagnostics are in Russian, the CLI help is in English, the audit is in Russian (pinned by the `binds-audit` spec). When the system language changes, the interface stays in the old language; a language mix (a Russian UI on top of an English system) looks unfinished. A single mechanism is needed: the language is selected by the system locale.

## What Changes

- A new localization module `l10n`: detecting the system language from the environment (`LC_ALL` → `LC_MESSAGES` → `LANG`), `ru` and `en` string catalogs, fallback to `en`.
- All user-facing surfaces are localized:
  - the "Center" overlay (the «Поиск…» placeholder, the counter, the footer hint, «Важные клавиши»);
  - the CLI help of all commands and flags (clap);
  - error diagnostics (the stderr messages of `lib.rs`, `exec.rs`, `fuzzel.rs`, `ui/app.rs`);
  - `kbinds audit` findings in human-readable form;
  - the default titles and group names of binds (the `groups.rs` dictionary, ~84 actions + the groups «Окна», «Воркспейсы», «Навигация», «Скриншоты», «Система», «Программы», «Прочее»).
- Not localized (intentionally): titles from the user's config (`hotkey-overlay-title`, `title`), the fuzzel prompt `binds> `, JSON fields, `id`, `keys_display`, `niri msg` action names, code comments.
- **BREAKING** (soft): the `title`/`group` values of default binds in the `kbinds list --json` output become locale-dependent (the JSON schema, fields, `id`, and exit codes do not change). User values from the config never change.

## Capabilities

### New Capabilities

- `localization`: selecting the interface language by the system locale — locale detection rules, catalogs, fallback, surface coverage, stability guarantees (`id`, the JSON schema, exit codes, and the prompt do not depend on the language).

### Modified Capabilities

- `center-overlay`: the requirements «The "Center" card» and «Important-keys cheat sheet» pinned the Russian text («Поиск…», «N из M биндов», the footer, «Важные клавиши») — the strings are taken from the localization catalog according to the system language.
- `binds-audit`: the «Audit output» requirement pinned the findings language to Russian — findings are printed in the system locale language.

## Impact

- Code: a new `src/l10n.rs`; changes in `src/groups.rs` (group keys + titles in two languages), `src/main.rs` (clap help), `src/ui/app.rs`, `src/audit.rs`, `src/lib.rs`, `src/exec.rs`, `src/fuzzel.rs`; tests (`src/ui/items.rs`, `src/parser.rs`, `src/audit.rs`, `tests/integration.rs`) — explicit language fixation.
- Dependencies: no new crates — the string catalogs are static, locale detection via `std::env`.
- Specs: the changes do not affect `binds-parsing`, `fuzzel-dmenu` (the item format and the prompt are preserved), exec behavior, exit codes, or the JSON schema.
