// PitFlag firmware: draws the frames the PitCrew Flags plugin sends over USB serial.
//
// PitFlag wire v1. Every packet is
//   0xA5 | type | len | payload[len] | crc8
// where crc8 (poly 0x07, init 0) covers type, len and the payload.
// The reference implementation and its tests are crates/pitflag-wire in this repository.

#include "config.h"

#if PITFLAG_MATRIX == PITFLAG_WS2812
#include <FastLED.h>
#endif

enum : uint8_t {
  T_HELLO = 0x01,
  T_PALETTE = 0x02,
  T_FRAME = 0x03,
  T_BRIGHTNESS = 0x04,
  T_CLEAR = 0x05,
  T_PING = 0x06,
  T_PONG = 0x07,
};

const uint8_t START = 0xA5;
const uint8_t WIRE_VERSION = 1;
const uint8_t MAX_PAYLOAD = 64;
const uint8_t SIDE = 8;

uint8_t palette[16][3];
uint8_t paletteSize = 1;
uint8_t frame[64];
uint8_t brightness = 40;
unsigned long lastPacketMs = 0;
bool lit = false;

// ---------------------------------------------------------------- matrix --

uint8_t ledIndex(uint8_t x, uint8_t y) {
  if (FLIP_X) x = SIDE - 1 - x;
  if (FLIP_Y) y = SIDE - 1 - y;
  uint8_t rx = x, ry = y;
#if ROTATE == 90
  rx = SIDE - 1 - y; ry = x;
#elif ROTATE == 180
  rx = SIDE - 1 - x; ry = SIDE - 1 - y;
#elif ROTATE == 270
  rx = y; ry = SIDE - 1 - x;
#endif
#if PITFLAG_MATRIX == PITFLAG_WS2812 && WS2812_SERPENTINE
  if (ry & 1) rx = SIDE - 1 - rx;
#endif
  return ry * SIDE + rx;
}

#if PITFLAG_MATRIX == PITFLAG_WS2812

CRGB leds[64];

void matrixBegin() {
  FastLED.addLeds<WS2812B, WS2812_DATA_PIN, WS2812_COLOR_ORDER>(leds, 64);
  FastLED.setMaxPowerInVoltsAndMilliamps(5, WS2812_MAX_MILLIAMPS);
  FastLED.setBrightness(brightness);
  FastLED.clear(true);
}

void matrixShow(bool on) {
  for (uint8_t y = 0; y < SIDE; y++) {
    for (uint8_t x = 0; x < SIDE; x++) {
      uint8_t i = frame[y * SIDE + x];
      CRGB c = CRGB::Black;
      if (on && i != 0 && i < paletteSize) c = CRGB(palette[i][0], palette[i][1], palette[i][2]);
      leds[ledIndex(x, y)] = c;
    }
  }
  FastLED.setBrightness(brightness);
  FastLED.show();
}

#else  // MAX7219

void maxSend(uint8_t reg, uint8_t value) {
  digitalWrite(MAX7219_CS_PIN, LOW);
  shiftOut(MAX7219_DIN_PIN, MAX7219_CLK_PIN, MSBFIRST, reg);
  shiftOut(MAX7219_DIN_PIN, MAX7219_CLK_PIN, MSBFIRST, value);
  digitalWrite(MAX7219_CS_PIN, HIGH);
}

void matrixBegin() {
  pinMode(MAX7219_DIN_PIN, OUTPUT);
  pinMode(MAX7219_CLK_PIN, OUTPUT);
  pinMode(MAX7219_CS_PIN, OUTPUT);
  digitalWrite(MAX7219_CS_PIN, HIGH);
  maxSend(0x0F, 0x00);  // display test off
  maxSend(0x09, 0x00);  // no BCD decode
  maxSend(0x0B, 0x07);  // scan all 8 rows
  maxSend(0x0C, 0x01);  // normal operation
  for (uint8_t r = 1; r <= 8; r++) maxSend(r, 0);
}

// One colour only: any pixel whose palette entry is not black is on.
void matrixShow(bool on) {
  uint8_t rows[8] = {0};
  for (uint8_t y = 0; y < SIDE; y++) {
    for (uint8_t x = 0; x < SIDE; x++) {
      uint8_t i = frame[y * SIDE + x];
      bool px = on && i != 0 && i < paletteSize && (palette[i][0] | palette[i][1] | palette[i][2]);
      if (px) {
        uint8_t n = ledIndex(x, y);
        rows[n / SIDE] |= 0x80 >> (n % SIDE);
      }
    }
  }
  maxSend(0x0A, brightness >> 4);
  for (uint8_t r = 0; r < 8; r++) maxSend(r + 1, rows[r]);
}

