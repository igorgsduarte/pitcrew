# Serial devices

A plugin can drive an Arduino, an ESP32 or another USB serial device. PitCrew owns the port. The
plugin gets it only through a `serialPort` setting that the driver fills.

## Open the port

```json
"permissions": ["devices:serial"],
"settings": [{ "key": "port", "type": "serialPort", "label": "Device port" }]
```

```rust
match serial::open("port", 115_200) {
    Ok(port) => { /* keep it */ }
    Err(e) => host::set_status(StatusLevel::Warning, &format!("{e}. Retrying every 2 s.")),
}
```

## Rules for a good device link

1. **Never block.** `read` returns what is waiting now. Read a little on each `tick`.
2. **Expect the device to go away.** A cable comes out in the middle of a race. When `read` or
   `write` returns `err`, drop the port, set a warning status and try `open` again every 2 s.
3. **Wait for the device.** An Arduino Uno or Nano resets when its DTR line changes, for example
   when another app opened the port. Its bootloader takes about 1.6 s. Ask the device who it is, and
   send nothing else until it answers.
4. **Send what changed, and refresh the rest.** A frame that did not change is not sent on each
   tick. But send the whole state again about once a second: a lost packet must not leave the device
   wrong until the next change.
5. **Send a heartbeat.** Make the device go dark when the PC stops talking. Then a crash never leaves
   a stale flag on the matrix.
6. **Turn the device off in `on-stop`.**
7. **Use a checksum.** Serial lines drop and corrupt bytes.
8. **Write one burst per tick, and let the device work between bursts.** A WS2812B redraw turns the
   interrupts off for about 2 ms on an AVR, and the bytes that arrive then are lost. The PitFlag
   firmware redraws only after the line is quiet for 2 ms.

## Write firmware to the device

Do not write a bootloader client in your plugin. Ask for `devices:firmware`, carry your images, and
call `firmware::start-upload`. PitCrew resets the board into its bootloader, writes the image, checks
it and restarts the board. See the `firmware` section of [api.md](api.md).

```rust
static IMAGE: &[u8] = include_bytes!("../firmware/dist/my-esp32c3.bin");

firmware::start_upload("port", "esp32c3", IMAGE)?;   // PitCrew has the port now
// on each tick:
if let Some(job) = firmware::job() {
    if !job.running { /* show job.error or success, then open the port again */ }
}
```

## PitFlag wire v2

The Flags plugin uses PitFlag, a small protocol that follows these rules. You can use it for any
matrix device. The reference code is [crates/pitflag-wire](../crates/pitflag-wire/src/lib.rs) and
the firmware is in [plugins/flags/firmware](../plugins/flags/firmware/).

**Link:** 115200 baud, 8N1.

**Packet:** `0xA5 | type | len | payload[len] | crc8`. The CRC-8 uses the polynomial 0x07 with an
initial value of 0. It covers `type`, `len` and the payload. A receiver drops a packet with a bad CRC
and looks for the next `0xA5`.

| Type | Name | Direction | Payload |
| --- | --- | --- | --- |
| `0x01` | HELLO | device → PC | 14 bytes, see below |
| `0x01` | HELLO | PC → device | empty: "say hello" |
| `0x02` | PALETTE | PC → device | up to 16 colours, 3 bytes each (R, G, B). Index 0 is always off. |
| `0x03` | FRAME | PC → device | 32 bytes: 64 palette indices, 4 bits each, row-major, high nibble first |
| `0x04` | BRIGHTNESS | PC → device | 1 byte, 0 to 255 |
| `0x05` | CLEAR | PC → device | empty: all LEDs off |
| `0x06` | PING | PC → device | 1 byte sequence number |
| `0x07` | PONG | device → PC | the same byte |
| `0x08` | CONFIG | PC → device | 10 bytes, see below |
| `0x09` | CONFIG_ACK | device → PC | result code, CRC-8 of the config the device uses now |
| `0x0A` | BOOTLOADER | PC → device | `B` `O` `O` `T`: restart into the ROM download mode. Only the ESP32-C3 acts on it. The Flags plugin no longer sends it: PitCrew resets the board itself. |

### HELLO (device → PC)

