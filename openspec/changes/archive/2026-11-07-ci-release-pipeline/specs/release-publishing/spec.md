# Spec Delta

## Purpose

Tag-driven publication of releases: pushing a version tag produces a public GitHub Release with a downloadable tar.gz bundle of the two binaries, so users can install the tool without building from source.

## ADDED Requirements

### Requirement: Releases are triggered by version tags

A release workflow SHALL run automatically when a tag whose name starts with `v` followed by a version (`v*`) is pushed to the hosted repository. Pushes of branches or tags not matching `v*` SHALL NOT produce releases. The published release SHALL be tied to that tag.

#### Scenario: Version tag publishes a release
- **WHEN** a tag named like `v0.1.0` is pushed to the hosted repository
- **THEN** the release workflow builds the bundle and publishes a GitHub Release for that tag with the bundle attached

#### Scenario: Non-version push publishes nothing
- **WHEN** commits are pushed to a branch, or a tag not starting with `v` is pushed
- **THEN** no release workflow runs and no release is created

### Requirement: Release bundle contents

The release workflow SHALL publish exactly one bundle artifact per release: a `.tar.gz` archive named after the tag and target triplet `x86_64-unknown-linux-gnu`, containing the two built binaries `kbinds` and `kbinds-overlay`. The binaries SHALL be release builds built from the tagged commit with the committed lockfile in locked mode.

#### Scenario: Bundle is installable
- **WHEN** a user downloads the bundle from a release, unpacks it, and runs `./kbinds` or `./kbinds-overlay`
- **THEN** both binaries execute on a Linux x86_64 system with glibc no older than the release runner's

#### Scenario: Bundle matches the tag
- **WHEN** the release workflow runs for tag `v1.2.3`
- **THEN** the bundle file name contains `v1.2.3` and its binaries are built from the commit the tag points to

### Requirement: Release publication

The published GitHub Release SHALL be created for the pushed tag with the bundle attached and with release notes auto-generated from the commits between the previous release and this tag.

#### Scenario: Release is visible on the repository
- **WHEN** the release workflow succeeds
- **THEN** the repository's Releases page shows a release for the tag with the `.tar.gz` bundle downloadable and generated notes listing the included changes
