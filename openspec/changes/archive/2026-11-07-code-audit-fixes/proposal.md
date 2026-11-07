# Proposal

## Why

A manual code audit (2026-11-07) of the ~5 000-line Rust codebase found two real defects and a batch of non-idiomatic patterns. The defects are worth fixing now: one is a reachable panic that already violates an existing spec scenario (corrupt `theme.json` must never panic), the other is a model field the spec promises "for audit" but no audit rule ever reads. The idiomatic-pattern items are cheap, mechanical cleanups that reduce maintenance cost and let the compiler catch more.

## What Changes

- **Fix a panic vector in `parse_hex`** (`src/ui/theme.rs`): byte slicing `&hex[0..2]` panics on a non-ASCII cache value like `"#中中"` (6 bytes, matches the `len()==6` arm). The `center-overlay` spec already requires the overlay to open "without errors or panics" on a corrupt cache — the code must comply.
- **Resolve the dead `raw` field** (`src/model.rs`, `src/parser.rs`): parser fills `Bind::raw`, tests assert it, but no production code reads it. The `binds-parsing` spec says "kept in `raw` for audit" — the requirement wording is amended to match reality (kept verbatim for diagnostics; audit rules A1–A3 do not use it).
- **Distinguish broken pipe from real IO errors**: `print_lines` (`src/lib.rs`) and the dry-run path (`src/exec.rs`) currently treat *every* write error as "downstream closed the pipe" and report success. Only `ErrorKind::BrokenPipe` stays a quiet success; other errors propagate to the standard exit-code-2 path of `binds-cli`.
- **Remove `std::process::exit` from library code**: `run_action`'s dry-run branch (`src/exec.rs`) exits the process directly on a closed pipe; it returns instead, consistent with `print_lines`.
- **Replace stringly-typed discriminants with enums**: `Action.kind` (`"builtin" | "spawn" | "spawn-sh"`) and `Bind.title_source` (`overlay-title | action-name | raw-action | command`) become enums with serde renames — the JSON output is byte-identical, but matching becomes exhaustive and typo-proof. No new capability; `binds-cli`'s JSON field list is unchanged.
- **Fix the comment/code mismatch in `cmd_list`** (`src/lib.rs`): the human-readable list is sorted alphabetically by group name, not "in config order" as the comment claims; the comment is corrected to match the stable, deliberate behavior.
- **Unify test fixtures**: six test modules hand-write full 14-field `Bind` literals (one with a pointless `..Bind { … }.clone()` functional update that clippy flags as a redundant clone); a shared sample-based fixture replaces them.
- **A batch of mechanical idiomatic cleanups** (details in `tasks.md`): remove the double error context and the double `canonicalize` in `parser.rs`, drop the needless `cli.config.clone()` in `main.rs`, import `BTreeSet` at the top of `audit.rs`, make `print_lines` generic over `AsRef<str>`, delegate `is_dictionary_action` to `dictionary_entry`, collapse the `find`+`position` double scans, filter keyboard events out of the generic `listen()` subscription, replace `vec!["niri".to_string(), …]`, name the `MenuItem` fields via a struct, `or_else` in `typography::resolve`, return `Bind` (not `Result<Bind>`) from `parse_bind`, declare `[lints]` in `Cargo.toml`.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `binds-parsing`: the "Tolerance of unknown constructs" requirement currently promises unknown properties are "kept in `raw` for audit" — no audit rule reads `raw`. The requirement is reworded to keep the tolerance guarantee (unknown constructs never fail parsing) while describing `raw` accurately as verbatim diagnostics data in the model, not an audit input.

## Impact

- **Code**: `src/ui/theme.rs`, `src/model.rs`, `src/parser.rs`, `src/lib.rs`, `src/main.rs`, `src/exec.rs`, `src/audit.rs`, `src/groups.rs`, `src/fuzzel.rs`, `src/ui/app.rs`, `src/ui/items.rs`, `src/ui/typography.rs`, `src/l10n.rs`, `Cargo.toml` (new `[lints]` section).
- **Behavior**: only two observable changes — real (non-broken-pipe) stdout write errors now surface as exit code 2 instead of a silent success, and a corrupt `theme.json` with non-ASCII hex no longer aborts the overlay. JSON schemas (`list --json`, `audit --json`) are unchanged.
- **Dependencies**: none added or removed.
- **Tests**: fixture helpers replace duplicated literals; new regression tests for the `parse_hex` panic vector and the broken-pipe distinction.
