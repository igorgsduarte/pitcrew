# Talking to your engineer

Press a key or a button on your wheel, ask a question, get an answer in your engineer's own voice.
Everything happens on your own PC after a one-time model download: no account, no key, and nothing
you say leaves the machine.

## Turning it on

Open **Voice commands** from the rail to set up your microphone, push-to-talk key or wheel button,
and the phrases PitCrew listens for.

Start on **Voice commands**:

1. **Switch it on.** It is off on a fresh install.
2. **Check the key.** `Alt+V` by default. You can change it, and a change takes effect the next time
   you start PitCrew. If what you type will not work — a key Windows will not hand over, a letter with
   no `Alt` or `Ctrl` in front of it, or a chord PitCrew already uses — the screen says so under the
   field, and *Restore Alt+V* puts it back.
3. **Bind a button on your wheel.** See below. This is the one most drivers want.

On the right-hand side:

4. **Pick your microphone.** Leave it on *System default* unless you have more than one and PitCrew
   picks the wrong one.
5. **Check the level.** Switch on *Check the level* and say something — the bar should move. If it
   does not, PitCrew is not hearing you, and no amount of recording is going to fix that. It also
   tells you when you are too loud. Turn on *Hear yourself* to be played back through the same
   speakers your engineer talks out of, which is the quickest way to judge how you actually sound —
   **wear headphones for this**, or your speakers will feed back into the microphone. It closes the
   microphone the moment you switch it off or leave the screen.
6. **Choose the phrasings you want to view or record.** See below.
7. **Download Parakeet TDT** — about 670 MB, once. PitCrew never fetches it on its own, which is why
   the button is there. Your engineer can speak without it; it is only needed to hear your commands.

If you already have the matching Parakeet TDT 0.6B v3 int8 export on your PC, put its folder — with
the encoder, decoder, joiner and `tokens.txt` together — in *Model folder* and skip the download.
If the model is incomplete or damaged, the screen tells you which files need attention and offers a
download.

## The language you ask in

Your engineer's language and your own are two separate settings.

The one your engineer *answers* in is on the Voice screen. Parakeet detects Portuguese and English
utterances automatically, even if you switch between them from one question to the next. The
**Phrasings to edit** picker on Voice commands chooses which language's recorded phrasings you see
and add; it does not force the recogniser to decode in just that language. A Brazilian Portuguese
engineer answering a question asked in English is a perfectly ordinary setup, and so is the reverse.

The picker also offers Spanish, French and German. Choosing one includes its phrasings alongside
Portuguese and English while leaving your engineer's output language alone.

Anything you have recorded stays filed under the language you recorded it in. Switching the picker
shows that language's phrasings; switching back brings the others straight back. Nothing is thrown away.

## Using a button on your wheel

A key means letting go of the wheel, which is the last thing you want to do in the middle of a corner.
Bind a button instead:

1. `Voice commands → Push to talk → Wheel or gamepad`.
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
`Voice commands → Hold it, or tap it` to **Tap on, tap off**. Then one press
opens the microphone and the next one closes it. If you tap it open and get distracted, it closes
itself after twenty seconds, so you can never leave it open by accident.

You will hear a **radio click** when the microphone opens, and a lower one when it closes. **Wait for
the first one before you start talking** — that pause is not politeness, it is what stops your first
word being cut off. PitCrew discards the click from the recording. If you would
rather it was a plain tone, or silent, `Voice commands → Sound when the mic opens` has
both. Silent works, and the first word is on you.

You do not have to wait for a gap in the conversation. Ask whenever you like; the answer arrives
when the engineer is next able to speak. A spotter call always wins that
race — a car on your left matters more than the fuel number you asked for a moment ago.

## What you can ask

Say any of these, or something close to them. Portuguese and English both work without switching
language; the examples below are in Portuguese regardless of what language your engineer speaks.

| Ask | Say | You get |
| --- | --- | --- |
| Fuel | *"quanto combustível eu tenho?"* | How many laps are left in the tank, and how much is in it |
| Position | *"em que posição eu estou?"* | Where you are running |
| The car ahead | *"quanto tempo pro carro da frente?"* | Who is ahead in your class, and by how much |
| The car behind | *"quanto tempo pro carro de trás?"* | Who is behind in your class, and by how much |
| How much is left | *"quantas voltas faltam?"* | Laps remaining, or minutes in a timed race |
| Your last lap | *"quanto foi minha última volta?"* | Your last lap time |
| The engine | *"como está o motor?"* | Water and oil temperature, and oil pressure |
| The leader | *"quanto pro líder?"* | Who leads your class, and by how much time or how many laps |
| Lap times around you | *"última volta do carro da frente"*, *"... do carro de trás"*, *"... do líder"* | That car's last lap time |
| Your best lap | *"qual foi minha melhor volta?"* | Your best lap time this session |
| Time left | *"quanto tempo falta?"* | Time remaining, or laps in a race run to a lap count |
| Fuel to the end | *"quanto combustível pra terminar?"* | How much fuel you still need to finish, or that you have enough |
| Laps since the stop | *"quantas voltas desde o box?"* | Laps on this stint, or that you have not stopped yet |
| Tyres | *"como estão os pneus?"* | Tread left on each corner, as measured at your last stop |
| Repairs | *"quanto tempo de reparo?"* | Required and optional repair time, or that there is nothing to fix |
| Incidents | *"contador de incidentes"* | Your incident count, and how many are left before the limit |
| iRating | *"qual o meu irating?"* | The iRating you joined the session with |
| Licence | *"minha licença"* | Your licence class and safety rating |
| Strength of field | *"força do grid"* | The strength of field of your class |
| Temperatures | *"temperatura da pista"* | Track and air temperature |
| Say that again | *"fala de novo"* | The last call, repeated |
| Silence | *"cala a boca"* | The engineer stops speaking |
| Speak again | *"volta a falar"* | It starts again |

