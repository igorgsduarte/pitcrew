# What PitCrew sends

PitCrew is free, has no accounts and no ads, and nobody is being sold anything. It does send some
information about how the app is used, and this page is the complete list. There is no setting to turn
it off — so the least this page owes you is to be specific rather than reassuring.

This page is the short, conversational version. [The privacy policy](privacy-policy.md) covers the same
facts in the fuller shape a policy needs — third parties, retention, contact — and is also what
Settings → About → Privacy shows inside the app.

**An earlier version of the release notes said PitCrew made no analytics requests at all. That is no
longer true**, and this page exists because saying so plainly is better than quietly leaving the old
sentence in place.

---

## The short version

PitCrew reports **what happens in the app**, never **who you are or how you drive**.

It knows that a race was finished, where you were classified, and how much the engineer said. It does
not know your name, your iRacing account, or where any of it lives on your disk.

---

## What is sent

Usage measurement goes to Google Analytics. Every one of these is an event about the app:

| What | Why it is worth knowing |
| --- | --- |
| The app was opened, and which screens you visited | Which parts of the app are actually used |
| The first-run voice download finished, or **failed** | This is the biggest one. A failed download leaves an app that cannot speak, and most people never report it |
| You finished setup, and how far you got if you did not | Whether the setup steps are getting in the way |
| PitCrew found iRacing and started reading telemetry | An install that never manages this looks exactly like one that was never opened, and the two need opposite fixes |
| A race was logged: laps, where you were classified, whether you finished, how many calls the engineer made, and whether you beat your own best lap | Whether the app is actually being used to race, for whole races |
| You changed the engineer's voice | Whether people get sound working |
| An ElevenLabs key was checked, and which of the five answers came back | "The studio voice doesn't work" has five different causes and five different fixes |
| A phrase cache finished warming, and whether you stopped it part-way | Stopping it means a cost surprised you, which is a bug in the app, not a statistic |
| The overlay was asked to appear | Whether the in-race overlay is worth the work it takes |
| You clicked through to PayPal, and which amount | Whether the donation card is worth its space. **The click, not a payment** — what happens on PayPal is not visible to PitCrew |
| Which language the engineer speaks, whether a studio voice is configured, whether setup is done, and the app's version | So all of the above can be read separately for, say, a Portuguese install on 0.1.1 |

Track and car names are sent with some of this. They are iRacing's own content names — the same ones
on any leaderboard — and they decide which tracks and cars are worth spending work on.

Separately, if PitCrew **crashes**, a crash report goes to Sentry so the crash can be fixed. It carries
the error and where in the code it happened. It does not carry personal details unless it is explicitly
switched on with `RACE_ENGINEER_SENTRY_PII=1`, which is off by default. `RACE_ENGINEER_SENTRY=off`
disables crash reporting completely.

---

## Checking for updates

A few times a day PitCrew asks GitHub whether there is a new engineer or new phrases. It is a request
for a small file, it **sends nothing about you or your PC**, and it is not measurement — there is no
identifier on it and nothing is recorded on our side, because there is no "our side": the file is
served by GitHub like any other public download.

Nothing is downloaded or installed unless you say so, twice — once for the download, once for the
install. If you would rather it did not check at all, blocking `raw.githubusercontent.com` stops it,
and PitCrew keeps working exactly as it does now.

---

## What is never sent

- **Your name.** Not the one you gave PitCrew, and not the one on your iRacing account.
- **The names you gave your engineers.**
- **Your ElevenLabs API key**, or which specific voice you use. Only *whether* a studio voice is set up.
- **Your lap times, your telemetry, or your inputs.** Your best lap stays on your machine. What a
  finished race reports is a result and a lap count, not a trace of your driving.
- **Anything you typed**, any file path, and any file name — including the name of a recording you
  replay.
- **The words the engineer says.** Phrase text never leaves your machine for measurement. (If you have
  configured a studio voice, phrase text does go to ElevenLabs to be spoken — that is the service you
  set up, billed to your own account, and it is not part of this.)
- **Anything you say.** If you switch voice commands on, PitCrew opens your microphone *only while you
  hold the push-to-talk key* — there is no wake word and it is never listening in the background. The
  recording is turned into text on your own PC, matched against a fixed list of questions, and thrown
  away. **No audio and no transcript is uploaded anywhere, to us or to anyone else**, and neither is
  measured — not what you asked, not whether you were understood, not how often you ask. The screen
  shows you the last thing it heard; that readout is local and goes no further. See
  [talking to your engineer](voice-commands.md).
- **Anything to do with advertising.** Google's advertising and ad-personalisation features are
  switched off, so none of this can be joined to an ad profile or used to target you.

---

## How you are counted

PitCrew generates a random identifier the first time it runs and keeps it on your machine. It is a
random number — nothing about it comes from your PC, your name, or your hardware.

Its whole purpose is to tell "somebody used this app twice this week" from "two people used it once".
That is the difference between knowing whether drivers come back and not knowing, and it cannot do
anything else. Clearing PitCrew's stored data resets it, and PitCrew then counts you as new.

---

## If this is not acceptable to you

There is no in-app switch, and pretending otherwise would be worse than saying so. Two things that do
work:

- **Block it at the network.** All measurement goes to `googletagmanager.com` and
  `google-analytics.com`. Blocking those — in your DNS, a firewall, or a hosts file — stops it. PitCrew
  is built so that a blocked request cannot break anything: the engineer, the overlay and the download
  button all keep working exactly as they do now.
- **Say so.** [Open an issue](reporting-issues.md). If people want a switch, that is a good enough
  reason to build one.
