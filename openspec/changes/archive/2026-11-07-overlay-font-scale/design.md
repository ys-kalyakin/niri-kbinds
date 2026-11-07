# Design

## Context

All overlay text sizes are hardcoded literals in `src/ui/app.rs` (`.size(14)`, `.size(11)`, …), the two `scrollable` widgets (bind list `LIST_SCROLL`, cheat sheet `CHEATSHEET_SCROLL`) draw iced's default 10px scrollbar, and the overlay already reads optional theme tokens from `~/.cache/kbinds/theme.json` (`Palette::load`, per-token fallback). Two entry points (`kbinds overlay` in `src/main.rs`, standalone `src/bin/overlay.rs`) share `ui::app::OverlayOptions`. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- One knob (`scale`) driving every text size, resolved CLI → theme cache → 1.0, clamped to 0.5–3.0.
- Base sizes ~25% larger at scale 1.0 (the new default look).
- No visible scrollbar on either panel; wheel/keyboard scrolling behavior unchanged.
- Pure, unit-testable resolution logic (no GUI needed to test it).

**Non-Goals:**

- Paddings and radii do not scale with the font (only text and the card box grow).
- No live/interactive size adjustment inside the overlay; no per-element size overrides.
- No new config file format — the token rides the existing theme cache.

## Decisions

**D1 — Scale lives in a new `src/ui/typography.rs`, not in `Palette`.**
`Palette` is a color role struct (Copy, captured by view closures); text metrics are a separate concern. The module holds the base-size consts, the clamp bounds, and two pure functions:

- `resolve(cli: Option<f32>, cache: Option<f32>) -> f32` — priority `cli → cache → 1.0`, then clamp into `[0.5, 3.0]`;
- `cache_scale(json: &str) -> Option<f32>` — extracts `font_scale` from the cache JSON; only a finite, positive number is valid, anything else (absent, string, null, NaN) is `None`.

`app::run` reads the CLI value from `OverlayOptions.font_scale: Option<f32>`, calls `cache_scale` on the same cache file `Palette` reads (one extra `read_to_string` + parse per launch — negligible), and stores the resolved `f32` in the `Overlay` state. View code renders `.size(typography::ROW_TITLE * state.scale)` etc. Alternative rejected: folding the scale into `Palette` (muddies a color struct every style closure captures).

**D2 — Base sizes (scale 1.0), matching the spec table:**

| Element | Old | New |
|---|---|---|
| Bind row title / cheat-sheet row title | 14 | 18 |
| Keycap label (mono) | 11 | 14 |
| Search field text | 16 | 20 |
| Group header / cheat-sheet header | 12 | 15 |
| Counter / footer hints | 11 | 13 |

Integers at scale 1.0 for crisp rendering; fractional results at other scales are fine (iced takes `f32` sizes). A `radii_match_spec_table`-style unit test pins the consts to the spec table.

**D3 — Scrollbar: `scrollable::Scrollbar::hidden()`.**
iced 0.14 ships exactly this: "Create a Scrollbar with zero width to allow a Scrollable to scroll without a visible scroller." Both widgets get `.direction(scrollable::Direction::Vertical(scrollable::Scrollbar::hidden()))`. The scrollbar is drawn over the content at the right edge, so zero width reserves no space and rows keep the full panel width. Wheel events, `operation::snap_to` (filter reset) and the selection-follow logic are independent of the scrollbar and stay untouched. Alternative rejected: transparent rail styling (`vertical_rail.background = None`) — the 10px rail still reserves its hit area at the edge.

**D4 — CLI plumbing via `OverlayOptions`.**
Both parsers (`main.rs` `Overlay` variant, `bin/overlay.rs`) gain `#[arg(long, value_name = "SCALE")] font_scale: Option<f32>`; a malformed value is a standard clap usage error (exit 2), consistent with every other flag. The raw `Option<f32>` travels in `OverlayOptions`; resolution happens once inside `ui::app::run` so both binaries share identical behavior with no duplicated logic. Help texts go through the l10n catalog (`apply_cli_texts` / `apply_overlay_cli_texts`) — ru/en entries added like for the existing flags.

**D5 — Cache token semantics follow the color-token pattern.**
`font_scale` is optional and per-token: absent, malformed, non-numeric, zero, negative, or non-finite → `None` → falls through to 1.0 (or the CLI value). No error is reported; older noctalia templates that don't emit the key keep working unchanged.

**D6 — Card geometry scales with the font scale (added during apply).**
With the +25% base fonts the fixed 900×640 card no longer fits its content comfortably, per user feedback. `card_rect` and the `CHEATSHEET_WIDTH` call site take the resolved scale: 600/1000 width, 640 height, 400 panel all multiply by it, keeping the 92% cap in each dimension. The outside-click hit test calls the same `card_rect`, so layout and hit testing stay one source of truth (design D2) for free. Alternatives rejected: keeping the card fixed and shrinking fonts to fit (defeats the change), and scaling paddings/radii too (more churn, no fit benefit — rows grow with their text anyway; only the card box was the bottleneck). At extreme scales the 92% cap bites and the panels scroll — same behavior as today on small monitors. During acceptance the user also asked to widen the cheat-sheet panel itself (titles + keycaps wrapped crooked at 300px): panel base 300→400px, card base 900→1000px, the left list column keeps its width.

## Risks / Trade-offs

- [Larger text in a fixed card → cramped/overflowing content] → Resolved by D6: geometry scales with the same factor (raised by the user during apply).
- [At large scales (near 3.0) long titles/keycap rows can crowd the cheat-sheet panel] → iced text wraps, rows grow taller and scroll; the panel width scales too; extremes are clamped.
- [Theme-cache token drift between noctalia template versions] → Only strict numeric acceptance (D5); anything else is silently ignored like an unknown color token.
- [Two reads of the same cache file at boot] → Negligible (once per launch); keeping `typography` independent of `Palette` buys testability.

## Migration Plan

None needed: the flag is optional, the token is optional, and without both the overlay renders the new base sizes at scale 1.0. Rollback = revert the commit; the cache JSON gains only an optional key.

## Open Questions

None — the config mechanism (CLI flag + theme token), knob form (multiplier), and default bump (+25%) were confirmed with the user before planning.
