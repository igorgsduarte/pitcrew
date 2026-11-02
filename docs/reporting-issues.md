# Reporting a problem

**[Open an issue →](https://github.com/igorgsduarte/pitcrew/issues/new/choose)**

Before you do, one pass through [troubleshooting](troubleshooting.md) is worth it — several of the
most common reports have an answer there, and the rest of that page tells you where the log is.

## What makes a report actionable

PitCrew talks to you during a live race, which means most bugs are about **a moment**: something was
said, or was not said, at a particular point in a particular session. Reconstructing that moment is
the whole job, so the useful details are the ones that pin it down.

Please include:

1. **The PitCrew version.** It is on the release you installed, and in the app.
2. **What you expected to hear, and what happened instead.** "It should have warned me about fuel on
   lap 12 and said nothing" is a report. "Fuel is broken" is not yet one.
3. **The log file.** This is the single most valuable thing you can attach:

   ```
   %APPDATA%\race-engineer\race-engineer.log
   ```

   Attach the file itself rather than pasting a fragment — the line that explains a problem is often
   nowhere near the line where you noticed it. If you have raced since, check
   `race-engineer.log.1` too; the log rotates at 1 MB.

4. **The session**, if the bug is about something spoken: car, track, session type (practice /
   qualifying / race), and whether it was multiclass. Several calculations behave differently across
   those, so this often *is* the reproduction.
5. **Your spoken language and voice**, and whether you have an ElevenLabs key configured. A
   keyless install and a keyed one take different paths through the speech engine.
6. **Windows version**, and your monitor setup if the problem is with the overlay.

## What it is safe to attach

The log records events, paths and errors — not your iRacing credentials and not an API key. When a
call fails to play, the log quotes that call, and a call can name another driver. Have a look before
you attach it if you would rather be sure; it is plain text.

## Files that name other drivers

The issue tracker is public: anyone can read and download what you attach. Two files can help with a
bug about a call, but they hold data about other people. **Do not attach them to an issue.**

| File | Where it is | What it holds |
| --- | --- | --- |
| Decision log (`decisions-*.ndjson`) | `%APPDATA%\race-engineer\decision-logs\` | Every call of one session, with its text. A call can name another driver. |
| Session recording (`.bin`) | The file that **Record this session** shows in Sims → iRacing | The session data from iRacing, with the name and iRacing ID of every driver in the session. |

If you have one of these files for the session with the problem, say so in the issue and keep the
file.

## Asking for something new

Feature requests are welcome, including small ones — **"I want it to tell me X"** is a completely
reasonable issue, and it is often cheap to add, because what PitCrew says lives in a phrase catalog
rather than in code.

The more useful version of that request describes the *moment* rather than the sentence: when you
would want to hear it, and what you would do differently as a result. That is what decides whether a
call is worth the airtime — during a race, every line spoken is a line that talked over another one.

## Pull requests

Please open an issue instead. This repository holds only the documentation and the releases; the
source is developed privately, so there is nothing here for a code change to land against.

Documentation fixes are the exception in spirit but not in mechanics — the files here are generated
from the private repository, so a PR against them would be overwritten on the next release. Report
the error in an issue and it will be fixed at the source.
