# Design

## Context

User-facing strings are scattered across the code with no single mechanism: the overlay (`src/ui/app.rs`), diagnostics (`src/lib.rs`, `src/exec.rs`, `src/fuzzel.rs`, `src/ui/app.rs`), audit findings (`src/audit.rs`), CLI help (`src/main.rs`, clap derive doc-comments), and the largest chunk — the dictionary of default titles/groups (`src/groups.rs`, ~84 actions + 7 groups, prefix rules, and «Прочее» (Other)). The project minimizes dependencies (see the winit pinning in `Cargo.toml`). The language of a process in a Wayland session is set by the standard locale variables.

## Goals / Non-Goals

**Goals:**
- Language selection by the system locale for all user-facing strings, with no new crates.
- A compile-time guarantee of catalog completeness (both languages in a single record).
- Deterministic tests: the language is passed explicitly, without env manipulation in tests.

**Non-Goals:**
- Live language switching in a running process; the locale is fixed at startup.
- Localizing user config data and comments/documentation.
- Localizing JSON field names, `id`, the fuzzel prompt, `niri` action names.
- Languages beyond `ru`/`en` (adding a new language must come down to one new record per message).

## Decisions

### D1. Own static catalog, no crates
`src/l10n.rs` with `pub enum Lang { En, Ru }`. A single catalog module; messages are records of "key + a translation into both languages". Alternatives: `fluent-rs` (an FTL runtime, overkill for 2 languages), `rust-i18n` (code generation, an external dependency) — rejected: the scope is small and the project keeps its dependency tree minimal.

### D2. Message enum + `tr(Msg)`, parameterized ones as functions
Simple strings: `pub enum Msg { SearchPlaceholder, FooterHints, … }`, `pub fn tr(lang: Lang, msg: Msg) -> &'static str` — a single `match`; a missing translation in either locale does not compile. Strings with substitutions (the counter «{visible} из {total}», diagnostics with paths/ids) are typed functions in `l10n` (`pub fn counter(lang, visible, total) -> String` and similar), because dynamic `format!` arguments in Rust require a literal format string. The alternative (string keys + `HashMap`) is rejected: a key typo is caught only at runtime; the enum catches it at compile time.

### D3. Language: a global `OnceLock<Lang>` for UI/CLI, an explicit `Lang` argument for the core
`l10n::init()` at the start of `main()` (and in `src/bin/overlay.rs`, if it is a separate entry point) reads `LC_ALL` → `LC_MESSAGES` → `LANG`, parses the subtag up to `_`/`-`/`.`, `C`/`POSIX`/empty/unknown → `En`. Core functions that produce strings (`groups.rs` title/group resolution, `audit.rs` finding construction) take `Lang` as a parameter; UI and CLI read the global. Rationale: `env::set_var` in tests is racy and `unsafe` in newer editions — tests call the core with an explicit `Lang::Ru`/`Lang::En`.

### D4. The `groups.rs` dictionary: a group key + a pair of titles
`ACTION_TITLES` → `&[(&str action, Group, &str title_en, &str title_ru)]`, where `enum Group { Windows, Workspaces, Navigation, Screenshots, System, Apps, Other }`; `group_by_prefix`/`spawn_group` return `Group`; `group_name(Group, Lang) -> &'static str` localizes at output time. `Bind.group` remains a `String` — filled with the localized name when the model is built (the language is already fixed), so the JSON schema, the `cmd_list` sort by `group`, and the `search_text` search (which includes title/group) keep working without model changes. The "action name is in the dictionary" check for A2 is by action name; the locale does not affect it.

### D5. CLI: keep derive, apply the texts via the builder
In `main()`: `let mut cmd = Cli::command(); l10n::apply_cli_texts(&mut cmd, lang);` — overriding the `about` of the root command, the subcommands, and argument help via the builder API (`find_subcommand_mut`, `mut_arg`), then `cmd.get_matches()` + `Cli::from_arg_matches`. Alternatives: a full switch to the builder API (a large diff with no gain), localization inside derive (not supported) — rejected. Doc-comments in `main.rs` become internal documentation; the authoritative texts live in `l10n`.

### D6. Diagnostics and audit findings via `tr`/functions
All `.context(...)`/`bail!`/`eprintln!` with human-readable text are replaced with `l10n` calls (substitution formats are typed functions). Audit findings assemble their messages at construction time with an explicit `Lang`; `--json` writes the same message text (a field value, not a field name).

## Risks / Trade-offs

- [The `title`/`group` values of default binds in `list --json` are locale-dependent] → documented in the proposal as a soft BREAKING; fields/`id`/exit codes are stable; user values do not change. Consumers that need a stable title can rely on `overlay_title` or `id`.
- [The group order in `list`/the overlay is determined by sorting on the localized name] → the order is stable within a locale; it may differ between locales — acceptable for human-readable output (the spec requires stability within a run, not across locales).
- [~120 translations — a risk of semantic errors in en] → acceptance by running with `LANG=ru_RU.UTF-8` and `LC_ALL=en_US.UTF-8` against a surface checklist; completeness is guaranteed by compilation.
- [Tests that pin Russian defaults (`items.rs`, `parser.rs`, `audit.rs`, integration)] → re-check and fix `Lang` explicitly in the calls; do not touch env in tests.
- [Overriding clap texts diverges from doc-comments] → reduce doc-comments to short internal descriptions so there are no two "sources of truth".

## Migration Plan

A single binary; no data or config migration is required. Rollback is a commit revert. After implementation: `cargo test`, then manual acceptance of `--help`, `list`, `audit`, `dmenu`, `overlay` under the two locales.

## Open Questions

None — the decisions cover all surfaces; adding languages later does not change the approach.
