//! The plugin itself: settings, the pack, the player, the link and the screen.

use std::collections::BTreeSet;

use pitflag_wire as wire;
use serde::Deserialize;

use crate::firmware::{self, Board};
use crate::host::{BoardInfo, FirmwareJob, Host, LogLevel, StatusLevel};
use crate::link::{Link, Want, PORT_SETTING};
use crate::matrix_test;
use crate::pack::{Animation, BUILTIN_PACK, CONTENT_TYPE, Pack, PackFile};
use crate::player::{OFF, Player, STANDBY, Source};
use crate::setup::{self, Check, DRAFT_FILE, Device, PREFS_FILE, Prefs, Step};
use crate::signal::Signal;
use crate::ui::{Event, Gap, Kind, Node, Tone, Tree, Variant};

pub const TESTS: [&str; 13] = [
    "lights-ready",
    "lights-set",
    "green-flash",
    "yellow",
    "yellow-waving",
    "debris",
    "blue-flash",
    "white",
    "checkered",
    "red",
    "black",
    "meatball",
    "black-x",
];
/// The Animations tab's flag switches, in groups: (group id, heading, flags). Covers every flag in `TESTS`.
pub const FLAG_GROUPS: [(&str, &str, &[&str]); 4] = [
    ("start", "Start", &["lights-ready", "lights-set", "green-flash"]),
    ("caution", "Caution", &["yellow", "yellow-waving", "debris"]),
    ("race", "Race", &["blue-flash", "white", "checkered"]),
    ("stop", "Stop and penalties", &["red", "black", "meatball", "black-x"]),
];
pub const PLAYBACK_SETTINGS: [&str; 3] = ["animationPack", "greenHoldS", "standby"];
pub const MY_PACK_ID: &str = "local.my-pack";
const UI_EVERY_MS: u64 = 200;
/// How long a board may take to answer after an upload before the screen says it has not.
const ANSWER_WAIT_MS: u64 = 20_000;
/// A matrix test screen stays this long without a click, then the matrix goes back to the flag.
pub const MATRIX_TEST_MS: u64 = 120_000;

/// The name the screen gives a flag animation.
pub fn flag_label(id: &str) -> &str {
    match id {
        "lights-ready" => "Start lights: ready",
        "lights-set" => "Start lights: set",
        "green-flash" => "Green",
        "yellow" => "Yellow",
        "yellow-waving" => "Yellow, waving",
        "debris" => "Debris",
        "blue-flash" => "Blue",
        "white" => "White",
        "checkered" => "Checkered",
        "red" => "Red",
        "black" => "Black",
        "meatball" => "Meatball (repair)",
        "black-x" => "Disqualified",
        other => other,
    }
}

