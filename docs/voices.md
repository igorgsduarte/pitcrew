# Voices and languages

## The two engines

PitCrew can speak with two different speech engines, and they are **split by role, not ranked
against each other**.

**Kokoro** runs on your own machine. Native, no network, no Python. It handles the lines that are
assembled around a live number — your position, a gap, a lap time, a comparison against your best —
because those cannot be recorded in advance and have to be produced the instant the number exists.

**ElevenLabs** is a cloud service, billed to *your own* account. It handles the fixed-text lines —
the spotter, fuel, pit, penalties, session calls — which are generated once and then played from
disk, so a race does not wait on the network.

You can override the assignment per category in the app's settings if you disagree with it.

## You do not need an API key

**An install with no API key is a supported configuration, not a crippled one.** With no key saved,
Kokoro speaks everything, including the lines it would otherwise hand over — and PitCrew stays fully
offline.

What a keyless install gives up is the studio *voice*. It never gives up a call. Every warning,
every gap, every spotter callout still arrives, at the same moment, saying the same thing.

## If you do want to use ElevenLabs

Paste your key in the app's settings. Two things worth knowing before you do:

- **It costs you money.** ElevenLabs bills per character against your account, not PitCrew's. The
  app tells you what it is about to generate, and the pre-generation it kicks off can be cancelled.
- **PitCrew never synthesizes remotely unless you asked it to.** Warming the cache in a new voice is
  a real purchase, so it is always something you trigger.

> **Status, honestly:** the cloud engine's audio quality has not yet been through the same
> listening check the local voice passed. If a release note has not said otherwise, treat
> ElevenLabs support in PitCrew as unproven — the local voice is what has actually been
> validated. Nothing about this affects *what* is said.

## Languages

The language you **hear** and the language of the **interface** are two separate settings. The
interface is English. What you hear is your choice:

| Language | Status |
|---|---|
| Portuguese (Brazil) | Available — the first language, and the one with the most listening behind it |
| English (US) | Available |
| Spanish | Written, not released — shown as *coming soon* |
| German | Written, not released — shown as *coming soon* |
| French | Written, not released — shown as *coming soon* |

A language is not a translation layer bolted over Portuguese: the phrase catalog carries a real
sentence per language, so each one is phrased the way an engineer would actually say it in that
language.

The three unreleased ones already have their sentences written. What they are waiting on is a voice
that has been listened to properly in that language — shipping a language that sounds wrong is worse
than not shipping it.

## Changing voice or language later

Both are in the app's settings, and neither needs a reinstall. Changing the language downloads
whatever that language needs and re-prepares the cached lines. Changing the voice does the same for
the new voice pack (~0.5 MB).

You can also adjust the speech rate, the output device, and the volume — useful if you want the
engineer on your headset while the sim plays through speakers.
