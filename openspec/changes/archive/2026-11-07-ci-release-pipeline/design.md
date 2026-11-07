# Design

## Context

The repository is local-only (no `origin`); the MVP, overlay, cheatsheet, and localization work is complete, and `openspec/specs/` covers 6 capabilities, none related to CI or releases. Verified current state of the tree: `cargo clippy --all-targets -- -D warnings` passes, `cargo test` passes headless (one test is `#[ignore]` and requires a live niri session — CI skips ignored tests by default), but `cargo fmt --check` **fails** in `src/exec.rs`. `Cargo.lock` is committed; `winit-core`/`winit-common` are pinned to exact beta versions, so locked resolution is deterministic. `gh` CLI is authenticated on this machine as `ys-kalyakin` (ssh protocol, `repo` scope). See proposal.md for motivation.

## Goals / Non-Goals

**Goals**
- Two GitHub Actions workflows: checks on push/PR, release on `v*` tag.
- A public hosted repository with `main` pushed and the workflows active.
- A reproducible release bundle users can download and unpack.

**Non-Goals**
- No musl/aarch64 builds, no RPM/Flatpak packaging (user chose tar.gz x86_64 gnu).
- No crates.io publishing (user chose GitHub Releases only).
- No CHANGELOG file, version-bump automation, or tag↔manifest version consistency gate.
- No branch protection rules beyond workflow-based reporting (can be added later).

## Decisions

1. **Hosting and CI: GitHub + GitHub Actions.** The only hosting option with zero new infrastructure: `gh` is already authenticated with `repo` scope, and Actions needs no setup beyond workflow files. Alternative (local scripts only) cannot "publish" anywhere public.
   - Repository: `ys-kalyakin/niri-kbinds`, **public** (a release others can download implies public), ssh remote `origin`. Recorded assumption — confirm at repo-creation time in apply.

2. **Actions pinned to major tags, minimal set.** `actions/checkout@v5`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` for CI; release workflow additionally uses preinstalled `gh`. Alternative — installing rustup manually in YAML — is more lines with no benefit. Pinned-by-major-tag is the project's acceptable supply-chain trade-off; a SHA pinning pass can come later.

3. **Release publishing via `gh release create` with `GITHUB_TOKEN`.** Avoids a third-party release action; `gh` is preinstalled on runners; the workflow sets `permissions: contents: write` for the job. Release notes via `--generate-notes` (auto-generated from commits since the previous tag), so no CHANGELOG upkeep.

4. **Bundle: `tar.gz` built on `ubuntu-24.04`, binaries stripped at archive root.**
   - Name: `niri-kbinds-<tag>-x86_64-linux-gnu.tar.gz`.
   - Contents: `kbinds`, `kbinds-overlay` (release, `strip`ped), `README.md` for context.
   - `ubuntu-24.04` gives a glibc 2.39 baseline — covers Fedora 40+ and similar-era distros (the dev machine is Fedora 45). Build with `cargo build --release --locked`, then `strip` into a staging dir before `tar czf`.

5. **All builds `--locked`.** `Cargo.lock` is committed; the exact beta pins (`winit-core =0.31.0-beta.2`, `winit-common =0.31.0-beta.2`) make CI resolve exactly what the dev machine resolves. Fulfills the locked-resolution requirement in `ci-checks`.

6. **Tag drives the release, not the manifest version.** The bundle name and release come from the pushed tag; `Cargo.toml` may stay at `0.1.0`. Rationale: zero-friction releases for a personal tool; a tag↔version gate is a possible future change, not a requirement now. Recorded assumption.

7. **One-time `cargo fmt` commit precedes the first CI run.** The tree currently fails `cargo fmt --check`, so the check workflow must be enabled together with (or after) a formatting-only commit, or the very first run would be red. Formatting-only, no behavior changes; clippy and tests already pass. The broken local rustup proxies are irrelevant to CI — runners install their own toolchain via `dtolnay/rust-toolchain`.

## Risks / Trade-offs

- [glibc 2.39 baseline excludes older LTS distros] → Accepted (user chose gnu over musl); musl remains a possible future capability.
- [Third-party actions are a supply-chain surface] → Minimal set, pinned to major tags; SHA-pinning later is additive.
- [Tag/manifest version drift confuses users about what they run] → Bundle name carries the tag; binaries report their own build; revisit if it matters in practice.
- [First release (`v0.1.0`) has no "previous tag" for generated notes] → `--generate-notes` handles the empty range fine (full-history notes); acceptable for a first release.
- [Repo creation is an outward-facing, hard-to-undo step] → Done explicitly and separately in apply (create repo → push → verify), not bundled into silent steps.

## Migration Plan

Apply order: (1) `cargo fmt` commit; (2) add workflow files; (3) create GitHub repo and push `main`; (4) watch the first check run go green; (5) push tag `v0.1.0`; (6) verify the release page and unpack-and-run the bundle. Rollback: delete the GitHub repo (nothing else changes locally); workflows are inert files until the remote exists.

## Open Questions

None — visibility (public) and tag-driven versioning are recorded as explicit assumptions to confirm during apply.
