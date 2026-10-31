//! The device link: open the port, wait for HELLO, send what changed, keep the heartbeat.

use pitflag_wire as wire;

use crate::host::{Host, LogLevel, Port, StatusLevel};

pub const PORT_SETTING: &str = "port";
pub const BAUD: u32 = 115_200;
pub const RETRY_MS: u64 = 2_000;
/// An Uno resets when the port opens; its bootloader takes about 1.6 s.
pub const ASK_HELLO_AFTER_MS: u64 = 1_800;
pub const GIVE_UP_AFTER_MS: u64 = 3_500;
/// A CONFIG with no CONFIG_ACK after this long is sent again.
pub const CONFIG_RESEND_MS: u64 = 1_000;
/// The brightness, the palette and the frame go again this often, even when nothing changed.
pub const REFRESH_MS: u64 = 1_000;

enum State {
    Closed { retry_at: u64 },
    Opening { port: Box<dyn Port>, since: u64, asked: bool },
    Open(Box<Open>),
}

struct Open {
    port: Box<dyn Port>,
    hello: wire::Hello,
    last_sent: u64,
    last_refresh: u64,
    seq: u8,
    palette: Option<Vec<u32>>,
    frame: Option<[u8; 64]>,
    brightness: Option<u8>,
    /// The CRC of the config the device uses, from HELLO and then from each CONFIG_ACK.
    config_crc: u8,
    /// The CRC of the CONFIG waiting for its ACK, and when it was sent.
    config_pending: Option<(u8, u64)>,
    /// The CRC of a CONFIG the device refused. It is not sent again until the settings change.
    config_refused: Option<u8>,
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
    pub config: wire::Config,
}

/// The header pill's text. The device card names the chip and the firmware.
fn connected_text(_h: &wire::Hello) -> String {
    "Connected".into()
}

