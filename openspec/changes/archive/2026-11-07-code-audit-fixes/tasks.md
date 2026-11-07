# Tasks

## 1. P1 — `parse_hex` panic vector (D1)

- [x] 1.1 Rewrite `parse_hex` (`src/ui/theme.rs`) to parse `hex.as_bytes()` 2-byte chunks instead of str slicing; add regression tests: `"#中中"` (6 bytes, non-ASCII), `"#ёёё"`, truncated multibyte input — all return `None`, none panic. Verify: `cargo test -p niri-kbinds theme` passes including the new cases.
- [x] 1.2 Confirm the corrupt-cache scenario end-to-end: unit test feeding `Palette::from_json` a cache with `"primary": "#中中"` falls back per-token to the §6 value. Verify: test asserts `palette.accent() == Palette::fallback().accent()`.

## 2. P2 — Broken-pipe semantics (D3)

- [x] 2.1 Make `print_lines` (`src/lib.rs`) special-case `std::io::ErrorKind::BrokenPipe` as quiet success (write and flush paths); every other write error returns `Err`. Verify: unit test with a `head -1`-style child or an error-injecting writer — broken pipe → `Ok(())`, injected non-pipe error → `Err`.
- [x] 2.2 Remove `std::process::exit(0)` from the dry-run branch of `run_action` (`src/exec.rs`): BrokenPipe → `Ok(())`, other errors → propagate `Err`; keep stderr diagnostics unchanged. Verify: `cargo test exec` passes; `kbinds exec mod-t --dry-run | head -1` exits 0 quietly.
- [x] 2.3 Assert observable exit codes: a real (non-pipe) stdout write failure produces exit code 2 via the standard error path. Verify: test or manual check documented in the task notes. (Integration tests `list_stdout_write_failure_exits_2` on `/dev/full` and `list_broken_pipe_is_quiet_success` in `tests/integration.rs`.)

## 3. P2 — Enum discriminants (D4)

- [x] 3.1 Add `enum ActionKind { Builtin, Spawn, SpawnSh }` and `enum TitleSource { OverlayTitle, ActionName, RawAction, Command }` in `src/model.rs` with `#[serde(rename_all = "kebab-case")]`; replace `Action.kind: String` and `Bind.title_source: String`. Verify: the golden test `json_matches_schema` passes unchanged.
- [x] 3.2 Convert every `kind.as_str()` / `title_source` string comparison to exhaustive matches (`src/groups.rs`, `src/exec.rs`, `src/audit.rs`, `src/fuzzel.rs`, `src/ui/app.rs`); give the enums the string forms `describe()`/`search_text()`/`bind_detail()` need. Verify: `cargo build --all-targets` compiles with zero `as_str` kind remnants (`grep -rn '"spawn-sh"' src/` shows only the enum/mapping definitions); full `cargo test` green. (Remaining `"spawn-sh"` literals: the enum mapping, the niri-msg argv, KDL node-name matching, and the argv-prefix inspection in `spawn_direct`.)

## 4. P2 — Comment/model honesty fixes (D2, D5, D8)

- [x] 4.1 Fix the `cmd_list` comment (`src/lib.rs`): human-readable output is clustered by group name alphabetically (stable within group), not "config order". Verify: the comment matches the code; `cargo test -p niri-kbinds lib` green (no behavior change).
- [x] 4.2 Update `Bind::raw`'s doc comment (`src/model.rs`) to "verbatim diagnostics data; audit rules A1–A3 do not read it" per the amended spec. Verify: doc renders (`cargo doc --no-deps` clean).
- [x] 4.3 Change `parse_bind` (`src/parser.rs`) to return `Bind` directly (no `Result`); adjust callers. Verify: `cargo clippy --all-targets -- -W clippy::unnecessary_wraps` no longer flags it.

## 5. P2 — Shared test fixtures (D6)

- [x] 5.1 Add `#[cfg(test)] pub(crate) mod testutil` to the lib with `sample_bind()` (promoted from `model.rs`) and the `builtin(name)` / `spawn(argv)` / `spawn_sh(line)` builders; delete the per-module duplicates in `model.rs`, `parser.rs` (including the pointless `..Bind { … }.clone()`), `groups.rs`, `fuzzel.rs`, `ui/items.rs`, `ui/app.rs`, switching them to structural-update syntax. Verify: `cargo test` full suite green; `grep -c 'title_source: String::new' src/` finds no test duplicates.

## 6. P3 — Mechanical idiomatic cleanups

- [x] 6.1 `parser.rs`: collapse the double error context (`map_err(anyhow!("parsing {}: {e}"))` + `with_context("parsing …")`) into one, and drop the redundant `canonicalize` (the path is already canonical when `collect_top_nodes` is called from `parse_config`). Verify: `cargo test parser` green; error message for a bad config still names the file once.
- [x] 6.2 `main.rs`: remove the needless `cli.config.clone()` (disjoint field moves). Verify: `cargo clippy -- -W clippy::redundant_clone` silent on it (nursery) and build green.
- [x] 6.3 `audit.rs`: import `BTreeSet` at the top alongside `BTreeMap`; drop the two fully-qualified inline uses. Verify: `cargo fmt --check` + build green.
- [x] 6.4 `lib.rs`: make `print_lines` generic over `impl AsRef<str>` items; drop the `.map(String::from)` in `cmd_list` and `cmd_audit`. Verify: full `cargo test` green; no allocation change observable.
- [x] 6.5 `groups.rs`: implement `is_dictionary_action` as `dictionary_entry(name).is_some()`; remove the `unwrap_or(p).to_string()` allocation in `spawn_group` (compare borrowed `&str`). Verify: `cargo test groups` green.
- [x] 6.6 `ui/app.rs` + `ui/items.rs`: collapse the `find`-then-`position` double scans (`scroll_to_selected`, `move_selection`) into single `position` passes. Verify: `cargo test items app` green (selection/wrap tests unchanged).
- [x] 6.7 `ui/app.rs`: filter keyboard events out of the generic `listen()` subscription so keyboard events are delivered only via `listen_raw`. Verify: overlay behavior unchanged in a manual smoke run (`Esc` closes, arrows move); `cargo test` green.
- [x] 6.8 `exec.rs`: replace `vec!["niri".to_string(), …]` with an array `map(Into::into)`; `fuzzel.rs`: replace `pub type MenuItem = (String, String)` with a named struct (fields `item`, `id`) and update uses; `ui/typography.rs`: `.or(...)` → `.or_else(...)`. Verify: `cargo test` green, `cargo clippy --all-targets -D warnings` clean.

## 7. Lint gating (D7)

- [x] 7.1 Add `[lints.rust] missing_docs = "warn"` to `Cargo.toml`. Verify: `cargo clippy --all-targets` and `cargo doc` clean with the gate on.

## 8. Integration verification

- [x] 8.1 JSON equivalence: `kbinds list --json` output byte-identical before/after the change on the live config (`kbinds list --json > old.json` pre-refactor, diff after). Verify: `diff` empty. (Baseline captured before the first edit: 108 583 bytes, diff empty.)
- [x] 8.2 Full gate: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` green from a clean build.

## Workflow follow-up

- Sync the delta into `openspec/specs/binds-parsing/spec.md` and archive the change after the project's review requirements are satisfied.
