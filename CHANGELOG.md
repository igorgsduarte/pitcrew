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

### Changed

### Fixed

### Removed

## [0.1.0] - 2026-08-05

### Added

- **The first public beta.** PitCrew speaks to you during an iRacing session: the spotter (a car
  alongside, which side, and when you are clear), fuel (consumption per lap, whether it reaches the
  end, and warnings as the tank runs down), gaps and position (filtered by class, so another class's
  leader is not reported as if it were your race), your lap times and personal bests, session flags
  and pit and penalty calls, and a memory of your best known lap for each car/track combination.
- **Brazilian Portuguese or English**, chosen independently of the interface, which is English.
  Spanish, German and French are written but not released yet.
- **A local voice that needs no account and no network.** Speech is synthesized on your own machine;
  an ElevenLabs API key is an optional upgrade to the studio voice, never a requirement, and an
  install without one gives up no information at all.
- **An overlay** over the sim — proximity bars and the last message as text — transparent,
  always-on-top and click-through, so it cannot steal a click mid-race. It keeps working with
  iRacing in exclusive fullscreen.
- **Three global shortcuts:** `Alt+H` shows or hides the overlay, `Alt+R` repeats the last thing
  said, `Alt+M` mutes the engineer.
- **Timing you can tune yourself** without waiting for a release — spotter debounce, fuel warning
  thresholds, the gap window and the opening silence, all in an optional `tuning.json`.
- Windows installer builds and GitHub releases are now produced by CI, and published here together
  with these notes.
