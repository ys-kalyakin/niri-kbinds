# Proposal: kbinds-overlay-mvp2

## Why

MVP1 provides the data and the execution of binds (CLI, fuzzel-dmenu), but the cheat sheet remains textual: the user has to read a list instead of seeing it. The approved `mockup.html` mockup («Центр» / "Center") defines a graphical overlay — a layer-shell surface with search, groups, and keycaps — covering the "look → find → execute" scenario. The GTK stack is unavailable (no devel packages, sudo is closed), so the stack is fixed as `iced` + `iced_layershell` 0.19.1 (pure Rust, crates.io only).

## What Changes

- A new binary `kbinds-overlay` (a second `[[bin]]` in the `niri-kbinds` crate): `kbinds overlay [--config PATH] [--dry-run] [--mode center]`.
- A new surface part of the UI: an `overlay` layer-shell layer, keyboard interactivity `exclusive`, a fullscreen scrim on the focused monitor (`niri msg focused-output --json` at startup).
- A «Центр» ("Center") card per the mockup: a search field with autofocus, a grouped list (groups and their order — as in MVP1 `groups`), keycaps from `keys_display`, a «N из M биндов» counter, a footer hint.
- Navigation: `↑/↓` cyclically, `Enter` — execute and close, `Esc` / a click outside the card — close, a click on a row — execute, mouse-wheel scrolling.
- `--dry-run`: the bind's command is printed to **stderr** (unlike `kbinds exec --dry-run`, which prints to stdout), nothing is executed, the overlay closes.
- New dependencies: `iced`, `iced_layershell` 0.19.1 (+ their dependency trees); no system devel packages required.
- README: installation, the `Mod+Shift+/` bind in niri, an overlay screenshot, differences from `kbinds dmenu`.
- The MVP1 fuzzel adapter and CLI commands are unchanged.

## Capabilities

### New Capabilities

- `center-overlay`: the graphical «Центр» ("Center") overlay — a layer-shell surface above the desktop with a scrim, a search card, a grouped list of binds, keyboard and mouse navigation, execution of the selected bind through MVP1 `exec`, and a `--dry-run` mode.

### Modified Capabilities

(none — MVP1 behavior (`binds-parsing`, `binds-cli`, `binds-audit`, `fuzzel-dmenu`) is unchanged; `kbinds-overlay` reuses the library modules as is)

## Impact

- **Cargo.toml**: +2 dependencies (`iced`, `iced_layershell` 0.19.1), +1 `[[bin]]` `kbinds-overlay`.
- **New code**: `src/bin/overlay.rs` (arguments, UI launch), `src/ui/{app,items,theme}.rs` (iced state, pure filtering functions, palette).
- **Reused without changes**: `parser`, `model`, `groups`, `exec` (`action_argv` + `display_command` — public; the overlay's dry-run prints to stderr, so `exec_bind` is not called directly in dry-run mode), `keys_display`.
- **README.md**: an overlay section + a screenshot.
- **The user's niri config**: a `Mod+Shift+/` bind (+ a duplicate `Mod+Shift+backslash` for the ru layout) → `spawn "kbinds" "overlay"`; per the user's decision the stock `show-hotkey-overlay` moves to the free `Mod+Shift+U` (niri would reject a duplicate combo entirely) (M5).
- **Risks**: the `iced_layershell` 0.19 API (layer/anchors/exclusive keyboard/background transparency) — M1 is the arbiter; if the crate blocks, come back with facts and options instead of silently changing the stack.
