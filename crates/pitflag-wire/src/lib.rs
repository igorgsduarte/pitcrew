//! PitFlag wire v1: the framing between the Flags plugin and the matrix firmware.
//!
//! A packet is `0xA5 | type | len | payload[len] | crc8`, where the CRC-8 (poly 0x07, init 0)
//! covers `type`, `len` and the payload. The firmware in `plugins/flags/firmware/` implements
//! the same table. See `docs/serial-devices.md`.

pub const START: u8 = 0xA5;
pub const WIRE_VERSION: u8 = 1;
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

/// What a device says about itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hello {
    pub wire: u8,
    pub firmware: [u8; 3],
    pub width: u8,
    pub height: u8,
    pub rgb: bool,
    pub max_fps: u8,
}

impl Hello {
    pub fn encode(&self) -> Vec<u8> {
        packet(
            Kind::Hello,
            &[
                self.wire,
                self.firmware[0],
                self.firmware[1],
                self.firmware[2],
                self.width,
                self.height,
                self.rgb as u8,
                self.max_fps,
            ],
        )
    }

    pub fn decode(payload: &[u8]) -> Option<Hello> {
        if payload.len() < 8 {
            return None;
        }
        Some(Hello {
            wire: payload[0],
            firmware: [payload[1], payload[2], payload[3]],
            width: payload[4],
            height: payload[5],
            rgb: payload[6] != 0,
            max_fps: payload[7],
        })
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
    fn hello_round_trips() {
        let h = Hello { wire: 1, firmware: [1, 0, 0], width: 8, height: 8, rgb: true, max_fps: 30 };
        let p = Decoder::new().push(&h.encode());
        assert_eq!(Hello::decode(&p[0].payload), Some(h));
    }

    #[test]
    fn palette_is_three_bytes_per_colour() {
        let p = palette(&[0x000000, 0xFFD400]);
        assert_eq!(&p[..3], &[START, Kind::Palette as u8, 6]);
        assert_eq!(&p[6..9], &[0xFF, 0xD4, 0x00]);
    }
}
