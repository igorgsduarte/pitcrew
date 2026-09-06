# Talking to your engineer

Press a key or a button on your wheel, ask a question, get an answer in your engineer's own voice.
Everything happens on your own PC: no account, no key, no network, and nothing you say leaves the
machine.

## Turning it on

It is two screens, and the split is simple: **Settings** is how you open the microphone, **Voice
commands** is the microphone itself and everything it hears.

Start at `Settings → Voice commands`:

1. **Switch it on.** It is off on a fresh install.
2. **Check the key.** `Alt+V` by default. You can change it, and a change takes effect the next time
   you start PitCrew. If what you type will not work — a key Windows will not hand over, a letter with
   no `Alt` or `Ctrl` in front of it, or a chord PitCrew already uses — the screen says so under the
   field, and *Restore Alt+V* puts it back.
3. **Bind a button on your wheel.** See below. This is the one most drivers want.

Then press **Voice commands** on that same card to open the screen where the rest lives. Down the
right-hand side:

4. **Pick your microphone.** Leave it on *System default* unless you have more than one and PitCrew
   picks the wrong one.
5. **Check the level.** Switch on *Check the level* and say something — the bar should move. If it
   does not, PitCrew is not hearing you, and no amount of recording is going to fix that. It also
   tells you when you are too loud. Turn on *Hear yourself* to be played back through the same
   speakers your engineer talks out of, which is the quickest way to judge how you actually sound —
   **wear headphones for this**, or your speakers will feed back into the microphone. It closes the
   microphone the moment you switch it off or leave the screen.
6. **Choose the language you ask in.** See below.
7. **Download the recogniser** — about 43 MB, once. PitCrew never fetches it on its own, which is why
   the button is there.

If you already have these weights on your PC — the ONNX export of Whisper tiny — put the folder in
*Model folder* and skip the download.

## The language you ask in

Your engineer's language and your own are two separate settings.

The one your engineer *answers* in is on the Voices screen. The one *you ask* in is **You ask in**, on
the Voice commands screen, and it decides what PitCrew listens for and how it writes down what you
said. They do not have to match: a Brazilian Portuguese engineer answering questions asked in English
is a perfectly ordinary setup, and so is the reverse.

All five languages are offered here, including the ones your engineer cannot speak yet — PitCrew needs
a voice to answer in one and only needs to recognise you to be asked in one.

Anything you have recorded stays filed under the language you recorded it in. Switching shows you that
language's phrasings; switching back brings the others straight back. Nothing is thrown away.

## Using a button on your wheel

A key means letting go of the wheel, which is the last thing you want to do in the middle of a corner.
Bind a button instead:

1. `Settings → Voice commands → Push to talk — wheel or gamepad`.
2. Press **Press a button**.
3. Press the button you want, on the wheel, the button box or the pad. That is it — no restart.

It works with anything Windows sees as a game controller: a wheel, a separate button box, a handbrake
with buttons on it, an Xbox or PlayStation pad. Your **key still works too** — the two live side by
side, so you can keep `Alt+V` for setting things up at the desk and use the wheel button on track.

Two things worth knowing:

- **Pedals and most rotary dials cannot be bound.** Your rig reports those as dials rather than
  buttons, and PitCrew can only bind a button. If nothing happens when you press one, that is why.
- **If PitCrew says no controller is detected**, it is not seeing your wheel at all. Check it works in
  iRacing first, then start PitCrew again.

To change it later, press *Rebind*. To go back to the key alone, press *Clear*.

## Using it

**Hold it while you speak** and let go when you have finished — the key and the button both. The
microphone opens when it goes down and closes when it comes up, so there is no wake word and nothing
is ever listening in the background.

If holding a button through a corner is awkward — plenty of wheel buttons are — switch
`Settings → Voice commands → Hold it, or tap it` to **Tap to start, tap to stop**. Then one press
opens the microphone and the next one closes it. If you tap it open and get distracted, it closes
itself after twenty seconds, so you can never leave it open by accident.

You will hear a **radio click** when the microphone opens, and a lower one when it closes. **Wait for
the first one before you start talking** — that pause is not politeness, it is what stops your first
word being cut off, and PitCrew does not begin listening until the click has finished. If you would
rather it was a plain tone, or silent, `Settings → Voice commands → Sound when the mic opens` has
both. Silent works, and the first word is on you.

You do not have to wait for a gap in the conversation. Ask whenever you like; the answer takes about
half a second and arrives when the engineer is next able to speak. A spotter call always wins that
race — a car on your left matters more than the fuel number you asked for a moment ago.

## What you can ask

Say any of these, or something close to them. They work in every language PitCrew speaks, in that
language — the Portuguese below is what a pt-BR engineer expects.

| Ask | Say | You get |
| --- | --- | --- |
| Fuel | *"quanto combustível eu tenho?"* | How many laps are left in the tank, and how much is in it |
| Position | *"em que posição eu estou?"* | Where you are running |
| The car ahead | *"quanto tempo pro carro da frente?"* | Who is ahead in your class, and by how much |
| The car behind | *"quanto tempo pro carro de trás?"* | Who is behind in your class, and by how much |
| How much is left | *"quantas voltas faltam?"* | Laps remaining, or minutes in a timed race |
| Your last lap | *"quanto foi minha última volta?"* | Your last lap time |
| Say that again | *"fala de novo"* | The last call, repeated |
| Silence | *"cala a boca"* | The engineer stops speaking |
| Speak again | *"volta a falar"* | It starts again |

