# Installing PitCrew

## Requirements

- **Windows 10 or 11, 64-bit.** PitCrew reads iRacing's telemetry through a Windows shared-memory
  interface, so there is no macOS or Linux build and there cannot be one.
- **iRacing**, obviously. PitCrew does not need to be running when you start the sim, and it does
  not need to be restarted when you change session.
- **An internet connection for the first run** — see [The first run downloads a
  voice](#the-first-run-downloads-a-voice) below. After that PitCrew works entirely offline.

Nothing else. No .NET, no Visual C++ redistributable, no Python. The installer works on a freshly
imaged Windows machine with no preparation.

## Install

1. Download the `-setup.exe` from the [releases page](https://github.com/igorgsduarte/pitcrew/releases)
   — the newest build is at the top. (Not `/releases/latest`: that shortcut skips pre-releases, and
   every beta is one, so it stays empty until the first stable build.)
2. Run it. It shows a license screen — PitCrew's terms and a short privacy notice, condensed onto the
   one screen the installer can show. The full [license](license.md) and
   [privacy policy](privacy-policy.md) are always reachable from **Settings → About** once PitCrew is
   installed. Accepting installs it for your Windows user, in `%LOCALAPPDATA%\PitCrew`, and adds a
   Start-menu entry. It does not ask for administrator rights.
3. Launch **PitCrew**.

Older PitCrew versions came as an `.msi` installed for every user. The new installer removes that
one first, and Windows asks your permission for that step, once.

### Windows will warn you about the installer

The installer is **not code-signed**, so Windows SmartScreen shows *"Windows protected your PC"* and
hides the Run button behind **More info → Run anyway**.

This is expected and it is not a judgment about the file — it is what Windows shows for any
installer from a publisher that has not bought a signing certificate. If that is not a trade you
want to make, that is a completely reasonable place to stop.

Verify what you downloaded came from here: the release page is the only place PitCrew is
published, and each release lists the file it built.

## The first run downloads a voice

The installer is small because it **does not bundle the voice model** — those are hundreds of
megabytes of neural-network weights, and shipping them inside every update would mean
re-downloading them on every update.

So on the first run — from the welcome screen, or later from the **Voice** screen — PitCrew fetches:

- the **Kokoro** speech model, about **310 MB**, and
- one **voice pack**, about **0.5 MB** each.

They come from the [Kokoro-82M-v1.0-ONNX repository on Hugging
Face](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX). The download shows progress and
can be resumed by relaunching if it fails. Once it is done, **PitCrew never needs the network
again** — speech is synthesized on your own machine.

If the download stalls, see [troubleshooting](troubleshooting.md#the-voice-download-stalls-or-fails).

## Where PitCrew keeps your things

Everything PitCrew writes lives under:

```
%APPDATA%\race-engineer\
```

That is your settings, your car/track profiles, the overlay position, the downloaded voice model,
and the cache of already-spoken audio. It is deliberately **outside** the folder PitCrew is
installed in, which means:

- an update never overwrites your settings or makes you download the voice again, and
- uninstalling PitCrew leaves that folder behind.

To remove PitCrew completely: uninstall it from **Settings → Apps**, then delete
`%APPDATA%\race-engineer\`.

## Updating

**PitCrew tells you.** A small panel appears in the bottom-right corner when there is a new engineer
or new phrases, and it waits. You accept the download; later, and separately, you accept the install.
Two answers rather than one, because a download spends your connection while you are doing something
else and an install stops the engineer for a few seconds to start it again.

- **Nothing happens on its own.** No download, no install, no restart you did not press.
- **Never during a session.** PitCrew does not even ask while iRacing is running, and the install
  button refuses if you go on track between the two answers.
- **"Not now" costs you nothing.** A download you already made stays on disk, and you are offered the
  install again next time.
- Everything is signed, and a file that is not exactly the one that was signed for is thrown away
  rather than installed. If an update does not start, the engineer you had comes back by itself.

**A new PitCrew itself comes through the same panel.** When you accept the install, PitCrew closes,
a small progress window installs the new version, and PitCrew opens again. Your settings, profiles
and downloaded voice are untouched. If your PitCrew came from the old `.msi`, Windows asks your
permission once, to remove it; after that, updates ask nothing.

You can still download the newest `-setup.exe` from the releases page and run it yourself. It
replaces the installed version in place.

## Uninstalling

**Settings → Apps → Installed apps → PitCrew → Uninstall.** Add
`%APPDATA%\race-engineer\` to that if you want the voice model and settings gone too; it is the
biggest thing PitCrew leaves on disk.
