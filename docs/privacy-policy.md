<!--
This page mirrors `apps/desktop-app/src/app/content/privacyPolicy.ts`, which is the canonical text and
also drives the Privacy dialog in Settings → About inside the app. The two are kept in sync by
convention, the same way `docs/analytics.md` and `what-pitcrew-sends.md` are — a change to what PitCrew
collects updates both. `what-pitcrew-sends.md` is the short, conversational version of the same facts;
this page is the fuller policy shape (third parties, retention, contact) that a privacy policy needs and
a driver FAQ doesn't.
-->

# Privacy policy

> **Draft for legal review.** This policy accurately describes what the current build of PitCrew does,
> but it has not been reviewed by a lawyer and is not yet Igor's final word on the subject — treat it as
> a statement of fact about the code, not as a finished legal document.

Effective date: *[effective date — set when this leaves draft]*

## The short version

PitCrew is free, has no accounts, and shows no ads. It reports what happens in the app — that a race was
finished, where you were classified, how much the engineer said — never who you are or how you drive.
There is no setting to turn this off, which is exactly why this page has to be specific rather than
reassuring.

## The phone app collects nothing

Everything else on this page is about PitCrew for Windows. The PitCrew phone app is a separate program:
it finds and remembers a rig, then opens its Companion or PIT page in a WebView. Its answer is much
shorter: it collects nothing at all. No account, no sign-in, no usage measurement, no crash reporting,
no advertising identifier, no ads.

- It talks to exactly one thing: the PC on your own network that is running PitCrew. What passes between
  them stays on that network, and none of it reaches us.
- The camera is used for one thing — reading the QR code on the desktop's screen so the phone can find
  the rig. The frames are read on the device, and are never stored and never sent anywhere. It asks the
  first time you open the scanner, and the app works without it if you type the address instead.
- The rig you last connected to is kept on the phone itself. Companion and PIT keep their layout and
  appearance in the browser or phone WebView that opened them, and nowhere else.

## What we collect, and why

Usage measurement goes to Google Analytics. Every one of the following is an event about the app, not
about you:

- That the app was opened, and which screens you visited — which parts of the app are actually used.
- That the first-run voice download finished, or failed — a failed download leaves an app that cannot
  speak, and most people never report it.
- That you finished setup, and how far you got if you did not — whether the setup steps are getting in
  the way.
- That PitCrew found iRacing and started reading telemetry — an install that never manages this looks
  exactly like one that was never opened, and the two need opposite fixes.
- That a race was logged: laps, where you were classified, whether you finished, how many calls the
  engineer made, and whether you beat your own best lap — whether the app is actually used to race, for
  whole races.
- That you changed the engineer's voice — whether people get sound working.
- That an ElevenLabs key was checked, and which of five outcomes came back — a studio voice can fail
  five different ways, each with a different fix.
- That a phrase cache finished warming, and whether you stopped it part-way — stopping it means a cost
  surprised you, which is a bug in the app, not a statistic.
- That the in-race overlay was asked to appear — whether it is worth the work it takes.
- That you clicked through to PayPal, and which amount — the click, not a payment; what happens on
  PayPal is not visible to PitCrew.
- Which language the engineer speaks, whether a studio voice is configured, whether setup is done, and
  the app's version — so all of the above can be read separately per language and version.
- Track and car names, with some of the events above. These are iRacing's own content names — the same
  ones on any leaderboard — and they decide which tracks and cars are worth the work of authoring more
  phrases for.

## What we never collect

Not as a field, not as a parameter, not as a user property:

- **Your name** — not the one you gave PitCrew, and not the one on your iRacing account.
- **The names you gave your engineers.**
- **Your ElevenLabs API key**, or which specific voice you use. Only whether a studio voice is set up.
- **Your lap times, your telemetry, or your inputs.** What a finished race reports is a result and a lap
  count, never a trace of your driving.
- **Anything you typed**, any file path, and any file name — including the name of a recording you
  replay.
- **The words the engineer says.** Phrase text never leaves your machine for measurement. (If you
  configure a studio voice, phrase text does go to ElevenLabs to be spoken — that is a service you set
  up yourself, billed to your own account, and it is not part of this collection; see "Third parties"
  below.)
- **Anything to do with advertising.** Google's advertising and ad-personalization signals are switched
  off in the app's configuration, so none of this can be joined to an ad profile or used to target you.

## Crash reports

Separately from usage measurement, if PitCrew crashes, a crash report goes to Sentry so the crash can be
fixed. It carries the error and where in the code it happened. It does not carry personal details unless
that is explicitly switched on with the environment variable `RACE_ENGINEER_SENTRY_PII=1`, which is off
by default. Setting `RACE_ENGINEER_SENTRY=off` disables crash reporting completely.

## Third parties and processors

PitCrew sends data to three outside services, and only three:

- **Google Analytics 4** — the usage events described above, under Google's own property for this app.
  Google's advertising features are switched off for it.
- **Sentry** — crash reports only, described above, and only when a crash actually happens.
- **ElevenLabs** — only if you paste your own API key into PitCrew's settings to use the studio voice.
  When configured, the text of the phrase being spoken is sent to ElevenLabs to render the audio. This
  is a service you opted into and pay for on your own account; it is not part of PitCrew's own data
  collection and PitCrew does not see your ElevenLabs usage or billing.

## Retention, and the client id

PitCrew generates a random identifier (a v4 UUID) the first time it runs and keeps it on your machine.
It is a random number — nothing about it is derived from your PC, your name, or your hardware. Its only
purpose is to tell "the same install used the app twice" from "two installs used it once", which is what
makes retention measurable at all; it cannot do anything else. Clearing PitCrew's stored data resets it,
and PitCrew then counts the install as new.

Data sent to Google Analytics and Sentry is retained under those providers' own standard retention terms
for the properties PitCrew uses; PitCrew does not operate a separate database of its own usage data.

## Your choices

There is no in-app switch for this, and pretending otherwise would be worse than saying so plainly. Two
things do work:

- **Block it at the network.** All measurement goes to `googletagmanager.com` and `google-analytics.com`.
  Blocking those — in your DNS, a firewall, or a hosts file — stops it. PitCrew is built so a blocked
  request cannot break anything: the engineer, the overlay, and the download button all keep working
  exactly as they do now.
- **Say so.** [Open an issue](reporting-issues.md) if a switch matters to you — that is a good enough
  reason to build one.

## Changes to this policy

If what PitCrew collects changes, this policy changes with it, in the same pull request — the app's own
contribution rules require that a change to collection updates the changelog and this policy together,
not one and then the other later.

## Contact

Questions, or a request about your data: [open an issue](reporting-issues.md) on the public repository.
PitCrew has no support email and no account system to look you up by — the issue tracker is the one
channel.
