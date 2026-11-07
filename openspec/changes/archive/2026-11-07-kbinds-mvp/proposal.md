# Proposal: kbinds-mvp

## Why

The only bind cheatsheet in niri 26.04 is a static PNG from `show-hotkey-overlay`: no search, no way to execute an action, conflicts and real commands are invisible. The user's config has 144 binds (40 with Cyrillic keys); scanning a picture by eye is unrealistic; there is already a real conflict (`Mod+D` assigned twice) that the overlay does not show. A tool is needed whose source of truth is `config.kdl` itself (the only one: `niri msg` does not expose the bind list).

## What Changes

A CLI tool `kbinds` is created (Rust, one static binary, new crate `niri-kbinds` at the repository root):

- Parser of the `binds` section of `~/.config/niri/config.kdl`: `include` support (relative resolution, cycle protection), a KDL dialect compatible with what niri 26.04 parses with (knuffel 3.2 / KDL v1), tolerance of unknown properties.
- Full bind model (serde): actions (builtin/spawn/spawn-sh), canonical `combo`, `keys_display` (arrows, Enter, Cyrillic per the reference table), grouping, a `search` field.
- `kbinds list [--json]` — machine-readable list (JSON schema §6 of the SPEC).
- `kbinds exec <id> [--dry-run]` — action execution via `niri msg action` (the `spawn -- <COMMAND>...` syntax verified on the target machine).
- `kbinds dmenu` — interactive search via fuzzel (`--dmenu`): found → Enter → executed.
- `kbinds audit [--json] [--strict]` — rules A1 (duplicated combo, error), A2 (bind without overlay-title, warn), A3 (niri actions with no bind, info, nice-to-have).
- Fixtures and tests (unit + integration), README (installation, the niri bind, usage scenarios).

Non-goals (fixed by the SPEC): a layer-shell GUI overlay (v2), editing config.kdl, a waybar module, a daemon, other compositors. A Vicinae adapter is a separate investigation, not blocking the MVP (the outcome is either an adapter or a documented "not supported" note in the README).

## Capabilities

### New Capabilities

- `binds-parsing`: parsing `config.kdl` into a full bind model — the niri KDL dialect, `include` with cycle protection, combo canonicalization, key display (Cyrillic per the reference table), deterministic grouping, a search field. Reused by the future v2 overlay.
- `binds-cli`: the `list --json` and `exec` commands — the bind JSON schema, action execution via `niri msg action`, `--dry-run`, exit codes and stderr diagnostics.
- `fuzzel-dmenu`: interactive search over binds via the fuzzel dmenu protocol with execution of the selected bind.
- `binds-audit`: static analysis of the config — combo conflicts, binds invisible in the stock PNG overlay, missing binds of common actions.

### Modified Capabilities

_(none — greenfield project, no existing specifications)_

## Impact

- **New code:** the `niri-kbinds` crate at the root (`Cargo.toml`, `src/`, `tests/`, `README.md`).
- **Dependencies (runtime):** `clap` v4, `kdl` (version — see design; the dialect must parse the live config), `serde` + `serde_json`, `anyhow`.
- **Environment:** Rust stable toolchain (rustup proxies are broken on the machine — invoke the toolchain directly from `~/.rustup/toolchains/*/bin`, verified by the SPEC); crates.io on the first build.
- **External interfaces:** reading `~/.config/niri/config.kdl` (+ includes), `niri msg action`, the fuzzel dmenu protocol. No network required.
- **Acceptance risk:** the bind count on the live config must match the `grep` arbiter (144/144) — any mismatch blocks acceptance.
