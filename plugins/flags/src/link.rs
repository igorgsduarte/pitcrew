//! The device link: open the port, wait for HELLO, send what changed, keep the heartbeat.

use pitflag_wire as wire;

use crate::host::{Host, LogLevel, Port, StatusLevel};

pub const PORT_SETTING: &str = "port";
pub const BAUD: u32 = 115_200;
pub const RETRY_MS: u64 = 2_000;
/// An Uno resets when the port opens; its bootloader takes about 1.6 s.
pub const ASK_HELLO_AFTER_MS: u64 = 1_800;
pub const GIVE_UP_AFTER_MS: u64 = 3_500;

enum State {
    Closed { retry_at: u64 },
    Opening { port: Box<dyn Port>, since: u64, asked: bool },
    Open(Box<Open>),
}

struct Open {
    port: Box<dyn Port>,
    hello: wire::Hello,
    last_sent: u64,
    seq: u8,
    palette: Option<Vec<u32>>,
    frame: Option<[u8; 64]>,
    brightness: Option<u8>,
}

pub struct Link {
    state: State,
    decoder: wire::Decoder,
    status: Option<(StatusLevel, String)>,
    bytes_this_second: u64,
    second_started: u64,
    pub bytes_per_second: u64,
}

impl Default for Link {
    fn default() -> Self {
        Link {
            state: State::Closed { retry_at: 0 },
            decoder: wire::Decoder::new(),
            status: None,
            bytes_this_second: 0,
            second_started: 0,
            bytes_per_second: 0,
        }
    }
}

/// What the matrix should show now.
pub struct Want<'a> {
    pub colors: &'a [u32],
    pub frame: &'a [u8; 64],
    pub brightness: u8,
}

impl Link {
    pub fn hello(&self) -> Option<wire::Hello> {
        match &self.state {
            State::Open(o) => Some(o.hello),
            _ => None,
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, State::Open(_))
    }

    pub fn status(&self) -> Option<&(StatusLevel, String)> {
        self.status.as_ref()
    }

    /// Drops the port; the next tick opens it again (after a settings change).
    pub fn reopen(&mut self, host: &mut dyn Host, now: u64) {
        self.state = State::Closed { retry_at: now };
        self.decoder = wire::Decoder::new();
        host.log(LogLevel::Info, "reopening the device port");
    }

    pub fn close(&mut self) {
        if let State::Open(o) = &mut self.state {
            let _ = o.port.write(&wire::packet(wire::Kind::Clear, &[]));
        }
        self.state = State::Closed { retry_at: u64::MAX };
    }

    fn set_status(&mut self, host: &mut dyn Host, level: StatusLevel, text: String) {
        if self.status.as_ref() != Some(&(level, text.clone())) {
            host.set_status(level, &text);
            host.invalidate_ui();
            self.status = Some((level, text));
        }
    }

    pub fn tick(&mut self, host: &mut dyn Host, now: u64, want: &Want) {
        if now.saturating_sub(self.second_started) >= 1_000 {
            self.bytes_per_second = self.bytes_this_second;
            self.bytes_this_second = 0;
            self.second_started = now;
        }
        let state = std::mem::replace(&mut self.state, State::Closed { retry_at: now + RETRY_MS });
        self.state = match state {
            State::Closed { retry_at } if now < retry_at => State::Closed { retry_at },
            State::Closed { .. } => match host.open_port(PORT_SETTING, BAUD) {
                Ok(port) => {
                    self.decoder = wire::Decoder::new();
                    self.set_status(host, StatusLevel::Idle, "Waiting for the device to say hello…".into());
                    State::Opening { port, since: now, asked: false }
                }
                Err(e) => {
                    self.set_status(host, StatusLevel::Warning, format!("{e}. Retrying every 2 s."));
                    State::Closed { retry_at: now + RETRY_MS }
                }
            },
            State::Opening { mut port, since, mut asked } => self.opening(host, now, &mut port, since, &mut asked).unwrap_or(State::Opening { port, since, asked }),
            State::Open(mut open) => match self.open(host, now, &mut open, want) {
                Ok(()) => State::Open(open),
                Err(e) => {
                    self.set_status(host, StatusLevel::Warning, format!("The device stopped answering ({e}). Retrying every 2 s."));
                    State::Closed { retry_at: now + RETRY_MS }
                }
            },
        };
    }

