# PitFlag firmware

The firmware for the Flags plugin's LED matrix. It speaks
[PitFlag wire v2](../../../docs/serial-devices.md#pitflag-wire-v2). Its `HELLO` starts with the
magic `PFLG`, so the plugin knows the board runs this firmware.

## Parts

| Part | Notes |
| --- | --- |
| A board | Arduino Uno, Nano, or an ESP32 dev board. |
| A matrix | An 8×8 WS2812B panel (colour), or an 8×8 MAX7219 module (one colour). |
| For WS2812B | A 330 Ω resistor in the data line, and a 1000 µF capacitor across 5 V and GND. |
| A USB cable | With data lines, not a charge-only cable. |

## Wiring

**WS2812B:** data in → pin 6 (Uno/Nano) or GPIO 13 (ESP32), through the resistor. 5 V and GND to the
board. At brightness 40 the panel draws less than 500 mA. The firmware caps the current at
`WS2812_MAX_MILLIAMPS`.

**MAX7219:** DIN → 11, CLK → 13, CS → 10, VCC → 5 V, GND → GND.

## Boards and images

One image per chip. Each image has both matrix drivers. The matrix type, the pins and the
orientation are settings in PitCrew: the plugin sends them to the board, and the board keeps them in
EEPROM. You never edit the sketch to change them.

| Image | Boards | Default pins | Status |
| --- | --- | --- | --- |
| `pitflag-atmega328p.hex` | Uno, Nano (new and old bootloader) | WS2812B 6 · MAX7219 DIN 11, CLK 13, CS 10 | Tested |
| `pitflag-atmega168.hex` | Nano ATmega168 | same | Experimental |
| `pitflag-atmega32u4.hex` | Pro Micro, Leonardo | same | Experimental |
| `pitflag-esp32.bin` | ESP32 dev board (classic ESP32) | WS2812B 13 · MAX7219 DIN 23, CLK 18, CS 5 | Experimental |

"Tested" means it ran on real hardware. Report an "Experimental" board that works, with the board and
the matrix you used.

## Build the images

On Windows, with PowerShell:

```powershell
.\build.ps1 -Setup   # once: arduino-cli, the pinned cores and Adafruit NeoPixel
.\build.ps1          # the four images into dist\, and dist\size-report.txt
```

`build.ps1` pins arduino-cli, the AVR and ESP32 cores and the library. It fails if an image uses more
than 90% of the flash or 75% of the RAM (the NeoPixel buffer included). The ATmega168 is the tight
one: 16 KB of flash and 1 KB of RAM.

The ESP32 image is a merged binary for address `0x0`, padded to 4 MB. It compresses to a small file.

## Upload

PitCrew uploads the image for you (planned, GOT-672). Until then, use the Arduino IDE or
`arduino-cli upload` with the image for your chip, or open `pitflag/pitflag.ino` and install the
**Adafruit NeoPixel** library.

## Check it

1. Power the board. A blue dot runs around the border: the board is booting. It stops at the first
   packet from PitCrew, or after 3 s.
2. In PitCrew, open **Flags → Device** and choose the board's port.
3. The badge must say **Connected · firmware 2.1.0 · ATmega328P · 8×8 RGB** (with your chip, or
   **mono**).
4. On **Device**, click **Start the test** and follow the screens. See
   [the plugin README](../README.md#test-the-matrix).

If the image is mirrored or turned, change **Zigzag wiring**, **Mirror** or **Rotate** in the
**Matrix** settings. The board takes the change at once. No new upload is needed.

Without PitCrew: `pcplugin run plugins/flags --wasm … --port COM4 --flag yellow+waving`.

## How it draws

The plugin sends every frame. The firmware only draws what it gets, and the boot animation is the one
thing it draws by itself. A WS2812B redraw turns the interrupts off for about 2 ms, and serial bytes
that arrive then are lost. So a packet only marks the matrix dirty, and the firmware redraws once the
line is quiet for 2 ms. The plugin sends the whole state again every second, so a lost packet heals.

## Status

Tested on a Nano clone (ATmega328P, Optiboot, FTDI) with a BTF-Lighting WS2812B-64 panel on pin 4.
That panel is wired row by row: turn **Zigzag wiring** off. Report problems with the board, the matrix
and the settings you used.