/// An animation's name on screen: the flag's label, or the id with a capital and spaces ("my-flag" → "My flag").
pub fn anim_label(id: &str) -> String {
    let label = flag_label(id);
    if label != id {
        return label.to_string();
    }
    let spaced = id.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// The scale per channel that makes a WS2812B look like the screen: its green and blue look brighter
/// than its red at the same value. The values are FastLED's TypicalSMD5050 correction.
const LED_CORRECTION: [u32; 3] = [255, 176, 240];

/// `rgb` with `LED_CORRECTION` applied.
pub fn led_color(rgb: u32) -> u32 {
    let [_, r, g, b] = rgb.to_be_bytes();
    let scale = |v: u8, k: u32| (v as u32 * k + 127) / 255;
    (scale(r, LED_CORRECTION[0]) << 16) | (scale(g, LED_CORRECTION[1]) << 8) | scale(b, LED_CORRECTION[2])
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub port: Option<String>,
    #[serde(default = "default_brightness")]
    pub brightness: u8,
    #[serde(default = "default_pack")]
    pub animation_pack: String,
    #[serde(default = "default_hold")]
    pub green_hold_s: f64,
    /// Outside a race, play the pack's `standby` animation instead of a dark matrix.
    #[serde(default = "default_true")]
    pub standby: bool,
    #[serde(default = "default_matrix")]
    pub matrix: String,
    /// Empty means the board's default pin.
    #[serde(default)]
    pub data_pin: Option<u8>,
    #[serde(default)]
    pub clock_pin: Option<u8>,
    #[serde(default)]
    pub cs_pin: Option<u8>,
    #[serde(default = "default_color_order")]
    pub color_order: String,
    /// Not on the Flags screen; only in PitCrew's settings for the plugin.
    #[serde(default = "default_true")]
    pub serpentine: bool,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    /// Applies `LED_CORRECTION` to the colors sent to a WS2812B. The screen keeps the pack's colors.
    #[serde(default = "default_true")]
    pub color_correction: bool,
    #[serde(default = "default_rotate")]
    pub rotate: String,
    #[serde(default = "default_max_milliamps")]
    pub max_milliamps: u16,
    /// The board the firmware upload is for. See `firmware::Board::id`.
    #[serde(default = "default_board")]
    pub board: String,
}

fn default_board() -> String {
    Board::Atmega328p.id().into()
}

fn default_brightness() -> u8 {
    40
}
fn default_matrix() -> String {
    "ws2812".into()
}
fn default_color_order() -> String {
    "grb".into()
}
fn default_true() -> bool {
    true
}
fn default_rotate() -> String {
    "0".into()
}
fn default_max_milliamps() -> u16 {
    500
}
fn default_pack() -> String {
    BUILTIN_PACK.into()
}
fn default_hold() -> f64 {
    3.0
}

impl Settings {
    pub fn parse(json: &str) -> Settings {
        serde_json::from_str(json).unwrap_or_else(|_| serde_json::from_str("{}").expect("defaults"))
    }

    /// The matrix settings as the firmware stores them. Unknown values fall back to the defaults.
    pub fn config(&self) -> wire::Config {
        let d = wire::Config::default();
        wire::Config {
            matrix: if self.matrix == "max7219" { wire::Matrix::Max7219 } else { wire::Matrix::Ws2812 },
            data_pin: self.data_pin.unwrap_or(wire::DEFAULT_PIN),
            clock_pin: self.clock_pin.unwrap_or(wire::DEFAULT_PIN),
            cs_pin: self.cs_pin.unwrap_or(wire::DEFAULT_PIN),
            color_order: match self.color_order.as_str() {
                "rgb" => wire::ColorOrder::Rgb,
                "brg" => wire::ColorOrder::Brg,
                _ => wire::ColorOrder::Grb,
            },
            serpentine: self.serpentine,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            rotate: match self.rotate.as_str() {
                "90" => 1,
                "180" => 2,
                "270" => 3,
                _ => 0,
            },
            max_milliamps: if self.max_milliamps == 0 { d.max_milliamps } else { self.max_milliamps },
        }
    }
}

struct Editor {
    anim: Animation,
    frame: usize,
    /// The editor holds changes that are not saved.
    dirty: bool,
}

pub struct App {
    settings: Settings,
    pack: Pack,
    signal: Option<Signal>,
    connected: bool,
    in_car: bool,
    player: Player,
    link: Link,
    blank: Animation,
    shown_name: String,
    shown_source: Source,
    shown_frame: [u8; 64],
    shown_colors: Vec<u32>,
    tab: String,
    prefs: Prefs,
    /// The setup step on screen. `None` shows the Live and Animations tabs.
    setup: Option<Step>,
    editor: Option<Editor>,
    /// What the last Save to My pack did, shown under the editor.
    save_note: Option<(StatusLevel, String)>,
    /// The animation picked while the editor held unsaved changes.
    pending_edit: Option<String>,
    /// The flag the Play button plays.
    play_pick: String,
    /// The matrix test step on screen, if the test runs.
    matrix_test: Option<usize>,
    saved: BTreeSet<String>,
    /// PitCrew holds the port while it detects the board or writes it; the link waits.
    fw_running: bool,
    /// The job as PitCrew last reported it.
    fw_job: Option<FirmwareJob>,
    fw_result: Option<(StatusLevel, String)>,
    /// The heading of `fw_result`, when it has one.
    fw_title: Option<String>,
    /// The running job writes the board; otherwise it only asks the board what it is.
    fw_uploading: bool,
    /// When the last upload finished well, until the board answers with the new firmware.
    fw_landed_at: Option<u64>,
    /// The board gave no answer for `ANSWER_WAIT_MS` after the upload.
    fw_silent: bool,
    /// An upload to the current port finished well, so the port is the driver's board.
    wrote_port: bool,
    /// The driver said the port that gives no PitFlag answer is their matrix board, or Detect board
    /// found the chip picked in Board on it. Upload waits for one of the two.
    port_confirmed: bool,
    /// The matrix test runs on its own, opened from the device card rather than inside the setup.
    test_only: bool,
    /// What the port's USB ids say.
    port_info: Option<BoardInfo>,
    now: u64,
    last_invalidate: u64,
}

impl App {
    pub fn init(host: &mut dyn Host) -> Result<App, String> {
        let settings = Settings::parse(&host.settings_json());
        let pack = load_pack(host, &settings.animation_pack)?;
        let port_info = describe(host, &settings);
        let prefs = Prefs::parse(host.content_read_own(PREFS_FILE).ok().as_deref());
        let saved = my_pack_animations(host);
        let draft = read_draft(host, &pack);
        let job = host.firmware_job().filter(|j| j.running);
        host.log(LogLevel::Info, &format!("using pack {} ({} animations)", pack.id, pack.animations.len()));
        Ok(App {
            settings,
            pack,
            signal: None,
            connected: false,
            in_car: false,
            player: Player::default(),
            link: Link::default(),
            blank: Animation::blank(),
            shown_name: "idle".into(),
            shown_source: Source::OutOfRace,
            shown_frame: [0; 64],
            shown_colors: vec![0],
            tab: "live".into(),
            prefs,
            setup: None,
            editor: None,
            save_note: None,
            pending_edit: None,
            play_pick: TESTS[0].to_string(),
            matrix_test: None,
            saved,
            fw_running: false,
            fw_job: None,
            fw_result: None,
            fw_title: None,
            fw_uploading: false,
            fw_landed_at: None,
            fw_silent: false,
            wrote_port: false,
            port_confirmed: false,
            test_only: false,
            port_info,
            now: 0,
            last_invalidate: 0,
        })
        .map(|mut app| {
            if !app.configured() {
                let step = app.prefs.setup_step.as_deref().and_then(Step::from_label).unwrap_or(Step::Board);
                app.open_setup(step);
            }
            if let Some(anim) = draft {
                app.editor = Some(Editor { anim, frame: 0, dirty: true });
                app.save_note = Some((StatusLevel::Idle, "These are the changes you had not saved.".into()));
            }
            if let Some(job) = job {
                app.resume_job(job);
            }
            app
        })
    }

    pub fn on_flag(&mut self, signal: Signal) {
        self.signal = Some(signal);
    }

    pub fn on_session(&mut self, connected: bool, in_car: bool) {
        self.connected = connected;
        self.in_car = in_car;
    }

    pub fn settings_changed(&mut self, host: &mut dyn Host) {
        let next = Settings::parse(&host.settings_json());
        if next.animation_pack != self.settings.animation_pack {
            match load_pack(host, &next.animation_pack) {
                Ok(p) => self.pack = p,
                Err(e) => host.log(LogLevel::Warn, &e),
            }
        }
        let port_changed = next.port != self.settings.port;
        self.settings = next;
        if port_changed {
            // A running job keeps the port it started on; `end_job` reopens the link.
            if !self.fw_running {
                self.link.reopen(host, self.now);
            }
            self.port_info = describe(host, &self.settings);
            self.fw_result = None;
            self.port_confirmed = false;
            self.wrote_port = false;
        }
        if let Some(i) = self.matrix_test {
            self.show_matrix_test(i);
        }
        if !self.configured() && self.setup.is_none() {
            self.open_setup(Step::Board);
        }
        host.invalidate_ui();
    }

    pub fn tick(&mut self, host: &mut dyn Host, now: u64) {
        self.now = now;
        let signal = if self.connected && self.in_car { self.signal.as_ref() } else { None };
        let outside = if self.settings.standby { STANDBY } else { OFF };
        let choice = self.player.choose(now, signal, &self.pack, self.settings.green_hold_s, &self.blank, outside);
        let off = matches!(choice.source, Source::Rule(_)) && self.prefs.flags_off.contains(&choice.name);
        let anim = if off { &self.blank } else { choice.anim };
        let frame = *anim.frame_at(now.saturating_sub(choice.started));
        let colors = anim.colors.clone();
        let (name, source) = if off { (format!("{} (turned off)", choice.name), choice.source) } else { (choice.name, choice.source) };
        let correct = self.settings.color_correction && self.settings.matrix != "max7219";
        let sent: Vec<u32> = if correct { colors.iter().map(|&c| led_color(c)).collect() } else { colors.clone() };
        if self.fw_landed_at.is_some() && self.board_ready() {
            self.fw_landed_at = None;
        }
        if self.fw_landed_at.is_some_and(|t| now.saturating_sub(t) >= ANSWER_WAIT_MS) {
            self.fw_landed_at = None;
            self.fw_silent = true;
            host.invalidate_ui();
        }
        if self.fw_running {
            self.poll_job(host, now);
        } else {
            self.link.tick(host, now, &Want { colors: &sent, frame: &frame, brightness: self.settings.brightness, config: self.settings.config() });
        }
        // The test screen timed out, or another test took its place.
        if self.matrix_test.is_some() && !(source == Source::Test && name.starts_with(matrix_test::NAME_PREFIX)) {
            self.matrix_test = None;
            host.invalidate_ui();
        }
        let changed = frame != self.shown_frame || name != self.shown_name;
        self.shown_frame = frame;
        self.shown_colors = colors;
        self.shown_name = name;
        self.shown_source = source;
        let live_on_screen = self.setup.is_none() && self.tab == "live";
        if changed && live_on_screen && now.saturating_sub(self.last_invalidate) >= UI_EVERY_MS {
            self.last_invalidate = now;
            host.invalidate_ui();
        }
    }

    pub fn on_action(&mut self, id: &str) {
        if id == "test" {
            self.play_all();
        }
    }

    pub fn shutdown(&mut self) {
        self.fw_running = false;
        self.link.close();
    }

    fn start_upload(&mut self, host: &mut dyn Host) {
        let board = Board::from_id(&self.settings.board);
        let invalid = setup::matrix_checks(&self.settings).iter().any(Check::blocks);
        if self.settings.port.is_none() || self.fw_running || self.board_mismatch(board) || invalid || !self.port_trusted() {
            return;
        }
        self.link.suspend();
        match host.firmware_upload(PORT_SETTING, board.id(), board.image()) {
            Ok(()) => {
                self.start_job(host, true);
                let v = firmware::version_text(firmware::VERSION);
                host.log(LogLevel::Info, &format!("PitCrew is uploading PitFlag {v} to the {}", board.chip().label()));
                self.link.set_status(host, StatusLevel::Idle, "Uploading the firmware…".into());
            }
            Err(e) => self.end_job(host, StatusLevel::Error, Some("The upload did not start"), e),
        }
    }

    /// The port answers as PitFlag, or the driver confirmed it is their matrix board.
    fn port_trusted(&self) -> bool {
        self.link.hello().is_some() || self.port_confirmed || self.wrote_port
    }

    fn start_probe(&mut self, host: &mut dyn Host) {
        if self.settings.port.is_none() || self.fw_running {
            return;
        }
        self.link.suspend();
        match host.firmware_probe(PORT_SETTING) {
            Ok(()) => {
                self.start_job(host, false);
                self.link.set_status(host, StatusLevel::Idle, "Looking at the board…".into());
            }
            Err(e) => self.end_job(host, StatusLevel::Error, Some("The board could not be checked"), e),
        }
    }

    fn start_job(&mut self, host: &mut dyn Host, upload: bool) {
        self.fw_running = true;
        self.fw_uploading = upload;
        self.fw_job = None;
        self.fw_result = None;
        self.fw_title = None;
        self.fw_landed_at = None;
        self.fw_silent = false;
        host.invalidate_ui();
    }

    /// Takes up a job PitCrew still runs from before the plugin restarted, on the step it belongs to.
    fn resume_job(&mut self, job: FirmwareJob) {
        self.link.suspend();
        self.open_setup(if job.upload { Step::Upload } else { Step::Board });
        self.fw_running = true;
        self.fw_uploading = job.upload;
        self.fw_job = Some(job);
    }

    fn end_job(&mut self, host: &mut dyn Host, level: StatusLevel, title: Option<&str>, text: String) {
        self.fw_running = false;
        let line = title.map_or(text.clone(), |t| format!("{t}: {text}"));
        host.log(if level == StatusLevel::Error { LogLevel::Error } else { LogLevel::Info }, &line);
        self.fw_title = title.map(String::from);
        self.fw_result = Some((level, text));
        self.link.reopen(host, self.now);
        host.invalidate_ui();
    }

    /// Reads PitCrew's job. Cheap: the host keeps the latest state ready.
    fn poll_job(&mut self, host: &mut dyn Host, now: u64) {
        let Some(job) = host.firmware_job() else {
            self.end_job(host, StatusLevel::Error, None, "PitCrew lost track of the firmware job. Try again.".into());
            return;
        };
        let changed = self.fw_job.as_ref() != Some(&job);
        self.fw_job = Some(job.clone());
        if job.running {
            if changed && now.saturating_sub(self.last_invalidate) >= UI_EVERY_MS {
                self.last_invalidate = now;
                host.invalidate_ui();
            }
            return;
        }
        let (level, title, text) = match (&job.error, job.upload) {
            (Some(e), true) => (StatusLevel::Error, Some("The upload did not finish"), e.clone()),
            (Some(e), false) => (StatusLevel::Error, Some("The board could not be checked"), e.clone()),
            (None, true) => {
                self.fw_landed_at = Some(now);
                self.wrote_port = true;
                let v = firmware::version_text(firmware::VERSION);
                (StatusLevel::Ok, None, format!("PitFlag {v} is on the board. It restarts and connects again."))
            }
            (None, false) => {
                let (level, text) = self.probe_result(job.board.as_ref());
                if level == StatusLevel::Ok {
                    self.port_confirmed = true;
                }
                (level, None, text)
            }
        };
        self.end_job(host, level, title, text);
    }

    fn probe_result(&self, found: Option<&BoardInfo>) -> (StatusLevel, String) {
        let picked = Board::from_id(&self.settings.board);
        match found.and_then(|b| b.chip.as_deref()).map(|c| (c, firmware::board_for_chip(c))) {
            Some((_, Some(b))) if b == picked => {
                (StatusLevel::Ok, format!("This board is an {}, the board picked in Board.", b.chip().label()))
            }
            Some((_, Some(b))) => (
                StatusLevel::Warning,
                format!("This board is an {}. Pick it in Board before you upload.", b.chip().label()),
            ),
            Some((chip, None)) => (StatusLevel::Warning, format!("This board is an {chip}, which PitFlag does not run on.")),
            None => (
                StatusLevel::Warning,
                format!(
                    "PitCrew could not tell which chip this is. {}",
                    found.map(|b| b.detail.as_str()).unwrap_or_default()
                ),
            ),
        }
    }

    /// The connected board says it is another chip than the one picked.
    fn board_mismatch(&self, board: Board) -> bool {
        self.link.hello().is_some_and(|h| h.chip != board.chip())
    }

    fn play_all(&mut self) {
        self.player.play_tests(TESTS.iter().map(|s| s.to_string()).collect(), self.now, 1_500);
    }

    /// The device on the port is the one a finished setup was for.
    pub fn configured(&self) -> bool {
        self.prefs.device.as_ref().is_some_and(|d| self.settings.port.as_deref() == Some(d.port.as_str()))
    }

    fn save_prefs(&mut self, host: &mut dyn Host) {
        if let Err(e) = host.content_write_own(PREFS_FILE, &self.prefs.to_bytes()) {
            host.set_status(StatusLevel::Warning, &format!("Could not save the Flags preferences: {e}"));
        }
    }

    /// The board on the port runs our firmware and is the chip picked in Board.
    fn board_ready(&self) -> bool {
        let board = Board::from_id(&self.settings.board);
        !self.fw_running && self.link.hello().is_some_and(|h| h.chip == board.chip() && h.firmware >= firmware::VERSION)
    }

    /// The matrix test steps for the picked matrix. A one-color matrix skips the color screens.
    fn test_steps(&self) -> Vec<usize> {
        matrix_test::steps(setup::is_max7219(&self.settings))
    }

    fn open_setup(&mut self, step: Step) {
        self.stop_matrix_test();
        self.setup = Some(step);
        self.test_only = false;
        self.fw_result = None;
        self.fw_title = None;
        self.fw_landed_at = None;
        self.fw_silent = false;
        if step == Step::Test {
            self.start_test();
        }
    }

    fn start_test(&mut self) {
        if let Some(&first) = self.test_steps().first() {
            self.show_matrix_test(first);
        }
    }

    fn finish_setup(&mut self, host: &mut dyn Host) {
        let Some(port) = self.settings.port.clone() else { return };
        self.prefs.device = Some(Device { port, board: self.settings.board.clone() });
        self.save_prefs(host);
        self.stop_matrix_test();
        self.setup = None;
        self.tab = "live".into();
        self.player.play_tests(vec!["green-flash".into(), "checkered".into()], self.now, 1_500);
    }

    /// Leaves the matrix test opened from the device card.
    fn close_test(&mut self) {
        self.stop_matrix_test();
        self.setup = None;
        self.test_only = false;
    }

    /// Keeps the setup step in prefs, so a restart reopens it.
    fn remember_step(&mut self, host: &mut dyn Host) {
        let step = self.setup.filter(|_| !self.test_only && !self.configured()).map(|s| s.label().to_string());
        if step != self.prefs.setup_step {
            self.prefs.setup_step = step;
            self.save_prefs(host);
        }
    }

    /// Keeps the editor's unsaved animation, so a restart brings it back.
    fn remember_draft(&mut self, host: &mut dyn Host) {
        let body = match &self.editor {
            Some(e) if e.dirty => serde_json::to_vec(&e.anim.to_file()).expect("serialisable"),
            _ => Vec::new(),
        };
        if let Err(e) = host.content_write_own(DRAFT_FILE, &body) {
            host.log(LogLevel::Warn, &format!("could not keep the editor draft: {e}"));
        }
    }

    /// Whether the driver may leave `step` forward.
    fn step_done(&self, step: Step) -> bool {
        match step {
            Step::Board => self.settings.port.is_some() && !self.board_mismatch(Board::from_id(&self.settings.board)),
            Step::Matrix => !setup::matrix_checks(&self.settings).iter().any(Check::blocks),
            Step::Upload => self.board_ready(),
            Step::Test => self.board_ready() && self.matrix_test == self.test_steps().last().copied(),
        }
    }

    pub fn render(&self) -> Vec<Node> {
        let mut t = Tree::default();
        let page = t.add("page", None, Kind::Page { title: "Flags".into(), subtitle: Some("Race flags on your LED matrix".into()) });
        match self.setup {
            Some(step) => self.render_setup(&mut t, &page, step),
            None => {
                let attention = self.device_needs_attention();
                if attention {
                    self.render_device_bar(&mut t, &page);
                }
                let tabs = t.add("tabs", Some(&page), Kind::Tabs { selected: self.tab.clone() });
                for (id, label) in [("live", "Live"), ("animations", "Animations")] {
                    t.add(id, Some(&tabs), Kind::Tab(label.into()));
                }
                match self.tab.as_str() {
                    "animations" => self.render_animations(&mut t),
                    _ => {
                        self.render_live(&mut t);
                        if !attention {
                            self.render_device_bar(&mut t, "live");
                        }
                    }
                }
            }
        }
        t.nodes
    }

    /// The board is missing, unhappy or on old firmware: its card goes above the tabs.
    fn device_needs_attention(&self) -> bool {
        !self.link.is_open()
            || self.link.status().is_some_and(|(l, _)| *l != StatusLevel::Ok)
            || self.link.hello().is_some_and(|h| h.firmware < firmware::VERSION)
    }

    fn render_device_bar(&self, t: &mut Tree, parent: &str) {
        let card = t.add("device-card", Some(parent), Kind::Card { title: Some("Device".into()), description: None });
        // Only a status that needs attention; the header pill shows every status.
        if let Some((level, text)) = self.link.status().filter(|(l, _)| *l != StatusLevel::Ok).cloned() {
            t.add("conn", Some(&card), Kind::Badge { text, level });
        }
        let port = self.settings.port.as_deref().unwrap_or("no port");
        let chip = Board::from_id(&self.settings.board).chip().label();
        let about = match self.link.hello() {
            Some(h) => format!("{chip} on {port} · PitFlag {}", firmware::version_text(h.firmware)),
            None => format!("{chip} on {port}"),
        };
        t.text("device-about", &card, about, Tone::Muted, true);
        let old = self.link.hello().is_some_and(|h| h.firmware < firmware::VERSION);
        if !self.link.is_open() && !old {
            t.text("device-hint", &card, "Check the USB cable. If you changed the board, set it up again.", Tone::Muted, false);
        }
        let row = t.add("device-actions", Some(&card), Kind::Grid { columns: if old { 3 } else { 2 }, gap: Gap::Sm });
        if old {
            let ours = firmware::version_text(firmware::VERSION);
            t.add("device:update", Some(&row), Kind::Button { label: format!("Update to PitFlag {ours}"), variant: Variant::Primary, icon: Some("zap".into()), disabled: false });
        }
        t.add("device:test", Some(&row), Kind::Button { label: "Test the matrix".into(), variant: Variant::Secondary, icon: Some("lightbulb".into()), disabled: false });
        t.add("device:setup", Some(&row), Kind::Button { label: "Set up again".into(), variant: Variant::Secondary, icon: Some("usb".into()), disabled: false });
        t.add("device-settings", Some(&card), Kind::SettingsForm(vec!["brightness".into()]));
    }

    fn render_setup(&self, t: &mut Tree, page: &str, step: Step) {
        if self.test_only {
            let card = t.add("setup-card", Some(page), Kind::Card {
                title: Some("Test the matrix".into()),
                description: Some("Compare the matrix with each screen. Change a setting while the screen shows and the board applies it at once.".into()),
            });
            self.render_step_test(t, &card);
            t.add("setup:done", Some(page), Kind::Button { label: "Done".into(), variant: Variant::Primary, icon: None, disabled: false });
            return;
        }
        let n = Step::ALL.len();
        let labels = Step::ALL.iter().map(|s| s.label()).collect::<Vec<_>>().join(" · ");
        t.add("setup-steps", Some(page), Kind::Progress {
            value: (step.index() + 1) as f32 / n as f32,
            label: Some(format!("Step {} of {n}: {labels}", step.index() + 1)),
        });
        let description = match step {
            Step::Board => "Plug the board into a USB port, then pick its port and the board type.",
            Step::Matrix => "Which matrix is wired to the board, on which pins, and how much current it may draw. Colors and rotation come in Test, where you can see them.",
            Step::Upload => "PitCrew checks your settings, then writes the PitFlag firmware to the board.",
            Step::Test => "Compare the matrix with each screen. Change a setting while the screen shows and the board applies it at once.",
        };
        let card = t.add("setup-card", Some(page), Kind::Card { title: Some(step.label().into()), description: Some(description.into()) });
        match step {
            Step::Board => self.render_step_board(t, &card),
            Step::Matrix => self.render_step_matrix(t, &card),
            Step::Upload => self.render_step_upload(t, &card),
            Step::Test => self.render_step_test(t, &card),
        }
        let nav = t.add("setup-nav", Some(page), Kind::Grid { columns: if self.configured() { 3 } else { 2 }, gap: Gap::Sm });
        t.add("setup:back", Some(&nav), Kind::Button { label: "Back".into(), variant: Variant::Secondary, icon: None, disabled: step == Step::Board || self.fw_running });
        if self.configured() {
            t.add("setup:cancel", Some(&nav), Kind::Button { label: "Cancel".into(), variant: Variant::Secondary, icon: None, disabled: self.fw_running });
        }
        if step == Step::Test {
            t.add("setup:finish", Some(&nav), Kind::Button { label: "Finish setup".into(), variant: Variant::Primary, icon: None, disabled: !self.step_done(step) });
        } else {
            t.add("setup:next", Some(&nav), Kind::Button { label: format!("Next: {}", step.next().label()), variant: Variant::Primary, icon: None, disabled: !self.step_done(step) || self.fw_running });
        }
    }

    fn render_checks(t: &mut Tree, parent: &str, prefix: &str, checks: &[Check]) {
        for (i, c) in checks.iter().enumerate() {
            t.add(format!("{prefix}-{i}"), Some(parent), Kind::Badge { text: c.text.clone(), level: c.level });
        }
    }

    fn render_step_board(&self, t: &mut Tree, card: &str) {
        if self.fw_running {
            let port = self.settings.port.as_deref().unwrap_or("");
            t.text("board-locked", card, format!("Port and board stay as they are while PitCrew works on {port}."), Tone::Muted, false);
        } else {
            t.add("board-settings", Some(card), Kind::SettingsForm(vec!["port".into(), "board".into()]));
        }
        if let Some(info) = &self.port_info {
            t.text("board-usb", card, format!("The port says: {}", info.usb_name), Tone::Muted, true);
        }
        if let Some(h) = self.link.hello() {
            let picked = Board::from_id(&self.settings.board);
            if h.chip != picked.chip() {
                t.add("board-mismatch", Some(card), Kind::Alert { title: None, text: format!("The board says it is an {}. Pick that board.", h.chip.label()), level: StatusLevel::Warning });
            } else {
                t.add("board-found", Some(card), Kind::Badge { text: format!("PitFlag {} answers on this port.", firmware::version_text(h.firmware)), level: StatusLevel::Ok });
            }
        }
        if self.fw_running {
            self.render_job(t, card);
            t.add("firmware:probe", Some(card), Kind::Button { label: "Detecting…".into(), variant: Variant::Secondary, icon: Some("cpu".into()), disabled: true });
            return;
        }
        if self.settings.port.is_some() && self.link.hello().is_none() {
            t.text("board-detect-hint", card, "Not sure which board this is? PitCrew can ask it. This restarts the board.", Tone::Muted, false);
            t.add("firmware:probe", Some(card), Kind::Button { label: "Detect board".into(), variant: Variant::Secondary, icon: Some("cpu".into()), disabled: false });
        }
        self.render_result(t, card);
    }

    fn render_result(&self, t: &mut Tree, card: &str) {
        if let Some((level, text)) = &self.fw_result {
            t.add("firmware-result", Some(card), Kind::Alert { title: self.fw_title.clone(), text: text.clone(), level: *level });
        }
    }

    fn render_step_matrix(&self, t: &mut Tree, card: &str) {
        t.add("matrix-settings", Some(card), Kind::SettingsForm(setup::matrix_keys(&self.settings)));
        t.text("matrix-defaults", card, setup::default_pins_text(&self.settings), Tone::Muted, false);
        Self::render_checks(t, card, "matrix-check", &setup::matrix_checks(&self.settings));
    }

    fn render_step_upload(&self, t: &mut Tree, card: &str) {
        let board = Board::from_id(&self.settings.board);
        let ours = firmware::version_text(firmware::VERSION);
        let mut checks = vec![match (&self.settings.port, &self.port_info) {
            (None, _) => Check { level: StatusLevel::Error, text: "No port. Go back to step 1 and pick one.".into() },
            (Some(p), Some(i)) => Check { level: StatusLevel::Ok, text: format!("Port {p} ({}).", i.usb_name) },
            (Some(p), None) => Check { level: StatusLevel::Ok, text: format!("Port {p}.") },
        }];
        checks.push(match self.link.hello() {
            Some(h) if h.chip != board.chip() => Check { level: StatusLevel::Error, text: format!("The board says it is an {}, not an {}. Go back to step 1.", h.chip.label(), board.chip().label()) },
            _ => Check { level: StatusLevel::Ok, text: format!("Board: {}.", board.chip().label()) },
        });
        checks.extend(setup::matrix_checks(&self.settings));
        match self.link.hello() {
            Some(h) if h.firmware >= firmware::VERSION => checks.push(Check { level: StatusLevel::Ok, text: format!("PitFlag {} is on the board already. You can upload again or go on.", firmware::version_text(h.firmware)) }),
            Some(h) => checks.push(Check { level: StatusLevel::Warning, text: format!("The board has PitFlag {}. Upload {ours} to go on.", firmware::version_text(h.firmware)) }),
            None if self.fw_running || !self.port_confirmed => {}
            None => checks.push(Check { level: StatusLevel::Idle, text: "PitFlag does not answer on this port yet. Upload it to go on.".into() }),
        }
        let blocked = checks.iter().any(Check::blocks);
        let (fine, attention): (Vec<Check>, Vec<Check>) = checks.into_iter().partition(|c| c.level == StatusLevel::Ok);
        if !fine.is_empty() && attention.iter().all(|c| !c.blocks()) {
            let port = self.settings.port.as_deref().unwrap_or("");
            t.add("upload-ready", Some(card), Kind::Badge { text: format!("Settings checked: {} on {port}.", board.chip().label()), level: StatusLevel::Ok });
        }
        Self::render_checks(t, card, "upload-check", &attention);
        if self.fw_running {
            self.render_job(t, card);
            let label = if self.fw_uploading { "Uploading…" } else { "Detecting…" };
            t.add("firmware:upload", Some(card), Kind::Button { label: label.into(), variant: Variant::Primary, icon: Some("zap".into()), disabled: true });
            return;
        }
        let unknown = self.settings.port.is_some() && self.link.hello().is_none() && !self.wrote_port;
        if unknown {
            let port = self.settings.port.as_deref().unwrap_or("");
            let usb = self.port_info.as_ref().map(|i| format!(" ({})", i.usb_name)).unwrap_or_default();
            t.add("upload-unknown", Some(card), Kind::Alert {
                title: Some("Is this your matrix board?".into()),
                text: format!("Nothing on {port}{usb} answers as PitFlag yet. Uploading replaces whatever program is on it. If this port is a button box, a wheel or another sim device, go back and pick another port."),
                level: StatusLevel::Warning,
            });
            t.add("upload:confirm", Some(card), Kind::Switch { label: format!("{port} is my matrix board"), checked: self.port_confirmed, disabled: false });
            if !self.port_confirmed {
                t.add("firmware:probe", Some(card), Kind::Button { label: "Detect board".into(), variant: Variant::Secondary, icon: Some("cpu".into()), disabled: false });
            }
        }
        let failed = matches!(&self.fw_result, Some((StatusLevel::Error, _)));
        let landed = matches!(&self.fw_result, Some((StatusLevel::Ok, _))) && self.board_ready();
        if landed {
            t.add("firmware-result", Some(card), Kind::Alert {
                title: Some(format!("PitFlag {ours} is running")),
                text: "The board answered with the new firmware. Next, test the matrix.".into(),
                level: StatusLevel::Ok,
            });
        } else if !self.fw_silent {
            self.render_result(t, card);
        }
        if failed {
            t.text(
                "firmware-retry-hint",
                card,
                "Nothing on the board is lost for good: its bootloader is still there, so you can upload again. Check the cable and the port first.",
                Tone::Normal,
                false,
            );
            t.text("firmware-hint", card, board.hint(), Tone::Muted, false);
        } else if matches!(&self.fw_result, Some((StatusLevel::Ok, _))) && !self.board_ready() && self.fw_silent {
            t.add("firmware-silent", Some(card), Kind::Alert {
                title: Some("The board has not answered".into()),
                text: format!("The upload finished {} seconds ago. Unplug the board, plug it back in and wait a few seconds. If PitFlag still does not answer, upload again.", ANSWER_WAIT_MS / 1000),
                level: StatusLevel::Warning,
            });
        } else if matches!(&self.fw_result, Some((StatusLevel::Ok, _))) && !self.board_ready() {
            t.text("firmware-wait", card, "Waiting for the board to restart and answer…", Tone::Muted, false);
        } else if !landed {
            t.text("firmware-hint", card, board.hint(), Tone::Normal, false);
        }
        t.add("firmware:upload", Some(card), Kind::Button {
            label: if failed { "Try the upload again".into() } else { format!("Upload PitFlag {ours}") },
            variant: if self.board_ready() { Variant::Secondary } else { Variant::Primary },
            icon: Some("zap".into()),
            disabled: blocked || !self.port_trusted(),
        });
    }

    fn render_job(&self, t: &mut Tree, card: &str) {
        let (value, label) = match &self.fw_job {
            Some(j) => (j.progress, j.text.clone()),
            None => (0.0, "Starting…".into()),
        };
        let text = if self.fw_uploading {
            let secs = Board::from_id(&self.settings.board).upload_seconds();
            format!("This takes about {secs} seconds. If the cable comes out, upload again: the board keeps its bootloader.")
        } else {
            "PitCrew restarts the board to ask what it is. This takes a few seconds.".to_string()
        };
        t.add("firmware-keep", Some(card), Kind::Alert { title: Some("Keep the board plugged in".into()), text, level: StatusLevel::Warning });
        t.add("firmware-progress", Some(card), Kind::Progress { value, label: Some(label) });
    }

    fn render_step_test(&self, t: &mut Tree, card: &str) {
        if !self.board_ready() {
            t.add("test-offline", Some(card), Kind::Alert { title: None, text: "The board is not connected, so the screens show only here. Check the cable, or go back to Upload.".into(), level: StatusLevel::Warning });
        }
        let steps = self.test_steps();
        let Some(i) = self.matrix_test else {
            t.text("test-stopped", card, "The test paused after two minutes. Start it again when you are ready.", Tone::Muted, false);
            t.button("matrix-test:start", card, "Start the test again", Variant::Primary);
            return;
        };
        let pos = steps.iter().position(|&s| s == i).unwrap_or(0);
        let step = &matrix_test::STEPS[i];
        let shot = matrix_test::screen(i);
        let grid = t.add("test-grid", Some(card), Kind::Grid { columns: 2, gap: Gap::Md });
        let left = t.add("test-screen-col", Some(&grid), Kind::Stack(Gap::Sm));
        t.add("matrix-test-screen", Some(&left), Kind::PixelGrid { width: 8, height: 8, palette: shot.colors, pixels: shot.frames[0].to_vec(), editable: false });
        t.text("matrix-test-step", &left, format!("Screen {} of {}", pos + 1, steps.len()), Tone::Muted, true);
        let right = t.add("test-fix-col", Some(&grid), Kind::Stack(Gap::Sm));
        t.text("matrix-test-expect", &right, step.expect, Tone::Normal, false);
        t.text("matrix-test-fix", &right, step.fix, Tone::Muted, false);
        let keys = matrix_test::fix_keys(step.id, setup::is_max7219(&self.settings));
        if !keys.is_empty() {
            t.add("matrix-test-settings", Some(&right), Kind::SettingsForm(keys.iter().map(|k| k.to_string()).collect()));
        }
        let row = t.add("matrix-test-actions", Some(&right), Kind::Grid { columns: 2, gap: Gap::Sm });
        t.add("matrix-test:prev", Some(&row), Kind::Button { label: "Previous screen".into(), variant: Variant::Secondary, icon: None, disabled: pos == 0 });
        if pos + 1 == steps.len() {
            let text = if self.test_only { "That is the last screen. Press Done when the matrix looks right." } else { "That is the last screen. Finish the setup when the matrix looks right." };
            t.text("matrix-test-last", &right, text, Tone::Normal, false);
        } else {
            t.add("matrix-test:next", Some(&row), Kind::Button { label: "Looks right".into(), variant: Variant::Primary, icon: None, disabled: false });
        }
    }

    fn render_live(&self, t: &mut Tree) {
        let card = t.add("now-card", Some("live"), Kind::Card { title: Some("Now".into()), description: None });
        let row = t.add("now-row", Some(&card), Kind::Grid { columns: 2, gap: Gap::Lg });
        t.add("matrix", Some(&row), Kind::PixelGrid {
            width: 8,
            height: 8,
            palette: self.shown_colors.clone(),
            pixels: self.shown_frame.to_vec(),
            editable: false,
        });
        let signal = match (&self.signal, self.connected, self.in_car) {
            (_, false, _) => "No sim connected".to_string(),
            (_, true, false) => "Not in the car".to_string(),
            (Some(s), true, true) => capitalized(&s.describe()),
            (None, true, true) => "Waiting for the first flag".to_string(),
        };
        let lines = t.add("now-lines", Some(&row), Kind::Stack(Gap::Sm));
        t.text("now", &lines, format!("iRacing shows: {signal}"), Tone::Normal, false);
        let why = match self.shown_source {
            Source::Rule(_) => String::new(),
            Source::Test => ", a test".into(),
            Source::OutOfRace => ", outside a race".into(),
            Source::NoRule => ", no animation matches this flag".into(),
        };
        let showing = if self.link.is_open() {
            format!("The matrix shows: {}{why}", anim_label(&self.shown_name))
        } else {
            format!("The matrix would show: {}{why}. The board isn't connected.", anim_label(&self.shown_name))
        };
        t.text("now-showing", &lines, showing, Tone::Muted, false);

        let card = t.add("test-card", Some("live"), Kind::Card {
            title: Some("Play a flag".into()),
            description: Some("Plays on the matrix even with no sim running, including flags you turned off.".into()),
        });
        let row = t.add("tests", Some(&card), Kind::Grid { columns: 3, gap: Gap::Sm });
        t.add("play-pick", Some(&row), Kind::Select {
            label: "Flag".into(),
            options: TESTS.iter().map(|n| (n.to_string(), flag_label(n).to_string())).collect(),
            value: self.play_pick.clone(),
        });
        t.add("play-one", Some(&row), Kind::Button { label: "Play".into(), variant: Variant::Primary, icon: Some("flag".into()), disabled: false });
        t.button("play-all", &row, "Play every flag", Variant::Secondary);
    }

    fn render_animations(&self, t: &mut Tree) {
        let grid = t.add("anim-grid", Some("animations"), Kind::Grid { columns: 2, gap: Gap::Md });
        let card = t.add("flags-card", Some(&grid), Kind::Card {
            title: Some("Flags on the matrix".into()),
            description: Some("Turn a flag off and the matrix stays dark while iRacing shows it.".into()),
        });
        let groups = t.add("flag-groups", Some(&card), Kind::Grid { columns: 2, gap: Gap::Lg });
        for pair in FLAG_GROUPS.chunks(2) {
            let column = t.add(format!("flag-col:{}", pair[0].0), Some(&groups), Kind::Stack(Gap::Lg));
            for (id, heading, names) in pair {
                let group = t.add(format!("flag-group:{id}"), Some(&column), Kind::Stack(Gap::Sm));
                t.text(&format!("flag-heading:{id}"), &group, *heading, Tone::Muted, false);
                for &name in *names {
                    t.add(format!("flag:{name}"), Some(&group), Kind::Switch { label: flag_label(name).into(), checked: !self.prefs.flags_off.contains(name), disabled: false });
                }
            }
        }
        let card = t.add("behavior-card", Some(&grid), Kind::Card { title: Some("Playback".into()), description: None });
        t.add("behavior", Some(&card), Kind::SettingsForm(PLAYBACK_SETTINGS.iter().map(|s| s.to_string()).collect()));

        let card = t.add("anims-card", Some(&grid), Kind::Card {
            title: Some(self.pack.name.clone()),
            description: Some("Pick an animation to edit it.".into()),
        });
        t.add("anim-table", Some(&card), Kind::Table {
            columns: vec!["Animation".into(), "Frames".into()],
            rows: {
                let mut rows: Vec<Vec<String>> =
                    self.pack.animations.values().map(|a| vec![anim_label(&a.id), a.frames.len().to_string()]).collect();
                rows.sort();
                rows
            },
            selectable: true,
        });
        let card = t.add("editor-card", Some(&grid), Kind::Card {
            title: Some(self.editor.as_ref().map_or("Editor".into(), |e| {
                if e.dirty { format!("Editing {} (not saved)", anim_label(&e.anim.id)) } else { format!("Editing {}", anim_label(&e.anim.id)) }
            })),
            description: None,
        });
        if let (Some(e), Some(next)) = (&self.editor, &self.pending_edit) {
            t.add("editor-pending", Some(&card), Kind::Alert {
                title: None,
                text: format!("{} has changes that are not saved. Save them before you open {}?", anim_label(&e.anim.id), anim_label(next)),
                level: StatusLevel::Warning,
            });
            let row = t.add("editor-pending-actions", Some(&card), Kind::Grid { columns: 3, gap: Gap::Sm });
            t.button("editor:save-open", &row, "Save and open", Variant::Primary);
            t.button("editor:discard", &row, "Discard changes", Variant::Destructive);
            t.button("editor:keep", &row, "Keep editing", Variant::Secondary);
        }
        let Some(e) = &self.editor else {
            t.text("editor-empty", &card, "No animation picked.", Tone::Muted, false);
            return;
        };
        let body = t.add("editor-body", Some(&card), Kind::Stack(Gap::Lg));
        t.add("editor", Some(&body), Kind::PixelGrid {
            width: 8,
            height: 8,
            palette: e.anim.colors.clone(),
            pixels: e.anim.frames[e.frame].to_vec(),
            editable: true,
        });
        let count = e.anim.frames.len();
        let button = |t: &mut Tree, id: &str, parent: &str, label: &str, variant: Variant, disabled: bool| {
            t.add(id, Some(parent), Kind::Button { label: label.into(), variant, icon: None, disabled });
        };
        let group = t.add("editor-frame-group", Some(&body), Kind::Stack(Gap::Sm));
        let frames = t.add("editor-frames", Some(&group), Kind::Grid { columns: 3, gap: Gap::Sm });
        button(t, "frame-prev", &frames, "Previous", Variant::Secondary, count < 2);
        t.text("frame-label", &frames, format!("Frame {} of {count}", e.frame + 1), Tone::Normal, true);
        button(t, "frame-next", &frames, "Next", Variant::Secondary, count < 2);
        let edits = t.add("editor-frame-edits", Some(&group), Kind::Grid { columns: 2, gap: Gap::Sm });
        button(t, "frame-dup", &edits, "Duplicate frame", Variant::Secondary, count >= crate::pack::MAX_FRAMES);
        button(t, "frame-del", &edits, "Delete frame", Variant::Secondary, count < 2);
        t.add("editor-fps", Some(&group), Kind::Slider {
            label: "Speed (frames per second)".into(),
            min: 1.0,
            max: crate::pack::MAX_FPS as f64,
            step: 1.0,
            value: e.anim.fps as f64,
        });
        let actions = t.add("editor-actions", Some(&body), Kind::Grid { columns: if e.dirty { 3 } else { 2 }, gap: Gap::Sm });
        button(t, "editor-play", &actions, "Play on the matrix", Variant::Secondary, !self.link.is_open());
        button(t, "editor-save", &actions, "Save to My pack", Variant::Primary, false);
        if e.dirty {
            button(t, "editor-revert", &actions, "Discard changes", Variant::Destructive, false);
        }
        if let Some((level, text)) = &self.save_note {
            t.add("editor-saved", Some(&card), Kind::Alert { title: None, text: text.clone(), level: *level });
            if *level == StatusLevel::Ok && self.settings.animation_pack != MY_PACK_ID {
                t.add("editor-use-pack", Some(&card), Kind::SettingsForm(vec!["animationPack".into()]));
            }
        }
    }

    fn show_matrix_test(&mut self, i: usize) {
        let shot = matrix_test::screen(i);
        let name = shot.id.clone();
        self.matrix_test = Some(i);
        self.player.play_custom(shot, name, self.now, MATRIX_TEST_MS);
    }

    fn stop_matrix_test(&mut self) {
        if self.matrix_test.take().is_some() {
            self.player.stop();
        }
    }

    /// Moves the matrix test `by` screens along the steps for this matrix.
    fn step_matrix_test(&mut self, forward: bool) {
        let steps = self.test_steps();
        let Some(pos) = self.matrix_test.and_then(|i| steps.iter().position(|&s| s == i)) else { return };
        let next = if forward { (pos + 1).min(steps.len() - 1) } else { pos.saturating_sub(1) };
        self.show_matrix_test(steps[next]);
    }

    pub fn on_ui_event(&mut self, host: &mut dyn Host, node: &str, event: Event) {
        match (node, event) {
            ("tabs", Event::Selected(tab)) => self.tab = tab,
            ("play-all", Event::Pressed) => self.play_all(),
            (id, Event::Pressed) if id.starts_with("test:") => {
                self.player.play_tests(vec![id["test:".len()..].to_string()], self.now, 2_000);
            }
            (id, Event::Toggled(on)) if id.starts_with("flag:") => {
                let name = id["flag:".len()..].to_string();
                if on {
                    self.prefs.flags_off.remove(&name);
                } else {
                    self.prefs.flags_off.insert(name);
                }
                self.save_prefs(host);
            }
            ("device:setup", Event::Pressed) => self.open_setup(Step::Board),
            ("device:update", Event::Pressed) => self.open_setup(Step::Upload),
            ("play-pick", Event::Selected(name)) => self.play_pick = name,
            ("play-one", Event::Pressed) => self.player.play_tests(vec![self.play_pick.clone()], self.now, 2_000),
            ("device:test", Event::Pressed) => {
                self.open_setup(Step::Test);
                self.test_only = true;
            }
            ("setup:done", Event::Pressed) => self.close_test(),
            ("upload:confirm", Event::Toggled(on)) => self.port_confirmed = on,
            ("setup:next", Event::Pressed) => {
                if let Some(step) = self.setup.filter(|s| self.step_done(*s) && !self.fw_running) {
                    self.open_setup(step.next());
                }
            }
            ("setup:back", Event::Pressed) => {
                if let Some(step) = self.setup.filter(|_| !self.fw_running) {
                    self.open_setup(step.prev());
                }
            }
            ("setup:finish", Event::Pressed) => {
                if self.setup == Some(Step::Test) && self.step_done(Step::Test) {
                    self.finish_setup(host);
                }
            }
            ("setup:cancel", Event::Pressed) => {
                if self.configured() && !self.fw_running {
                    self.close_test();
                }
            }
            ("anim-table", Event::Selected(row)) if self.anim_id(&row).is_some() => {
                let name = self.anim_id(&row).unwrap_or_default();
                if self.editor.as_ref().is_some_and(|e| e.dirty && e.anim.id != name) {
                    self.pending_edit = Some(name);
                } else {
                    self.open_editor(&name);
                }
            }
            ("editor:discard", Event::Pressed) => {
                if let Some(name) = self.pending_edit.take() {
                    self.open_editor(&name);
                }
            }
            ("editor:keep", Event::Pressed) => self.pending_edit = None,
            ("editor:save-open", Event::Pressed) => {
                self.save(host);
                if let Some(name) = self.pending_edit.take() {
                    if !matches!(self.save_note, Some((StatusLevel::Error, _))) {
                        self.open_editor(&name);
                    }
                }
            }
            ("editor", Event::Painted { x, y, color }) => self.with_editor(|e| {
                if x < 8 && y < 8 && (color as usize) < e.anim.colors.len() {
                    e.anim.frames[e.frame][y as usize * 8 + x as usize] = color;
                    e.dirty = true;
                }
            }),
            ("editor", Event::Filled(color)) => self.with_editor(|e| {
                if (color as usize) < e.anim.colors.len() {
                    e.anim.frames[e.frame] = [color; 64];
                    e.dirty = true;
                }
            }),
            ("editor", Event::Palette(colors)) => self.with_editor(|e| {
                let old = &e.anim.colors;
                if colors.len() >= old.len() && colors.len() <= crate::pack::MAX_COLORS {
                    let next: Vec<u32> = colors.iter().enumerate().map(|(i, &c)| if i == 0 { old[0] } else { c & 0xFF_FF_FF }).collect();
                    e.dirty |= next != *old;
                    e.anim.colors = next;
                }
            }),
            ("frame-prev", Event::Pressed) => self.with_editor(|e| e.frame = (e.frame + e.anim.frames.len() - 1) % e.anim.frames.len()),
            ("frame-next", Event::Pressed) => self.with_editor(|e| e.frame = (e.frame + 1) % e.anim.frames.len()),
            ("frame-dup", Event::Pressed) => self.with_editor(|e| {
                if e.anim.frames.len() < crate::pack::MAX_FRAMES {
                    let f = e.anim.frames[e.frame];
                    e.anim.frames.insert(e.frame + 1, f);
                    e.frame += 1;
                    e.dirty = true;
                }
            }),
            ("frame-del", Event::Pressed) => self.with_editor(|e| {
                if e.anim.frames.len() > 1 {
                    e.anim.frames.remove(e.frame);
                    e.frame = e.frame.min(e.anim.frames.len() - 1);
                    e.dirty = true;
                }
            }),
            ("editor-revert", Event::Pressed) => {
                if let Some(id) = self.editor.as_ref().map(|e| e.anim.id.clone()) {
                    self.open_editor(&id);
                }
            }
            ("editor-fps", Event::Number(v)) => self.with_editor(|e| {
                let fps = (v.round() as u32).clamp(1, crate::pack::MAX_FPS);
                e.dirty |= fps != e.anim.fps;
                e.anim.fps = fps;
            }),
            ("editor-play", Event::Pressed) => {
                if let (Some(e), true) = (&self.editor, self.link.is_open()) {
                    self.player.play_custom(e.anim.clone(), format!("{} (editing)", e.anim.id), self.now, 3_500);
                }
            }
            ("matrix-test:start", Event::Pressed) => self.start_test(),
            ("matrix-test:prev", Event::Pressed) => self.step_matrix_test(false),
            ("matrix-test:next", Event::Pressed) => self.step_matrix_test(true),
            ("editor-save", Event::Pressed) => self.save(host),
            ("firmware:upload", Event::Pressed) => self.start_upload(host),
            ("firmware:probe", Event::Pressed) => self.start_probe(host),
            _ => {}
        }
        self.remember_step(host);
        let editor_event = ["editor", "frame-", "anim-table"].iter().any(|p| node.starts_with(p));
        if editor_event {
            self.remember_draft(host);
        }
        host.invalidate_ui();
    }

    /// The animation a table row names: an id, or the label `anim_label` drew for one.
    fn anim_id(&self, row: &str) -> Option<String> {
        if self.pack.get(row).is_some() {
            return Some(row.to_string());
        }
        self.pack.animations.keys().find(|id| anim_label(id) == row).cloned()
    }

    fn open_editor(&mut self, name: &str) {
        if let Some(a) = self.pack.get(name) {
            self.editor = Some(Editor { anim: a.clone(), frame: 0, dirty: false });
            self.save_note = None;
        }
        self.pending_edit = None;
    }

    fn with_editor(&mut self, f: impl FnOnce(&mut Editor)) {
        if let Some(e) = &mut self.editor {
            let was_dirty = e.dirty;
            f(e);
            if e.dirty && !was_dirty {
                self.save_note = None;
            }
        }
    }

    fn save(&mut self, host: &mut dyn Host) {
        let Some(e) = &self.editor else { return };
        let id = e.anim.id.clone();
        if !is_file_id(&id) {
            self.save_note = Some((
                StatusLevel::Error,
                format!("Could not save {id}. Its name can only use letters, digits, - and _, so it can't be a file in My pack."),
            ));
            return;
        }
        let name = anim_label(&id);
        let body = serde_json::to_vec_pretty(&e.anim.to_file()).expect("serialisable");
        if let Err(err) = host.content_write_own(&format!("packs/my-pack/animations/{id}.json"), &body) {
            self.save_note = Some((StatusLevel::Error, format!("Could not save {name}. {err}")));
            return;
        }
        self.saved.insert(id.clone());
        let pack = PackFile {
            schema: None,
            id: MY_PACK_ID.into(),
            name: "My pack".into(),
            content_type: CONTENT_TYPE.into(),
            version: "0.0.0".into(),
            animations: self.saved.iter().cloned().collect(),
            has_mapping: false,
        };
        let body = serde_json::to_vec_pretty(&pack).expect("serialisable");
        if let Err(err) = host.content_write_own("packs/my-pack/pack.json", &body) {
            self.save_note = Some((StatusLevel::Error, format!("Could not save My pack. {err}")));
            return;
        }
        host.log(LogLevel::Info, &format!("saved {id} to My pack"));
        self.with_editor(|e| e.dirty = false);
        if self.settings.animation_pack == MY_PACK_ID {
            match load_pack(host, MY_PACK_ID) {
                Ok(p) => self.pack = p,
                Err(e) => host.log(LogLevel::Warn, &e),
            }
            self.save_note = Some((StatusLevel::Ok, format!("Saved {name}. The matrix uses it now.")));
        } else {
            self.save_note = Some((StatusLevel::Ok, format!("Saved {name} to My pack. Pick My pack below to show it on the matrix.")));
        }
    }
}

/// An animation id that is safe as `animations/<id>.json`: 1 to 64 ASCII letters, digits, `-` or `_`.
/// "yellow · waving" → "Yellow · waving".
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// The editor's unsaved animation from the last run, when its animation is still in the pack.
fn read_draft(host: &mut dyn Host, pack: &Pack) -> Option<Animation> {
    let bytes = host.content_read_own(DRAFT_FILE).ok().filter(|b| !b.is_empty())?;
    let file: crate::pack::AnimationFile = serde_json::from_slice(&bytes).ok()?;
    pack.get(&file.id)?;
    Animation::compile(&file).ok()
}

fn is_file_id(id: &str) -> bool {
    (1..=64).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn load_pack(host: &mut dyn Host, id: &str) -> Result<Pack, String> {
    let base = {
        let mut read = |p: &str| host.content_read(BUILTIN_PACK, p);
        Pack::load(&mut read, None).map_err(|e| format!("{BUILTIN_PACK}: {e}"))?
    };
    if id == BUILTIN_PACK || !host.content_packs(CONTENT_TYPE).iter().any(|p| p == id) {
        return Ok(base);
    }
    let mut read = |p: &str| host.content_read(id, p);
    match Pack::load(&mut read, Some(&base)) {
        Ok(p) => Ok(p),
        Err(e) => {
            host.log(LogLevel::Warn, &format!("{id}: {e}; using {BUILTIN_PACK}"));
            Ok(base)
        }
    }
}

/// The animations already in My pack, so a new save keeps them in its pack.json.
fn my_pack_animations(host: &mut dyn Host) -> BTreeSet<String> {
    host.content_read_own("packs/my-pack/pack.json")
        .ok()
        .and_then(|b| serde_json::from_slice::<PackFile>(&b).ok())
        .map(|p| p.animations.into_iter().collect())
        .unwrap_or_default()
}

/// What PitCrew says about the picked port, without touching it.
fn describe(host: &mut dyn Host, settings: &Settings) -> Option<BoardInfo> {
    settings.port.as_ref()?;
    host.firmware_describe(PORT_SETTING).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::tests::FakeHost;
    use crate::signal::Flag;
    use pitflag_wire as wire;

    /// A device on COM4 that a finished setup remembers.
    fn running() -> (App, FakeHost) {
        let prefs = Prefs { device: Some(Device { port: "COM4".into(), board: "atmega168".into() }), ..Prefs::default() };
        let mut host = fresh_host();
        host.seeded.push((PREFS_FILE.into(), prefs.to_bytes()));
        let mut app = App::init(&mut host).unwrap();
        app.on_session(true, true);
        (app, host)
    }

    /// An ATmega168 with this plugin's firmware on COM4 and nothing saved yet.
    fn fresh_host() -> FakeHost {
        let mut host = FakeHost::default();
        host.settings = r#"{"port":"COM4","board":"atmega168","brightness":40,"animationPack":"pitcrew.flags.classic","greenHoldS":3}"#.into();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        host.device.borrow_mut().firmware = firmware::VERSION;
        host
    }

    fn has(app: &App, id: &str) -> bool {
        app.render().iter().any(|n| n.id == id)
    }

    fn disabled(app: &App, id: &str) -> bool {
        app.render().iter().any(|n| n.id == id && matches!(n.kind, Kind::Button { disabled: true, .. }))
    }

    fn frames_received(host: &FakeHost) -> Vec<[u8; 64]> {
        host.device.borrow().received.iter().filter(|p| p.kind == wire::Kind::Frame).filter_map(|p| wire::unpack_frame(&p.payload)).collect()
    }

    #[test]
    fn a_waving_yellow_reaches_the_device_as_the_yellow_waving_frames() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Yellow), waving: true, ..Signal::default() });
        for t in (0..1_000).step_by(33) {
            app.tick(&mut host, t);
        }
        let got = frames_received(&host);
        let want = &app.pack.get("yellow-waving").unwrap().frames;
        assert!(got.len() >= 4, "frames: {}", got.len());
        assert!(got.iter().all(|f| want.contains(f)));
    }

    #[test]
    fn outside_a_race_the_device_gets_the_standby_screen() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Red), ..Signal::default() });
        app.on_session(true, false);
        for t in (0..300).step_by(33) {
            app.tick(&mut host, t);
        }
        assert_eq!(app.shown_name, STANDBY);
        let standby = &app.pack.get(STANDBY).unwrap().frames;
        let last = frames_received(&host).last().copied().unwrap();
        assert!(standby.contains(&last));
        assert_ne!(last, [0u8; 64]);
    }

    #[test]
    fn with_standby_off_and_no_sim_the_device_gets_a_blank_frame() {
        let (mut app, mut host) = running();
        host.settings = r#"{"port":"COM4","standby":false}"#.into();
        app.settings_changed(&mut host);
        app.on_flag(Signal { primary: Some(Flag::Red), ..Signal::default() });
        app.on_session(false, false);
        for t in (0..300).step_by(33) {
            app.tick(&mut host, t);
        }
        assert_eq!(frames_received(&host).last(), Some(&[0u8; 64]));
    }

    #[test]
    fn every_tab_renders_a_tree_the_host_would_accept() {
        let (mut app, mut host) = running();
        app.tick(&mut host, 0);
        app.tick(&mut host, 40);
        for tab in ["live", "animations"] {
            app.on_ui_event(&mut host, "tabs", Event::Selected(tab.into()));
            crate::ui::validate(&app.render()).unwrap_or_else(|e| panic!("{tab}: {e}"));
        }
        for step in Step::ALL {
            app.open_setup(step);
            crate::ui::validate(&app.render()).unwrap_or_else(|e| panic!("{step:?}: {e}"));
        }
        app.on_ui_event(&mut host, "anim-table", Event::Selected("checkered".into()));
        crate::ui::validate(&app.render()).unwrap();
    }

    #[test]
    fn fill_and_clear_set_every_cell_of_the_frame() {
        let (mut app, mut host) = editing("yellow");
        let last = app.editor.as_ref().unwrap().anim.colors.len() as u8 - 1;
        app.on_ui_event(&mut host, "editor", Event::Filled(last));
        let e = app.editor.as_ref().unwrap();
        assert!(e.dirty && e.anim.frames[e.frame].iter().all(|&p| p == last));
        app.on_ui_event(&mut host, "editor", Event::Filled(0));
        let e = app.editor.as_ref().unwrap();
        assert!(e.anim.frames[e.frame].iter().all(|&p| p == 0));
        let before = e.anim.frames[e.frame];
        app.on_ui_event(&mut host, "editor", Event::Filled(99));
        assert_eq!(app.editor.as_ref().unwrap().anim.frames[0], before, "an index past the palette is ignored");
    }

    #[test]
    fn a_new_palette_changes_and_adds_colors_and_keeps_off() {
        let (mut app, mut host) = editing("yellow");
        let old = app.editor.as_ref().unwrap().anim.colors.clone();
        let mut next = old.clone();
        next[0] = 0xFF_FF_FF;
        if next.len() > 1 {
            next[1] = 0x12_34_56;
        }
        next.push(0xFF_00_AB_CD);
        app.on_ui_event(&mut host, "editor", Event::Palette(next.clone()));
        let e = app.editor.as_ref().unwrap();
        assert!(e.dirty);
        assert_eq!(e.anim.colors[0], old[0], "off stays off");
        assert_eq!(e.anim.colors.len(), old.len() + 1);
        assert_eq!(*e.anim.colors.last().unwrap(), 0x00_AB_CD, "only the RGB bits are kept");
        assert!(host.written.iter().any(|(p, b)| p == DRAFT_FILE && !b.is_empty()), "the draft keeps it");

        let shorter = old[..old.len() - 1].to_vec();
        app.on_ui_event(&mut host, "editor", Event::Palette(shorter));
        assert_eq!(app.editor.as_ref().unwrap().anim.colors.len(), old.len() + 1, "a shorter palette is refused");
        app.on_ui_event(&mut host, "editor", Event::Palette(vec![0; crate::pack::MAX_COLORS + 1]));
        assert_eq!(app.editor.as_ref().unwrap().anim.colors.len(), old.len() + 1, "more than 16 is refused");
    }

    #[test]
    fn the_editor_paints_and_saves_a_pack_the_plugin_can_load_again() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        app.on_ui_event(&mut host, "anim-table", Event::Selected("yellow".into()));
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 0 });
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        let files: Vec<_> = host.written.iter().map(|(p, _)| p.as_str()).filter(|p| p.starts_with("packs/")).collect();
        assert_eq!(files, vec!["packs/my-pack/animations/yellow.json", "packs/my-pack/pack.json"]);
        let written = host.written.clone();
        let mut read = |p: &str| {
            written.iter().find(|(w, _)| w == &format!("packs/my-pack/{p}")).map(|(_, b)| b.clone()).ok_or(p.to_string())
        };
        let base = crate::pack::tests::classic();
        let mine = Pack::load(&mut read, Some(&base)).unwrap();
        assert_eq!(mine.get("yellow").unwrap().frames[0][0], 0);
        assert_ne!(base.get("yellow").unwrap().frames[0][0], 0);
    }

    #[test]
    fn old_firmware_offers_an_update_that_opens_only_the_upload_step() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "device:update", Event::Pressed);
        assert_eq!(app.setup, Some(Step::Upload));
    }

    #[test]
    fn the_play_button_plays_the_picked_flag() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "play-pick", Event::Selected("yellow".into()));
        app.on_ui_event(&mut host, "play-one", Event::Pressed);
        assert!(!has(&app, "test:yellow"));
        crate::ui::validate(&app.render()).unwrap();
    }

    #[test]
    fn the_orientation_screen_offers_the_wiring_switches() {
        assert!(matrix_test::fix_keys("orientation", false).contains(&"serpentine"));
        assert!(!matrix_test::fix_keys("orientation", true).contains(&"serpentine"));
    }

    #[test]
    fn unsaved_edits_survive_picking_another_animation() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        app.on_ui_event(&mut host, "anim-table", Event::Selected("yellow".into()));
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 0 });
        app.on_ui_event(&mut host, "anim-table", Event::Selected("red".into()));
        assert_eq!(app.editor.as_ref().map(|e| e.anim.id.as_str()), Some("yellow"));
        assert!(has(&app, "editor-pending"));
        crate::ui::validate(&app.render()).unwrap();
        app.on_ui_event(&mut host, "editor:save-open", Event::Pressed);
        assert_eq!(app.editor.as_ref().map(|e| e.anim.id.as_str()), Some("red"));
        assert!(host.written.iter().any(|(p, _)| p == "packs/my-pack/animations/yellow.json"));

        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 0 });
        app.on_ui_event(&mut host, "anim-table", Event::Selected("blue-flash".into()));
        app.on_ui_event(&mut host, "editor:discard", Event::Pressed);
        assert_eq!(app.editor.as_ref().map(|e| (e.anim.id.as_str(), e.dirty)), Some(("blue-flash", false)));
    }

    #[test]
    fn a_failed_upload_offers_a_retry_and_says_the_board_is_fine() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        finish_job(&mut host, Some("The board did not answer."), None);
        app.tick(&mut host, 66);
        let nodes = app.render();
        assert!(nodes.iter().any(|n| n.id == "firmware-retry-hint"));
        assert!(nodes.iter().any(|n| matches!(&n.kind, Kind::Button { label, .. } if label == "Try the upload again")));
    }

    #[test]
    fn my_pack_keeps_earlier_saves_after_a_restart() {
        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        assert!(has(&app, "editor-saved"));
        let mut again = App::init(&mut host).unwrap();
        again.on_ui_event(&mut host, "anim-table", Event::Selected("red".into()));
        again.on_ui_event(&mut host, "editor-save", Event::Pressed);
        let pack: PackFile = serde_json::from_slice(&host.content_read_own("packs/my-pack/pack.json").unwrap()).unwrap();
        assert_eq!(pack.animations, vec!["red".to_string(), "yellow".to_string()]);
    }

    fn last_palette(host: &FakeHost) -> Vec<u8> {
        host.device.borrow().received.iter().rev().find(|p| p.kind == wire::Kind::Palette).map(|p| p.payload.clone()).unwrap_or_default()
    }

    #[test]
    fn the_test_step_walks_the_screens_and_hands_back_to_the_flag() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Blue), ..Signal::default() });
        app.tick(&mut host, 0);
        app.on_ui_event(&mut host, "device:test", Event::Pressed);
        app.tick(&mut host, 40);
        app.tick(&mut host, 73);
        assert_eq!(app.shown_name, "matrix test: red");
        assert_eq!(frames_received(&host).last(), Some(&[1u8; 64]));
        crate::ui::validate(&app.render()).unwrap();
        for _ in 0..4 {
            app.on_ui_event(&mut host, "matrix-test:next", Event::Pressed);
        }
        app.tick(&mut host, 106);
        assert_eq!(app.shown_name, "matrix test: orientation");
        assert_eq!(frames_received(&host).last(), Some(&matrix_test::screen(4).frames[0]));
        assert!(has(&app, "matrix-test-settings"), "the orientation screen shows Rotate");
        app.on_ui_event(&mut host, "matrix-test:prev", Event::Pressed);
        app.tick(&mut host, 139);
        assert_eq!(app.shown_name, "matrix test: white");
        app.on_ui_event(&mut host, "setup:cancel", Event::Pressed);
        app.tick(&mut host, 172);
        assert_eq!(app.shown_name, "blue-flash");
        assert_eq!((app.matrix_test, app.setup), (None, None));
    }

    #[test]
    fn the_test_from_the_device_card_is_not_the_setup() {
        let (mut app, mut host) = running();
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        app.on_ui_event(&mut host, "device:test", Event::Pressed);
        assert!(has(&app, "setup:done") && !has(&app, "setup-steps") && !has(&app, "setup:back") && !has(&app, "setup:finish"));
        crate::ui::validate(&app.render()).unwrap();
        app.on_ui_event(&mut host, "setup:done", Event::Pressed);
        assert_eq!((app.matrix_test, app.setup), (None, None));
        assert!(!app.player.testing());
    }

    #[test]
    fn finish_unlocks_only_on_the_last_screen_and_plays_green_then_checkered() {
        let (mut app, mut host) = running();
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        app.open_setup(Step::Test);
        assert!(disabled(&app, "setup:finish"));
        for _ in 1..matrix_test::STEPS.len() {
            app.on_ui_event(&mut host, "matrix-test:next", Event::Pressed);
        }
        assert_eq!(app.matrix_test, Some(matrix_test::STEPS.len() - 1));
        assert!(!has(&app, "matrix-test:next"));
        assert!(!disabled(&app, "setup:finish"));
        app.on_ui_event(&mut host, "setup:finish", Event::Pressed);
        assert_eq!((app.matrix_test, app.setup), (None, None));
        app.tick(&mut host, 66);
        app.tick(&mut host, 99);
        assert_eq!(app.shown_name, "green-flash");
    }

    #[test]
    fn a_setting_change_restarts_the_test_timer() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "device:test", Event::Pressed);
        app.tick(&mut host, MATRIX_TEST_MS - 1_000);
        app.settings_changed(&mut host);
        app.tick(&mut host, MATRIX_TEST_MS + 500);
        assert_eq!(app.matrix_test, Some(0));
    }

    #[test]
    fn an_unknown_port_needs_a_yes_before_the_upload() {
        let mut host = fresh_host();
        host.device.borrow_mut().hello_on_open = false;
        let mut app = App::init(&mut host).unwrap();
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        app.open_setup(Step::Upload);
        assert!(has(&app, "upload-unknown"));
        assert!(disabled(&app, "firmware:upload"));
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        assert!(host.fw_calls.is_empty());
        app.on_ui_event(&mut host, "upload:confirm", Event::Toggled(true));
        assert!(!disabled(&app, "firmware:upload"));
        crate::ui::validate(&app.render()).unwrap();
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        assert_eq!(host.fw_calls.len(), 1);
        assert!(disabled(&app, "firmware:upload"), "the button stays, disabled, while it uploads");
    }

    #[test]
    fn a_restart_reopens_the_setup_step_and_the_unsaved_edit() {
        let mut host = fresh_host();
        let mut app = App::init(&mut host).unwrap();
        app.on_ui_event(&mut host, "setup:next", Event::Pressed);
        assert_eq!(app.setup, Some(Step::Matrix));
        assert_eq!(App::init(&mut host).unwrap().setup, Some(Step::Matrix));

        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 0 });
        let again = App::init(&mut host).unwrap();
        let e = again.editor.as_ref().unwrap();
        assert_eq!((e.anim.id.as_str(), e.dirty, e.anim.frames[0][0]), ("yellow", true, 0));
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 1, y: 0, color: 1 });
        app.on_ui_event(&mut host, "editor-revert", Event::Pressed);
        assert!(App::init(&mut host).unwrap().editor.is_none());
    }

    #[test]
    fn a_matrix_test_left_alone_times_out() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "device:test", Event::Pressed);
        app.tick(&mut host, 40);
        assert_eq!(app.matrix_test, Some(0));
        app.tick(&mut host, MATRIX_TEST_MS + 100);
        assert_eq!(app.matrix_test, None);
        assert_eq!(app.shown_name, STANDBY);
        assert!(has(&app, "matrix-test:start"));
    }

    #[test]
    fn another_test_takes_the_place_of_the_matrix_test() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "device:test", Event::Pressed);
        app.tick(&mut host, 40);
        app.on_ui_event(&mut host, "test:checkered", Event::Pressed);
        app.tick(&mut host, 80);
        assert_eq!(app.shown_name, "checkered");
        assert_eq!(app.matrix_test, None);
    }

    #[test]
    fn a_fresh_install_shows_only_the_setup() {
        let mut host = fresh_host();
        let app = App::init(&mut host).unwrap();
        assert_eq!(app.setup, Some(Step::Board));
        assert!(!has(&app, "tabs"));
        assert!(has(&app, "setup-card"));
        assert!(!has(&app, "setup:cancel"), "nothing to go back to");
    }

    #[test]
    fn the_setup_walks_to_the_end_and_is_remembered() {
        let mut host = fresh_host();
        let mut app = App::init(&mut host).unwrap();
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        for step in [Step::Board, Step::Matrix, Step::Upload] {
            assert_eq!(app.setup, Some(step));
            assert!(!disabled(&app, "setup:next"), "{step:?}");
            app.on_ui_event(&mut host, "setup:next", Event::Pressed);
        }
        assert_eq!(app.setup, Some(Step::Test));
        for _ in 1..matrix_test::STEPS.len() {
            app.on_ui_event(&mut host, "matrix-test:next", Event::Pressed);
        }
        app.on_ui_event(&mut host, "setup:finish", Event::Pressed);
        assert_eq!(app.setup, None);
        assert!(has(&app, "tabs") && has(&app, "device-card"));

        let again = App::init(&mut host).unwrap();
        assert!(again.configured(), "prefs.json brings the device back");
        assert_eq!(again.setup, None);
    }

    #[test]
    fn upload_waits_for_this_plugins_firmware_before_the_test() {
        let mut host = fresh_host();
        host.device.borrow_mut().firmware = [2, 0, 0];
        let mut app = App::init(&mut host).unwrap();
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        app.open_setup(Step::Upload);
        assert!(disabled(&app, "setup:next"));
        assert!(!disabled(&app, "firmware:upload"));
        app.on_ui_event(&mut host, "setup:next", Event::Pressed);
        assert_eq!(app.setup, Some(Step::Upload));
    }

    #[test]
    fn a_bad_pin_blocks_the_matrix_step_and_the_upload() {
        let mut host = fresh_host();
        let mut app = App::init(&mut host).unwrap();
        host.settings = r#"{"port":"COM4","board":"atmega168","dataPin":25}"#.into();
        app.settings_changed(&mut host);
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        app.open_setup(Step::Matrix);
        assert!(disabled(&app, "setup:next"));
        app.open_setup(Step::Upload);
        assert!(disabled(&app, "firmware:upload"));
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        assert!(host.fw_calls.is_empty());
    }

    #[test]
    fn a_new_port_opens_the_setup_again() {
        let (mut app, mut host) = running();
        host.settings = r#"{"port":"COM9","board":"atmega168"}"#.into();
        app.settings_changed(&mut host);
        assert_eq!(app.setup, Some(Step::Board));
    }

    fn editing(name: &str) -> (App, FakeHost) {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        app.on_ui_event(&mut host, "anim-table", Event::Selected(name.into()));
        (app, host)
    }

    #[test]
    fn a_frame_can_be_deleted_but_never_the_last_one() {
        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "frame-dup", Event::Pressed);
        let frames = app.editor.as_ref().unwrap().anim.frames.len();
        app.on_ui_event(&mut host, "frame-del", Event::Pressed);
        let e = app.editor.as_ref().unwrap();
        assert_eq!(e.anim.frames.len(), frames - 1);
        assert!(e.frame < e.anim.frames.len());

        while app.editor.as_ref().unwrap().anim.frames.len() > 1 {
            app.on_ui_event(&mut host, "frame-del", Event::Pressed);
        }
        app.on_ui_event(&mut host, "frame-del", Event::Pressed);
        assert_eq!(app.editor.as_ref().unwrap().anim.frames.len(), 1);
        assert!(disabled(&app, "frame-del") && disabled(&app, "frame-prev") && disabled(&app, "frame-next"));
    }

    #[test]
    fn duplicate_is_disabled_at_the_frame_limit() {
        let (mut app, mut host) = editing("yellow");
        while app.editor.as_ref().unwrap().anim.frames.len() < crate::pack::MAX_FRAMES {
            app.on_ui_event(&mut host, "frame-dup", Event::Pressed);
        }
        assert!(disabled(&app, "frame-dup"));
        app.on_ui_event(&mut host, "frame-dup", Event::Pressed);
        assert_eq!(app.editor.as_ref().unwrap().anim.frames.len(), crate::pack::MAX_FRAMES);
    }

    #[test]
    fn discard_changes_returns_to_the_pack_version() {
        let (mut app, mut host) = editing("yellow");
        let before = app.editor.as_ref().unwrap().anim.frames.clone();
        app.on_ui_event(&mut host, "frame-dup", Event::Pressed);
        assert!(has(&app, "editor-revert"));
        app.on_ui_event(&mut host, "editor-revert", Event::Pressed);
        let e = app.editor.as_ref().unwrap();
        assert_eq!((e.anim.frames.clone(), e.dirty), (before, false));
        assert!(!has(&app, "editor-revert"));
    }

    #[test]
    fn an_edit_after_a_save_clears_the_saved_note() {
        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        assert!(has(&app, "editor-saved"));
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 1 });
        assert!(!has(&app, "editor-saved"));
    }

    #[test]
    fn a_row_that_is_not_in_the_pack_is_ignored() {
        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "frame-dup", Event::Pressed);
        app.on_ui_event(&mut host, "anim-table", Event::Selected("gone".into()));
        assert!(app.pending_edit.is_none());
        assert_eq!(app.editor.as_ref().unwrap().anim.id, "yellow");
    }

    #[test]
    fn save_refuses_an_id_that_is_not_a_plain_file_name() {
        let (mut app, mut host) = editing("yellow");
        app.editor.as_mut().unwrap().anim.id = "../escape".into();
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        assert!(matches!(app.save_note, Some((StatusLevel::Error, _))));
        assert!(host.content_read_own("packs/my-pack/animations/../escape.json").is_err());
        assert!(is_file_id("yellow-waving") && is_file_id("my_flag_2"));
        assert!(!is_file_id("") && !is_file_id("a/b") && !is_file_id("a.b") && !is_file_id(&"x".repeat(65)));
    }

    #[test]
    fn the_table_names_animations_and_a_row_opens_by_its_name() {
        assert_eq!((anim_label("yellow-waving"), anim_label("standby"), anim_label("my_flag")), ("Yellow, waving".into(), "Standby".into(), "My flag".into()));
        let (mut app, mut host) = editing("yellow");
        app.on_ui_event(&mut host, "anim-table", Event::Selected("Yellow, waving".into()));
        assert_eq!(app.editor.as_ref().unwrap().anim.id, "yellow-waving");
        app.on_ui_event(&mut host, "anim-table", Event::Selected("Standby".into()));
        assert_eq!(app.editor.as_ref().unwrap().anim.id, "standby");
    }

    #[test]
    fn the_flag_groups_hold_every_flag_once() {
        let mut grouped: Vec<&str> = FLAG_GROUPS.iter().flat_map(|(_, _, names)| names.iter().copied()).collect();
        grouped.sort_unstable();
        let mut all = TESTS.to_vec();
        all.sort_unstable();
        assert_eq!(grouped, all);
    }

    #[test]
    fn a_flag_turned_off_leaves_the_matrix_dark_and_is_remembered() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "flag:yellow", Event::Toggled(false));
        app.on_flag(Signal { primary: Some(Flag::Yellow), ..Signal::default() });
        for t in (0..300).step_by(33) {
            app.tick(&mut host, t);
        }
        assert_eq!(frames_received(&host).last(), Some(&[0u8; 64]));
        assert_eq!(app.shown_name, "yellow (turned off)");
        assert!(App::init(&mut host).unwrap().prefs.flags_off.contains("yellow"));

        app.on_ui_event(&mut host, "test:yellow", Event::Pressed);
        app.tick(&mut host, 340);
        app.tick(&mut host, 373);
        assert_eq!(app.shown_name, "yellow", "a test plays a flag that is off");
    }

    #[test]
    fn the_live_matrix_refreshes_the_screen_only_on_the_live_tab() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Yellow), waving: true, ..Signal::default() });
        for t in (0..300).step_by(33) {
            app.tick(&mut host, t);
        }
        assert!(app.link.is_open());
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        let before = host.invalidated;
        for t in (330..2_000).step_by(33) {
            app.tick(&mut host, t);
        }
        assert_eq!(host.invalidated, before);
        app.on_ui_event(&mut host, "tabs", Event::Selected("live".into()));
        let before = host.invalidated;
        for t in (2_000..4_000).step_by(33) {
            app.tick(&mut host, t);
        }
        assert!(host.invalidated > before);
    }

    #[test]
    fn the_led_correction_tames_green_and_blue_but_the_screen_keeps_the_pack_colors() {
        assert_eq!(led_color(0xFFD400), 0xFF9200);
        assert_eq!(led_color(0xFFFFFF), 0xFFB0F0);
        assert_eq!(led_color(0x000000), 0x000000);

        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Yellow), ..Signal::default() });
        for t in [0, 33, 66] {
            app.tick(&mut host, t);
        }
        assert_eq!(&last_palette(&host)[3..6], &[0xFF, 0x92, 0x00]);
        assert_eq!(app.shown_colors[1], 0xFFD400);

        host.settings = r#"{"port":"COM4","colorCorrection":false}"#.into();
        app.settings_changed(&mut host);
        app.tick(&mut host, 99);
        assert_eq!(&last_palette(&host)[3..6], &[0xFF, 0xD4, 0x00]);
    }

    fn on_device_tab(board: &str, chip: wire::Chip) -> (App, FakeHost) {
        let (mut app, mut host) = running();
        host.device.borrow_mut().chip = chip;
        host.seeded[0].1 = Prefs { device: Some(Device { port: "COM4".into(), board: board.into() }), ..Prefs::default() }.to_bytes();
        host.settings = format!(r#"{{"port":"COM4","board":"{board}"}}"#);
        app.settings_changed(&mut host);
        app.tick(&mut host, 0);
        app.tick(&mut host, 33);
        assert!(app.link.is_open());
        app.open_setup(Step::Upload);
        (app, host)
    }

    fn upload_button(app: &App) -> Option<Kind> {
        let nodes = app.render();
        crate::ui::validate(&nodes).unwrap();
        nodes.into_iter().find(|n| n.id == "firmware:upload").map(|n| n.kind)
    }

    fn finish_job(host: &mut FakeHost, error: Option<&str>, chip: Option<&str>) {
        let job = host.fw_job.as_mut().expect("a job");
        job.running = false;
        job.progress = 1.0;
        job.error = error.map(String::from);
        if !job.upload {
            job.board = Some(BoardInfo { chip: chip.map(String::from), detail: "asked".into(), ..BoardInfo::default() });
        }
    }

    #[test]
    fn upload_hands_the_boards_image_to_pitcrew_and_takes_the_port_back_after() {
        for (board, chip) in [("esp32c3", wire::Chip::Esp32c3), ("atmega328p", wire::Chip::Atmega328p), ("atmega32u4", wire::Chip::Atmega32u4)] {
            let (mut app, mut host) = on_device_tab(board, chip);
            assert!(matches!(upload_button(&app), Some(Kind::Button { disabled: false, .. })), "{board}");
            app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
            let b = Board::from_id(board);
            assert_eq!(host.fw_calls, vec![("upload".to_string(), board.to_string(), b.image().len())]);
            assert!(!app.link.is_open(), "the port is PitCrew's while it uploads");
            host.fw_job.as_mut().unwrap().progress = 0.5;
            host.fw_job.as_mut().unwrap().text = "Writing the firmware: 50%".into();
            app.tick(&mut host, 66);
            let nodes = app.render();
            crate::ui::validate(&nodes).unwrap();
            assert!(nodes.iter().any(|n| n.id == "firmware-progress"));
            assert!(!app.link.is_open(), "the link waits while the job runs");

            finish_job(&mut host, None, None);
            app.tick(&mut host, 99);
            assert_eq!(app.fw_result.as_ref().map(|r| r.0), Some(StatusLevel::Ok), "{board}");
            app.tick(&mut host, 132);
            app.tick(&mut host, 165);
            assert!(app.link.is_open(), "the link takes the port again");
        }
    }

    #[test]
    fn a_failed_upload_shows_pitcrews_reason() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        finish_job(&mut host, Some("The board did not enter its download mode."), None);
        app.tick(&mut host, 66);
        let (level, text) = app.fw_result.clone().unwrap();
        assert_eq!(level, StatusLevel::Error);
        assert!(text.contains("did not enter its download mode"), "{text}");
        crate::ui::validate(&app.render()).unwrap();
    }

    #[test]
    fn a_plugin_restarted_mid_upload_takes_up_pitcrews_job() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        app.shutdown();

        let mut app = App::init(&mut host).expect("starts again");
        assert!(app.fw_running && app.fw_uploading);
        assert_eq!(app.setup, Some(Step::Upload));
        app.tick(&mut host, 33);
        app.tick(&mut host, 66);
        assert!(!app.link.is_open(), "the port stays PitCrew's");
        assert!(has(&app, "firmware-progress"));
        assert!(disabled(&app, "firmware:upload"));

        finish_job(&mut host, None, None);
        app.tick(&mut host, 99);
        assert_eq!(app.fw_result.as_ref().map(|r| r.0), Some(StatusLevel::Ok));
        app.tick(&mut host, 132);
        app.tick(&mut host, 165);
        assert!(app.link.is_open());
    }

    #[test]
    fn a_finished_job_from_before_a_restart_is_left_alone() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        finish_job(&mut host, Some("old failure"), None);
        let app = App::init(&mut host).expect("starts again");
        assert!(!app.fw_running);
        assert!(app.fw_result.is_none());
    }

    #[test]
    fn a_port_change_mid_upload_leaves_the_port_to_pitcrew() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        host.settings = r#"{"port":"COM7","board":"esp32c3"}"#.into();
        app.settings_changed(&mut host);
        app.tick(&mut host, 66);
        app.tick(&mut host, 99);
        assert!(app.fw_running);
        assert!(!app.link.is_open(), "no link while the job runs");
    }

    #[test]
    fn the_port_and_board_are_locked_while_a_probe_runs() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.open_setup(Step::Board);
        app.on_ui_event(&mut host, "firmware:probe", Event::Pressed);
        assert!(app.fw_running);
        assert!(!has(&app, "board-settings"));
        assert!(has(&app, "board-locked"));
    }

    #[test]
    fn a_board_that_never_answers_after_the_upload_says_what_to_do() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        host.device.borrow_mut().present = false;
        finish_job(&mut host, None, None);
        app.tick(&mut host, 100);
        assert!(has(&app, "firmware-wait"));
        app.tick(&mut host, 100 + ANSWER_WAIT_MS - 1);
        assert!(!has(&app, "firmware-silent"));
        app.tick(&mut host, 100 + ANSWER_WAIT_MS);
        assert!(has(&app, "firmware-silent"));
        assert!(!has(&app, "firmware-wait"));
        assert!(!has(&app, "firmware-result"), "no \"it connects again\" beside the warning");
        assert!(!disabled(&app, "firmware:upload"), "upload again is offered");
        assert!(!has(&app, "upload-unknown"), "the port took our firmware, so it is not asked about again");
        crate::ui::validate(&app.render()).unwrap();
    }

    #[test]
    fn a_healthy_board_sits_under_the_live_matrix_and_a_missing_one_leads() {
        let parent = |app: &App| app.render().into_iter().find(|n| n.id == "device-card").and_then(|n| n.parent);
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        app.close_test();
        app.setup = None;
        app.tick(&mut host, 100);
        assert_eq!(parent(&app).as_deref(), Some("live"));
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        assert_eq!(parent(&app), None, "no device card on Animations");

        let (app, _host) = running();
        assert!(!app.link.is_open());
        assert_eq!(parent(&app).as_deref(), Some("page"));
    }

    #[test]
    fn a_refused_start_reopens_the_link_at_once() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Esp32c3);
        host.fw_refuse = Some("COM4 is in use by the plugin other".into());
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        assert!(!app.fw_running);
        assert!(app.fw_result.as_ref().unwrap().1.contains("in use"));
        app.tick(&mut host, 66);
        app.tick(&mut host, 99);
        assert!(app.link.is_open());
    }

    #[test]
    fn upload_is_refused_when_the_board_says_it_is_another_chip() {
        let (mut app, mut host) = on_device_tab("esp32c3", wire::Chip::Atmega328p);
        assert!(matches!(upload_button(&app), Some(Kind::Button { disabled: true, .. })));
        assert!(app.render().iter().any(|n| matches!(&n.kind, Kind::Badge { level: StatusLevel::Error, text } if text.contains("says it is"))));
        app.on_ui_event(&mut host, "firmware:upload", Event::Pressed);
        assert!(host.fw_calls.is_empty());
        assert!(app.link.is_open());
    }

    #[test]
    fn detect_board_reports_the_chip_pitcrew_found() {
        let (mut app, mut host) = on_device_tab("atmega328p", wire::Chip::Atmega328p);
        app.on_ui_event(&mut host, "firmware:probe", Event::Pressed);
        assert_eq!(host.fw_calls[0].0, "probe");
        finish_job(&mut host, None, Some("esp32c3"));
        app.tick(&mut host, 66);
        let (level, text) = app.fw_result.clone().unwrap();
        assert_eq!(level, StatusLevel::Warning);
        assert!(text.contains("ESP32-C3") && text.contains("Pick it in Board"), "{text}");

        app.on_ui_event(&mut host, "firmware:probe", Event::Pressed);
        finish_job(&mut host, None, Some("atmega328p"));
        app.tick(&mut host, 99);
        assert_eq!(app.fw_result.as_ref().map(|r| r.0), Some(StatusLevel::Ok));
    }

    #[test]
    fn the_port_card_names_what_the_usb_ids_say() {
        let (mut app, _host) = on_device_tab("atmega328p", wire::Chip::Atmega328p);
        app.open_setup(Step::Board);
        assert!(app.render().iter().any(|n| matches!(&n.kind, Kind::Text { text, .. } if text == "The port says: Fake USB")));
    }

    #[test]
    fn a_test_plays_ahead_of_the_flag() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Blue), ..Signal::default() });
        app.tick(&mut host, 0);
        app.on_ui_event(&mut host, "test:checkered", Event::Pressed);
        app.tick(&mut host, 40);
        app.tick(&mut host, 80);
        assert_eq!(app.shown_name, "checkered");
    }
}
