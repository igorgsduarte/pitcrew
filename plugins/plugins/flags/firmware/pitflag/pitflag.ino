// PitFlag firmware: draws the frames the PitCrew Flags plugin sends over USB serial.
//
// PitFlag wire v2. Every packet is
//   0xA5 | type | len | payload[len] | crc8
// where crc8 (poly 0x07, init 0) covers type, len and the payload.
// The reference implementation and its tests are crates/pitflag-wire in this repository.

#include <Adafruit_NeoPixel.h>
#include <EEPROM.h>

#include "config.h"

#if defined(CONFIG_IDF_TARGET_ESP32C3)
#include "soc/rtc_cntl_reg.h"
#endif

enum : uint8_t {
  T_HELLO = 0x01,
  T_PALETTE = 0x02,
  T_FRAME = 0x03,
  T_BRIGHTNESS = 0x04,
  T_CLEAR = 0x05,
  T_PING = 0x06,
  T_PONG = 0x07,
  T_CONFIG = 0x08,
  T_CONFIG_ACK = 0x09,
  T_BOOTLOADER = 0x0A,
};

// BOOTLOADER payload. Anything else is ignored, so line noise never restarts the board.
const uint8_t BOOTLOADER_MAGIC[4] = {'B', 'O', 'O', 'T'};

enum : uint8_t { ACK_OK = 0, ACK_BAD_LAYOUT = 1, ACK_BAD_MATRIX = 2, ACK_BAD_PIN = 3, ACK_BAD_VALUE = 4 };
enum : uint8_t { M_WS2812 = 1, M_MAX7219 = 2 };

const uint8_t START = 0xA5;
const uint8_t WIRE_VERSION = 2;
const uint8_t MAX_PAYLOAD = 64;
const uint8_t SIDE = 8;
const uint8_t MAX_FPS = 30;

// CONFIG payload, byte by byte. It must match pitflag_wire::Config::payload.
const uint8_t CONFIG_LEN = 10;
const uint8_t CONFIG_LAYOUT = 1;
enum : uint8_t { C_LAYOUT, C_MATRIX, C_DATA, C_CLOCK, C_CS, C_ORDER, C_FLAGS, C_ROTATE, C_MA_HI, C_MA_LO };
const uint8_t F_SERPENTINE = 1, F_FLIP_X = 2, F_FLIP_Y = 4;
const uint8_t PIN_DEFAULT = 0xFF;

// EEPROM: 'P' 'F', the CONFIG payload, its crc8.
const uint8_t EE_MAGIC_0 = 0, EE_MAGIC_1 = 1, EE_CONFIG = 2, EE_CRC = EE_CONFIG + CONFIG_LEN, EE_SIZE = EE_CRC + 1;

uint8_t cfg[CONFIG_LEN] = {
  CONFIG_LAYOUT, M_WS2812, PIN_DEFAULT, PIN_DEFAULT, PIN_DEFAULT, 0, F_SERPENTINE, 0,
  DEFAULT_MAX_MILLIAMPS >> 8, DEFAULT_MAX_MILLIAMPS & 0xFF,
};

uint8_t palette[16][3];
uint8_t paletteSize = 1;
uint8_t frame[64];
uint8_t brightness = DEFAULT_BRIGHTNESS;
unsigned long lastPacketMs = 0;
bool lit = false;

// A WS2812B redraw turns interrupts off for about 2 ms, and serial bytes that arrive then are lost.
// So a packet only marks the matrix dirty, and loop() redraws once the line is quiet.
const unsigned long QUIET_US = 2000;
bool dirty = false;
unsigned long lastByteUs = 0;

// The boot animation runs from power-on until the first packet, for BOOT_ANIMATION_MS at most.
bool booting = true;
unsigned long bootFrameMs = 0;
uint8_t bootPos = 0;

// ------------------------------------------------------------------ crc --

uint8_t crc8(uint8_t crc, uint8_t b) {
  crc ^= b;
  for (uint8_t i = 0; i < 8; i++) crc = (crc & 0x80) ? (uint8_t)((crc << 1) ^ 0x07) : (uint8_t)(crc << 1);
  return crc;
}

uint8_t crcOf(const uint8_t *p, uint8_t len) {
  uint8_t crc = 0;
  for (uint8_t i = 0; i < len; i++) crc = crc8(crc, p[i]);
  return crc;
}

// ----------------------------------------------------------------- pins --

