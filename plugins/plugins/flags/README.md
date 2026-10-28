# Flags

Shows the race flags on an 8×8 LED matrix, with animations. This is the official PitCrew plugin for
flag displays.

- **Sims:** every sim PitCrew reads (iRacing, Assetto Corsa, Assetto Corsa EVO). PitCrew decides
  the flag; the plugin only shows it.
- **Hardware:** an Arduino Uno or Nano, or an ESP32, with an 8×8 WS2812B panel (colour) or a MAX7219
  module (one colour), running the PitFlag firmware in [`firmware/`](firmware/).
- **Screen:** a **Flags** item in the sidebar, with three tabs:
  - **Live:** what the matrix shows now, the flag, and test buttons that work with no sim.
  - **Animations:** the pack's animations and an editor. **Save to My pack** keeps your changes.
  - **Device:** the connection, the firmware, the serial link use, the settings and the matrix test.

## Test the matrix

On the **Device** tab, click **Start the test**. The matrix shows six fixed screens, one at a time:
red, green, blue, white, an orientation screen and a brightness row. Each screen says what you must
see and which setting to change if you do not. Change the **Matrix** settings while a screen shows:
the board applies them at once. **Next** goes on, **Stop** gives the matrix back to the flag. A
screen left alone for 2 minutes stops by itself.

Without PitCrew: `pcplugin run plugins/flags --wasm … --port COM4 --press matrix-test:start@1
--press matrix-test:next@7 …`.

## Set up

1. Build the device. See [firmware/README.md](firmware/README.md).
2. In PitCrew, open **Plugins → Discover → Flags** and click **Download**.
3. Click **Turn on**. Check what it asks for: the flag, the session, the serial port, and its own
   folder.
4. Open **Flags** in the sidebar. On the **Device** tab, choose the port of your board.
5. On the **Live** tab, click **Play every flag**. The matrix shows each flag in turn.

## Settings

| Setting | Default | What it does |
| --- | --- | --- |
| Device port | none | The USB serial port of the board. |
| Brightness | 40 | 1 to 255. Keep it low on USB power. |
| Animations | Classic | The content pack to use. |
| Show green for (s) | 3 | How long the green flash stays after the start or a restart. |
| Animate outside a race | on | With no sim, or out of the car, play the pack's `standby` animation. Off: the matrix stays dark. |
| Color order | GRB | The order the panel reads red, green and blue. Use the matrix test to find it. |
| Correct LED colors | on | Lowers green and blue a little, so the LEDs look like the screen (yellow is not lime). |
| Zigzag wiring | on | On when every second row runs back. The orientation screen of the matrix test shows it. |
| Mirror, Rotate | off, 0° | Turn the image without a new upload. |

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
