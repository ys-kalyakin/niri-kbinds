# Tasks: kbinds-mvp

Run all build commands in the sandbox after `export PATH="$(ls -d ~/.rustup/toolchains/*/bin | head -1):$PATH"` (the broken-rustup-proxy workaround, SPEC §13 / design D8).

## 1. Crate skeleton

- [x] 1.1 Create `Cargo.toml` (package `niri-kbinds`, bin `kbinds`, `[lib]`+`[[bin]]`; deps: clap v4 derive, kdl 6.x, serde derive + serde_json, anyhow), `src/lib.rs` with the modules `model, parser, normalize, groups, audit, exec, fuzzel` and a thin `src/main.rs` with the clap skeleton of the `list/exec/dmenu/audit` commands and the `--json/--dry-run/--strict/--launcher/--config` flags. Verify: `cargo build` passes, crates download.
- [x] 1.2 Set up cleanliness from day one: `cargo clippy -- -D warnings` and `cargo fmt --check` give zero remarks on the skeleton. Verify: both commands green.

## 2. Model, parser, normalization (M1)

- [x] 2.1 `model.rs`: the `Bind`/`Action` types (`kind`: builtin|spawn|spawn-sh, `name`, `cmd`) with `serde::Serialize` and every field of the §6 schema (`id`, `title`, `title_source`, `actions`, `keys_raw`, `keys_display`, `combo`, `group`, `repeat`, `overlay_title_present`, `file`, `line`, `search`). Verify: a unit test serializes a sample bind and compares against the expected JSON.
- [x] 2.2 `parser.rs`: parsing the `binds { … }` block nodes with the `kdl` crate in v1 mode (design D1), `include` resolved relative to the including file, splicing top-level nodes, cycle protection (a parse error naming the file), preserving `file`/`line`, accumulating unknown properties into `raw`, tolerance of `XF86*`/`condition`. Verify: unit tests on mini KDL strings — block form with `repeat=false` and `hotkey-overlay-title`, several actions in a body, spawn with arguments, spawn-sh, `//` and `/* */` comments, include, cyclic include → error, unknown property → no failure.
- [x] 2.3 `normalize.rs`: `combo` canonicalization (modifier order mod, ctrl, shift, alt; lowercase key), `keys_display` (`Return`→`Enter`, `Left/Right/Up/Down`→`←/→/↑/↓`, `Slash`→`/`), the Cyrillic table — 24 names from §7.2 extracted from `/usr/include/X11/keysymdef.h` (design D3), pinned as `const`; an unknown name stays raw without an error; `keys_raw` preserves the original. Verify: a unit test per each of the 24 Cyrillic pairs + special-key tests + a canonicalization test.
- [x] 2.4 Fixtures `tests/fixtures/`: `edge.kdl` (every §7.1 case: Cyrillic — all 24 names, block form, spawn-sh, `repeat=false`, a duplicated combo, comments, a plain include, unknown properties, `XF86*`), `edge-inc.kdl` (the include target), `cycle-a.kdl`+`cycle-b.kdl` (a cycle), `conflict.kdl` (a minimal A1 conflict), `clean.kdl` (no findings). Verify: the parser on `edge.kdl` yields the expected bind count with correct `file`/`line`; `cycle-a.kdl` yields a parse error (code 2).
- [x] 2.5 `groups.rs`: the dictionary action → (Russian title, group) + prefix rules (design D5), the fallback (Other/Apps), building the `search` field (a lowercase concatenation of title, keys, actions, group), the title priority overlay-title → dictionary → spawn command. Verify: unit tests — group determinism between runs, title priority, the fallback for an unknown action.
- [x] 2.6 Completeness check on the live config (the §7.3 arbiter): an integration `#[ignore]` test — parsing `~/.config/niri/config.kdl` yields exactly the grep-arbiter bind count (144 at the time). Verify: `cargo test -- --ignored` green on the live machine; a mismatch is a blocker — resolve before the next groups.

## 3. CLI: list and exec (M2)

