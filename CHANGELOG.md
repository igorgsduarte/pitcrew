# Changelog

Every notable change to PitCrew is recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**This file is the source of the release notes the driver reads.** It is mirrored verbatim into the
public repository ([igorgsduarte/pitcrew](https://github.com/igorgsduarte/pitcrew)) on every
release, and the section for a version becomes that release's body. Write entries for the driver,
not for the compiler: *"the spotter no longer calls a car that has already pitted"* rather than
*"guard `CarIdxOnPitRoad` in `spotter::alongside`"*.

Every pull request adds at least one entry under `## [Unreleased]`, in one of the four subsections
below — the CI gate rejects a PR that does not. The next version number lives in
[`.release/next.json`](.release/next.json), not here; cutting a release moves this section into a
dated one. See `docs/release-process.md`.

## [Unreleased]

### Added

- Windows installer builds and GitHub releases are now produced by CI on every commit to `main`,
  and published to the public repository together with these notes.

### Changed

### Fixed

### Removed
