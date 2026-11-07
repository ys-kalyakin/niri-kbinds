# Tasks

## 1. Typography module

- [x] 1.1 Create `src/ui/typography.rs` (registered in `src/ui/mod.rs`): base-size consts `ROW_TITLE=18`, `KEYCAP=14`, `SEARCH=20`, `GROUP_HEADER=15`, `CHEATSHEET_HEADER=15`, `COUNTER_FOOTER=13`, clamp bounds `MIN_SCALE=0.5` / `MAX_SCALE=3.0`, and the pure functions `resolve(cli: Option<f32>, cache: Option<f32>) -> f32` (priority cli → cache → 1.0, then clamp) and `cache_scale(json: &str) -> Option<f32>` (only a finite positive number is valid). Verify: new unit tests pass — priority order, clamping at both ends, absent/malformed/zero/negative/NaN token → 1.0, and a `consts_match_spec_table` test pinning the six sizes.

## 2. CLI flag and plumbing

- [x] 2.1 Add `font_scale: Option<f32>` as `--font-scale SCALE` to `Command::Overlay` in `src/main.rs` and to `Args` in `src/bin/overlay.rs`; add the raw `Option<f32>` field to `ui::app::OverlayOptions`, resolve it once inside `ui::app::run` via `typography::resolve(cli, typography::cache_scale(...))` and store the result in the `Overlay` state. Verify: `cargo build` succeeds; `kbinds overlay --font-scale abc` prints a usage error and exits 2; `--help` lists the flag for both entry points.
- [x] 2.2 Add ru/en catalog entries for the `--font-scale` help text in the l10n tables behind `apply_cli_texts` / `apply_overlay_cli_texts`. Verify: `kbinds overlay --help` under `LANG=ru_RU.UTF-8` and under `LC_ALL=en_US.UTF-8` shows the localized help; existing l10n tests still pass.
- [x] 2.3 Update README: `kbinds overlay [--font-scale SCALE]` in the Commands block and a sentence on the `font_scale` token in `~/.cache/kbinds/theme.json` (CLI flag wins; 0.5–3.0 clamp). Verify: the README usage line matches the actual `--help` output.

## 3. Overlay rendering

- [x] 3.1 In `src/ui/app.rs` replace every hardcoded text size (search 16, group headers 12, counter/footer 11, row titles 14, keycaps 11, cheat-sheet header 12, cheat-sheet row titles 14) with the `typography` const × `state.scale`. Verify: `cargo build` succeeds and no raw `.size(<integer literal>)` remains in `app.rs`; the `consts_match_spec_table` test still passes.
- [x] 3.2 Give both scrollables (`LIST_SCROLL`, `CHEATSHEET_SCROLL`) `.direction(scrollable::Direction::Vertical(scrollable::Scrollbar::hidden()))`. Verify: `cargo build` succeeds; a quick live run shows no scrollbar on either panel's edge.
- [x] 3.3 (added during apply, user feedback) Scale the card geometry with the font scale: `items::card_rect(window, has_cheatsheet, font_scale)` multiplies the 600/900 width and 640 height by the scale (92% cap kept), `is_outside_card` passes it through, the `CHEATSHEET_WIDTH` call site multiplies too; update the geometry unit tests (1.0 numbers unchanged, 1.5 growth, cap case). Verify: `cargo test items` green; a live screenshot at `--font-scale 1.5` shows the card grown and the text fitting.
- [x] 3.4 (added during acceptance, user feedback) Widen the cheat-sheet panel: `CHEATSHEET_WIDTH` 300→400 and `CARD_WIDTH_CHEATSHEET` 900→1000 (the left list column keeps its width), comments and geometry tests updated. Verify: `cargo test items` green; a live screenshot shows «Важные клавиши» rows on one line (title + keycaps), nothing wrapped crooked.

## 4. Integration verification

- [x] 4.1 Run the full check suite: `cargo test && cargo clippy -- -D warnings && cargo fmt --check` (use the toolchains PATH workaround — rustup proxies are broken on this machine). Verify: all three green.
- [x] 4.2 Interactive acceptance on the live niri session: at `--font-scale 1.0` the base look (18/14/20/15/13) with no scrollbar and full-width rows; wheel scrolls both panels; `↑↓` keeps the selection visible; typing snaps the list to top; `--font-scale 1.5` scales all text; `--font-scale 10` clamps to 3×; `"font_scale": 1.2` in `~/.cache/kbinds/theme.json` without the flag gives 1.2×; the flag beats the token; `kbinds-overlay --font-scale 1.5` matches `kbinds overlay --font-scale 1.5`. Verify: every check observed on screen (use `--dry-run` where an execute would fire).

## Workflow follow-up

- After review, archive the change (syncs the delta into `openspec/specs/center-overlay/spec.md`).
- Optionally re-shoot `docs/screenshot-overlay.png` — the old screenshot shows the smaller pre-change font.