**Say the whole phrase, not one word.** Shorter forms are listed in the app — *"combustível?"*,
*"posição?"* — but a whole sentence gives the recogniser more to go on and lets the matcher forgive
one missed word. *"Gap"* on its own is better said as *"quanto tempo pro carro da frente"*.

## When it says "não entendi"

That is the feature, not a fault. PitCrew understands a **fixed list** of questions rather than
anything you care to say, and anything it is not certain of gets *"não entendi, fala de novo"* —
it asks you to say it again rather than guessing. A wrong answer during a race is worse than no
answer: being told you have five laps of fuel when you have one is not a mistake you get to notice
in time.

If it happens often, `Voice commands` shows the last thing it actually heard. That is the
quickest way to tell the two problems apart:

- **Nothing shown, or nonsense** — it is not hearing you. Open the Voice commands screen and switch on
  *Check the level*: if the bar does not move when you speak, the microphone is the problem and no
  phrasing will fix it. Also check you are waiting for the click before you speak rather than talking
  over it. Portuguese and English need no input-language switch.
- **Roughly what you said, but no answer** — it heard you and that phrasing is not on the list. Try
  one of the phrasings in the table above, or teach it yours: see *Teaching it how you ask* below.

Some questions have an honest answer that is not a number. On the opening lap there is no fuel
average yet, so *"combustível?"* gets *"ainda não tenho esse dado"* rather than an invented figure. The
tyres are the same: iRacing reports tread only while the car sits in its pit box, so before your
first stop the engineer tells you they have not been measured yet, and after it the answer is the
reading from that stop.

## Changing the pit stop

You can also set the next stop by voice. The engineer sends it to iRacing's black box, then tells you
what the car shows.

| Change | Say | What is set |
| --- | --- | --- |
| Add fuel | *"coloca vinte litros"* | That many litres to add |
| Fuel to the end | *"abastece até o fim"* | Enough to finish if you stop at the end of this lap, with a small reserve |
| All four tyres | *"troca os quatro pneus"* | All four corners |
| Some tyres | *"só os dianteiros"*, *"só os traseiros"*, *"pneus da esquerda"*, *"pneus da direita"* | Those corners only |
| No tyres | *"sem pneus"* | Every corner unticked |
| Fast repair | *"reparo rápido"* | The fast repair box |
| Tear-off | *"tear off"* | The windscreen tear-off |

A few things work differently from a question:

- **Each command changes only what it names.** Asking for fuel leaves your tyres, fast repair and
  tear-off exactly as they were.
- **What you hear back is what the car shows.** If iRacing caps the fuel to the space left in the
  tank, you hear the capped amount. If the car never shows the change, the engineer tells you to
  check the black box instead of pretending it worked.
- **Your pit plan waits.** After you change fuel or tyres by voice, automatic arming from your pit
  plan does not overwrite it. The plan takes over again after you make the stop, or when you press
  the preset Clear button. Fast repair and the tear-off never stop the plan.
- **It has to hear the whole command.** A question can be understood with a word missing; a command
  cannot, and a command heard with *"não"*, *"sem"* or *"not"* where it does not belong is ignored.
  *"Fuel to the end"* with no measured fuel usage yet, or with enough fuel already on board, sends
  nothing and tells you why.

## Teaching it how you ask

The phrasings above are ours. The ones you record on your own headset may be easier for the model to
hear when you are driving. **How you say it matters.**

Open **Language → You ask** to see every answer your engineer can give and the ways you can ask for
it. The **Voice commands** screen holds your microphone settings and the *Phrasings to edit* picker;
choose a language there before recording it on the Language screen.

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

The recogniser is [NVIDIA Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3),
under CC BY 4.0, and it is credited on the [license page](license.md). PitCrew does not redistribute
it: your PC downloads a pinned ONNX export when you press the button.

## What it does not do yet

- **It cannot command the car or change strategy.** *"Boxes essa volta"* is not a command it takes.
  A misheard question costs a wrong sentence; a misheard instruction could cost a pit stop.
