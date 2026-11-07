# Acceptance run of SPEC §15 — kbinds-mvp (2026-11-07)

Binary: `target/release/kbinds` (release, 1.7 MB). Tests: 44 unit + 2 integration
green, 1 `#[ignore]` test on the live config — green. `cargo clippy -- -D warnings`
and `cargo fmt --check` — clean.

| # | §15 criterion | Status |
|---|---|---|
| 1 | `list` on the live config = the grep arbiter | ✅ **with a caveat**: all binds **159** = the full grep over the config's key prefixes; Mod-binds **144** = the literal SPEC grep `grep -cE '^\s*Mod'`. The SPEC arbiter did not see 15 binds (10 XF86, Alt+Tab, Print ×3, Ctrl+Print, Alt+Print, Ctrl+Alt+Delete) |
| 2 | `list --json` — valid JSON, the §6 schema, combo/keys_display/group/search filled | ✅ (`jq -e .` passes; the `Mod+Cyrillic_ve` sample — all fields) |
| 3 | Cyrillic as a character: `Mod+Cyrillic_ve` → `Mod+в` | ✅ **with a caveat**: → `Mod+в` (lowercase). The table was captured from `/usr/include/X11/keysymdef.h`: `XK_Cyrillic_ve = 0x06d7 = U+0432 CYRILLIC SMALL LETTER VE`. The SPEC example wrote the capital «В», but the reference keysyms of the base block are lowercase letters; the §7.2 mandated procedure prevails |
| 4 | `exec mod-t --dry-run` prints the command, executes nothing | ✅ `niri msg action spawn -- kitty`, exit 0; builtin: `niri msg action close-window` |
| 5 | `dmenu`: searching «воркспейс» → Enter executes, Esc — nothing, repeat works | ⏳ **handed to the user** (task 4.2): needs an interactive session. The protocol is covered by unit tests (item format, dedup, reverse mapping, empty stdout → 0 without actions); the fuzzel 1.15 man page does not document exit codes — the empty-stdout branch covers cancellation |
| 6 | `audit` on the live config finds `Mod+D` (error), `--strict` → exit 1 | ✅ **with a caveat**: in the current live config the duplicate has already been removed by the user (the `vicinae toggle` bind is commented out, line 213) → the live audit yields **0** A1 errors, `--strict` exit 0. The A1 demo — on `tests/fixtures/conflict.kdl`: the finding lists both variants with files/lines, `--strict` → exit 1. Duplicate semantics researched (notes/niri-duplicate-binds.md): niri 26.04 rejects a config with a duplicate entirely |
| 7 | Negative test: `clean.kdl` — zero findings, exit 0 | ✅ «Находок нет.» ("No findings."), exit 0 |
| 8 | `cargo test` green, clippy/fmt clean, `cargo build --release` | ✅ 44+2 tests, `-D warnings` clean, fmt clean, release built |
| 9 | README: installation on Fedora, the niri bind, the three §4 scenarios | ✅ README.md (Russian) |
| 10 | No command requires the network or crashes on unexpected properties | ✅ the edge fixture (unknown properties, XF86, condition — parses); the build needs crates.io only once |

## Additionally verified

- EPIPE (`kbinds list | head`) — a quiet exit 0 without panics (SPEC §16: no panics).
- Exit codes: 0 success / 1 strict findings / 2 error (a nonexistent config, an unknown id — verified, stdout empty, diagnostics in stderr).
- A2 on the live config: 15 warnings (XF86 binds and spawn without overlay-title) — expected.
- The live config is a symlink into dotfiles; the `file` field shows the canonical path.

## Remnants

- Task 4.2 (S1 by hand) — on the user: add
  `Mod+Shift+Slash { spawn "kbinds" "dmenu"; }` to config.kdl and check search/Enter/Esc.
