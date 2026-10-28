//! PitFlag wire v2: the framing between the Flags plugin and the matrix firmware.
//!
//! A packet is `0xA5 | type | len | payload[len] | crc8`, where the CRC-8 (poly 0x07, init 0)
//! covers `type`, `len` and the payload. The firmware in `plugins/flags/firmware/` implements
//! the same table. See `docs/serial-devices.md`.

pub const START: u8 = 0xA5;
pub const WIRE_VERSION: u8 = 2;
/// The first four bytes of every v2 HELLO. A reply that starts with them comes from PitFlag firmware.
pub const MAGIC: [u8; 4] = *b"PFLG";
pub const MAX_PAYLOAD: usize = 64;
/// The device blanks itself after this long without a packet.
pub const HEARTBEAT_TIMEOUT_MS: u64 = 1500;
pub const PING_EVERY_MS: u64 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    /// Device → host: who it is. Host → device with no payload: "say hello".
    Hello = 0x01,
    Palette = 0x02,
    Frame = 0x03,
    Brightness = 0x04,
    Clear = 0x05,
    Ping = 0x06,
    Pong = 0x07,
    /// Host → device: the matrix settings, stored in EEPROM.
    Config = 0x08,
    /// Device → host: the answer to `Config`.
    ConfigAck = 0x09,
}

impl Kind {
    pub fn from_u8(v: u8) -> Option<Kind> {
        Some(match v {
            0x01 => Kind::Hello,
            0x02 => Kind::Palette,
            0x03 => Kind::Frame,
            0x04 => Kind::Brightness,
            0x05 => Kind::Clear,
            0x06 => Kind::Ping,
            0x07 => Kind::Pong,
            0x08 => Kind::Config,
            0x09 => Kind::ConfigAck,
            _ => return None,
        })
    }
}

pub fn crc8(bytes: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &b in bytes {
        crc ^= b;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
        }
    }
    crc
}

/// One whole packet, ready to write.
pub fn packet(kind: Kind, payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() <= MAX_PAYLOAD, "payload too long");
    let mut out = Vec::with_capacity(payload.len() + 4);
    out.push(START);
    out.push(kind as u8);
    out.push(payload.len() as u8);
    out.extend_from_slice(payload);
    out.push(crc8(&out[1..]));
    out
}

/// Up to 16 colours as 0xRRGGBB.
pub fn palette(colors: &[u32]) -> Vec<u8> {
    let colors = &colors[..colors.len().min(16)];
    let payload: Vec<u8> = colors
        .iter()
        .flat_map(|c| [(c >> 16) as u8, (c >> 8) as u8, *c as u8])
        .collect();
    packet(Kind::Palette, &payload)
}

/// An 8×8 frame of palette indices (0–15), row-major, packed two pixels per byte, high nibble first.
pub fn frame(pixels: &[u8; 64]) -> Vec<u8> {
    let mut payload = [0u8; 32];
    for (i, pair) in pixels.chunks(2).enumerate() {
        payload[i] = ((pair[0] & 0x0F) << 4) | (pair[1] & 0x0F);
    }
    packet(Kind::Frame, &payload)
}

pub fn unpack_frame(payload: &[u8]) -> Option<[u8; 64]> {
    if payload.len() != 32 {
        return None;
    }
    let mut px = [0u8; 64];
    for (i, b) in payload.iter().enumerate() {
        px[i * 2] = b >> 4;
        px[i * 2 + 1] = b & 0x0F;
    }
    Some(px)
}

pub fn brightness(level: u8) -> Vec<u8> {
    packet(Kind::Brightness, &[level])
}

pub fn ping(seq: u8) -> Vec<u8> {
    packet(Kind::Ping, &[seq])
}

pub fn hello_request() -> Vec<u8> {
    packet(Kind::Hello, &[])
}

/// The chip a firmware image was built for. Each chip has one image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
    Atmega328p,
    Atmega168,
    Atmega32u4,
    Esp32,
    Other(u8),
}

impl Chip {
    pub fn from_u8(v: u8) -> Chip {
        match v {
            1 => Chip::Atmega328p,
            2 => Chip::Atmega168,
            3 => Chip::Atmega32u4,
            4 => Chip::Esp32,
            n => Chip::Other(n),
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Chip::Atmega328p => 1,
            Chip::Atmega168 => 2,
            Chip::Atmega32u4 => 3,
            Chip::Esp32 => 4,
            Chip::Other(n) => n,
        }
    }

