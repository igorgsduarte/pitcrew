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

1. Download the `.msi` from the [latest release](https://github.com/igorgsduarte/pitcrew/releases/latest).
2. Run it. It installs to `C:\Program Files\PitCrew` and adds a Start-menu entry.
3. Launch **PitCrew**.

### Windows will warn you about the installer

The `.msi` is **not code-signed**, so Windows SmartScreen shows *"Windows protected your PC"* and
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

So the first time you launch PitCrew, onboarding downloads:

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
and the cache of already-spoken audio. It is deliberately **outside** `C:\Program Files`, which
means:

- an update never overwrites your settings or makes you download the voice again, and
- uninstalling PitCrew leaves that folder behind.

To remove PitCrew completely: uninstall it from **Settings → Apps**, then delete
`%APPDATA%\race-engineer\`.

## Updating

Download the newer `.msi` and run it — it replaces the installed version in place. There is no
in-app updater yet. Your settings, profiles and downloaded voice are untouched.

## Uninstalling

**Settings → Apps → Installed apps → PitCrew → Uninstall.** Add
`%APPDATA%\race-engineer\` to that if you want the voice model and settings gone too; it is the
biggest thing PitCrew leaves on disk.
