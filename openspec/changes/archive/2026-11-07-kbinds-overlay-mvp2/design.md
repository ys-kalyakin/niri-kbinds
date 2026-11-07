# Design

## Context

The `niri-kbinds` crate (MVP1, archived as `2026-11-07-kbinds-mvp`) contains the library modules `parser`, `model`, `normalize`, `groups`, `audit`, `exec`, `fuzzel` and the CLI binary `kbinds`. On the live session `kbinds list` reports **161** binds (the "144" figure from the task spec is a historical count of Mod binds; the overlay shows everything the parser yields). The motivation is in proposal.md; the behavior is in specs/center-overlay.

Verified environment facts (2026-11-07): Fedora 45, niri 26.04, `WAYLAND_DISPLAY=wayland-1`, focused monitor DP-2 2560×1440; the GTK4/gtk4-layer-shell devel packages are not installed, sudo is unavailable; `niri msg --json focused-output` (the global flag **before** the subcommand) returns `{"name":"DP-2",…}` — the `niri msg focused-output --json` form from task spec §9.2 does not exist; the user's config has 0 Cyrillic duplicates of letter binds; the rustup proxies in the sandbox are broken — run cargo with `PATH="$(ls -d ~/.rustup/toolchains/*/bin | head -1):$PATH"`.

## Goals / Non-Goals

**Goals:**
- A second `[[bin]]` `kbinds-overlay` on top of the shared library, without edits to MVP1 logic.
- Testability without a GUI: filtering, grouping, and navigation are pure functions.
- Building from crates.io only (the acceptance criterion §11.1 of the task spec).
- A transparent scrim and an exclusive keyboard via layer-shell.

**Non-Goals:**
- The `--mode panel` mode (stretch, a separate task spec), theme settings, animations, guarding against a second instance, edits to the MVP1 modules.
- `-D warnings` cleanliness is achieved, but refactoring the foreign MVP1 code is not included.

## Decisions

- **D1. Stack: `iced` + `iced_layershell` 0.19.1** (waycrate/exwlshelleventloop). Rejected alternatives: GTK4+gtk4-layer-shell (no devel packages and no sudo), bare wayland-client/smithay (manual rendering and event loop — disproportionate), iced without layershell (no overlay layer). Take the compatible `iced` version from the `Cargo.toml`/examples of `iced_layershell` 0.19 (docs.rs); the `Overlay` layer, `KeyboardInteractivity::Exclusive`, anchors on all 4 edges, the target output — by name from D3. If the API does not provide an exclusive keyboard or transparency — come back with facts and options (task spec §9.1), do not change the stack silently. The arbiter is M1.
- **D2. Layout**: `src/bin/overlay.rs` — a thin entry point (clap arguments, config parsing by the library, errors → stderr + exit 2, UI launch); `lib.rs` gains a `pub mod ui` with `ui/items.rs` (row/group models, the pure `filter` and `move_selection` functions, group assembly — tested without a GUI), `ui/app.rs` (iced state + a `Message` enum, only applying the pure functions), `ui/theme.rs` (palette tokens of task spec §6, card/row/keycap styles). This way the bin stays thin, like `main.rs` in MVP1, and the logic is covered by unit tests.
- **D3. Focused output**: at startup, `niri msg --json focused-output`, parsing the `name` field with `serde_json` (already a dependency). Fallback: the command is unavailable/an error → a message to stderr, exit 2. The surface lives on the chosen output; monitor switching after opening is not handled (task spec §9.2).
- **D4. Dry-run to stderr without edits to exec**: `exec_bind(_, true)` prints to stdout (MVP1 behavior, pinned by tests). The overlay needs stderr (task spec §5.8), so it prints `exec::display_command(&exec::action_argv(action))` for each action of the bind to stderr itself — both functions are public. Actual execution — `exec_bind(bind, false)`. `exec.rs` is unchanged.
- **D5. Transparency**: the iced scene clear-color — `rgba(8,6,14,0.55)`; a surface in the RGBA format from layershell. Checked visually on M1 (windows must show through); if the render fills it opaque — a §11.2 blocker, come back with facts.
- **D6. Selection model**: a flat `Vec` of visible rows (groups expanded in the MVP1 order), `selected: usize`; `move_selection` wraps modulo the length; on a filter change the selection resets to 0 (predictable, with no "jumps"); the selected row is scrolled into the visible area.
- **D7. Closing**: any close (Esc, a click outside, a successful Enter, dry-run) — exit from the iced application, process code 0. Nothing is executed only on Esc/a click outside.
- **D8. Bind in niri (M5)**: per the user's decision — replace the stock overlay: `Mod+Shift+Slash { spawn "kbinds" "overlay"; }` **and a duplicate `Mod+Shift+backslash`** (ru layout: the slash key has no Cyrillic keysym — the pattern is already used in the config); `show-hotkey-overlay` moves to the free `Mod+Shift+U`. Check after reload: niri 26.04 rejects a duplicate combo ENTIRELY; look for the real error in `journalctl --user` (`niri msg request-error` returns a demo stub).

## Risks / Trade-offs

- [The `iced_layershell` 0.19 API may not match expectations (exclusive keyboard, anchors, output choice, alpha)] → M1 is the arbiter; if blocked — come back with facts and 2–3 options (including fallbacks to bare layer-shell), do not change the stack silently.
- [iced's wgpu renderer may fail to raise a transparent surface on this machine/driver] → check on M1 before the other milestones; the scrim is the core of the overlay's value.
- [Cyrillic may not be typeable into the iced search field on the ru layout] → live scenario §10.5 of the task spec; the field receives system IME input via winit, the risk is low, but it is checked before M3.
- [`grim` is not installed → no screenshot for the README] → a non-blocking step (task spec §10.3): an interactive screenshot by niri's own means, by hand.
- [Slide-of-scale: 161 rows with keycaps] → trivial for iced; the grouping is expanded once per filter change.
- [Changing the bind touches the working niri config] → an edit of two lines + moving one; rollback — restore the previous three lines; check the reload via journalctl.

## Migration Plan

The changes are additive: the new binary and the `ui` module do not touch the existing code paths; the MVP1 CLI commands remain as they were (the regression test §10.4 of the task spec). The niri config edit is the only change outside the repository, and it is reversible by hand. Rollback: delete the `[[bin]]` entry and `ui/`, restore the config lines.

## Open Questions

(none blocking: the `iced_layershell` API details are deliberately postponed to M1 per D1 — a manageable risk with a return plan, not a missed decision)

## Addendum (2026-11-07, at the user's request): the noctalia theme

- **D9. The theme follows the noctalia palette**: instead of the fixed table of §6, the colors are taken from `~/.cache/kbinds/theme.json` — a JSON cache written by the user's noctalia template (`~/.config/noctalia/templates/kbinds-overlay.json`, registration in `~/.config/noctalia/templates.toml`). Noctalia itself re-renders user templates when the palette or the dark-light mode changes — the overlay picks up the current colors on its next launch. Token mapping: scrim→scrim, surface_container→card, outline/outline_variant→borders, on_surface/on_surface_variant→text, primary→accent/highlight; the alphas of §6 are preserved. The active scheme on the machine is `builtin Kanagawa`, dark (`noctalia msg color-scheme-get`). Fallback when the cache is missing/corrupt — the fixed palette of §6 (the fallback constants remain in theme.rs).