    /// The name people read on the chip, e.g. "ATmega168".
    pub fn label(self) -> String {
        match self {
            Chip::Atmega328p => "ATmega328P".into(),
            Chip::Atmega168 => "ATmega168".into(),
            Chip::Atmega32u4 => "ATmega32U4".into(),
            Chip::Esp32 => "ESP32".into(),
            Chip::Other(n) => format!("chip {n}"),
        }
    }
}

/// What a device says about itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hello {
    pub wire: u8,
    pub firmware: [u8; 3],
    pub chip: Chip,
    pub width: u8,
    pub height: u8,
    pub rgb: bool,
    pub max_fps: u8,
    /// `Config::crc` of the settings the device uses now.
    pub config_crc: u8,
}

pub const HELLO_LEN: usize = 14;

impl Hello {
    pub fn encode(&self) -> Vec<u8> {
        let mut p = Vec::with_capacity(HELLO_LEN);
        p.extend_from_slice(&MAGIC);
        p.extend_from_slice(&[
            self.wire,
            self.firmware[0],
            self.firmware[1],
            self.firmware[2],
            self.chip.as_u8(),
            self.width,
            self.height,
            self.rgb as u8,
            self.max_fps,
            self.config_crc,
        ]);
        packet(Kind::Hello, &p)
    }

    /// `None` when the payload does not start with `MAGIC` or is too short.
    pub fn decode(payload: &[u8]) -> Option<Hello> {
        if payload.len() < HELLO_LEN || payload[..4] != MAGIC {
            return None;
        }
        let p = &payload[4..];
        Some(Hello {
            wire: p[0],
            firmware: [p[1], p[2], p[3]],
            chip: Chip::from_u8(p[4]),
            width: p[5],
            height: p[6],
            rgb: p[7] != 0,
            max_fps: p[8],
            config_crc: p[9],
        })
    }

    /// The firmware version of a PitFlag 1.x board, which sends the 8-byte wire v1 HELLO.
    pub fn legacy_firmware(payload: &[u8]) -> Option<[u8; 3]> {
        (payload.len() == 8 && payload[0] == 1).then(|| [payload[1], payload[2], payload[3]])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Matrix {
    Ws2812 = 1,
    Max7219 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOrder {
    Grb = 0,
    Rgb = 1,
    Brg = 2,
}

/// A pin value that tells the firmware to use its chip's default pin.
pub const DEFAULT_PIN: u8 = 0xFF;
pub const CONFIG_LAYOUT: u8 = 1;
pub const CONFIG_LEN: usize = 10;

/// The matrix settings. The firmware checks the pins against its chip and stores the block in EEPROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub matrix: Matrix,
    /// WS2812B data in, or MAX7219 DIN. `DEFAULT_PIN` for the chip's default.
    pub data_pin: u8,
    /// MAX7219 CLK. `DEFAULT_PIN` for the chip's default.
    pub clock_pin: u8,
    /// MAX7219 CS. `DEFAULT_PIN` for the chip's default.
    pub cs_pin: u8,
    pub color_order: ColorOrder,
    pub serpentine: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    /// Quarter turns clockwise, 0 to 3, applied after the flips.
    pub rotate: u8,
    pub max_milliamps: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            matrix: Matrix::Ws2812,
            data_pin: DEFAULT_PIN,
            clock_pin: DEFAULT_PIN,
            cs_pin: DEFAULT_PIN,
            color_order: ColorOrder::Grb,
            serpentine: true,
            flip_x: false,
            flip_y: false,
            rotate: 0,
            max_milliamps: 500,
        }
    }
}

impl Config {
    pub fn payload(&self) -> [u8; CONFIG_LEN] {
        let flags = self.serpentine as u8 | (self.flip_x as u8) << 1 | (self.flip_y as u8) << 2;
        let [ma_hi, ma_lo] = self.max_milliamps.to_be_bytes();
        [
            CONFIG_LAYOUT,
            self.matrix as u8,
            self.data_pin,
            self.clock_pin,
            self.cs_pin,
            self.color_order as u8,
            flags,
            self.rotate & 3,
            ma_hi,
            ma_lo,
        ]
    }

    pub fn encode(&self) -> Vec<u8> {
        packet(Kind::Config, &self.payload())
    }

