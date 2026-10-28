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
3. **Wait for the device.** An Arduino Uno or Nano resets when the port opens. Its bootloader takes
   about 1.6 s. Ask the device who it is, and send nothing else until it answers.
4. **Send only what changed.** A frame that did not change is not sent again.
5. **Send a heartbeat.** Make the device go dark when the PC stops talking. Then a crash never leaves
   a stale flag on the matrix.
6. **Turn the device off in `on-stop`.**
7. **Use a checksum.** Serial lines drop and corrupt bytes.

## PitFlag wire v1

The Flags plugin uses PitFlag, a small protocol that follows these rules. You can use it for any
matrix device. The reference code is [crates/pitflag-wire](../crates/pitflag-wire/src/lib.rs) and
the firmware is in [plugins/flags/firmware](../plugins/flags/firmware/).

**Link:** 115200 baud, 8N1.

**Packet:** `0xA5 | type | len | payload[len] | crc8`. The CRC-8 uses the polynomial 0x07 with an
initial value of 0. It covers `type`, `len` and the payload. A receiver drops a packet with a bad CRC
and looks for the next `0xA5`.

| Type | Name | Direction | Payload |
| --- | --- | --- | --- |
| `0x01` | HELLO | device → PC | `wire` (1), firmware major, minor, patch, width, height, rgb (0/1), max fps |
| `0x01` | HELLO | PC → device | empty: "say hello" |
| `0x02` | PALETTE | PC → device | up to 16 colours, 3 bytes each (R, G, B). Index 0 is off. |
| `0x03` | FRAME | PC → device | 32 bytes: 64 palette indices, 4 bits each, row-major, high nibble first |
| `0x04` | BRIGHTNESS | PC → device | 1 byte, 0 to 255 |
| `0x05` | CLEAR | PC → device | empty: all LEDs off |
| `0x06` | PING | PC → device | 1 byte sequence number |
| `0x07` | PONG | device → PC | the same byte |

**Session:**

1. The device sends HELLO when it starts.
2. The PC opens the port and waits for HELLO. After 1.8 s with no HELLO, it sends an empty HELLO.
   After 3.5 s, it reports "No PitFlag device answered on this port".
3. The PC checks `wire` = 1 and the size. It refuses other devices with a clear status.
4. The PC sends BRIGHTNESS, PALETTE, then FRAME. Later it sends only what changed. A new PALETTE is
   always followed by a FRAME.
5. If 250 ms pass with nothing sent, the PC sends a PING.
6. The device turns all LEDs off after 1500 ms without a packet.

A full frame is 36 bytes. At 30 frames a second that is 1 080 B/s, about 9% of the link.