    /// `Some(next state)` when the opening attempt is over.
    fn opening(&mut self, host: &mut dyn Host, now: u64, port: &mut Box<dyn Port>, since: u64, asked: &mut bool) -> Option<State> {
        let bytes = match port.read(256) {
            Ok(b) => b,
            Err(e) => {
                self.set_status(host, StatusLevel::Warning, format!("{e}. Retrying every 2 s."));
                return Some(State::Closed { retry_at: now + RETRY_MS });
            }
        };
        for p in self.decoder.push(&bytes) {
            if p.kind != wire::Kind::Hello {
                continue;
            }
            let Some(hello) = wire::Hello::decode(&p.payload) else { continue };
            if hello.wire != wire::WIRE_VERSION {
                self.set_status(host, StatusLevel::Error, format!("The device speaks PitFlag wire v{}; this plugin speaks v{}. Update its firmware.", hello.wire, wire::WIRE_VERSION));
                return Some(State::Closed { retry_at: now + RETRY_MS * 5 });
            }
            if (hello.width, hello.height) != (8, 8) {
                self.set_status(host, StatusLevel::Error, format!("The device is {}×{}; this plugin drives 8×8 matrices.", hello.width, hello.height));
                return Some(State::Closed { retry_at: now + RETRY_MS * 5 });
            }
            let [a, b, c] = hello.firmware;
            self.set_status(host, StatusLevel::Ok, format!("Connected · firmware {a}.{b}.{c} · 8×8 {}", if hello.rgb { "RGB" } else { "mono" }));
            host.log(LogLevel::Info, &format!("device said hello: {hello:?}"));
            let placeholder: Box<dyn Port> = Box::new(Gone);
            let port = std::mem::replace(port, placeholder);
            return Some(State::Open(Box::new(Open { port, hello, last_sent: 0, seq: 0, palette: None, frame: None, brightness: None })));
        }
        let waited = now.saturating_sub(since);
        if !*asked && waited >= ASK_HELLO_AFTER_MS {
            *asked = true;
            if let Err(e) = port.write(&wire::hello_request()) {
                self.set_status(host, StatusLevel::Warning, format!("{e}. Retrying every 2 s."));
                return Some(State::Closed { retry_at: now + RETRY_MS });
            }
        }
        if waited >= GIVE_UP_AFTER_MS {
            self.set_status(host, StatusLevel::Warning, "No PitFlag device answered on this port. Check the port and the firmware.".into());
            return Some(State::Closed { retry_at: now + RETRY_MS });
        }
        None
    }

    fn open(&mut self, _host: &mut dyn Host, now: u64, o: &mut Open, want: &Want) -> Result<(), String> {
        o.port.read(256)?;
        let send = |o: &mut Open, bytes: Vec<u8>, counter: &mut u64| -> Result<(), String> {
            o.port.write(&bytes)?;
            o.last_sent = now;
            *counter += bytes.len() as u64;
            Ok(())
        };
        let mut counter = 0;
        if o.brightness != Some(want.brightness) {
            send(o, wire::brightness(want.brightness), &mut counter)?;
            o.brightness = Some(want.brightness);
        }
        if o.palette.as_deref() != Some(want.colors) {
            send(o, wire::palette(want.colors), &mut counter)?;
            o.palette = Some(want.colors.to_vec());
            o.frame = None;
        }
        if o.frame.as_ref() != Some(want.frame) {
            send(o, wire::frame(want.frame), &mut counter)?;
            o.frame = Some(*want.frame);
        }
        if now.saturating_sub(o.last_sent) >= wire::PING_EVERY_MS {
            o.seq = o.seq.wrapping_add(1);
            let seq = o.seq;
            send(o, wire::ping(seq), &mut counter)?;
        }
        self.bytes_this_second += counter;
        Ok(())
    }
}

struct Gone;

impl Port for Gone {
    fn write(&mut self, _: &[u8]) -> Result<(), String> {
        Err("closed".into())
    }
    fn read(&mut self, _: u32) -> Result<Vec<u8>, String> {
        Err("closed".into())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A PitFlag device in memory: says hello when asked or on open, records what it receives.
    #[derive(Default)]
    pub struct FakeDevice {
        pub present: bool,
        pub hello_on_open: bool,
        pub unplugged: bool,
        pub outbox: Vec<u8>,
        pub received: Vec<wire::Packet>,
        decoder: wire::Decoder,
    }

    pub struct FakePort(pub Rc<RefCell<FakeDevice>>);

    impl Port for FakePort {
        fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
            let mut d = self.0.borrow_mut();
            if d.unplugged {
                return Err("the port is gone".into());
            }
            let packets = d.decoder.push(bytes);
            for p in packets {
                if p.kind == wire::Kind::Hello && p.payload.is_empty() {
                    let h = hello().encode();
                    d.outbox.extend(h);
                }
                d.received.push(p);
            }
            Ok(())
        }
        fn read(&mut self, _: u32) -> Result<Vec<u8>, String> {
            let mut d = self.0.borrow_mut();
            if d.unplugged {
                return Err("the port is gone".into());
            }
            Ok(std::mem::take(&mut d.outbox))
        }
    }