    /// The value a device reports in `Hello::config_crc` when it uses this config.
    pub fn crc(&self) -> u8 {
        crc8(&self.payload())
    }

    pub fn decode(p: &[u8]) -> Option<Config> {
        if p.len() != CONFIG_LEN || p[0] != CONFIG_LAYOUT || p[7] > 3 {
            return None;
        }
        Some(Config {
            matrix: match p[1] {
                1 => Matrix::Ws2812,
                2 => Matrix::Max7219,
                _ => return None,
            },
            data_pin: p[2],
            clock_pin: p[3],
            cs_pin: p[4],
            color_order: match p[5] {
                0 => ColorOrder::Grb,
                1 => ColorOrder::Rgb,
                2 => ColorOrder::Brg,
                _ => return None,
            },
            serpentine: p[6] & 1 != 0,
            flip_x: p[6] & 2 != 0,
            flip_y: p[6] & 4 != 0,
            rotate: p[7],
            max_milliamps: u16::from_be_bytes([p[8], p[9]]),
        })
    }
}

/// The device's answer to `Config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckCode {
    Ok,
    /// Wrong length or layout version.
    BadLayout,
    BadMatrix,
    /// A pin the chip cannot use (a serial, flash or input-only pin).
    BadPin,
    BadValue,
    Other(u8),
}

