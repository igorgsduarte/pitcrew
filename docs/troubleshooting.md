# Troubleshooting

Before anything else: **the log file is the answer to most of these.**

```
%APPDATA%\race-engineer\race-engineer.log
```

Paste that path into Explorer's address bar. It records startups, loads, state changes, errors and
fallbacks — including the cases where PitCrew recovered from something and kept going, which is
exactly the class of problem you would otherwise never see. It is capped at 1 MB, with one older
generation kept beside it as `race-engineer.log.1`.

---

## Windows blocked the installer

Expected — the `.msi` is not code-signed. **More info → Run anyway.** See
[installing](install.md#windows-will-warn-you-about-the-installer).

## The voice download stalls or fails

The first run downloads ~310 MB from Hugging Face. If it stops:

1. **Relaunch PitCrew and start it again.** A failed download does not have to begin from scratch.
2. **Check whether you can reach the source at all** —
   [huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX).
   Some corporate and school networks block it.
3. **Check the disk.** Budget about 400 MB free under `%APPDATA%`.
4. **Look at the log** — a refused connection, a proxy, and a full disk look completely different
   there and only one of them is worth retrying.

## PitCrew says nothing at all

In roughly the order worth checking:

1. **Is it muted?** Press **Alt + M**. Muting is silent by design, which makes it easy to forget.
2. **Is it connected to iRacing?** The main window's header says so. If it does not connect, make
   sure you are actually in a session — PitCrew reads live telemetry, and the sim publishes nothing
   from the menus.
3. **Is it playing to the device you are listening to?** The output device is a setting. If your
   headset arrived after PitCrew started, it may be talking to the speakers.
4. **Did the voice finish downloading?** With no model on disk there is nothing to speak with.
   Onboarding will say so; the log will too.
5. **Press Alt + R.** If the last message repeats, speech works and the real question is why nothing
   *triggered* — which is a different problem, and a much more interesting bug report.

If the log has a line about the announcer giving up, that is a genuine defect and worth an issue with
the log attached.

## It speaks, but the wrong language

The spoken language and the interface language are separate settings; the interface is always
English. Set the spoken one in the app's settings — see [voices and
languages](voices.md#languages).

If you picked a language that shows as *coming soon*, it is not released yet and will not be spoken.

## The overlay is invisible

1. **Press Alt + H.** It toggles, and it may simply be hidden.
2. **It may be off-screen.** The position is remembered, so a monitor you unplugged can leave it
   parked outside the desktop. Put it back with the overlay's edit mode from the main window.
3. **iRacing in exclusive fullscreen** is supported. If the overlay is invisible only in
   fullscreen and fine in windowed mode, that is a bug — please report it with your resolution and
   monitor layout.

## I clicked and it went into the game instead

That is deliberate: the overlay is click-through so it can never steal a click mid-race. To interact
with it, put it in edit mode from the main window.

## A call comes too early, too late, or too often

This is tunable without waiting for a release — spotter debounce, fuel thresholds, the gap window
and the opening silence all live in an optional `tuning.json`. See [tuning what it says and
when](getting-started.md#tuning-what-it-says-and-when).

If a call is not just mistimed for your taste but **wrong** — the spotter naming a side that has no
car on it, a fuel projection that contradicts the tank — that is a bug, and a good one to report.

## Multiclass gaps look wrong

Gaps and deltas are filtered by car class, so the car reported should be in your class. If you are
being told about a car from another class as though it were racing you, that is a defect worth
reporting with the session details.

## An update lost my settings

It should not: your settings, profiles and voice model live in `%APPDATA%\race-engineer\`, which no
installer touches. If something did disappear, please report it — and mention which version you
upgraded from.

## Still stuck

[Open an issue](https://github.com/igorgsduarte/pitcrew/issues/new/choose), and read
[reporting a problem](reporting-issues.md) first — mainly so you attach the log, which is the
difference between a bug that gets fixed and one that cannot be.
