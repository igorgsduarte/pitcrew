// PitFlag firmware settings. Change these to match your build, then upload.
// See docs/serial-devices.md and plugins/flags/firmware/README.md.

#pragma once

// ---- Which matrix ---------------------------------------------------------
// PITFLAG_WS2812  : 8x8 WS2812B / NeoPixel panel, full colour (needs the FastLED library).
// PITFLAG_MAX7219 : 8x8 single-colour module with a MAX7219 chip (no library needed).
#define PITFLAG_WS2812 1
#define PITFLAG_MAX7219 2
#define PITFLAG_MATRIX PITFLAG_WS2812

// ---- WS2812B --------------------------------------------------------------
#define WS2812_DATA_PIN 6         // Uno/Nano: 6. ESP32: 13 is a safe choice.
#define WS2812_COLOR_ORDER GRB
// Most panels are wired in a zigzag: row 0 left to right, row 1 right to left, ...
#define WS2812_SERPENTINE 1
// Caps the current the panel may draw from USB. 64 LEDs at full white draw ~3.8 A.
#define WS2812_MAX_MILLIAMPS 500

// ---- MAX7219 --------------------------------------------------------------
#define MAX7219_DIN_PIN 11
#define MAX7219_CLK_PIN 13
#define MAX7219_CS_PIN 10

// ---- Orientation (both matrices) ------------------------------------------
#define FLIP_X 0
#define FLIP_Y 0
// 0, 90, 180 or 270, clockwise, applied after the flips.
#define ROTATE 0

// ---- Link -----------------------------------------------------------------
#define PITFLAG_BAUD 115200
// Dark after this long without a packet, so a crashed PC never leaves a stale flag up.
#define HEARTBEAT_TIMEOUT_MS 1500

#define FIRMWARE_MAJOR 1
#define FIRMWARE_MINOR 0
#define FIRMWARE_PATCH 0
