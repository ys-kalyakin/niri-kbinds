# Research: duplicated-combo semantics in niri 26.04 (task 5.1, SPEC §14.5)

Method: reading the niri v26.04 sources (git clone --branch v26.04, sparse: niri-config), no experiments on the live session.

## Conclusion

- `niri-config/src/binds.rs` (~lines 775–805): when decoding the `binds` block niri keeps
  a `seen_keys: HashMap<Key, _>`; a second node with the same key produces **two decode errors**:
  «keybind first defined here» (on the first) and «duplicate keybind later defined here» (on the second).
- These errors go into the parse context (`ctx.emit_error`) and make
  `ConfigParseResult.config = Err` (`niri-config/src/error.rs`): **the config is rejected entirely**.
  niri keeps running the last valid config (or the default one) and shows a config-error dialog.

## Consequences for the audit

- Neither of the duplicate binds "wins" — a config with a duplicate is not applied at all.
  The A1 finding wording therefore stays winner-neutral (as the spec requires):
  it lists every variant with files and lines without naming the active one.
- No A1 wording change was needed: the neutral text is compatible with the established
  semantics. The `binds-audit` spec was updated only in its motivation part
  ("semantics not verified" → "verified by source reading: niri rejects the config entirely");
  the normative part is unchanged.

## Fact deviation from the SPEC (recorded)

- SPEC §2: "A real conflict in the config: `Mod+D` is assigned twice". In the current live config
  the second bind (`vicinae toggle`, line 213) is **commented out** — there is no active conflict anymore
  (verified: zero active duplicated combos in the config). The §15.6 acceptance criterion about "the audit
  finds Mod+D on the live config" is checked against the `tests/fixtures/conflict.kdl` fixture;
  on the live config the audit must yield zero error-level findings (exit 0 even with `--strict`).
