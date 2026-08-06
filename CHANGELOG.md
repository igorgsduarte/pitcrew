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
- The website uses Google Analytics to count visits and how many people click through to download.
- **PitCrew now measures how the app itself is used, and there is a page listing exactly what that
  means** — [what PitCrew sends](docs/what-pitcrew-sends.md). The short version: it reports what
  happens in the app, never who you are or how you drive. Whether the first-run voice download
  succeeded or failed, whether setup finished, whether PitCrew found iRacing, that a race was logged
  and how much the engineer said in it. Never your name, your engineers' names, your API key, your lap
  times or your telemetry, and nothing you typed. An earlier version of these notes said the app made
  no analytics requests at all; that is no longer true, and this replaces it rather than sitting next
  to it. There is no switch in the app — the page explains how to block it at the network, and PitCrew
  is built so that blocking it cannot break anything.
- **A way to support PitCrew, at the bottom of the home screen.** There is no paid version and
  donating unlocks nothing — every feature is already yours. It is there because the app has no paid
  tier, no ads and no account, and is written and tested by one person after work. Pick $5, $10, $25
  or your own amount; the button opens PayPal in your own browser, where you confirm before anything
  is charged. Your card details never reach PitCrew.
- **The same way to support PitCrew, on the website**, under the download button rather than in front
  of it. It offers the same $5, $10 and $25 as the app, or an amount you choose, and it says plainly
  what the money is for: the installer is not code-signed, so Windows warns you twice, and a signing
  certificate is the next thing that fixes for everyone who installs. Donating still unlocks nothing.

### Changed

- The website now says what the first launch actually does: PitCrew downloads a 310 MB voice model
  the first time you open it, and runs offline from then on. It previously promised that a clean
  install worked offline, which was never true — and a driver on a metered connection would have
  found that out after installing.
- The website lists the two languages PitCrew speaks today, Brazilian Portuguese and English, rather
  than the five it has phrases written for. Spanish, German and French still need a voice before they
  can be spoken, and the site now says so instead of counting them.
- Every quoted radio call on the website is shown in Portuguese *and* English. The page's own
  interface is English and its readers are not only Portuguese speakers, so a column of untranslated
  quotes was decoration where it was meant to be the evidence.
- The website's "who speaks what" table can now be read in any of the five languages PitCrew has
  phrases written for, and the lines in it are the real ones out of the phrase catalog rather than
  examples written for the page. Picking Spanish, German or French says plainly that the words exist
  but the voice does not yet.
- The website's sections have more room to breathe, and the closing invitation to download gets some
  runway before the ask.
- The website no longer assumes your engineer is a man. It says "them" where it said "him", and the
  two example engineers now carry voice names that match the names they are given — the local one was
  labelled with a female voice under a man's name.
- The website's links and buttons are big enough to hit with a thumb. On a phone the four section
  links at the top were about a third of the size they should have been, and they are the only way to
  skip a section on a page that runs to some nine thousand pixels.
- The website is served with a strict content-security policy and the usual browser protection
  headers, so nothing it did not ship can run on it, and it cannot be framed by another site.

### Fixed

- **The website's download button now downloads PitCrew.** Every link on the page pointed back into
  the page itself, so the button at the bottom — the one every other button leads to — did nothing at
  all when clicked. It, and the two above it, now go to the latest release.
- The website's footer links go where they say: the install guide, getting started, voices and
  languages, troubleshooting, the release notes and the issue tracker. All twelve previously scrolled
  back to the top of the page.
- The four numbered items in the website's header now scroll to the sections they name, and the
  header's own menu lists them in the order the page presents them. Section 03 was missing from the
  menu entirely, and the menu disappeared completely on a phone.
- Picking a voice in the website's voice-designer panel no longer has the panel change it back a few
  seconds later.
- The website no longer renders as a blank page when the browser is not running animations — a
  restored background tab, low-power mode, or an in-app browser. All of its text was hidden until an
  animation revealed it.
- Sharing a link to the website now produces a preview card with an image, instead of an empty one.

### Removed

- The website's hero is much quieter: eight things were moving continuously behind the headline, and
  now two are.
