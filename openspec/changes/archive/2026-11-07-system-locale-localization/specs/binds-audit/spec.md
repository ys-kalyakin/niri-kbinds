# Spec Delta

## MODIFIED Requirements

### Requirement: Audit output

`kbinds audit` SHALL print findings in a human-readable form in the system locale language (capability `localization`): under `ru` — in Russian, under `en` and locales without a translation — in English; the meaning, level, and composition of findings do not depend on the language. With `--json` — a machine-readable list of findings (rule id, level, message, references to binds); field names and rule ids do not depend on the locale, and the message texts in JSON follow the selected locale. With `--strict` and at least one `error` finding the exit code SHALL be 1; without `--strict` — 0 regardless of findings. On a config without findings the audit SHALL produce zero findings and exit code 0.

#### Scenario: Finding language follows the locale

- **WHEN** `kbinds audit` is run on a config with the `Mod+D` conflict under `ru_RU.UTF-8` and then under `en_US.UTF-8`
- **THEN** in both cases the `error` finding is printed, in Russian in the first case and in English in the second; the rule id and level match

#### Scenario: Strict mode

- **WHEN** `kbinds audit --strict` is run on the live config with the `Mod+D` conflict under `en_US.UTF-8`
- **THEN** exit code 1, the English-language `error` finding is printed to stdout or stderr per the regular output rules

#### Scenario: Negative test

- **WHEN** `kbinds audit` is run on a fixture without findings (`clean.kdl`)
- **THEN** zero findings, exit code 0