| Byte | Field |
| --- | --- |
| 0–3 | the magic `P` `F` `L` `G`. A HELLO with this magic comes from PitFlag firmware. |
| 4 | wire version: 2 |
| 5–7 | firmware major, minor, patch |
| 8 | chip: 1 ATmega328P, 2 ATmega168, 3 ATmega32U4, 4 ESP32, 5 ESP32-C3 |
| 9–10 | width, height |
| 11 | 1 for an RGB matrix, 0 for one colour |
| 12 | max frames per second |
| 13 | CRC-8 of the config the device uses now |

PitFlag 1.x firmware sends the 8-byte wire v1 HELLO (`1`, major, minor, patch, width, height, rgb,
max fps), with no magic. The plugin reads its version and asks for the new firmware.

### CONFIG (PC → device)

The matrix settings. The device checks them, stores them in EEPROM and uses them at once.

| Byte | Field |
| --- | --- |
| 0 | layout version: 1 |
| 1 | matrix: 1 WS2812B, 2 MAX7219 |
| 2 | data pin (WS2812B data in, or MAX7219 DIN) |
| 3 | MAX7219 CLK pin |
| 4 | MAX7219 CS pin |
| 5 | colour order: 0 GRB, 1 RGB, 2 BRG |
| 6 | bit 0 zigzag wiring, bit 1 mirror left and right, bit 2 mirror top and bottom |
| 7 | rotation in quarter turns clockwise, 0 to 3, after the mirrors |
| 8–9 | current limit in mA, big-endian |

A pin of `0xFF` means the chip's default pin. CONFIG_ACK result codes: 0 taken, 1 bad length or
layout, 2 bad matrix, 3 a pin the chip cannot use, 4 a value out of range. After a refusal the
device keeps its old config.

**Session:**

1. The device sends HELLO when it starts.
2. The PC opens the port and waits for HELLO. After 1.8 s with no HELLO, it sends an empty HELLO.
   After 3.5 s, it reports "No PitFlag device answered on this port".
3. The PC checks the magic, `wire` = 2 and the size. It refuses other devices with a clear status.
4. If the config CRC in HELLO is not the CRC of the PC's settings, the PC sends CONFIG. With no
   CONFIG_ACK after 1 s, it sends it again. After a refusal, it waits for the settings to change.
5. The PC sends BRIGHTNESS, PALETTE, then FRAME. Later it sends only what changed. A new PALETTE is
   always followed by a FRAME.
6. If 250 ms pass with nothing sent, the PC sends a PING.
7. The device turns all LEDs off after 1500 ms without a packet.

A full frame is 36 bytes. At 30 frames a second that is 1 080 B/s, about 9% of the link.

## Firmware upload

The Flags plugin writes PitFlag to an ESP32-C3 itself, over the same `serialPort` setting. It
carries the merged image from `plugins/flags/firmware/dist/pitflag-esp32c3.bin` in its `.wasm`.
The code is [esp_rom.rs](../plugins/flags/src/esp_rom.rs) and [firmware.rs](../plugins/flags/src/firmware.rs).

1. If the board runs PitFlag, the plugin sends BOOTLOADER. The firmware sets
   `RTC_CNTL_FORCE_DOWNLOAD_BOOT` and restarts. A new board goes into the download mode when the
   driver holds BOOT while plugging the cable in.
2. The plugin opens the port again every 500 ms until the ROM answers a SYNC. It gives up after 12 s.
3. It speaks the esptool serial protocol to the ROM loader, with no RAM stub:
   - It checks the chip id at `0x40001000`. It refuses a classic ESP32 and any other chip.
   - It holds the RTC and super watchdogs, as esptool does over USB.
   - It sends SPI_ATTACH and SPI_SET_PARAMS (4 MB), then FLASH_BEGIN.
   - It writes 1 KB FLASH_DATA blocks, one command at a time.
4. It compares SPI_FLASH_MD5 with the image's MD5.
5. It clears the forced download mode and resets the chip with the RTC watchdog.
6. The link opens the port again and waits for HELLO.

The host opens a port with DTR and RTS off and has no call to change them. So the plugin cannot
reset an Arduino into its bootloader, or an ESP32 behind a USB-serial bridge. The screen tells the
driver to upload those images with the Arduino IDE.
