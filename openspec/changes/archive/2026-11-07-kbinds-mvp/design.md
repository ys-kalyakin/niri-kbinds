# Design: kbinds-mvp

## Context

The project is greenfield: no code yet, there is SPEC.md (a self-contained task specification with verified facts about the target machine) and proposal.md with the specs of four capabilities. Target machine: Fedora 45, niri 26.04 (session active), fuzzel 1.15.0, rustc/cargo 1.99.1 (rustup proxies are broken in the sandbox — see Risks). Live config: `~/.config/niri/config.kdl`, 144 binds (the `grep -cE '^\s*Mod'` arbiter confirmed), `include "./noctalia.kdl"`, 40 binds with `Cyrillic_*`, a live `Mod+D` ×2 conflict.

Facts verified this session (the basis for decisions):
- `niri msg action spawn -- <COMMAND>...` — confirmed (`--help`).
- `niri msg action spawn-sh -- <COMMAND>` — confirmed: accepts a single shell line.
- niri 26.04 parses the config with the **knuffel 3.2.0** crate (niri-config/Cargo.toml), which wraps `kdl` 2.x, i.e. the **KDL v1** dialect.
- `xkbcli` is **absent** on the machine; `/usr/include/X11/keysymdef.h` **is present** — the Cyrillic table reference per §7.2 of the SPEC.

## Goals / Non-Goals

**Goals:**
- One static `kbinds` binary, synchronous, no network or runtime dependencies.
- Parsing/model logic lives in the crate's library modules (`lib.rs` + modules), a thin `main.rs`; the v2 overlay reuses the lib.
- The parser dialect must read the live config; the completeness arbiter is 144/144 (§7.3 of the SPEC).

**Non-Goals:**
- Everything fixed in the proposal (v2 GUI overlay, config editing, a daemon, other compositors, the vicinae adapter).
- KDL v2 as the target dialect — the target dialect is defined by what niri reads.

## Decisions

### D1. Parser: the `kdl` crate in KDL v1 mode
niri 26.04 uses knuffel 3.2 → kdl 2.x → **KDL v1** documents. We take the latest major `kdl` crate (6.x) and parse in v1 mode (`parse_v1`) to avoid dragging in an outdated major version just for the dialect. Alternatives: knuffel 3.2 (the same parser niri uses) — rejected: the derive schema resists the "unknown properties must not break parsing" requirement; kdl 2.x directly — the fallback if the fresh crate's v1 mode rejects the live config (see Risks). The decision arbiter is the 144/144 integration check on the live config; both versions satisfy it, so we take the fresh one.

### D2. Action execution — only via `niri msg action`
- `builtin` → `niri msg action <name> <args…>`.
- `spawn` → `niri msg action spawn -- <argv…>` (syntax confirmed).
- `spawn-sh` → `niri msg action spawn-sh -- "<line>"` (confirmed: a single shell-line argument).
- Fallback when niri routing is unavailable (niri msg does not respond): a direct `std::process::Command` with `process_group(0)` and stdio to null, `#[cfg(unix)]` — as §8 of the SPEC allows. Dry-run prints the final command (the whole argv) to stdout.
The alternative (always spawn directly) — rejected: niri semantics (working directory, env) differ, and the SPEC requires routing through niri.

### D3. Cyrillic table — from keysymdef.h only
`xkbcli` is absent, `/usr/include/X11/keysymdef.h` is present. At implementation time: extract the character codes for the 24 names of §7.2 of the SPEC (`a, be, de, el, em, er, es, ghe, ha, hardsign, i, ie, ka, o, sha, shcha, shorti, slash, tse, u, ve, yeru, yu, ze`) from keysymdef.h (format `XK_Cyrillic_ve 0x0432`), pin them as a `const` table in `normalize.rs`, with a unit test per "name → character" pair. Check: `Cyrillic_ve → в`, `Cyrillic_a → а` (unicode U+0432/U+0430), cross-check the chosen characters against a manual extraction from keysymdef.h at review. An unknown name stays raw, without an error.

### D4. Bind identifiers
`id` = kebab-case of the canonical combo (`mod+shift+w` → `mod-shift-w`; `+` → `-`, the other combo characters are already safe). Duplicate combos (legal in the config — the live `Mod+D`) get a `-2`, `-3`… suffix in model order — deterministic between runs. The model is built as an ordered list, `exec` looks up by exact id.

