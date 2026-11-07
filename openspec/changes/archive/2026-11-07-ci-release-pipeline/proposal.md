# Proposal

## Why

The project is built and installed entirely by hand (`cp target/release/… ~/.local/bin`): nothing verifies every push, and there is no way to publish a release — the repository has no remote at all. CI checks (fmt, clippy, tests, build) and automated release publishing are the standard next step now that the MVP, overlay, cheatsheet, and localization features are complete and specs are in English.

## What Changes

- Add a GitHub Actions check workflow running on every push and pull request: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release --locked`.
- Add a GitHub Actions release workflow triggered by pushing a `v*` tag: build a release binary bundle and publish a GitHub Release with the bundle attached.
- One-time fix: run `cargo fmt` before enabling the check workflow — the current tree does not pass `cargo fmt --check` (src/exec.rs).
- One-time hosting setup (operational, in apply phase): create the GitHub repository `ys-kalyakin/niri-kbinds`, add it as `origin` (ssh), push `main` and existing history. No code changes beyond formatting; no dependency changes.
- README install section gains a "download a release" option alongside the existing build-from-source instructions.

## Capabilities

### New Capabilities

- `ci-checks`: automated verification of every push and pull request — formatting, lints, tests, and a release build must pass on CI before changes are considered accepted.
- `release-publishing`: tag-driven publication of releases — pushing a `v*` tag produces a GitHub Release with a downloadable tar.gz containing the `kbinds` and `kbinds-overlay` binaries.

### Modified Capabilities

(none — no existing capability covers CI or releases)

## Impact

- New files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`.
- Source: formatting-only changes from `cargo fmt` (src/exec.rs and possibly others); no behavior changes.
- Repository hosting: GitHub (`github.com/ys-kalyakin/niri-kbinds`), ssh remote `origin`; `gh` CLI is already authenticated with `repo` scope.
- CI environment assumptions: Rust stable (edition 2024 requires ≥ 1.85), committed `Cargo.lock` honored via `--locked` (pins `winit-core`/`winit-common` betas), Linux x86_64 runner; tests pass headless (one niri-live test is already `#[ignore]`).
