# PitFlag firmware

The firmware for the Flags plugin's LED matrix. It speaks
[PitFlag wire v1](../../../docs/serial-devices.md#pitflag-wire-v1).

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

## Upload

1. Install the [Arduino IDE](https://www.arduino.cc/en/software) or `arduino-cli`.
2. For WS2812B, install the **FastLED** library (Library Manager).
3. Open `pitflag/pitflag.ino`.
4. Edit `pitflag/config.h`:
   - `PITFLAG_MATRIX`: `PITFLAG_WS2812` or `PITFLAG_MAX7219`.
   - The pins, if yours are different.
   - `WS2812_SERPENTINE`, `FLIP_X`, `FLIP_Y` and `ROTATE`, if the image is mirrored or turned.
5. Upload to the board.

## Check it

1. In PitCrew, open **Flags → Device** and choose the board's port.
2. The badge must say **Connected · firmware 1.0.0 · 8×8 RGB** (or **mono**).
3. On **Live**, click **Play every flag**.

If the image is mirrored or turned, change the orientation in `config.h` and upload again.

Without PitCrew: `pcplugin run plugins/flags --wasm … --port COM4 --flag yellow+waving`.

## Status

This firmware has not been compiled or run on hardware yet. Report problems with the board, the
matrix and the `config.h` you used.
