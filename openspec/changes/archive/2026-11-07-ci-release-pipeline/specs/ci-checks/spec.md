# Spec Delta

## Purpose

Automated verification of every push and pull request: formatting, lints, tests, and a release build run on a hosted CI service so regressions are caught before code is accepted, without manual local checks.

## ADDED Requirements

### Requirement: Checks run on every push and pull request

The CI check workflow SHALL run automatically on every push to the main branch and on every pull request targeting the main branch. A push or pull request for which any check fails SHALL be reported as failed and the commit SHALL NOT be treated as passing.

#### Scenario: Push to main triggers checks
- **WHEN** a commit is pushed to the main branch of the hosted repository
- **THEN** the check workflow starts and runs the full check suite for that commit

#### Scenario: Pull request triggers checks
- **WHEN** a pull request targeting the main branch is opened or updated
- **THEN** the check workflow starts and its overall result is visible on the pull request

### Requirement: Check suite composition

The check suite SHALL consist of four gates, each failing the workflow when it does not pass:
1. formatting check (`cargo fmt --check`),
2. lint check with warnings treated as errors (`cargo clippy --all-targets -- -D warnings`),
3. test run (`cargo test`), which MUST pass without a graphical session,
4. release build (`cargo build --release`).

#### Scenario: Formatting violation fails CI
- **WHEN** the pushed code does not satisfy `cargo fmt --check`
- **THEN** the check workflow fails and reports formatting as the failing gate

#### Scenario: Clippy warning fails CI
- **WHEN** `cargo clippy` emits at least one warning
- **THEN** the check workflow fails, because warnings are errors

#### Scenario: Failing test fails CI
- **WHEN** at least one unit or integration test fails
- **THEN** the check workflow fails and reports the failing test

#### Scenario: Clean tree passes CI
- **WHEN** the pushed code passes formatting, clippy, and all tests, and the release build succeeds
- **THEN** the check workflow reports success for that commit

### Requirement: Locked dependency resolution

The release build gate SHALL build with the committed lockfile in locked mode, so a change that would alter resolved dependency versions without committing the updated lockfile fails CI.

#### Scenario: Lockfile out of date fails CI
- **WHEN** a pushed change updates a dependency declaration in the manifest but not the committed `Cargo.lock`, so a locked build cannot resolve
- **THEN** the check workflow fails with a lockfile resolution error
