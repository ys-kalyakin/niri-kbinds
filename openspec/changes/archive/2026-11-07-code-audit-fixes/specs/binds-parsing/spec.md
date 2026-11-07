# Spec Delta

## MODIFIED Requirements

### Requirement: Tolerance of unknown constructs

Unknown properties and attributes of a bind SHALL be semantically ignored — parsing SHALL NOT fail on them; `XF86*` key names and `condition` constructs SHALL be tolerated. The verbatim representation of such constructs SHALL be preserved in the model's `raw` field as diagnostics data for humans and future tooling; the audit rules (A1–A3) SHALL NOT depend on it.

#### Scenario: Unknown property does not break parsing

- **WHEN** a bind contains an attribute or property unknown to the tool (e.g., `condition { … }`, `cooldown-ms=100`)
- **THEN** parsing succeeds and the property is preserved verbatim in `raw`; the key `XF86AudioMute` parses as a regular key

#### Scenario: Audit output does not depend on unknown constructs

- **WHEN** a bind carries arbitrary unknown attributes
- **THEN** the audit findings for the config are identical to the findings for the same config without those attributes (rules A1–A3 never read `raw`)