    pub fn hello() -> wire::Hello {
        wire::Hello { wire: 1, firmware: [1, 0, 0], width: 8, height: 8, rgb: true, max_fps: 30 }
    }

    #[derive(Default)]
    pub struct FakeHost {
        pub device: Rc<RefCell<FakeDevice>>,
        pub statuses: Vec<(StatusLevel, String)>,
        pub settings: String,
        pub written: Vec<(String, Vec<u8>)>,
    }

    impl Host for FakeHost {
        fn log(&mut self, _: LogLevel, _: &str) {}
        fn set_status(&mut self, level: StatusLevel, text: &str) {
            self.statuses.push((level, text.to_string()));
        }
        fn invalidate_ui(&mut self) {}
        fn open_port(&mut self, key: &str, _: u32) -> Result<Box<dyn Port>, String> {
            assert_eq!(key, PORT_SETTING);
            let mut d = self.device.borrow_mut();
            if !d.present || d.unplugged {
                return Err("No device on COM4".into());
            }
            if d.hello_on_open {
                let h = hello().encode();
                d.outbox.extend(h);
            }
            Ok(Box::new(FakePort(self.device.clone())))
        }
        fn settings_json(&mut self) -> String {
            self.settings.clone()
        }
        fn content_packs(&mut self, _: &str) -> Vec<String> {
            vec![crate::pack::BUILTIN_PACK.into()]
        }
        fn content_read(&mut self, _pack: &str, path: &str) -> Result<Vec<u8>, String> {
            std::fs::read(crate::pack::tests::classic_dir().join(path)).map_err(|e| e.to_string())
        }
        fn content_write_own(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
            self.written.push((path.into(), bytes.to_vec()));
            Ok(())
        }
    }

    fn want<'a>(frame: &'a [u8; 64]) -> Want<'a> {
        Want { colors: &[0x000000, 0xFFD400], frame, brightness: 40 }
    }

    #[test]
    fn it_opens_waits_for_hello_then_sends_brightness_palette_and_frame() {
        let mut host = FakeHost::default();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        let mut link = Link::default();
        let f = [1u8; 64];
        link.tick(&mut host, 0, &want(&f));
        assert!(!link.is_open());
        link.tick(&mut host, 33, &want(&f));
        assert!(link.is_open());
        link.tick(&mut host, 66, &want(&f));
        let kinds: Vec<_> = host.device.borrow().received.iter().map(|p| p.kind).collect();
        assert_eq!(kinds, vec![wire::Kind::Brightness, wire::Kind::Palette, wire::Kind::Frame]);
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Ok);
    }

    #[test]
    fn a_board_that_stays_quiet_is_asked_for_hello() {
        let mut host = FakeHost::default();
        host.device.borrow_mut().present = true;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        link.tick(&mut host, ASK_HELLO_AFTER_MS, &want(&f));
        link.tick(&mut host, ASK_HELLO_AFTER_MS + 33, &want(&f));
        assert!(link.is_open());
    }

    #[test]
    fn an_unchanged_frame_is_not_resent_but_the_heartbeat_is() {
        let mut host = FakeHost::default();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        let mut link = Link::default();
        let f = [1u8; 64];
        for t in (0..=1_000).step_by(33) {
            link.tick(&mut host, t, &want(&f));
        }
        let d = host.device.borrow();
        let frames = d.received.iter().filter(|p| p.kind == wire::Kind::Frame).count();
        let pings = d.received.iter().filter(|p| p.kind == wire::Kind::Ping).count();
        assert_eq!(frames, 1);
        assert!(pings >= 3, "pings: {pings}");
    }

    #[test]
    fn unplugging_reports_it_and_retries_every_two_seconds() {
        let mut host = FakeHost::default();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        let mut link = Link::default();
        let f = [1u8; 64];
        link.tick(&mut host, 0, &want(&f));
        link.tick(&mut host, 33, &want(&f));
        host.device.borrow_mut().unplugged = true;
        link.tick(&mut host, 66, &want(&f));
        assert!(!link.is_open());
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Warning);
        host.device.borrow_mut().unplugged = false;
        link.tick(&mut host, 1_000, &want(&f));
        assert!(!link.is_open(), "too early to retry");
        link.tick(&mut host, 66 + RETRY_MS, &want(&f));
        link.tick(&mut host, 99 + RETRY_MS, &want(&f));
        assert!(link.is_open());
    }

    #[test]
    fn a_device_on_another_wire_version_is_refused() {
        let mut host = FakeHost::default();
        host.device.borrow_mut().present = true;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        let mut h = hello();
        h.wire = 2;
        host.device.borrow_mut().outbox.extend(h.encode());
        link.tick(&mut host, 33, &want(&f));
        assert!(!link.is_open());
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Error);
    }
}