bool pinOk(uint8_t p) {
#if defined(CONFIG_IDF_TARGET_ESP32C3)
  if (p > 21) return false;                               // no such GPIO
  if (p >= 11 && p <= 17) return false;                   // the SPI flash
  if (p == 18 || p == 19) return false;                   // USB D- and D+, the link
  return true;
#elif defined(ARDUINO_ARCH_ESP32)
  if (p > 33) return false;                               // 34-39 are input only
  if (p == 1 || p == 3) return false;                     // UART0, the USB link
  if (p >= 6 && p <= 11) return false;                    // the SPI flash
  if (p == 20 || p == 24 || (p >= 28 && p <= 31)) return false;  // no such GPIO
  return true;
#else
  return p >= 2 && p < NUM_DIGITAL_PINS;                  // 0 and 1 are the serial link
#endif
}

uint8_t dataPin() {
  uint8_t p = cfg[C_DATA];
  if (p != PIN_DEFAULT) return p;
  return cfg[C_MATRIX] == M_WS2812 ? DEFAULT_WS2812_PIN : DEFAULT_MAX7219_DIN_PIN;
}

uint8_t clockPin() {
  return cfg[C_CLOCK] == PIN_DEFAULT ? DEFAULT_MAX7219_CLK_PIN : cfg[C_CLOCK];
}

uint8_t csPin() {
  return cfg[C_CS] == PIN_DEFAULT ? DEFAULT_MAX7219_CS_PIN : cfg[C_CS];
}

// --------------------------------------------------------------- config --

// An ACK_* code for a CONFIG payload. Reads the pins through a copy, so cfg is not touched.
uint8_t checkConfig(const uint8_t *p, uint8_t len) {
  if (len != CONFIG_LEN || p[C_LAYOUT] != CONFIG_LAYOUT) return ACK_BAD_LAYOUT;
  if (p[C_MATRIX] != M_WS2812 && p[C_MATRIX] != M_MAX7219) return ACK_BAD_MATRIX;
  if (p[C_ORDER] > 2 || p[C_FLAGS] > 7 || p[C_ROTATE] > 3) return ACK_BAD_VALUE;
  if (p[C_MA_HI] == 0 && p[C_MA_LO] == 0) return ACK_BAD_VALUE;
  uint8_t saved[CONFIG_LEN];
  memcpy(saved, cfg, CONFIG_LEN);
  memcpy(cfg, p, CONFIG_LEN);
  bool ok;
  if (cfg[C_MATRIX] == M_WS2812) {
    ok = pinOk(dataPin());
  } else {
    uint8_t d = dataPin(), c = clockPin(), s = csPin();
    ok = pinOk(d) && pinOk(c) && pinOk(s) && d != c && d != s && c != s;
  }
  memcpy(cfg, saved, CONFIG_LEN);
  return ok ? ACK_OK : ACK_BAD_PIN;
}

#if defined(ARDUINO_ARCH_ESP32)
#define EE_PUT(addr, v) EEPROM.write(addr, v)
#else
#define EE_PUT(addr, v) EEPROM.update(addr, v)
#endif

void loadConfig() {
#if defined(ARDUINO_ARCH_ESP32)
  EEPROM.begin(EE_SIZE);
#endif
  if (EEPROM.read(EE_MAGIC_0) != 'P' || EEPROM.read(EE_MAGIC_1) != 'F') return;
  uint8_t p[CONFIG_LEN];
  for (uint8_t i = 0; i < CONFIG_LEN; i++) p[i] = EEPROM.read(EE_CONFIG + i);
  if (EEPROM.read(EE_CRC) != crcOf(p, CONFIG_LEN)) return;
  if (checkConfig(p, CONFIG_LEN) != ACK_OK) return;
  memcpy(cfg, p, CONFIG_LEN);
}

void saveConfig() {
  EE_PUT(EE_MAGIC_0, 'P');
  EE_PUT(EE_MAGIC_1, 'F');
  for (uint8_t i = 0; i < CONFIG_LEN; i++) EE_PUT(EE_CONFIG + i, cfg[i]);
  EE_PUT(EE_CRC, crcOf(cfg, CONFIG_LEN));
#if defined(ARDUINO_ARCH_ESP32)
  EEPROM.commit();
#endif
}

// ---------------------------------------------------------------- matrix --

uint8_t ledIndex(uint8_t x, uint8_t y) {
  uint8_t flags = cfg[C_FLAGS];
  if (flags & F_FLIP_X) x = SIDE - 1 - x;
  if (flags & F_FLIP_Y) y = SIDE - 1 - y;
  uint8_t rx = x, ry = y;
  switch (cfg[C_ROTATE]) {
    case 1: rx = SIDE - 1 - y; ry = x; break;
    case 2: rx = SIDE - 1 - x; ry = SIDE - 1 - y; break;
    case 3: rx = y; ry = SIDE - 1 - x; break;
  }
  if (cfg[C_MATRIX] == M_WS2812 && (flags & F_SERPENTINE) && (ry & 1)) rx = SIDE - 1 - rx;
  return ry * SIDE + rx;
}

