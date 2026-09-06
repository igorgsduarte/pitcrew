# Voices and languages

## The two that are on by default

PitCrew ships with two speech engines turned on, and they are **split by role, not ranked against
each other**. There are more to try beside them — see *The engines in beta* below.

**Kokoro** runs on your own machine. Native, no network, no Python. It handles the lines that are
assembled around a live number — your position, a gap, a lap time, a comparison against your best —
because those cannot be recorded in advance and have to be produced the instant the number exists.

**ElevenLabs** is a cloud service, billed to *your own* account. It handles the fixed-text lines —
the spotter, fuel, pit, penalties, session calls — which are generated once and then played from
disk, so a race does not wait on the network.

You can override the assignment per category in the app's settings if you disagree with it, and you
can also choose which engine each of the three — the race engineer, the race strategist and the
spotter — speaks through.

## Where you choose it

`Voice → Engineers` has a card per engineer, and opening one asks three questions in order:

1. **On this PC, or online.** Not which engine — what *kind* of engine. On this PC downloads once, works
   offline and costs nothing; online needs an account and is billed per character.
2. **Which engine of that kind.**
3. **Everything that engine has** — its voice, its weights or its account, and how fast it reads.

On an engine that runs on your own PC, the voices are a list rather than a dropdown, and **every row
has a play button**: press it and you hear that voice read a line in this engineer's language, at this
engineer's speed, before you pick it. The row also shows a tick if it is the one this engineer is on,
and a download if the voice is not on your PC yet.

Picking the kind on its own changes nothing: it shows you that group's engines, and the engineer moves
only when you pick one of them.

Some of what you see in there belongs to your whole install rather than to one engineer — a key, a
download, a model file. Each of those is marked **All engineers**, because pasting a key under the
spotter's name is the same key the chief engineer uses.

## The engines in beta

Beside the two above there are three more you can switch an engineer to. All of them are **off until
you pick one**, and nothing about a fresh install changes if you never do.

**Piper** runs on your own machine, with fifteen voices across the five languages — nine of them in
English, three in Brazilian Portuguese and one each in Spanish, French and German. On this engine a
voice *is* a language, so the language you set for an engineer decides which voices they can be given;
set them to German and the German voice is what they get.

Each voice is a model of its own and **downloads on its own, about 63 MB** — you fetch the one you
want from the row it sits on, not the whole set. Piper counts as ready for an engineer as soon as
*their* voice is on disk. Moving an engineer to a language whose voice you have not fetched yet means
one more download, and the row says so before you start it.

It is a single pass through a small model, which makes it the quickest of the local engines to produce
a line on the machine it has been measured on; that measurement was not taken on Windows, so treat it
as promising rather than as a number.

One thing to know before you go looking: **Piper has no female voice in Brazilian Portuguese**, at any
quality. That is true of the voices published for it rather than a choice PitCrew made — all three of
its Portuguese voices are men. If that is what you want, Kokoro and Supertonic both have one.

**Supertonic** also runs on your own machine, with ten voices that come with the model and no key to
paste. There are **two versions of it** and the panel lets you pick: the newer one skips fewer words,
the older one sounds closer to Brazilian Portuguese. They share no files, so switching versions is a
second full download rather than a top-up — the panel shows the size of each before you commit.

**Azure AI Speech** is a cloud service like ElevenLabs, billed to your own account, with Brazilian
Portuguese voices of its own. It needs two things pasted rather than one: a key *and* the region it
belongs to.

Three things to know before you switch to any of them:

- **The files are fetched after installation, and PitCrew does it for you.** Nothing beyond the app
  itself is in the installer. It is one button in the engineer's panel with a progress bar, safe to
  press twice, and an interrupted download keeps whatever arrived in full — pressing again fetches
  only the rest. If you already have the files, point the panel's folder field at them instead: a
  second drive is fine, and nothing is copied.
- **Beta means the word.** Each of these has produced real audio in every language PitCrew offers,
  and each was listened to before it shipped. What none of them has is a season behind it, and how
  fast one produces a line on a rig that is also running the sim has not been measured.
- **Nothing is lost if one is not working.** If an engine's files are missing, or it fails on a line,
  Kokoro reads that line instead and PitCrew records that it did. You never lose a call over it.

Changing an engine, its voice or its version changes what comes out, so lines PitCrew has already
prepared keep the old sound until they are prepared again. That is why those controls say so, and why
*Warm cache* on the phrase catalog is the quickest way to hear a change everywhere at once.

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
- **If your key has restricted access, PitCrew says so when you paste it.** ElevenLabs lets you limit
  what a key is allowed to do, and a limited key still passes an ordinary "is this key valid?" check —
  it then refuses the thing you actually wanted, later, usually mid-race. So PitCrew checks the
  permissions too, names the one that is missing (`text_to_speech`, `voices_read`, …) and says what it
  costs you, with a link to the page where you enable it. A key that is missing one permission keeps
  working for everything else, and no call is ever lost: whatever the studio voice cannot say is read
  by the local voice instead.
- **When ElevenLabs refuses something, PitCrew tells you which of four reasons it was.** They look
  identical from the outside — the same refusal, on the same button — and they need four different
  things from you, so the app names the one that happened instead of showing you an error code:
  - your **plan** does not include that feature, with a link to the plans page;
  - your **key** is missing a permission, named, with a link to the page that has the checkbox;
  - ElevenLabs has **flagged the account** — it reports this as unusual activity, and a VPN or a shared
    connection on the free tier can trigger it. Pasting a different key will not help, because the
    block follows the account rather than the key. Only ElevenLabs can lift it;
  - the refusal **never reached ElevenLabs**: something between your PC and their API answered for it,
    which is what a filtering proxy, a company or school network, or an antivirus that inspects HTTPS
    traffic does. Nothing is wrong with your key or your account.

  In all four the engineer keeps talking — the local voice reads whatever the studio voice cannot.
- **PitCrew never synthesizes remotely unless you asked it to.** Warming the cache in a new voice is
  a real purchase, so it is always something you trigger — *Warm cache* on the phrase catalog,
  *Regenerate them now* on the voice screen, or the last step of setup, and nowhere else. Changing a
  setting never starts one. When phrases are missing, the header says so on every screen, with the
  count and roughly what generating them would cost; the run itself shows up as a bar at the bottom of
  the window, with a separate line for the voice on your PC and the studio voice, and a button to stop
  the paid half without stopping the free one.
- **You choose which model reads the lines, and PitCrew defaults to the cheap one.** Three are
  offered on *Voice → ElevenLabs*, with the price shown next to each: `eleven_flash_v2_5` (the
  default — fastest and half the cost per character), `eleven_turbo_v2_5` (steadier, same price) and
  `eleven_multilingual_v2` (the most expressive, at double). Switching models means the lines already
  generated were made by the old one, so PitCrew asks you to regenerate them; until you do, those
  calls fall back to the local voice.

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
| Spanish | Available |
| German | Available with Piper, Supertonic, Azure or ElevenLabs; Kokoro has no German voice |
| French | Available |

A language is not a translation layer bolted over Portuguese: the phrase catalog carries a real
sentence and number wording per language, so each one is phrased the way an engineer would actually
say it in that language. The language picker only offers choices the selected engine can synthesize.

## Changing voice or language later

Both are in the app's settings, and neither needs a reinstall. Changing the language downloads
whatever that language needs and re-prepares the cached lines. Changing the voice does the same for
the new voice pack (~0.5 MB).

You can also adjust the speech rate, the output device, and the volume — useful if you want the
engineer on your headset while the sim plays through speakers.
