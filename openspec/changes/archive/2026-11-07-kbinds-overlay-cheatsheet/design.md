# Design: important-keys cheat sheet in the overlay

## Context

The "Center" overlay (MVP2) is a single 600×640 card: search, list, counter,
footer (`src/ui/app.rs::view`). The model already contains
`overlay_title_present` for every bind (`src/model.rs`); there are 16 of them
on the live config. Nothing new needs to be parsed — motivation in
proposal.md, contract in the spec delta.

Constraints inherited from MVP2: iced 0.14 + iced_layershell 0.19.1 (pin
`winit-core/winit-common =0.31.0-beta.2`); window size from `niri msg --json
focused-output`; `#[to_layer_message]` above `#[derive]`; the layer surface
does not receive `Resized` through the subscription.

## Goals / Non-Goals

**Goals:**
- A right cheat-sheet column: binds with `hotkey-overlay-title`,
  "title + keycaps".
- Read-only: no clicks, no reaction to the search or `↑↓`.
- Update the card geometry and the "click outside the card" hit test.
- A pure selection function in `items.rs`, covered by unit tests (the MVP2
  pattern).

**Non-Goals:**
- Panel interactivity (click = execute) — rejected by the user.
- `--mode panel` (a full-screen docked panel on the right) — a stretch goal
  from §8 of SPEC-V2, not to be confused with the column inside the card.
- New theme tokens / edits to the noctalia template — we use the existing
  roles.
- Panel visibility control (a flag/a key) — the panel is permanent.

## Decisions

**D1. Cheat sheet selection — a pure function in `items.rs`.**
`cheatsheet_binds(&[Bind]) -> Vec<usize>` — indices of binds with
`overlay_title_present == true`, in config order. Computed once in `boot` and
stored in `Overlay` (the `cheatsheet: Vec<usize>` field); `update()` does not
touch it — hence "not filtered by search" for free. Alternative — a filter in
`view` on every frame: also cheap, but state in the struct is easier to test
and read. Alternative — capturing the binds by value: data duplication.

**D2. Geometry: 900 + 300, the column disappears dynamically.**
Constants: `CARD_WIDTH = 900`, `CHEATSHEET_WIDTH = 300` (the left column
`Length::Fill` ≈ 600 — the former width is preserved), `CARD_HEIGHT = 640`
unchanged. `items::card_rect(window)` → `card_rect(window, has_cheatsheet:
bool)`: with the panel the width is 900, without it — 600; `is_outside_card`
takes the same parameter, `app.rs` passes `!state.cheatsheet.is_empty()`.
The hit test and the layout use one function — the single source of truth
about the rectangle. The alternative "always 900 with an empty panel" — dead
space when there are zero titles.

**D3. Layout: `row![левая_колонка, разделитель, панель]` inside the card.**
The left column is the current `column![search, list, counter, footer]`.
Between the columns — a 1px vertical line in the `outline_variant` color (a
container with a background). The panel is `column![заголовок, scrollable(rows)]`: a «Важные клавиши» heading of size 12 in the secondary
color (visually separates the column from the list), then compact
"title + keycaps" rows — the same `items::keycaps`, sizes as in the left
list (title 14, keycaps 11), but without `mouse_area`, without selection
highlighting, and with smaller vertical padding (4px instead of 6px). A
separate `scrollable` with id `kbinds-overlay-cheatsheet` — the wheel over
the panel scrolls the panel, over the list the list (iced sorts it out per
widget); programmatic scrolling of the panel is not needed — there are no
new `Message` variants at all.

**D4. Theme — existing roles only.**
The panel uses `surface` (the card background, shared), `on_surface`/
`on_surface_variant` (text), `outline_variant` (separator), the keycap styles
as is. The noctalia cache and
`~/.config/noctalia/templates/kbinds-overlay.json` are not changed. The
alternative — a slightly darker panel background (`surface_container_low` and
similar) — was rejected: the new token exists neither in the cache nor in the
§6 fallback.

**D5. Order of work relative to mvp2.**
The "MODIFIED" delta requires `center-overlay` to already be in the main
specs — before applying this change, `kbinds-overlay-mvp2` must be archived
(`/opsx:archive`), otherwise `openspec validate` will not find the requirement
to modify. The order is fixed in proposal.md (Impact).

## Risks / Trade-offs

- [Layout and hit-test divergence: a click "inside" closes the overlay] → both
  sides read `items::card_rect(window, has_cheatsheet)`; geometry unit tests
  for both values of the flag (600 and 900).
- [The spec delta references a capability not yet archived] → archive mvp2
  before apply (D5); until then, `validate` of this change is expected to
  complain.
- [On a very narrow monitor the 92% cap will squeeze the left column into
  unreadability (the panel is fixed at 300px)] → the target machine is
  2560×1440; narrow-screen degradation makes no sense — recorded as a known
  limitation, not handled.
- [Long Cyrillic titles wrap and break the evenness of the rows] → as in the
  left list: a single-line `text` without wrapping, excess width is clipped;
  verified by screenshot on the live set (16 titles).

## Migration Plan

Rollback — revert the commit; the change touches no data or configs. The
binary is reinstalled with `cargo install --path .` as in MVP1/MVP2 (account
for working around the target machine's broken rustup proxies — PATH to
toolchains).

## Open Questions

None: the content (binds with `hotkey-overlay-title`) and the read-only
behavior were fixed by the user at propose time.