impl AckCode {
    pub fn from_u8(v: u8) -> AckCode {
        match v {
            0 => AckCode::Ok,
            1 => AckCode::BadLayout,
            2 => AckCode::BadMatrix,
            3 => AckCode::BadPin,
            4 => AckCode::BadValue,
            n => AckCode::Other(n),
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            AckCode::Ok => 0,
            AckCode::BadLayout => 1,
            AckCode::BadMatrix => 2,
            AckCode::BadPin => 3,
            AckCode::BadValue => 4,
            AckCode::Other(n) => n,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigAck {
    pub code: AckCode,
    /// The CRC of the config the device uses after this answer.
    pub crc: u8,
}

impl ConfigAck {
    pub fn encode(&self) -> Vec<u8> {
        packet(Kind::ConfigAck, &[self.code.as_u8(), self.crc])
    }

    pub fn decode(p: &[u8]) -> Option<ConfigAck> {
        (p.len() == 2).then(|| ConfigAck { code: AckCode::from_u8(p[0]), crc: p[1] })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub kind: Kind,
    pub payload: Vec<u8>,
}

/// Turns a byte stream into packets. Bytes before a start byte, unknown kinds and bad CRCs are dropped.
#[derive(Debug, Default)]
pub struct Decoder {
    buf: Vec<u8>,
}

impl Decoder {
    pub fn new() -> Decoder {
        Decoder::default()
    }

    pub fn push(&mut self, bytes: &[u8]) -> Vec<Packet> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        loop {
            match self.buf.iter().position(|&b| b == START) {
                None => {
                    self.buf.clear();
                    break;
                }
                Some(i) => {
                    self.buf.drain(..i);
                }
            }
            if self.buf.len() < 3 {
                break;
            }
            let len = self.buf[2] as usize;
            if len > MAX_PAYLOAD {
                self.buf.drain(..1);
                continue;
            }
            if self.buf.len() < len + 4 {
                break;
            }
            let body = &self.buf[1..3 + len];
            let crc = self.buf[3 + len];
            match (crc8(body) == crc, Kind::from_u8(self.buf[1])) {
                (true, Some(kind)) => {
                    out.push(Packet { kind, payload: self.buf[3..3 + len].to_vec() });
                    self.buf.drain(..len + 4);
                }
                _ => {
                    self.buf.drain(..1);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello() -> Hello {
        Hello { wire: 2, firmware: [2, 0, 0], chip: Chip::Atmega168, width: 8, height: 8, rgb: true, max_fps: 30, config_crc: Config::default().crc() }
    }

    #[test]
    fn crc_matches_the_reference_check_value() {
        assert_eq!(crc8(b"123456789"), 0xF4);
    }

    #[test]
    fn a_frame_round_trips() {
        let mut px = [0u8; 64];
        for (i, p) in px.iter_mut().enumerate() {
            *p = (i % 16) as u8;
        }
        let bytes = frame(&px);
        assert_eq!(bytes.len(), 36);
        let packets = Decoder::new().push(&bytes);
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].kind, Kind::Frame);
        assert_eq!(unpack_frame(&packets[0].payload), Some(px));
    }

    #[test]
    fn the_decoder_skips_noise_and_bad_crcs_and_waits_for_partial_packets() {
        let mut bad = ping(1);
        *bad.last_mut().unwrap() ^= 0xFF;
        let good = ping(2);
        let mut stream = vec![0x00, 0x13];
        stream.extend(&bad);
        stream.extend(&good[..3]);
        let mut d = Decoder::new();
        assert!(d.push(&stream).is_empty());
        let out = d.push(&good[3..]);
        assert_eq!(out, vec![Packet { kind: Kind::Ping, payload: vec![2] }]);
    }

    #[test]
    fn hello_round_trips_and_starts_with_the_magic() {
        let h = hello();
        let bytes = h.encode();
        assert_eq!(&bytes[3..7], b"PFLG");
        let p = Decoder::new().push(&bytes);
        assert_eq!(Hello::decode(&p[0].payload), Some(h));
    }

    #[test]
    fn a_hello_without_the_magic_is_not_pitflag() {
        let mut payload = hello().encode()[3..3 + HELLO_LEN].to_vec();
        payload[0] = b'X';
        assert_eq!(Hello::decode(&payload), None);
        assert_eq!(Hello::decode(&payload[..10]), None);
    }

    #[test]
    fn a_wire_v1_hello_shows_the_old_firmware_version() {
        let v1 = [1, 1, 0, 0, 8, 8, 1, 30];
        assert_eq!(Hello::decode(&v1), None);
        assert_eq!(Hello::legacy_firmware(&v1), Some([1, 0, 0]));
        assert_eq!(Hello::legacy_firmware(&hello().encode()[3..3 + HELLO_LEN]), None);
    }

    #[test]
    fn unknown_chips_survive_a_round_trip() {
        let mut h = hello();
        h.chip = Chip::Other(9);
        let p = Decoder::new().push(&h.encode());
        assert_eq!(Hello::decode(&p[0].payload).unwrap().chip, Chip::Other(9));
    }

    #[test]
    fn config_round_trips_and_its_crc_follows_every_field() {
        let c = Config {
            matrix: Matrix::Max7219,
            data_pin: 11,
            clock_pin: 13,
            cs_pin: 10,
            color_order: ColorOrder::Brg,
            serpentine: false,
            flip_x: true,
            flip_y: true,
            rotate: 3,
            max_milliamps: 1200,
        };
        let p = Decoder::new().push(&c.encode());
        assert_eq!(p[0].kind, Kind::Config);
        assert_eq!(Config::decode(&p[0].payload), Some(c));
        let mut d = c;
        d.rotate = 2;
        assert_ne!(c.crc(), d.crc());
        let mut d = c;
        d.max_milliamps = 1201;
        assert_ne!(c.crc(), d.crc());
    }

    #[test]
    fn a_config_with_a_bad_layout_or_value_is_refused() {
        let mut p = Config::default().payload();
        p[0] = 9;
        assert_eq!(Config::decode(&p), None);
        let mut p = Config::default().payload();
        p[1] = 3;
        assert_eq!(Config::decode(&p), None);
        let mut p = Config::default().payload();
        p[7] = 4;
        assert_eq!(Config::decode(&p), None);
        assert_eq!(Config::decode(&Config::default().payload()[..9]), None);
    }

    #[test]
    fn the_default_config_uses_the_chip_default_pins() {
        let p = Config::default().payload();
        assert_eq!(&p[2..5], &[DEFAULT_PIN; 3]);
        assert_eq!(p[6], 1, "serpentine on, no flips");
    }

    #[test]
    fn config_ack_round_trips() {
        let a = ConfigAck { code: AckCode::BadPin, crc: 0x42 };
        let p = Decoder::new().push(&a.encode());
        assert_eq!(p[0].kind, Kind::ConfigAck);
        assert_eq!(ConfigAck::decode(&p[0].payload), Some(a));
    }

    #[test]
    fn palette_is_three_bytes_per_colour() {
        let p = palette(&[0x000000, 0xFFD400]);
        assert_eq!(&p[..3], &[START, Kind::Palette as u8, 6]);
        assert_eq!(&p[6..9], &[0xFF, 0xD4, 0x00]);
    }
}