// The palette entry of a pixel, or 0 (off).
uint8_t colourAt(uint8_t i, bool on) {
  uint8_t c = frame[i];
  return (on && c != 0 && c < paletteSize) ? c : 0;
}

Adafruit_NeoPixel strip(64, DEFAULT_WS2812_PIN, NEO_GRB + NEO_KHZ800);

const uint16_t NEO_ORDER[3] = {NEO_GRB, NEO_RGB, NEO_BRG};

// About 20 mA per colour channel at full scale, and 1 mA per idle LED. Scales the brightness down
// so the estimate stays under the cap.
uint8_t cappedBrightness(bool on) {
  uint32_t sum = 0;
  for (uint8_t i = 0; i < 64; i++) {
    uint8_t c = colourAt(i, on);
    sum += (uint16_t)palette[c][0] + palette[c][1] + palette[c][2];
  }
  uint32_t idle = 64;
  uint32_t cap = ((uint16_t)cfg[C_MA_HI] << 8) | cfg[C_MA_LO];
  uint32_t lit_ma = sum * 20UL * brightness / (255UL * 255UL);
  if (idle + lit_ma <= cap) return brightness;
  if (cap <= idle) return 1;
  uint32_t b = (uint32_t)brightness * (cap - idle) / lit_ma;
  return b < 1 ? 1 : (uint8_t)b;
}

void maxSend(uint8_t reg, uint8_t value) {
  uint8_t d = dataPin(), c = clockPin(), s = csPin();
  digitalWrite(s, LOW);
  shiftOut(d, c, MSBFIRST, reg);
  shiftOut(d, c, MSBFIRST, value);
  digitalWrite(s, HIGH);
}

void matrixBegin() {
  if (cfg[C_MATRIX] == M_WS2812) {
    strip.updateType(NEO_ORDER[cfg[C_ORDER]] + NEO_KHZ800);
    strip.setPin(dataPin());
    strip.begin();
    strip.clear();
    strip.show();
  } else {
    pinMode(dataPin(), OUTPUT);
    pinMode(clockPin(), OUTPUT);
    pinMode(csPin(), OUTPUT);
    digitalWrite(csPin(), HIGH);
    maxSend(0x0F, 0x00);  // display test off
    maxSend(0x09, 0x00);  // no BCD decode
    maxSend(0x0B, 0x07);  // scan all 8 rows
    maxSend(0x0C, 0x01);  // normal operation
    for (uint8_t r = 1; r <= 8; r++) maxSend(r, 0);
  }
}

// Turns the current matrix off and frees its pins, before a CONFIG changes them.
void matrixEnd() {
  if (cfg[C_MATRIX] == M_WS2812) {
    strip.clear();
    strip.show();
    pinMode(dataPin(), INPUT);
  } else {
    for (uint8_t r = 1; r <= 8; r++) maxSend(r, 0);
    maxSend(0x0C, 0x00);  // shutdown
    pinMode(dataPin(), INPUT);
    pinMode(clockPin(), INPUT);
    pinMode(csPin(), INPUT);
  }
}

void matrixShow(bool on) {
  if (cfg[C_MATRIX] == M_WS2812) {
    // setBrightness before setPixelColor: the library scales the stored pixels when it changes.
    strip.setBrightness(cappedBrightness(on));
    for (uint8_t y = 0; y < SIDE; y++) {
      for (uint8_t x = 0; x < SIDE; x++) {
        uint8_t c = colourAt(y * SIDE + x, on);
        strip.setPixelColor(ledIndex(x, y), palette[c][0], palette[c][1], palette[c][2]);
      }
    }
    strip.show();
  } else {
    // One colour only: any pixel whose palette entry is not black is on.
    uint8_t rows[8] = {0};
    for (uint8_t y = 0; y < SIDE; y++) {
      for (uint8_t x = 0; x < SIDE; x++) {
        uint8_t c = colourAt(y * SIDE + x, on);
        if (palette[c][0] | palette[c][1] | palette[c][2]) {
          uint8_t n = ledIndex(x, y);
          rows[n / SIDE] |= 0x80 >> (n % SIDE);
        }
      }
    }
    maxSend(0x0A, brightness >> 4);
    for (uint8_t r = 0; r < 8; r++) maxSend(r + 1, rows[r]);
  }
}

// ------------------------------------------------------------------ wire --

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
  const uint8_t hello[14] = {
    'P', 'F', 'L', 'G',
    WIRE_VERSION, FIRMWARE_MAJOR, FIRMWARE_MINOR, FIRMWARE_PATCH,
    PITFLAG_CHIP, SIDE, SIDE,
    (uint8_t)(cfg[C_MATRIX] == M_WS2812 ? 1 : 0),  // rgb
    MAX_FPS,
    crcOf(cfg, CONFIG_LEN),
  };
  sendPacket(T_HELLO, hello, sizeof hello);
}