### D5. Grouping and the title dictionary
One table in `groups.rs`: action name → (Russian title, group). Two matching levels: exact name → then prefix rules (`focus-*` → Navigation, `move-*`/`consume-*`/`expel-*` → Windows, `workspace*` → Workspaces, `screenshot*` → Screenshots, spawn with known launcher programs → System). Fallback: unknown builtin → the Other group, title = the raw entry; spawn → Apps, title = the command as a line. The table starts from these rules and is tuned against the live config at implementation time (the real 144 actions are visible there); the mechanism is deterministic, the dictionary is extensible without changing the spec.

### D6. dmenu protocol and item uniqueness
`fuzzel --dmenu -p "binds> "`, items `<keys_display> — <title>` + the action name. An item must be a unique string for the reverse mapping to the id: on collision (possible with identical keys and identical overlay-titles on different actions) the item gets a ` [<id>]` disambiguator. Line breaks in items are replaced with spaces. Reverse mapping: exact item-string match → id → exec. Empty fuzzel stdout (Esc) → exit 0 with no actions. The exact fuzzel cancel behavior (empty stdout vs exit code) is checked against the fuzzel 1.15 docs at implementation — the empty-stdout branch covers both cases.

### D7. Errors and exit codes
`anyhow::Result` at the top, `with_context` on IO/parsing; no panics, no `unsafe`. `main` maps: success → 0; `--strict` with `error` findings → 1; any error (IO/parsing/usage/unknown id/fuzzel unavailable) → 2. Diagnostics go to stderr. Error printing is `{:?}` via `anyhow`/`eprintln!`, no `unwrap` on user input.

### D8. Build and sandbox environment
Cargo.toml: package `niri-kbinds`, bin `kbinds`, `[lib]` + `[[bin]]`; deps: clap v4 (derive), kdl 6.x, serde (derive) + serde_json, anyhow. `cargo install --path .` → `~/.cargo/bin/kbinds`. In the sandbox the toolchain is invoked directly: `export PATH="$(ls -d ~/.rustup/toolchains/*/bin | head -1):$PATH"` (verified by the SPEC §13: cargo 1.99.1 works). The first build needs crates.io access.

### D9. What `list` does without `--json` (assumption)
The SPEC does not specify the format; we adopt: one line per bind — `[group] keys_display — title (id)`, sorted by group and config order. The assumption is recorded; the behavior does not conflict with the spec (the spec requires JSON only for `--json`).

## Risks / Trade-offs

- [kdl 6.x in v1 mode may reject the live config (v1/v2 dialect subtleties)] → The arbiter is the 144/144 test on the live config on day one; the fallback plan is rolling back to the `kdl` 2.x crate (niri's dialect verbatim), the API is nearly identical for our read subset.
- [Duplicated combo in niri: the "winner" semantics are unverified] → A1 deliberately does not name a winner (a spec requirement); reading niri docs/sources is a research task in tasks.md — it only affects the finding's wording.
- [A manually extracted Cyrillic table may contain an error] → the source is a file on disk (keysymdef.h), not memory; a unit test per each of the 24 pairs; a sample cross-check at review.
- [The group dictionary is incomplete for the 144 live binds] → the fallback Other group and raw-entry title guarantee correct output without panics; the dictionary is extensible without changing the spec.
- [Broken rustup proxies in the sandbox] → the D8 workaround is verified by the SPEC; tasks never run `cargo` without export PATH.
- [The fuzzel scenario S1 needs a live session] → automated tests cover the protocol (stdin/stdout); the manual S1 check is a separate task per DoD M3.

## Migration Plan

Not applicable (a new component, no backward compatibility). Installation: `cargo install --path .` or copying `target/release/kbinds` into `~/.local/bin`; the niri bind follows the README instructions (`Mod+Shift+Slash { spawn "kbinds" "dmenu"; }`). Rollback — delete the binary.

## Open Questions

- niri's semantics for a duplicated combo (doc/source reading) — only polishes the A1 wording, does not change the approach.
- Exact fuzzel 1.15 cancel codes — the empty-stdout branch already covers the behavior; the refinement is cosmetic.
- Vicinae API — research task M5; the outcome (an adapter or "not supported" in the README) does not block the MVP and is out of this change's scope.
