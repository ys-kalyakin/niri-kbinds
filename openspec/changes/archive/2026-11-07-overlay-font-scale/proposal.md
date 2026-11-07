# Proposal

## Why

The overlay's text sizes are hardcoded and read small on a 1440p desktop (row title 14px, keycaps 11px, footer 11px), and there is no way to adjust them without recompiling. On top of that, both scrollable panels draw iced's default 10px scrollbar — visual noise for a launcher-style overlay that is already scrolled with the wheel and follows the keyboard selection.

## What Changes

- Raise the base text sizes by ~25% (the look at scale 1.0): row titles 14→18, keycap labels 11→14 (mono), search field 16→20, group headers 12→15, cheatsheet header 12→15, visible-binds counter and footer hints 11→13.
- Introduce a font scale: every text size in the overlay is `base × scale`.
- The scale is resolved with the priority: `--font-scale` CLI flag (both the `kbinds overlay` subcommand and the `kbinds-overlay` binary) → `font_scale` token in the noctalia theme cache `~/.cache/kbinds/theme.json` → default `1.0`. A malformed/missing token falls back per-token, consistent with the color tokens.
- Accepted scale range is 0.5–3.0; values outside it are clamped. A malformed CLI value is a clap error (exit 2), like other flags.
- Remove the visible scrollbar from both scrollable panels (the bind list and the cheat sheet): mouse-wheel scrolling, `↑↓` selection-follow scrolling, and snap-to-top after a filter change all keep working — only the drawn scrollbar disappears.
- Scale the card geometry with the same factor (added during apply after the bigger default fonts no longer fit the fixed card): card width 600/900, height 640, and the 300px cheat-sheet panel all multiply by the font scale, still capped at 92% of the monitor.
- Out of scope: paddings and radii do not scale with the font (only the card box and text grow); no live/interactive size adjustment inside the overlay.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `center-overlay`: the launch-arguments requirement gains `--font-scale`; a new typography requirement defines the base sizes and the scale factor, and a new requirement makes the card geometry follow the same scale; the «Center» card requirement's fixed-width clause becomes scale-relative; the mouse/cheat-sheet scrolling behavior changes to scroll with no visible scrollbar.

## Impact

- `src/ui/app.rs` — all hardcoded `.size(...)` values become base constants × scale; both `scrollable` widgets get the hidden-scrollbar direction/style; `OverlayOptions` carries the resolved scale.
- `src/ui/theme.rs` (or a sibling module) — reading and validating `font_scale` from the theme cache alongside the color tokens.
- `src/main.rs` (`overlay` subcommand) and `src/bin/overlay.rs` — the new `--font-scale` flag; `l10n` catalog entries for its help text (ru/en).
- README and `docs/` screenshots may lag visually until re-shot; no API or format breaking changes. The theme-cache JSON gains one optional key, so older templates keep working (the token is simply absent → 1.0).
