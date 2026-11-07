# Tasks

## 1. l10n core

- [x] 1.1 Create `src/l10n.rs`: `enum Lang { En, Ru }`, `detect()` (`LC_ALL` → `LC_MESSAGES` → `LANG`, the subtag up to `_`/`-`/`.`, `C`/`POSIX`/empty/unknown → `En`), a global `OnceLock<Lang>` + `init()`; unit tests of detection over a table of values (`ru_RU.UTF-8`→Ru, `en_US`→En, `de_DE`→En, `C`→En, empty→En, `LC_ALL` priority over `LANG`) — `cargo test l10n`
- [x] 1.2 Create the catalog: `enum Msg` and `tr(Lang, Msg) -> &'static str` for simple strings, typed functions for substitutions (the counter, diagnostics with `{path}`/`{id}`/`{code}`/`{launcher}` and the like); fill in the ru/en translations for all surfaces from the proposal (the overlay, CLI help, the diagnostics of `lib.rs`/`exec.rs`/`fuzzel.rs`/`ui/app.rs`, audit findings); verify that a new `Msg` variant without a translation fails the build

## 2. Group and title dictionary

- [x] 2.1 Convert `groups.rs` to the D4 scheme: `enum Group`, the `ACTION_TITLES: &[(&str, Group, &str en, &str ru)]` table, `group_by_prefix`/`spawn_group` return `Group`, `group_name(Group, Lang)`; title/group resolution takes `Lang`; unit test: `close-window` under `En` → "Close window"/the windows group, under `Ru` → «Закрыть окно»/«Окна» — `cargo test groups`
- [x] 2.2 Fix the dependent tests (`items.rs`, `parser.rs`, integration) by fixing `Lang` explicitly in the calls, without touching env; `cargo test` green

## 3. User-facing surfaces

- [x] 3.1 Overlay `ui/app.rs`: the search placeholder, the counter, the footer, the cheat-sheet title, and error contexts via `l10n` (the language from the global `init()`); running `kbinds overlay --dry-run` under `LANG=ru_RU.UTF-8` shows a Russian UI, under `LC_ALL=en_US.UTF-8` an English one
- [x] 3.2 Diagnostics `lib.rs`/`exec.rs`/`fuzzel.rs`: `bail!`/`.context`/`eprintln!` via `l10n`; test: `exec nonexistent-id` under `en` gives an English message on stderr, exit code 2
- [x] 3.3 Findings `audit.rs`: the A1–A3 rule messages and the summaries via `l10n` with an explicit `Lang`; test: an A2 finding on the fixture under `En` and `Ru` gives the same rule id and level, different text languages — `cargo test audit`
- [x] 3.4 CLI help in `main.rs`: `init()` at the start of `main`, `l10n::apply_cli_texts(&mut cmd, lang)` before `get_matches` (the root about, the subcommands, flag help); reduce doc-comments to internal descriptions; running `kbinds --help` under `ru_RU.UTF-8` and `en_US.UTF-8` shows the respective texts

## 4. Integration acceptance

- [x] 4.1 Run the whole CLI under the two locales: `list`, `list --json` (the field schema and `id` identical, the `title`/`group` of defaults differ), `audit`, `audit --strict` (exit code 1 regardless of language), `dmenu` with fuzzel (the `binds> ` prompt and the item format unchanged), `overlay` — the checklist against the spec scenarios is recorded in the task result
- [x] 4.2 `cargo test` and `cargo clippy` with no regressions; the full set of delta-spec scenarios (`localization`, `center-overlay`, `binds-audit`) passes manually or via automated tests

## Workflow follow-up

- Archiving the change and syncing the deltas into the main specs — a separate request via `/opsx:archive`.
