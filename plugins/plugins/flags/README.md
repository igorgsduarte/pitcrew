# Flags

Shows the race flags on an 8×8 LED matrix, with animations. This is the official PitCrew plugin for
flag displays.

- **Sims:** every sim PitCrew reads (iRacing, Assetto Corsa, Assetto Corsa EVO). PitCrew decides
  the flag; the plugin only shows it.
- **Hardware:** an Arduino Uno or Nano, an ESP32 or an ESP32-C3, with an 8×8 WS2812B panel (colour)
  or a MAX7219 module (one colour), running the PitFlag firmware in [`firmware/`](firmware/).
- **Screen:** a **Flags** item in the sidebar. Until a device is set up it shows only the setup.
  After that, a Device card (connection, firmware, brightness, **Test the matrix**, **Set up again**)
  sits above two tabs:
  - **Live:** what the matrix shows now, the flag, and a player that works with no sim.
  - **Animations:** the flags you allow, playback settings, the pack's animations and an editor.
    **Save to My pack** keeps your changes.

## Test the matrix

The Test step of the setup shows six fixed screens, one at a time: red, green, blue, white, an
orientation screen and a brightness row. A MAX7219 matrix has one color, so it shows only the white
and orientation screens. Each screen says what you must see and shows the setting to change if you
do not. The board applies a change at once. **Looks right** goes on. **Finish setup** unlocks on the
last screen, and plays green then checkered. A screen left alone for 2 minutes stops by itself;
changing a setting counts as a click.

To test again later, click **Test the matrix** on the Device card. **Done** gives the matrix back
to the flag.

Without PitCrew: `pcplugin run plugins/flags --wasm … --port COM4 --press device:test@1
--press matrix-test:next@7 …`.

## Upload the firmware

On the **Device** tab, the **Firmware** card shows the firmware in the plugin and the firmware on
the board. Set **Board** to the board you have.

Not sure which board it is? Click **Detect board**: PitCrew asks the board and says which one to
pick. Then click **Upload PitFlag**. PitCrew restarts the board into its bootloader, writes the image
the plugin carries, checks it and restarts the board. This works for every board in the list, with
no button to hold and no Arduino IDE (needs PitCrew with plugin API 1.2).

The matrix settings go to the board at run time. You never compile the firmware for your setup.

`pcplugin run` writes no firmware: on the fake port an upload ends at once, and a real port is
refused. To write a board without the app, use `pitcrew-engine firmware upload --port COM5 --chip
esp32c3 --image firmware/dist/pitflag-esp32c3.bin`.

## Set up

1. Build the device. See [firmware/README.md](firmware/README.md).
2. In PitCrew, open **Plugins → Discover → Flags** and click **Download**.
3. Click **Turn on**. Check what it asks for: the flag, the session, the serial port, and its own
   folder.
4. Open **Flags** in the sidebar. The setup opens, because no device is set up yet.
5. **Board:** choose the port and the board type. **Matrix:** choose the matrix type, its pins and the current limit.
   **Upload:** PitCrew checks the settings and writes the firmware. If nothing on the port answers
   as PitFlag yet, confirm that the port is your matrix board first. **Test:** compare the matrix with
   each screen, then click **Finish setup**.
6. On the **Live** tab, click **Play every flag**. The matrix shows each flag in turn.
7. On the **Animations** tab, turn off the flags you do not want on the matrix.

To change the board or the port, click **Set up again** on the Device card.

## Settings

| Setting | Default | What it does |
| --- | --- | --- |
| Device port | none | The USB serial port of the board. |
| Brightness | 16% | Keep it low on USB power. |
| Animation pack | Classic | The content pack to use. |
| Show green for | 3 s | How long the green flash stays after the start or a restart. |
| Animate outside a race | on | With no sim, or out of the car, play the pack's `standby` animation. Off: the matrix stays dark. |
| Color order | GRB | The order the panel reads red, green and blue. Use the matrix test to find it. |
| Correct LED colors | on | Lowers green and blue a little, so the LEDs look like the screen (yellow is not lime). |
| Rotate | 0° | Turn the image without a new upload. |
| Zigzag wiring | on | On when every second row runs back. Also on the orientation screen of the matrix test. |
| Mirror | off | Mirror the image. Also on the orientation screen of the matrix test. |
| Board | Arduino Nano or Uno (ATmega328P) | The board the firmware upload is for. |

## Which animation for which flag

The first rule that matches wins ([content/classic/mapping.json](content/classic/mapping.json)):

| Flag | Animation |
| --- | --- |
| Your car: disqualified, black flag, repair (meatball) | `black-x`, `black`, `meatball` |
| The same flags for the session | the same |
| Red | `red` |
| Chequered | `checkered` |
| Start lights: ready, set | `lights-ready`, `lights-set` |
| Start lights: go | `green-flash` for "Show green for", then `idle` |
| Full-course yellow | `yellow-waving` |
| Local yellow | `yellow` |
| Debris | `debris` |
| White | `white` |
| Blue | `blue-flash` |
| Green | `green-flash` for "Show green for", then `idle` |
| No flag | `idle` (all off) |
| No sim connected, or not in the car | `standby` (or all off, with "Animate outside a race" off) |

## Make your own animations

Use the editor on the **Animations** tab, or write a pack. See
[docs/content-packs.md](../../docs/content-packs.md#the-flags-format-flagsanimations).

## Build from source

```sh
cargo test -p pitcrew-flags
cargo build --release --target wasm32-wasip2 -p pitcrew-flags
pcplugin run plugins/flags --wasm target/wasm32-wasip2/release/pitcrew_flags.wasm --ui
```

Add `--port COM4` to drive your board.
