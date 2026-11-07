# Tasks

Note (not a task): the rustup proxies in the sandbox are broken — run cargo/clippy/fmt with `export PATH="$(ls -d ~/.rustup/toolchains/*/bin | head -1):$PATH"`.

## 1. Surface skeleton (M1)

- [x] 1.1 Cargo.toml: add `iced` (a version compatible with iced_layershell 0.19.1 — from the crate's Cargo.toml/examples), `iced_layershell = "0.19.1"`, a `[[bin]] name = "kbinds-overlay" path = "src/bin/overlay.rs"` section; in `lib.rs` — `pub mod ui;` with the stub files `src/ui/{app,items,theme}.rs`. Verify: `cargo build` passes, the crate builds from crates.io only.
- [x] 1.2 `src/bin/overlay.rs`: clap arguments `overlay [--config PATH] [--dry-run] [--mode center]` (center is the default), config parsing by the MVP1 library; parse errors → a human-readable message to stderr, exit 2; without `WAYLAND_DISPLAY` → «kbinds-overlay: не найдена Wayland-сессия» ("kbinds-overlay: no Wayland session found"), exit 2, no panics. Verify: `kbinds-overlay --config /нет/такого.kdl` → code 2 and a message; a run with `WAYLAND_DISPLAY` cleared → code 2 and the session text.
- [x] 1.3 Focused output detection: run `niri msg --json focused-output` (the global `--json` before the subcommand), parse the `name` field with serde_json; unavailable/error → stderr, exit 2. Verify: a parser unit test on the JSON sample `{"name":"DP-2",…}`; on the live session it returns `DP-2`.
- [x] 1.4 iced_layershell skeleton: the `Overlay` layer, `KeyboardInteractivity::Exclusive`, anchors on all 4 edges, the target output by name from 1.3, clear-color `rgba(8,6,14,0.55)`, `Esc` closes the application. Verify: a live run of `kbinds overlay` — a scrim on the focused monitor, windows show through, Esc closes with code 0. If the API blocks (no exclusivity/transparency) — stop, come back with facts and options (design D1/D5), do not change the stack silently.

## 2. Pure list logic (items.rs)

- [x] 2.1 Row/group models: expand `Vec<Bind>` into a flat list of visible rows with group headers, the group order — as in MVP1 `groups` (first appearance in the config). Verify: unit tests on a sample of binds (order, group composition, empty config).
- [x] 2.2 Filtering: substring search over the model's `search` field (no new normalizer), empty groups are hidden. Verify: unit tests — an empty query shows all rows; «воркспейс» finds only the matches; «mod+shift»; a Cyrillic query; a group with no matches is hidden.
- [x] 2.3 Navigation: `move_selection` wraps over the visible rows in both directions; on a filter change the selection resets to 0. Verify: unit tests — wrap down from the last row to the first, wrap up, reset after the list narrows.
- [x] 2.4 Keycap rows: build the caps from `keys_display` without new key parsing. Verify: a unit test — `Mod+В` (Cyrillic), arrows/Enter are rendered with the `keys_display` tokens.

## 3. The «Центр» card (M2)

- [x] 3.1 `src/ui/theme.rs`: palette tokens of task spec §6 (scrim `#080610`@55%, card `rgba(21,18,30,0.94)`/radius 20/border `rgba(255,255,255,0.09)`, text `#ece9f4`/`#9b93b3`, accent `#a78bfa`/`rgba(167,139,250,0.16)`, keycap style, radii 20/10/12/6), styles of the card, rows, field, and caps. Verify: `cargo test` (a const test on radii/colors) and a review against the table.
- [x] 3.2 `src/ui/app.rs` + view: a 600px card (max 92% of the width), a search field with autofocus, a grouped list of "title + keycaps" rows, a «N из M биндов» counter, the footer `↑↓ выбор · Enter выполнить · Esc закрыть`, wheel scrolling of the list. Verify: a live run — all 161 binds reachable by scrolling, the look visually matches the mockup, the secondary text is readable.

## 4. Live filter and navigation (M3)

- [x] 4.1 Typing in search → live filter via `items::filter` (a Message enum, state in a single struct); the N counter updates; the selection resets to 0. Verify: a unit test of the "query → visible rows" binding; a live run: typing «воркспейс» filters, including from the ru layout (scenario §10.5 of the task spec), clearing restores everything.
- [x] 4.2 `↑/↓` via `items::move_selection`, the selected row highlighted with the accent, auto-scroll to the selection. Verify: a live run — the selection is visible, the wrap-around works at both ends.
- [x] 4.3 Mouse: a click on a row executes its bind, a click outside the card closes. Verify: a live run in `--dry-run`: a row click prints the command to stderr and closes, a click on the scrim closes without printing.

## 5. Execution and closing (M4)

- [x] 5.1 Enter: dry-run → for each action `exec::display_command(&exec::action_argv(action))` to stderr, closing, exit 0; without dry-run → `exec::exec_bind(bind, false)`, then closing. Verify: live scenario §10.3 of the task spec: `kbinds overlay --dry-run` in the background, «воркспейс», Enter → the command `niri msg action …workspace…` in stderr, exit 0; without dry-run the action actually fires.
- [x] 5.2 All close paths: Esc, a click outside, a successful Enter/dry-run — code 0; no path executes actions except Enter without dry-run. Verify: a live run of all paths in a row, `echo $?` after each = 0.

## 6. Regression, bind, README (M5)

- [x] 6.1 Crate regression: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — clean; `kbinds list/exec/dmenu/audit` work as before (adding the binary did not break the CLI). Verify: a run of all four commands on the live config + green checks.
- [x] 6.2 Bind in niri: `Mod+Shift+Slash` and a duplicate `Mod+Shift+backslash` → `spawn "kbinds" "overlay"` (hotkey-overlay-title="Binds: overlay (kbinds)"); `show-hotkey-overlay` → the free `Mod+Shift+U`; reload the config. Verify: `Mod+Shift+/` opens the overlay (also in the ru layout), show-hotkey-overlay opens on `Mod+Shift+U`; no config errors in `journalctl --user` (not `niri msg request-error` — it returns a stub).
- [x] 6.3 README: installation, the overlay and dmenu binds, an overlay screenshot (`grim -o DP-2 -` or an interactive niri screenshot — non-blocking), differences from `kbinds dmenu`. Verify: the README has all sections, the screenshot is attached.

## Workflow follow-up

- Finish the live scenarios §10.3–10.5 of the task spec (an overlay screenshot for the README, the ru layout) after apply, if grim was missing at the time of 6.3.
- Archive the change after the user's acceptance (`/opsx:archive`).

## 7. The noctalia theme (an addendum at the user's request)

- [x] 7.1 A user noctalia template: `~/.config/noctalia/templates/kbinds-overlay.json` (tokens scrim/surface_container/surface_container_high/surface_container_highest/on_surface/on_surface_variant/outline/outline_variant/primary/shadow/secondary/tertiary) + registration in `~/.config/noctalia/templates.toml` → the output `~/.cache/kbinds/theme.json`; render with `noctalia msg templates-apply`. Verify: `~/.cache/kbinds/theme.json` exists and contains the hex colors of the active scheme (Kanagawa).
- [x] 7.2 `theme.rs`: `Palette::load()` — read the cache (hex → Color, the §6 alphas layered on top), a fallback to the fixed §6 palette when the file is missing/corrupt; the styles take the palette. Verify: unit tests of cache parsing and the fallback; `cargo test` green.
- [x] 7.3 Live check: a screenshot of the overlay — the card in theme colors (compare with the noctalia panel); README — a section on the noctalia template and its installation. Verify: the screenshot attached/updated, the README describes the theme setup.
- [x] 7.4 Fix: the first Esc closes the overlay — `keyboard::listen()` listens only to events with the Ignored status, and the focused search field intercepts Esc (unfocusing, Captured); a switch to `iced::event::listen_raw` for keys (Esc/↑↓ are delivered regardless of widget capture). Verify: the code fix + a repeated interactive check of the close paths together with 5.2.
