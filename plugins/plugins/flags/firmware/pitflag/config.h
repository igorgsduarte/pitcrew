// PitFlag firmware build constants.
//
// The matrix settings (type, pins, colour order, orientation, current cap) are not here. The Flags
// plugin sends them in a CONFIG packet and the firmware keeps them in EEPROM. The values below are
// the factory defaults, used until the first CONFIG arrives. See plugins/flags/firmware/README.md.

#pragma once

#define FIRMWARE_MAJOR 2
#define FIRMWARE_MINOR 2
#define FIRMWARE_PATCH 0

// ---- Link -----------------------------------------------------------------
#define PITFLAG_BAUD 115200
// Dark after this long without a packet, so a crashed PC never leaves a stale flag up.
#define HEARTBEAT_TIMEOUT_MS 1500

// ---- Boot animation -------------------------------------------------------
// From power-on until the first packet, a dot runs around the border. It stops after this long
// even if PitCrew never talks.
#define BOOT_ANIMATION_MS 3000
#define BOOT_FRAME_MS 50

// ---- Chip -----------------------------------------------------------------
// The number goes into HELLO. It must match pitflag_wire::Chip.
#if defined(CONFIG_IDF_TARGET_ESP32C3)
#define PITFLAG_CHIP 5
#if !ARDUINO_USB_CDC_ON_BOOT
#error "Build the ESP32-C3 image with USB CDC On Boot enabled: the link is the chip's own USB port."
#endif
#elif defined(CONFIG_IDF_TARGET_ESP32)
#define PITFLAG_CHIP 4
#elif defined(__AVR_ATmega32U4__)
#define PITFLAG_CHIP 3
#elif defined(__AVR_ATmega168__) || defined(__AVR_ATmega168P__)
#define PITFLAG_CHIP 2
#elif defined(__AVR_ATmega328P__)
#define PITFLAG_CHIP 1
#else
#error "PitFlag supports the ATmega328P, ATmega168, ATmega32U4, ESP32 and ESP32-C3."
#endif

// ---- Factory defaults -----------------------------------------------------
// A pin of 0xFF in CONFIG means "the default below".
#if defined(CONFIG_IDF_TARGET_ESP32C3)
#define DEFAULT_WS2812_PIN 6
#define DEFAULT_MAX7219_DIN_PIN 7
#define DEFAULT_MAX7219_CLK_PIN 6
#define DEFAULT_MAX7219_CS_PIN 10
#elif defined(ARDUINO_ARCH_ESP32)
#define DEFAULT_WS2812_PIN 13
#define DEFAULT_MAX7219_DIN_PIN 23
#define DEFAULT_MAX7219_CLK_PIN 18
#define DEFAULT_MAX7219_CS_PIN 5
#else
#define DEFAULT_WS2812_PIN 6
#define DEFAULT_MAX7219_DIN_PIN 11
#define DEFAULT_MAX7219_CLK_PIN 13
#define DEFAULT_MAX7219_CS_PIN 10
#endif
#define DEFAULT_MAX_MILLIAMPS 500
#define DEFAULT_BRIGHTNESS 40
