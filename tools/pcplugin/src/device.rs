//! A PitFlag device in memory, and the terminal drawing of what it shows.

use std::time::Instant;

use pitflag_wire as wire;

pub const HELLO: wire::Hello = wire::Hello { wire: 1, firmware: [1, 0, 0], width: 8, height: 8, rgb: true, max_fps: 30 };

/// Behaves as the firmware in plugins/flags/firmware does: hello on boot and on request, pong,
/// and dark after `HEARTBEAT_TIMEOUT_MS` without a packet.
pub struct FakeDevice {
    decoder: wire::Decoder,
    outbox: Vec<u8>,
    palette: Vec<u32>,
    frame: [u8; 64],
    brightness: u8,
    last_packet: Option<Instant>,
    pub packets: u64,
    pub frames: u64,
    pub bytes: u64,
}

impl FakeDevice {
    pub fn boot() -> FakeDevice {
        FakeDevice {
            decoder: wire::Decoder::new(),
            outbox: HELLO.encode(),
            palette: vec![0],
            frame: [0; 64],
            brightness: 255,
            last_packet: None,
            packets: 0,
            frames: 0,
            bytes: 0,
        }
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        self.bytes += bytes.len() as u64;
        for p in self.decoder.push(bytes) {
            self.packets += 1;
            self.last_packet = Some(Instant::now());
            match p.kind {
                wire::Kind::Hello if p.payload.is_empty() => self.outbox.extend(HELLO.encode()),
                wire::Kind::Palette => {
                    self.palette = p.payload.chunks_exact(3).map(|c| (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32).collect();
                }
                wire::Kind::Frame => {
                    if let Some(f) = wire::unpack_frame(&p.payload) {
                        self.frame = f;
                        self.frames += 1;
                    }
                }
                wire::Kind::Brightness => self.brightness = p.payload.first().copied().unwrap_or(255),
                wire::Kind::Clear => self.frame = [0; 64],
                wire::Kind::Ping => self.outbox.extend(wire::packet(wire::Kind::Pong, &p.payload)),
                _ => {}
            }
        }
    }

    pub fn take(&mut self, max: usize) -> Vec<u8> {
        let n = self.outbox.len().min(max);
        self.outbox.drain(..n).collect()
    }

    /// The 64 LED colours as 0xRRGGBB, after brightness; all off when the heartbeat stopped.
    pub fn leds(&self) -> [u32; 64] {
        let alive = self.last_packet.is_some_and(|t| t.elapsed().as_millis() as u64 <= wire::HEARTBEAT_TIMEOUT_MS);
        let mut out = [0u32; 64];
        if !alive {
            return out;
        }
        for (o, &i) in out.iter_mut().zip(&self.frame) {
            let c = self.palette.get(i as usize).copied().unwrap_or(0);
            *o = if i == 0 { 0 } else { scale(c, self.brightness) };
        }
        out
    }
}

/// Brightness as the firmware applies it, floored so a dim matrix still reads in a terminal.
fn scale(c: u32, b: u8) -> u32 {
    let k = (b as u32).max(64);
    let ch = |s: u32| ((c >> s) & 0xFF) * k / 255;
    ch(16) << 16 | ch(8) << 8 | ch(0)
}

/// Draws the matrix. `plain` prints one character per LED (`.` off, `#` lit) for logs and CI.
pub fn draw(leds: &[u32; 64], plain: bool) -> String {
    let mut s = String::new();
    for row in leds.chunks(8) {
        for &c in row {
            if plain {
                s.push(if c == 0 { '.' } else { '#' });
            } else if c == 0 {
                s.push_str("\x1b[38;2;40;40;40m● ");
            } else {
                s.push_str(&format!("\x1b[38;2;{};{};{}m● ", c >> 16, (c >> 8) & 0xFF, c & 0xFF));
            }
        }
        if !plain {
            s.push_str("\x1b[0m");
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_says_hello_on_boot_and_answers_pings() {
        let mut d = FakeDevice::boot();
        let hello = wire::Decoder::new().push(&d.take(256));
        assert_eq!(wire::Hello::decode(&hello[0].payload), Some(HELLO));
        d.receive(&wire::ping(7));
        assert_eq!(wire::Decoder::new().push(&d.take(256)), vec![wire::Packet { kind: wire::Kind::Pong, payload: vec![7] }]);
    }

    #[test]
    fn it_draws_the_last_frame_with_the_last_palette() {
        let mut d = FakeDevice::boot();
        d.receive(&wire::brightness(255));
        d.receive(&wire::palette(&[0, 0xFF0000]));
        let mut f = [0u8; 64];
        f[9] = 1;
        d.receive(&wire::frame(&f));
        let leds = d.leds();
        assert_eq!(leds[9], 0xFF0000);
        assert_eq!(leds[0], 0);
        assert_eq!(draw(&leds, true).lines().nth(1), Some(".#......"));
    }
}
