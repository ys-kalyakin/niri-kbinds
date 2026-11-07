# Design

## Context

Single-crate Rust workspace (`niri-kbinds` lib + `kbinds`/`kbinds-overlay` bins), edition 2024, no external deps to add or remove. Motivation in [proposal.md](proposal.md); the one requirement-level change is specified in the [binds-parsing delta](specs/binds-parsing/spec.md). Default clippy and rustfmt are already clean, so every fix must keep them clean (the `ci-checks` capability gates on that).

## Goals / Non-Goals

**Goals:**

- Eliminate the reachable panic in cache hex parsing (spec scenario of `center-overlay`: corrupt cache → no panics).
- Make stdout write-error semantics honest: broken pipe = quiet success, everything else = error exit 2.
- Make the compiler own the action-kind and title-source matching (exhaustive enums, identical JSON).
- Remove process-level side effects from library code (`std::process::exit`).
- Cheap idiomatic cleanups from the audit, each individually verifiable by tests/clippy.

**Non-Goals:**

- No new audit rules (an A4 rule over unknown constructs would be new behavior — out of scope; `raw` stays diagnostics-only).
- No change to the human-readable `list` ordering (comment fix only; re-ordering groups to match the overlay would be a separate behavior change).
- No new dependencies (the clippy-nursery suggestion to use `bytecount` for newline counting is declined — a config file line count does not need SIMD).
- No wholesale `clippy::pedantic` enablement; `[lints]` starts minimal (see D7).

## Decisions

### D1 — `parse_hex` parses bytes, not str slices

Replace `&hex[0..2]` slicing with `hex.as_bytes()` chunk parsing (`u8::from_str_radix` over `std::str::from_utf8` of each 2-byte chunk). This removes the whole panic class (byte-index-out-of-char-boundary) rather than guarding specific lengths, and treats `#中中`, `#ёё`, and truncated input uniformly as "unparseable → per-token fallback". The existing tests (`hex_parsing_handles_short_and_long_forms`) gain a non-ASCII regression case. Alternative considered: an `hex.is_ascii()` guard before slicing — smaller diff, but leaves the panicking code in place behind a gate; rejected.

### D2 — `raw` stays in the model; the spec wording is fixed instead

The field is pinned by `binds-parsing` and genuinely useful (verbatim unknown-construct record); deleting it would mean a REMOVED delta for zero gain. The audit finding was the false "for audit" promise: the delta rewords the requirement to "diagnostics data; A1–A3 do not depend on it" and adds a scenario pinning that audit output is invariant to unknown attributes. Code change: only the field's doc comment. Alternatives considered: implement an A4 rule (new behavior, scope creep); delete the field (spec churn, information loss). Rejected both.

### D3 — Broken-pipe discrimination in both stdout writers

`print_lines` (`src/lib.rs`) and the dry-run branch of `run_action` (`src/exec.rs`) currently treat any write error as "downstream closed the pipe". Both switch to: `ErrorKind::BrokenPipe` → stop quietly (success); any other error → `Err(...)` up the stack → the standard exit-2 path of `binds-cli`. In `exec.rs` the `std::process::exit(0)` disappears — the function returns `Ok(())` like `print_lines` does (library code must not exit the process). Subtlety kept in mind: `BufWriter` defers errors to `flush()`, so the flush path in `print_lines` applies the same kind check instead of the current unconditional `.ok()`. Tests: spawn `head -1` style children (existing fuzzel test pattern) or assert on the error type via a unit-testable helper. Alternatives considered: keep swallow-everything (dishonest exit codes on ENOSPC/EIO); install a SIGPIPE handler (unidiomatic in Rust, std deliberately ignores SIGPIPE).

### D4 — `ActionKind` and `TitleSource` enums

`Action.kind: String` → `enum ActionKind { Builtin, Spawn, SpawnSh }` with `#[serde(rename_all = "kebab-case")]`; `Bind.title_source: String` → `enum TitleSource { OverlayTitle, ActionName, RawAction, Command }` same rename. Serialization is byte-identical (the golden test `json_matches_schema` in `model.rs` pins it). Every `as_str()` match site (`groups.rs`, `exec.rs`, `audit.rs`, `fuzzel.rs`) becomes an exhaustive `match`, so adding a kind later is a compile error at all sites. `describe()`, `search_text()`, and `bind_detail()` need the string form — derived via a small `Display` impl or `match` in the enum. Alternatives considered: keep `String` with constructor validation (weaker, no exhaustiveness); a `strum` dep (unneeded). This is the only refactor with a wide blast radius; it is compiler-guided end to end.

### D5 — `cmd_list`: fix the comment, keep the sort

The stable alphabetical-by-group-name clustering is deliberate observable behavior of `binds-cli`; only the comment ("config order") is wrong. Fix the comment. Alternative (change the sort to first-appearance order to match the overlay's `build_groups`) — rejected here: user-visible change beyond a hygiene fix.

### D6 — Shared test fixture module

Six modules hand-write full 14-field `Bind` literals; `parser.rs` additionally wraps one in a pointless `..Bind { … }.clone()` (clippy: redundant clone). Add `#[cfg(test)] pub(crate) mod testutil` in the lib with a `sample_bind()` (the `model.rs` fixture promoted) plus tiny builders (`builtin(name)`, `spawn(argv)` already exist duplicated in three modules). Each test module switches to `Bind { line: 2, ..testutil::sample_bind() }`-style structural updates. Alternative: `Default for Bind` — rejected: a default-able domain model invites placeholder values in production code.

### D7 — Minimal `[lints]` in Cargo.toml

`[lints.rust] missing_docs = "warn"` — the crate is documented by convention and the lib is public; this locks the convention in. `clippy::pedantic` stays off (139 mostly-noise findings, e.g. blanket `must_use`); adopting a curated pedantic subset is a separate, later decision. The audit's P3 items are fixed in code regardless, so they don't depend on lint gating.

### D8 — `parse_bind` returns `Bind` directly

Clippy `unnecessary_wraps`: the function cannot fail by design (SPEC tolerance). Returning `Bind` documents "tolerant parsing" in the signature; the callers already treat it as infallible.

## Risks / Trade-offs

- [Enum refactor touches every action-matching site] → The compiler drives it (exhaustive matches); the JSON golden test and the full existing suite pin behavior; `list --json` output diff must be empty against the pre-change binary on the live config.
- [Exit-code semantics change: pathological stdout errors now exit 2] → Intended spec compliance (`binds-cli` exit codes); scripts piping to `head` keep working because `BrokenPipe` is still quiet success.
- [Test fixture consolidation churns many test diffs] → Mechanical, one module at a time; no assertion changes.
- [`write!` error kinds can vary by std/platform] → Only `BrokenPipe` is special-cased; every other kind fails loudly, which errs on the honest side.

## Migration Plan

No data, no deps, single-crate repo: land as one reviewable series of commits in the order of `tasks.md`, run `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` after each. Rollback = revert the commit(s).

## Open Questions

(none)
