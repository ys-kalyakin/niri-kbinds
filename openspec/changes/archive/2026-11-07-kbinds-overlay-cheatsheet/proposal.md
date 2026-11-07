# Proposal: important-keys cheat sheet in the overlay

## Why

Replacing the stock niri PNG overlay with `kbinds overlay` removed the
"instant glance" at the most important combos: they are now dissolved in the
general list of 162 binds. To see, say, "Mod+Space — layout switch" or
"Mod+Alt+L — screen lock", one has to hunt with the eyes or filter — although
niri used to show exactly this set as a separate cheat sheet.

## What Changes

- The overlay card gains a **second column on the right** — a static
  important-keys cheat sheet (read-only): a list of binds whose config entry
  sets `hotkey-overlay-title` (16 on the live config), in the same
  "title + keycaps" form as the left list.
- The cheat sheet is **not interactive**: it is not filtered by the search and
  does not react to clicks or `↑↓` — navigation and execution work only in the
  left list.
- The card becomes wider (two columns) and the left list narrows; the
  hit-test geometry of "click outside the card" updates together with the
  width.
- The cheat sheet content is driven by the config: adding/removing an
  important bind from the cheat sheet = adding/removing `hotkey-overlay-title`
  in `config.kdl` (just like in niri before), no code edits.

## Capabilities

### New Capabilities

<!-- no new capabilities -->

### Modified Capabilities

- `center-overlay`: the "Center" card gains a second (right) column — a
  static cheat sheet of binds with `hotkey-overlay-title`; the requirement
  "The "Center" card" changes (composition and width) and a requirement about
  the cheat sheet is added (read-only, independent of the search).

## Impact

- **Code**: `src/ui/app.rs` (card layout, state, width),
  `src/ui/items.rs` (pure bind-selection function for the cheat sheet, card
  and hit-test geometry), `src/ui/theme.rs` (panel style, if a new column
  separator/background is needed). The CLI (`main.rs`, `bin/overlay.rs`),
  parser, model, and exec — unchanged.
- **Data**: the binds for the cheat sheet are already in the model
  (`overlay_title_present`) — no new parsing needed.
- **Dependencies**: none new (iced 0.14 + iced_layershell 0.19.1 as is).
- **Specs**: the delta modifies the `center-overlay` capability, which lands
  in the main specs upon archiving `kbinds-overlay-mvp2`. Order of work: first
  `/opsx:archive` for `kbinds-overlay-mvp2`, then apply of this change —
  otherwise the "MODIFIED" delta references a main spec that does not exist
  yet.
- **Docs**: README (Overlay section) — a paragraph about the right panel +
  a new screenshot.