- [x] 3.1 The `list` command: text output (one line per bind — design D9) and `--json` (`serde_json::to_string_pretty`, the §6 schema). Verify: on the live config `kbinds list --json | jq '.[] | select(.group=="Воркспейсы")'` returns only workspace binds; the JSON is valid; every bind has `combo`, `keys_display`, `group`, `search` filled.
- [x] 3.2 `exec.rs`: routing `builtin` → `niri msg action <name> <args…>`, `spawn` → `niri msg action spawn -- <argv…>`, `spawn-sh` → `niri msg action spawn-sh -- "<line>"` (design D2); several actions in order; lookup by `id`, unknown id → code 2. Verify: unit tests of argv building for the three action kinds and the ordering; a live run on a known-safe builtin.
- [x] 3.3 The `--dry-run` flag: printing the final commands to stdout without executing, code 0. Verify: `kbinds exec mod-t --dry-run` prints the correct command and executes nothing (acceptance §15.4).
- [x] 3.4 Exit codes and diagnostics: an IO/parsing/usage/unknown-id error → code 2, a message in stderr, stdout empty (design D7). Verify: running with a nonexistent `--config` and `exec no-such-id` gives code 2 and empty stdout.

## 4. fuzzel dmenu (M3)

- [x] 4.1 `fuzzel.rs`: launching `fuzzel --dmenu -p "binds> "` (items on stdin, the selection from stdout), the item format `<keys_display> — <title>` + the action name, collision dedup with a ` [<id>]` suffix, replacing line breaks with spaces, the reverse mapping item → id → exec, empty stdout (Esc) → code 0 with no actions, fuzzel missing/failing to launch → code 2 (the fuzzel-dmenu spec; design D6; check fuzzel's cancel code against the 1.15 docs at implementation). Verify: unit tests of formatting, dedup, sanitizing and the reverse mapping through a replaceable launcher call.
- [x] 4.2 Live check of scenario S1 on the working session: the user adds `Mod+Shift+Slash { spawn "kbinds" "dmenu"; }`, searching «воркспейс» finds the move-to-workspace binds, Enter executes, Esc does nothing, a second invocation works. Verify: a manual run per DoD M3, the result recorded in the change report.

## 5. Audit (M4)

- [x] 5.1 Research niri's duplicated-combo semantics — reading niri 26.04 docs/sources, not experiments on the live session (SPEC §14.5). Verify: the conclusion (how niri resolves duplicates) is recorded in the change; the A1 finding wording refined if needed, its winner-neutrality preserved.
- [x] 5.2 `audit.rs` rule A1 (error): one canonical `combo` assigned to actions with different commands — the finding lists every variant with `file`/`line`, without claiming a winner; the same action on different keys is not a finding. Verify: unit tests on `conflict.kdl` (a finding with both variants) and on the `Mod+H`/`Mod+Left` pair with one action (empty).
- [x] 5.3 Rule A2 (warn): `spawn`/`spawn-sh` or an out-of-dictionary builtin without `hotkey-overlay-title`; a builtin whose name is in the dictionary is not warned. Verify: unit tests on both branches.
- [x] 5.4 Rule A3 (info, nice-to-have): a static list of ~20–30 common niri actions with no bind at all (screenshots, `toggle-window-floating`, `consume-or-expel-window-*`…). Verify: a unit test on `clean.kdl` yields the expected info-finding list.
- [x] 5.5 Audit output: human-readable in Russian, `--json` (rule id, level, message, bind references), `--strict` → code 1 with error-level findings, no findings → code 0. Verify: on the live config `kbinds audit` finds the `Mod+D` conflict (error), `--strict` gives code 1; `kbinds audit --config tests/fixtures/clean.kdl` — zero findings, code 0 (acceptance §15.6–7).

## 6. README and acceptance (M6)

- [x] 6.1 Research vicinae against current docs (M5 of the SPEC): if there is a supported way to add a result source — record the adapter plan in the README as a separate change; if not — write "use the fuzzel variant" in the README. Verify: the research conclusion and the README entry exist.
- [x] 6.2 README (Russian): installation on Fedora (`cargo install --path .` / copying `target/release/kbinds` into `~/.local/bin`), adding the bind `Mod+Shift+Slash { spawn "kbinds" "dmenu"; }`, the three §4 scenarios, the mode descriptions and non-goals. Verify: the README commands run as written on the target machine.
- [x] 6.3 Final cleanliness: `cargo test` fully green, `cargo clippy -- -D warnings` and `cargo fmt --check` without remarks, `cargo build --release` successful. Verify: all four commands green (acceptance §15.8).
- [x] 6.4 Acceptance run per §15 of the SPEC: the checklist of criteria 1–10 on the live machine (144/144, valid JSON, Cyrillic `Mod+Cyrillic_ve`→`в`, dry-run, S1, the Mod+D audit, cleanliness), results recorded. Verify: all 10 criteria checked, deviations listed explicitly.

## Workflow follow-up

- After artifact review, start the implementation with `/opsx:apply`.
- When the tasks are done — `openspec validate`, then archive the change (`/opsx:archive`); at archive time the deltas of the four capabilities become the main specs.
