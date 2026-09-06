# PitCrew

**A voice race engineer for iRacing.** It talks to you while you drive — the spotter, your fuel,
your gaps, your position — so you never have to look away from the track to read a number.

PitCrew uses **no voice-actor recordings**. Every line is synthesized, which buys it an unlimited
vocabulary: it reads your actual opponents' names out of the session, spells out lap times, and
combines words freely instead of picking from a fixed clip library.

Windows 10 / 11, 64-bit. One `.msi`, no .NET, no Python, nothing else to install.

---

## Download

**[Download PitCrew →](https://github.com/igorgsduarte/pitcrew/releases)**

PitCrew is in **beta**, so its releases are marked as pre-releases and the newest one is at the top
of that page. Grab the `.msi` from it. (The `/releases/latest` shortcut deliberately skips
pre-releases, so it will only start working once there is a stable build.)

Every release lists what changed in it. The full history is in [CHANGELOG.md](CHANGELOG.md).

## Documentation

| | |
|---|---|
| [Installing](docs/install.md) | The `.msi`, the Windows SmartScreen warning, and what the first run downloads |
| [Getting started](docs/getting-started.md) | Setting up your voice, the overlay, and what PitCrew says during a race |
| [Voices and languages](docs/voices.md) | The local voice, the optional cloud voice, and which languages are spoken |
| [Talking to your engineer](docs/voice-commands.md) | Hold a key and ask about fuel, position, gaps or laps left — recognised on your own PC |
| [Troubleshooting](docs/troubleshooting.md) | It says nothing, the overlay is invisible, the download stalls |
| [Reporting a problem](docs/reporting-issues.md) | What to include so a bug can actually be fixed |
| [What PitCrew sends](docs/what-pitcrew-sends.md) | The complete list of what is measured, what never leaves your machine, and how to block it |

## What it currently speaks

PitCrew speaks **Brazilian Portuguese** or **English (US)** — you pick, and it is independent of the
interface, which is in English. Spanish, German and French are written but not yet released; they
appear in the language list as *coming soon*.

## Status

**Beta.** The telemetry reader, the spotter, fuel, gaps and positions, cross-session memory and the
overlay are all built and unit-tested. What that does *not* mean is that every one of them has been
through a full race on somebody else's hardware — you are early, so expect rough edges, expect the
settings screens to keep moving, and please [report what
breaks](docs/reporting-issues.md).

The local voice is the one that has been listened to properly. The optional cloud voice is present
but unproven — see [voices and languages](docs/voices.md).

## Something to report?

**[Open an issue](https://github.com/igorgsduarte/pitcrew/issues/new/choose)** — bugs, a phrase that
came at the wrong moment, or something you want it to say. Please read
[docs/reporting-issues.md](docs/reporting-issues.md) first; a report with the app's log attached
gets fixed, and one without usually cannot be.

## About this repository

This repository is PitCrew's **public home**: the documentation, the changelog, the releases, and
the issue tracker. The source code is developed privately, and everything here is published from
there automatically — so please open issues rather than pull requests.
