# Tasks

## 1. Pure logic and geometry (src/ui/items.rs)

- [x] 1.1 Add `cheatsheet_binds(&[Bind]) -> Vec<usize>` — indices of binds with `overlay_title_present == true` in config order; unit tests: selection from a mixed set, empty config, order matches the config; `cargo test` green
- [x] 1.2 Extend `card_rect(window)` → `card_rect(window, has_cheatsheet: bool)` (900 when true, 600 when false; the 92% cap and the height unchanged) and `is_outside_card` with the same parameter; update the existing geometry/hit-test tests for both values of the flag; `cargo test` green

## 2. Layout and state (src/ui/app.rs)

- [x] 2.1 A `cheatsheet: Vec<usize>` field in `Overlay`, filled in `boot` via `items::cheatsheet_binds`; `update()` does not touch the panel (no new `Message` variants); `cargo clippy -- -D warnings` and `cargo build` clean
- [x] 2.2 Layout per design D2/D3: card `row![левая колонка, разделитель 1px (outline_variant), панель]`, width 900 (92% cap) or 600 without the panel; the panel — a «Важные клавиши» heading (12px, secondary color) + a `scrollable` (id `kbinds-overlay-cheatsheet`) with compact "title + keycaps" rows (4px vertical padding) from `items::keycaps`; `cargo build` clean
- [x] 2.3 The panel is read-only: rows without `mouse_area` and without selection highlighting, clicking a panel row neither executes an action nor closes the overlay (a click inside the card), `↑↓`/Enter move only the left list; verify with a `--dry-run` run on the live session and a screenshot
- [x] 2.4 Independent scrolling: the wheel over the panel scrolls the panel, over the list — the list; "click outside the card" with width 900 closes correctly (hit-test from 1.2); live run

## 3. Checks and documents

- [x] 3.1 Full gate: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — clean; the MVP1 command regression is unaffected (`kbinds list --json | wc -l` unchanged)
- [x] 3.2 Live spec scenarios on the session: the panel shows exactly the binds with `hotkey-overlay-title` (16 on the live config, Cyrillic as characters), the «воркспейс» filter does not change the panel, the long panel scrolls; screenshot for the README
- [x] 3.3 README: Overlay section — a paragraph about the right panel (composition = `hotkey-overlay-title`, read-only) and a new screenshot; `kbinds-overlay`/`kbinds` reinstalled (`cargo install --path .`, working around the target machine's rustup proxy via PATH toolchains)
- [x] 3.4 `openspec validate kbinds-overlay-cheatsheet --strict` — clean (after archiving `kbinds-overlay-mvp2`, see the follow-up)

## Workflow follow-up

- Before applying this change, archive `kbinds-overlay-mvp2` (`/opsx:archive`) — the "MODIFIED" delta references the `center-overlay` capability, which lands in the main specs upon archiving.
- After acceptance, archive this change (`/opsx:archive`) and verify that `center-overlay` in the main specs contains the modified requirements.
