# Tasks

Note for apply: on this machine rustup proxies are broken — run cargo via `PATH=~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH`.

## 1. Formatting pre-fix

- [x] 1.1 Run `cargo fmt`, verify `cargo fmt --check` exits 0 and `cargo clippy --all-targets -- -D warnings` plus `cargo test` still pass; commit the formatting-only change (`style: cargo fmt`)

## 2. Check workflow

- [x] 2.1 Create `.github/workflows/ci.yml`: triggers on push to `main` and `pull_request`; `actions/checkout@v5`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`; gates in order `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release --locked`; `concurrency` with cancel-in-progress per ref. Verify the YAML parses (`python3 -c "import yaml,yaml..."` / `yaml.safe_load`) and each gate command passes locally

## 3. Release workflow

- [x] 3.1 Create `.github/workflows/release.yml`: trigger on push of tags `v*`; job on `ubuntu-24.04` with `permissions: contents: write`; checkout, stable toolchain, `cargo build --release --locked`, `strip` both binaries into a staging dir with `README.md`, `tar czf niri-kbinds-<tag>-x86_64-linux-gnu.tar.gz`, then `gh release create <tag> <bundle> --generate-notes`. Verify the YAML parses and the bundle name derivation (`${GITHUB_REF_NAME}`) matches the spec example
- [x] 3.2 Add a "download a release" subsection to the README install section (unpack tar.gz, `cp kbinds kbinds-overlay ~/.local/bin/`). Verify the documented bundle file name matches what `release.yml` produces

## 4. Hosted repository

- [x] 4.1 Confirm name and public visibility with the user, then `gh repo create ys-kalyakin/niri-kbinds --public --source . --remote origin` (ssh). Verify `git remote -v` shows ssh `origin` and `gh repo view` succeeds
- [x] 4.2 Push `main` with full history (`git push -u origin main`). Verify `git ls-remote origin main` matches local HEAD

## 5. CI verification (integration)

- [x] 5.1 Watch the first check run on `main` (`gh run watch`), verify it is green and all four gates executed. PR-trigger behavior is verified by the first real pull request; no dummy PR is created

## 6. First release (integration)

- [x] 6.1 Tag `v0.1.0` on the release-workflow commit and push the tag; watch the release run (`gh run watch`). Verify the Releases page shows the bundle and auto-generated notes
- [x] 6.2 Download the bundle into a temp dir, unpack, and run `./kbinds --help` and `./kbinds-overlay --help`; verify both exit 0 and the bundle file name contains `v0.1.0`

## Workflow follow-up

- After acceptance, run `/opsx:archive` to sync `ci-checks` and `release-publishing` deltas into the main specs
- Later (out of scope here): branch protection requiring the check workflow; SHA-pinning the actions; musl/aarch64 bundles; tag↔manifest version consistency gate