fn refusal_text(code: wire::AckCode) -> &'static str {
    match code {
        wire::AckCode::BadPin => "a pin this board cannot use",
        wire::AckCode::BadMatrix => "an unknown matrix type",
        wire::AckCode::BadLayout => "a settings format it does not know",
        _ => "a value out of range",
    }
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

    /// Drops the port and leaves it closed until `reopen`, so PitCrew can use it for a firmware job.
    pub fn suspend(&mut self) {
        self.state = State::Closed { retry_at: u64::MAX };
        self.decoder = wire::Decoder::new();
    }

    pub fn set_status(&mut self, host: &mut dyn Host, level: StatusLevel, text: String) {
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
            if let Some([a, b, c]) = wire::Hello::legacy_firmware(&p.payload) {
                self.set_status(host, StatusLevel::Error, format!("This board runs PitFlag firmware {a}.{b}.{c}, which speaks wire v1. Upload the new firmware to it."));
                return Some(State::Closed { retry_at: now + RETRY_MS * 5 });
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
            self.set_status(host, StatusLevel::Ok, connected_text(&hello));
            host.log(LogLevel::Info, &format!("device said hello: {hello:?}"));
            let placeholder: Box<dyn Port> = Box::new(Gone);
            let port = std::mem::replace(port, placeholder);
            return Some(State::Open(Box::new(Open {
                port,
                hello,
                last_sent: 0,
                last_refresh: now,
                seq: 0,
                palette: None,
                frame: None,
                brightness: None,
                config_crc: hello.config_crc,
                config_pending: None,
                config_refused: None,
            })));
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

    fn open(&mut self, host: &mut dyn Host, now: u64, o: &mut Open, want: &Want) -> Result<(), String> {
        let bytes = o.port.read(256)?;
        for p in self.decoder.push(&bytes) {
            if p.kind != wire::Kind::ConfigAck {
                continue;
            }
            let Some(ack) = wire::ConfigAck::decode(&p.payload) else { continue };
            let sent = o.config_pending.take().map(|(crc, _)| crc);
            o.config_crc = ack.crc;
            if ack.code == wire::AckCode::Ok {
                o.hello.rgb = want.config.matrix == wire::Matrix::Ws2812;
                o.hello.config_crc = ack.crc;
                self.set_status(host, StatusLevel::Ok, connected_text(&o.hello));
                host.log(LogLevel::Info, "the device took the matrix settings");
            } else {
                o.config_refused = sent;
                self.set_status(host, StatusLevel::Warning, format!("The board refused the matrix settings: {}. Check the pins in Settings.", refusal_text(ack.code)));
            }
        }

        // A packet can be lost on the line (see the firmware's QUIET_US), so the whole state goes
        // again every REFRESH_MS.
        if now.saturating_sub(o.last_refresh) >= REFRESH_MS {
            o.last_refresh = now;
            o.brightness = None;
            o.palette = None;
        }
        // One write per tick: the device sees one burst, then a quiet line to draw in.
        let mut out = Vec::new();
        let want_crc = want.config.crc();
        let waiting = o.config_pending.is_some_and(|(crc, at)| crc == want_crc && now.saturating_sub(at) < CONFIG_RESEND_MS);
        if o.config_crc != want_crc && o.config_refused != Some(want_crc) && !waiting {
            out.extend(want.config.encode());
            o.config_pending = Some((want_crc, now));
        }
        if o.brightness != Some(want.brightness) {
            out.extend(wire::brightness(want.brightness));
            o.brightness = Some(want.brightness);
        }
        if o.palette.as_deref() != Some(want.colors) {
            out.extend(wire::palette(want.colors));
            o.palette = Some(want.colors.to_vec());
            o.frame = None;
        }
        if o.frame.as_ref() != Some(want.frame) {
            out.extend(wire::frame(want.frame));
            o.frame = Some(*want.frame);
        }
        if out.is_empty() && now.saturating_sub(o.last_sent) >= wire::PING_EVERY_MS {
            o.seq = o.seq.wrapping_add(1);
            out.extend(wire::ping(o.seq));
        }
        if !out.is_empty() {
            o.port.write(&out)?;
            o.last_sent = now;
            self.bytes_this_second += out.len() as u64;
        }
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
    use crate::host::{BoardInfo, Firmware, FirmwareJob};

    /// A PitFlag device in memory: says hello when asked or on open, takes CONFIG, records what it
    /// receives.
    pub struct FakeDevice {
        pub present: bool,
        pub hello_on_open: bool,
        pub unplugged: bool,
        /// Answer every CONFIG with this code instead of taking it.
        pub refuse_config: Option<wire::AckCode>,
        /// Say the 8-byte wire v1 HELLO of PitFlag 1.0.0.
        pub legacy: bool,
        pub chip: wire::Chip,
        /// The firmware version the HELLO names.
        pub firmware: [u8; 3],
        pub config: wire::Config,
        pub outbox: Vec<u8>,
        pub received: Vec<wire::Packet>,
        decoder: wire::Decoder,
    }

    impl Default for FakeDevice {
        fn default() -> Self {
            FakeDevice {
                present: false,
                hello_on_open: false,
                unplugged: false,
                refuse_config: None,
                legacy: false,
                chip: wire::Chip::Atmega168,
                firmware: [2, 0, 0],
                config: wire::Config::default(),
                outbox: Vec::new(),
                received: Vec::new(),
                decoder: wire::Decoder::new(),
            }
        }
    }

    impl FakeDevice {
        fn say_hello(&mut self) {
            let h = if self.legacy {
                wire::packet(wire::Kind::Hello, &[1, 1, 0, 0, 8, 8, 1, 30])
            } else {
                wire::Hello { chip: self.chip, firmware: self.firmware, ..hello(self.config.crc()) }.encode()
            };
            self.outbox.extend(h);
        }
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
                    d.say_hello();
                }
                if p.kind == wire::Kind::Config {
                    let code = match (d.refuse_config, wire::Config::decode(&p.payload)) {
                        (Some(code), _) => code,
                        (None, Some(c)) => {
                            d.config = c;
                            wire::AckCode::Ok
                        }
                        (None, None) => wire::AckCode::BadLayout,
                    };
                    let ack = wire::ConfigAck { code, crc: d.config.crc() }.encode();
                    d.outbox.extend(ack);
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

    pub fn hello(config_crc: u8) -> wire::Hello {
        wire::Hello { wire: 2, firmware: [2, 0, 0], chip: wire::Chip::Atmega168, width: 8, height: 8, rgb: true, max_fps: 30, config_crc }
    }

    #[derive(Default)]
    pub struct FakeHost {
        pub device: Rc<RefCell<FakeDevice>>,
        pub statuses: Vec<(StatusLevel, String)>,
        pub settings: String,
        pub written: Vec<(String, Vec<u8>)>,
        /// Files already in the data folder when the plugin starts.
        pub seeded: Vec<(String, Vec<u8>)>,
        pub invalidated: usize,
        /// Firmware calls in order: ("upload", chip, image bytes) or ("probe", "", 0).
        pub fw_calls: Vec<(String, String, usize)>,
        /// The job `firmware_job` reports.
        pub fw_job: Option<FirmwareJob>,
        /// Makes `firmware_upload` and `firmware_probe` refuse with this text.
        pub fw_refuse: Option<String>,
    }

    impl Firmware for FakeHost {
        fn firmware_describe(&mut self, key: &str) -> Result<BoardInfo, String> {
            assert_eq!(key, PORT_SETTING);
            Ok(BoardInfo { transport: "usb-jtag".into(), chip: None, usb_name: "Fake USB".into(), detail: String::new() })
        }
        fn firmware_probe(&mut self, key: &str) -> Result<(), String> {
            assert_eq!(key, PORT_SETTING);
            if let Some(e) = &self.fw_refuse {
                return Err(e.clone());
            }
            self.fw_calls.push(("probe".into(), String::new(), 0));
            self.fw_job = Some(FirmwareJob { upload: false, running: true, ..FirmwareJob::default() });
            Ok(())
        }
        fn firmware_upload(&mut self, key: &str, chip: &str, image: &[u8]) -> Result<(), String> {
            assert_eq!(key, PORT_SETTING);
            if let Some(e) = &self.fw_refuse {
                return Err(e.clone());
            }
            self.fw_calls.push(("upload".into(), chip.into(), image.len()));
            self.fw_job = Some(FirmwareJob { upload: true, running: true, ..FirmwareJob::default() });
            Ok(())
        }
        fn firmware_job(&mut self) -> Option<FirmwareJob> {
            self.fw_job.clone()
        }
    }

    impl Host for FakeHost {
        fn log(&mut self, _: LogLevel, _: &str) {}
        fn set_status(&mut self, level: StatusLevel, text: &str) {
            self.statuses.push((level, text.to_string()));
        }
        fn invalidate_ui(&mut self) {
            self.invalidated += 1;
        }
        fn open_port(&mut self, key: &str, _: u32) -> Result<Box<dyn Port>, String> {
            assert_eq!(key, PORT_SETTING);
            let mut d = self.device.borrow_mut();
            if !d.present || d.unplugged {
                return Err("No device on COM4".into());
            }
            if d.hello_on_open {
                d.say_hello();
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
        fn content_read_own(&mut self, path: &str) -> Result<Vec<u8>, String> {
            self.written
                .iter()
                .rev()
                .chain(self.seeded.iter())
                .find(|(p, _)| p == path)
                .map(|(_, b)| b.clone())
                .ok_or_else(|| format!("{path}: not found"))
        }
    }

    fn want(frame: &[u8; 64]) -> Want<'_> {
        Want { colors: &[0x000000, 0xFFD400], frame, brightness: 40, config: wire::Config::default() }
    }

    fn present_host() -> FakeHost {
        let host = FakeHost::default();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        host
    }

    fn kinds(host: &FakeHost) -> Vec<wire::Kind> {
        host.device.borrow().received.iter().map(|p| p.kind).collect()
    }

    #[test]
    fn it_opens_waits_for_hello_then_sends_brightness_palette_and_frame() {
        let mut host = present_host();
        let mut link = Link::default();
        let f = [1u8; 64];
        link.tick(&mut host, 0, &want(&f));
        assert!(!link.is_open());
        link.tick(&mut host, 33, &want(&f));
        assert!(link.is_open());
        link.tick(&mut host, 66, &want(&f));
        assert_eq!(kinds(&host), vec![wire::Kind::Brightness, wire::Kind::Palette, wire::Kind::Frame]);
        let (level, text) = host.statuses.last().unwrap();
        assert_eq!(*level, StatusLevel::Ok);
        assert_eq!(text, "Connected");
    }

    #[test]
    fn a_board_that_stays_quiet_is_asked_for_hello() {
        let mut host = present_host();
        host.device.borrow_mut().hello_on_open = false;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        link.tick(&mut host, ASK_HELLO_AFTER_MS, &want(&f));
        link.tick(&mut host, ASK_HELLO_AFTER_MS + 33, &want(&f));
        assert!(link.is_open());
    }

    #[test]
    fn an_unchanged_frame_is_not_resent_but_the_heartbeat_is() {
        let mut host = present_host();
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
    fn the_whole_state_goes_again_every_refresh_so_a_lost_packet_heals() {
        let mut host = present_host();
        let mut link = Link::default();
        let f = [1u8; 64];
        for t in (0..=2_200).step_by(33) {
            link.tick(&mut host, t, &want(&f));
        }
        let d = host.device.borrow();
        let count = |k: wire::Kind| d.received.iter().filter(|p| p.kind == k).count();
        assert_eq!(count(wire::Kind::Frame), 3);
        assert_eq!(count(wire::Kind::Palette), 3);
        assert_eq!(count(wire::Kind::Brightness), 3);
    }

    #[test]
    fn unplugging_reports_it_and_retries_every_two_seconds() {
        let mut host = present_host();
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
        let mut host = present_host();
        host.device.borrow_mut().hello_on_open = false;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        let mut h = hello(0);
        h.wire = 3;
        host.device.borrow_mut().outbox.extend(h.encode());
        link.tick(&mut host, 33, &want(&f));
        assert!(!link.is_open());
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Error);
    }

    #[test]
    fn a_pitflag_1_board_is_told_to_get_the_new_firmware() {
        let mut host = present_host();
        host.device.borrow_mut().legacy = true;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        link.tick(&mut host, 33, &want(&f));
        assert!(!link.is_open());
        let (level, text) = host.statuses.last().unwrap();
        assert_eq!(*level, StatusLevel::Error);
        assert!(text.contains("firmware 1.0.0"), "{text}");
    }

    #[test]
    fn a_hello_without_the_magic_is_ignored() {
        let mut host = present_host();
        host.device.borrow_mut().hello_on_open = false;
        let mut link = Link::default();
        let f = [0u8; 64];
        link.tick(&mut host, 0, &want(&f));
        let mut payload = hello(0).encode()[3..17].to_vec();
        payload[..4].copy_from_slice(b"XXXX");
        host.device.borrow_mut().outbox.extend(wire::packet(wire::Kind::Hello, &payload));
        link.tick(&mut host, 33, &want(&f));
        assert!(!link.is_open());
        link.tick(&mut host, GIVE_UP_AFTER_MS, &want(&f));
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Warning);
    }

    #[test]
    fn matching_settings_send_no_config() {
        let mut host = present_host();
        let mut link = Link::default();
        let f = [1u8; 64];
        for t in (0..=500).step_by(33) {
            link.tick(&mut host, t, &want(&f));
        }
        assert!(!kinds(&host).contains(&wire::Kind::Config));
    }

    #[test]
    fn changed_settings_send_one_config_and_the_device_keeps_them() {
        let mut host = present_host();
        let mut link = Link::default();
        let f = [1u8; 64];
        let config = wire::Config { rotate: 1, data_pin: 5, ..wire::Config::default() };
        for t in (0..=1_500).step_by(33) {
            link.tick(&mut host, t, &Want { config, ..want(&f) });
        }
        assert_eq!(kinds(&host).iter().filter(|k| **k == wire::Kind::Config).count(), 1);
        assert_eq!(kinds(&host)[0], wire::Kind::Config, "CONFIG goes before the first frame");
        assert_eq!(host.device.borrow().config, config);
        assert_eq!(link.hello().unwrap().config_crc, config.crc());
        assert_eq!(host.statuses.last().unwrap().0, StatusLevel::Ok);
    }

    #[test]
    fn a_refused_config_is_reported_and_not_sent_again_until_it_changes() {
        let mut host = present_host();
        host.device.borrow_mut().refuse_config = Some(wire::AckCode::BadPin);
        let mut link = Link::default();
        let f = [1u8; 64];
        let mut config = wire::Config { data_pin: 1, ..wire::Config::default() };
        for t in (0..=3_000).step_by(33) {
            link.tick(&mut host, t, &Want { config, ..want(&f) });
        }
        assert_eq!(kinds(&host).iter().filter(|k| **k == wire::Kind::Config).count(), 1);
        let (level, text) = host.statuses.last().unwrap();
        assert_eq!(*level, StatusLevel::Warning);
        assert!(text.contains("a pin this board cannot use"), "{text}");
        assert!(link.is_open(), "the flags keep going with the old settings");

        host.device.borrow_mut().refuse_config = None;
        config.data_pin = 7;
        link.tick(&mut host, 3_033, &Want { config, ..want(&f) });
        link.tick(&mut host, 3_066, &Want { config, ..want(&f) });
        assert_eq!(kinds(&host).iter().filter(|k| **k == wire::Kind::Config).count(), 2);
        assert_eq!(host.device.borrow().config, config);
    }

    #[test]
    fn a_lost_config_ack_is_sent_again_after_a_second() {
        let mut host = present_host();
        let mut link = Link::default();
        let f = [1u8; 64];
        let config = wire::Config { flip_x: true, ..wire::Config::default() };
        link.tick(&mut host, 0, &Want { config, ..want(&f) });
        link.tick(&mut host, 33, &Want { config, ..want(&f) });
        // The ACK is lost on the line.
        link.tick(&mut host, 66, &Want { config, ..want(&f) });
        host.device.borrow_mut().outbox.clear();
        link.tick(&mut host, 99, &Want { config, ..want(&f) });
        link.tick(&mut host, 66 + CONFIG_RESEND_MS, &Want { config, ..want(&f) });
        assert_eq!(kinds(&host).iter().filter(|k| **k == wire::Kind::Config).count(), 2);
    }
}