**Say the whole phrase, not one word.** Shorter forms are listed in the app and mostly work —
*"combustível?"*, *"posição?"* — but a single word gives the recogniser nothing to go on, and it is
the one thing that reliably fails. Measured on a real session, the same nine questions went from six
right to eight just by being asked as whole sentences. *"Gap"* on its own is the worst of them in
Portuguese and is better said as *"quanto tempo pro carro da frente"*.

## When it says "não entendi"

That is the feature, not a fault. PitCrew understands a **fixed list** of questions rather than
anything you care to say, and anything it is not certain of gets *"não entendi, fala de novo"* —
it asks you to say it again rather than guessing. A wrong answer during a race is worse than no
answer: being told you have five laps of fuel when you have one is not a mistake you get to notice
in time.

If it happens often, `Settings → Voice commands` shows the last thing it actually heard. That is the
quickest way to tell the two problems apart:

- **Nothing shown, or nonsense** — it is not hearing you. Open the Voice commands screen and switch on
  *Check the level*: if the bar does not move when you speak, the microphone is the problem and no
  phrasing will fix it. Also check you are waiting for the click before you speak rather than talking
  over it, and that *You ask in* is set to the language you are actually speaking.
- **Roughly what you said, but no answer** — it heard you and that phrasing is not on the list. Try
  one of the phrasings in the table above, or teach it yours: see *Teaching it how you ask* below.

Some questions have an honest answer that is not a number. On the opening lap there is no fuel
average yet, so *"combustível?"* gets *"ainda não tenho esse dado"* rather than an invented figure.

## Teaching it how you ask

The phrasings above are ours. Yours are better, and that is measurable: on a real session, changing
how the question was asked moved recognition from six right out of nine to eight. A bigger recogniser
moved nothing. **How you say it is the whole game.**

The **Voice commands** screen — the one you reached from Settings to set the microphone up — is where
this happens. On the left, every answer your engineer can give. In the middle, the ways you can ask
for the one you have selected. On the right, the microphone settings from earlier, so you can fix a
level or switch language without leaving.

**You record a phrasing — you never type it.** Press *Record a phrasing*, press and hold your
push-to-talk key or button, and say it the way you would say it on track. PitCrew shows you exactly what it heard and what would
happen if you kept it, and only then do you keep it.

That matters more than it looks. If you type *"gap a frente"*, you are guessing that your voice on
your microphone with an engine running produces that exact sentence — and you find out on track that
it does not. If you record it, it is something your microphone has already produced once.

**It keeps what it heard, not what you said, and that is on purpose.** If it hears *"quanta gasosa
ainda tem"* when you said *"quanta gasolina ainda tem"*, keeping the odd-looking version is the
right move: it is going to make the same mistake next time, and now the answer will be waiting for
it. Do not be put off by a phrasing that reads wrong. It is not a sentence, it is a fingerprint.

A few things worth knowing:

- **Record it three times.** Keep the ones that come back differently — each is another way the
  recogniser might hear you on the day.
- **Say a whole sentence.** Under three words PitCrew has to hear every single one; from three up it
  can miss one and still answer you. The screen tells you which side of that line you are on.
- **It will refuse some of them,** and it says why. Usually the phrasing already reaches a different
  answer, and having it reach two would mean *"não entendi"* every time you used it.
- **Ours and yours sit side by side,** each labelled, each removable on its own. *Reset to ours*
  puts one answer back the way it shipped.
- **Recording with the headset on beats recording at a desk.** The phrasings worth keeping are the
  ones that survive an engine at 8000 rpm.

Your phrasings are stored as *changes* rather than as a copy of the list, so a PitCrew update that
improves the built-in phrasings still reaches you. Removing one of ours keeps it removed.

Ours and yours are kept per language. Recording in Portuguese leaves your English phrasings exactly
where they were.

## What it costs

Nothing, and nothing leaves your PC. Recognition runs locally, so there is no account, no API key and
no per-question charge — the same standard as the voices that run on your own machine. The answers
come from the phrase catalog PitCrew already ships, in whichever voice that engineer already uses, so
asking a question never spends studio-voice credits either.

The recogniser is [Whisper tiny](https://github.com/openai/whisper), under the MIT license, and it is
credited on the [license page](license.md). PitCrew does not redistribute it: your PC downloads it
from the publisher when you press the button.

## What it does not do yet

- **A wheel button cannot be bound.** The binding is a keyboard shortcut. On Windows a wheel button
  is a different kind of device entirely, and PitCrew cannot see it yet.
- **It only asks.** Nothing you say changes the car or the strategy — *"boxes essa volta"* is not a
  command it takes. A misheard question costs you a wrong sentence; a misheard instruction would cost
  you a pit stop.