#endif

// ------------------------------------------------------------------ wire --

uint8_t crc8(uint8_t crc, uint8_t b) {
  crc ^= b;
  for (uint8_t i = 0; i < 8; i++) crc = (crc & 0x80) ? (uint8_t)((crc << 1) ^ 0x07) : (uint8_t)(crc << 1);
  return crc;
}

void sendPacket(uint8_t type, const uint8_t *payload, uint8_t len) {
  uint8_t crc = crc8(crc8(0, type), len);
  Serial.write(START);
  Serial.write(type);
  Serial.write(len);
  for (uint8_t i = 0; i < len; i++) {
    Serial.write(payload[i]);
    crc = crc8(crc, payload[i]);
  }
  Serial.write(crc);
}

void sendHello() {
  const uint8_t hello[8] = {
    WIRE_VERSION, FIRMWARE_MAJOR, FIRMWARE_MINOR, FIRMWARE_PATCH, SIDE, SIDE,
    PITFLAG_MATRIX == PITFLAG_WS2812 ? 1 : 0,  // rgb
    30,                                       // max fps
  };
  sendPacket(T_HELLO, hello, sizeof hello);
}

void handle(uint8_t type, const uint8_t *p, uint8_t len) {
  lastPacketMs = millis();
  switch (type) {
    case T_HELLO:
      if (len == 0) sendHello();
      break;
    case T_PALETTE:
      paletteSize = len / 3 > 16 ? 16 : len / 3;
      for (uint8_t i = 0; i < paletteSize; i++) {
        palette[i][0] = p[i * 3];
        palette[i][1] = p[i * 3 + 1];
        palette[i][2] = p[i * 3 + 2];
      }
      break;
    case T_FRAME:
      if (len != 32) break;
      for (uint8_t i = 0; i < 32; i++) {
        frame[i * 2] = p[i] >> 4;
        frame[i * 2 + 1] = p[i] & 0x0F;
      }
      matrixShow(true);
      lit = true;
      break;
    case T_BRIGHTNESS:
      if (len >= 1) brightness = p[0];
      if (lit) matrixShow(true);
      break;
    case T_CLEAR:
      memset(frame, 0, sizeof frame);
      matrixShow(false);
      lit = false;
      break;
    case T_PING:
      sendPacket(T_PONG, p, len);
      break;
  }
}

// A small state machine, so a packet split across reads or noise on the line never confuses it.
enum State : uint8_t { WAIT_START, READ_TYPE, READ_LEN, READ_PAYLOAD, READ_CRC };
State state = WAIT_START;
uint8_t rxType, rxLen, rxPos, rxCrc;
uint8_t rxBuf[MAX_PAYLOAD];

void feed(uint8_t b) {
  switch (state) {
    case WAIT_START:
      if (b == START) state = READ_TYPE;
      break;
    case READ_TYPE:
      rxType = b;
      rxCrc = crc8(0, b);
      state = READ_LEN;
      break;
    case READ_LEN:
      if (b > MAX_PAYLOAD) {
        state = b == START ? READ_TYPE : WAIT_START;
        break;
      }
      rxLen = b;
      rxPos = 0;
      rxCrc = crc8(rxCrc, b);
      state = b == 0 ? READ_CRC : READ_PAYLOAD;
      break;
    case READ_PAYLOAD:
      rxBuf[rxPos++] = b;
      rxCrc = crc8(rxCrc, b);
      if (rxPos == rxLen) state = READ_CRC;
      break;
    case READ_CRC:
      if (b == rxCrc) handle(rxType, rxBuf, rxLen);
      state = WAIT_START;
      break;
  }
}

// ----------------------------------------------------------------- main --

void setup() {
  Serial.begin(PITFLAG_BAUD);
  palette[0][0] = palette[0][1] = palette[0][2] = 0;
  memset(frame, 0, sizeof frame);
  matrixBegin();
  sendHello();
}

void loop() {
  while (Serial.available() > 0) feed((uint8_t)Serial.read());
  if (lit && millis() - lastPacketMs > HEARTBEAT_TIMEOUT_MS) {
    matrixShow(false);
    lit = false;
  }
}