void applyConfig(const uint8_t *p, uint8_t len) {
  uint8_t code = checkConfig(p, len);
  if (code == ACK_OK && memcmp(p, cfg, CONFIG_LEN) != 0) {
    matrixEnd();
    memcpy(cfg, p, CONFIG_LEN);
    saveConfig();
    matrixBegin();
    dirty = true;
  }
  const uint8_t ack[2] = {code, crcOf(cfg, CONFIG_LEN)};
  sendPacket(T_CONFIG_ACK, ack, sizeof ack);
}

// Restarts into the ROM download mode, so the plugin can write new firmware over the same port.
// Only the ESP32-C3 can: an Arduino is reset into its bootloader when the port opens.
void enterBootloader(const uint8_t *p, uint8_t len) {
  if (len != sizeof BOOTLOADER_MAGIC || memcmp(p, BOOTLOADER_MAGIC, len) != 0) return;
#if defined(CONFIG_IDF_TARGET_ESP32C3)
  matrixEnd();
  Serial.flush();
  REG_WRITE(RTC_CNTL_OPTION1_REG, RTC_CNTL_FORCE_DOWNLOAD_BOOT);
  esp_restart();
#endif
}

// ----------------------------------------------------------------- boot --

// Border position n (0-27, clockwise from the top-left corner) as a pixel index.
uint8_t borderPixel(uint8_t n) {
  if (n < 8) return n;                 // top row, left to right
  if (n < 15) return (n - 7) * 8 + 7;  // right column, down
  if (n < 22) return 63 - (n - 14);    // bottom row, right to left
  return (28 - n) * 8;                 // left column, up
}

// A dot with a short tail runs around the border.
void bootStep(unsigned long now) {
  if (now - bootFrameMs < BOOT_FRAME_MS) return;
  bootFrameMs = now;
  const uint8_t shades[4][3] = {{0, 0, 0}, {30, 107, 255}, {8, 40, 110}, {2, 10, 30}};
  memcpy(palette, shades, sizeof shades);
  paletteSize = 4;
  memset(frame, 0, sizeof frame);
  for (uint8_t t = 0; t < 3; t++) frame[borderPixel((bootPos + 28 - t) % 28)] = t + 1;
  bootPos = (bootPos + 1) % 28;
  dirty = true;
}

void endBoot() {
  booting = false;
  memset(frame, 0, sizeof frame);
  memset(palette, 0, sizeof palette);
  paletteSize = 1;
  dirty = true;
}

void handle(uint8_t type, const uint8_t *p, uint8_t len) {
  lastPacketMs = millis();
  if (booting) endBoot();
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
      palette[0][0] = palette[0][1] = palette[0][2] = 0;  // index 0 is off, whatever the PC sent
      break;
    case T_FRAME:
      if (len != 32) break;
      for (uint8_t i = 0; i < 32; i++) {
        frame[i * 2] = p[i] >> 4;
        frame[i * 2 + 1] = p[i] & 0x0F;
      }
      lit = true;
      dirty = true;
      break;
    case T_BRIGHTNESS:
      if (len >= 1) brightness = p[0];
      dirty = true;
      break;
    case T_CLEAR:
      memset(frame, 0, sizeof frame);
      lit = false;
      dirty = true;
      break;
    case T_PING:
      sendPacket(T_PONG, p, len);
      break;
    case T_CONFIG:
      applyConfig(p, len);
      break;
    case T_BOOTLOADER:
      enterBootloader(p, len);
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
#if defined(CONFIG_IDF_TARGET_ESP32C3)
  // With no PC on the USB port, a write would wait for one. Drop the bytes instead.
  Serial.setTxTimeoutMs(0);
#endif
  memset(palette, 0, sizeof palette);
  memset(frame, 0, sizeof frame);
  loadConfig();
  matrixBegin();
  sendHello();
}

void loop() {
  while (Serial.available() > 0) {
    feed((uint8_t)Serial.read());
    lastByteUs = micros();
  }
  unsigned long now = millis();
  if (booting) {
    if (now >= BOOT_ANIMATION_MS) endBoot();
    else bootStep(now);
  }
  if (lit && now - lastPacketMs > HEARTBEAT_TIMEOUT_MS) {
    lit = false;
    dirty = true;
  }
  if (dirty && state == WAIT_START && micros() - lastByteUs >= QUIET_US) {
    dirty = false;
    matrixShow(lit || booting);
  }
}
