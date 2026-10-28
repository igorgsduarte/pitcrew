//! The plugin itself: settings, the pack, the player, the link and the screen.

use std::collections::BTreeSet;

use pitflag_wire as wire;
use serde::Deserialize;

use crate::host::{Host, LogLevel, StatusLevel};
use crate::link::{Link, Want};
use crate::matrix_test;
use crate::pack::{Animation, BUILTIN_PACK, CONTENT_TYPE, Pack, PackFile};
use crate::player::{OFF, Player, STANDBY, Source};
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
pub const SETTINGS_SHOWN: [&str; 5] = ["port", "brightness", "animationPack", "greenHoldS", "standby"];
pub const MATRIX_SETTINGS: [&str; 11] =
    ["matrix", "dataPin", "clockPin", "csPin", "colorOrder", "colorCorrection", "serpentine", "flipX", "flipY", "rotate", "maxMilliamps"];
pub const MY_PACK_ID: &str = "local.my-pack";
const UI_EVERY_MS: u64 = 200;
/// A matrix test screen stays this long without a click, then the matrix goes back to the flag.
pub const MATRIX_TEST_MS: u64 = 120_000;

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
    /// Applies `LED_CORRECTION` to the colors sent to a WS2812B. The screen keeps the pack's colors.
    #[serde(default = "default_true")]
    pub color_correction: bool,
    #[serde(default = "default_true")]
    pub serpentine: bool,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "default_rotate")]
    pub rotate: String,
    #[serde(default = "default_max_milliamps")]
    pub max_milliamps: u16,
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
    editor: Option<Editor>,
    /// The matrix test step on screen, if the test runs.
    matrix_test: Option<usize>,
    saved: BTreeSet<String>,
    now: u64,
    last_invalidate: u64,
}

impl App {
    pub fn init(host: &mut dyn Host) -> Result<App, String> {
        let settings = Settings::parse(&host.settings_json());
        let pack = load_pack(host, &settings.animation_pack)?;
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
            editor: None,
            matrix_test: None,
            saved: BTreeSet::new(),
            now: 0,
            last_invalidate: 0,
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
        if next.port != self.settings.port {
            self.link.reopen(host, self.now);
        }
        self.settings = next;
        host.invalidate_ui();
    }

    pub fn tick(&mut self, host: &mut dyn Host, now: u64) {
        self.now = now;
        let signal = if self.connected && self.in_car { self.signal.as_ref() } else { None };
        let outside = if self.settings.standby { STANDBY } else { OFF };
        let choice = self.player.choose(now, signal, &self.pack, self.settings.green_hold_s, &self.blank, outside);
        let frame = *choice.anim.frame_at(now.saturating_sub(choice.started));
        let colors = choice.anim.colors.clone();
        let (name, source) = (choice.name, choice.source);
        let correct = self.settings.color_correction && self.settings.matrix != "max7219";
        let sent: Vec<u32> = if correct { colors.iter().map(|&c| led_color(c)).collect() } else { colors.clone() };
        self.link.tick(host, now, &Want { colors: &sent, frame: &frame, brightness: self.settings.brightness, config: self.settings.config() });
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
        if changed && now.saturating_sub(self.last_invalidate) >= UI_EVERY_MS {
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
        self.link.close();
    }

    fn play_all(&mut self) {
        self.player.play_tests(TESTS.iter().map(|s| s.to_string()).collect(), self.now, 1_500);
    }

    pub fn render(&self) -> Vec<Node> {
        let mut t = Tree::default();
        let page = t.add("page", None, Kind::Page { title: "Flags".into(), subtitle: Some("Race flags on your LED matrix".into()) });
        let tabs = t.add("tabs", Some(&page), Kind::Tabs { selected: self.tab.clone() });
        for (id, label) in [("live", "Live"), ("animations", "Animations"), ("device", "Device")] {
            t.add(id, Some(&tabs), Kind::Tab(label.into()));
        }
        match self.tab.as_str() {
            "animations" => self.render_animations(&mut t),
            "device" => self.render_device(&mut t),
            _ => self.render_live(&mut t),
        }
        t.nodes
    }

    fn render_live(&self, t: &mut Tree) {
        let grid = t.add("live-grid", Some("live"), Kind::Grid { columns: 2, gap: Gap::Md });
        let card = t.add("matrix-card", Some(&grid), Kind::Card { title: Some("Matrix".into()), description: None });
        t.add("matrix", Some(&card), Kind::PixelGrid {
            width: 8,
            height: 8,
            palette: self.shown_colors.clone(),
            pixels: if self.link.is_open() { self.shown_frame.to_vec() } else { vec![0; 64] },
            editable: false,
        });
        let showing = if self.link.is_open() { format!("showing {}", self.shown_name) } else { "nothing is reaching the matrix".into() };
        t.text("matrix-showing", &card, showing, Tone::Muted, true);

        let card = t.add("now-card", Some(&grid), Kind::Card { title: Some("Now".into()), description: None });
        let device = self.link.status().map_or("Starting…".to_string(), |s| s.1.clone());
        let signal = match (&self.signal, self.connected, self.in_car) {
            (_, false, _) => "No sim connected".to_string(),
            (_, true, false) => "Not in the car".to_string(),
            (Some(s), true, true) => s.describe(),
            (None, true, true) => "Waiting for the first flag".to_string(),
        };
        t.add("now", Some(&card), Kind::Table {
            columns: vec!["".into(), "".into()],
            rows: vec![
                vec!["Device".into(), device],
                vec!["Flag".into(), signal],
                vec!["Showing".into(), format!("{} ({})", self.shown_name, self.shown_source.describe())],
            ],
            selectable: false,
        });

        let card = t.add("test-card", Some("live"), Kind::Card {
            title: Some("Test without a sim".into()),
            description: Some("Plays on the matrix even with no sim running.".into()),
        });
        let row = t.add("tests", Some(&card), Kind::Stack(Gap::Sm));
        for name in TESTS {
            t.button(&format!("test:{name}"), &row, name, Variant::Secondary);
        }
        t.button("play-all", &card, "Play every flag", Variant::Primary);
    }

    fn render_animations(&self, t: &mut Tree) {
        let grid = t.add("anim-grid", Some("animations"), Kind::Grid { columns: 2, gap: Gap::Md });
        let card = t.add("anims-card", Some(&grid), Kind::Card { title: Some(format!("Animations · {}", self.pack.name)), description: None });
        t.add("anim-table", Some(&card), Kind::Table {
            columns: vec!["Animation".into(), "Frames".into(), "fps".into()],
            rows: self.pack.animations.values().map(|a| vec![a.id.clone(), a.frames.len().to_string(), a.fps.to_string()]).collect(),
            selectable: true,
        });
        let card = t.add("editor-card", Some(&grid), Kind::Card {
            title: Some(self.editor.as_ref().map_or("Editor".into(), |e| format!("Editing {}", e.anim.id))),
            description: None,
        });
        let Some(e) = &self.editor else {
            t.text("editor-empty", &card, "Pick an animation to edit it.", Tone::Muted, false);
            return;
        };
        t.add("editor", Some(&card), Kind::PixelGrid {
            width: 8,
            height: 8,
            palette: e.anim.colors.clone(),
            pixels: e.anim.frames[e.frame].to_vec(),
            editable: true,
        });
        let frames = t.add("editor-frames", Some(&card), Kind::Stack(Gap::Sm));
        t.button("frame-prev", &frames, "Previous", Variant::Secondary);
        t.text("frame-label", &frames, format!("Frame {} of {}", e.frame + 1, e.anim.frames.len()), Tone::Normal, true);
        t.button("frame-next", &frames, "Next", Variant::Secondary);
        t.button("frame-dup", &frames, "Duplicate frame", Variant::Secondary);
        t.add("editor-fps", Some(&card), Kind::Slider { label: "Speed (fps)".into(), min: 1.0, max: 30.0, step: 1.0, value: e.anim.fps as f64 });
        let actions = t.add("editor-actions", Some(&card), Kind::Stack(Gap::Sm));
        t.button("editor-play", &actions, "Play on the matrix", Variant::Secondary);
        t.button("editor-save", &actions, "Save to My pack", Variant::Primary);
    }

    fn render_device(&self, t: &mut Tree) {
        let grid = t.add("device-grid", Some("device"), Kind::Grid { columns: 2, gap: Gap::Md });
        let card = t.add("conn-card", Some(&grid), Kind::Card { title: Some("Connection".into()), description: None });
        let (level, text) = self.link.status().cloned().unwrap_or((StatusLevel::Idle, "Starting…".into()));
        t.add("conn", Some(&card), Kind::Badge { text, level });
        if let Some(h) = self.link.hello() {
            let [a, b, c] = h.firmware;
            t.text("fw", &card, format!("PitFlag wire v{} · firmware {a}.{b}.{c} · {} · {}×{} {} · up to {} fps", h.wire, h.chip.label(), h.width, h.height, if h.rgb { "RGB" } else { "mono" }, h.max_fps), Tone::Muted, true);
        }
        let bps = self.link.bytes_per_second;
        t.add("usage", Some(&card), Kind::Progress {
            value: (bps as f32 / 11_520.0).min(1.0),
            label: Some(format!("Serial link in use: {bps} of 11 520 B/s")),
        });
        let card = t.add("settings-card", Some(&grid), Kind::Card { title: Some("Settings".into()), description: None });
        t.add("settings", Some(&card), Kind::SettingsForm(SETTINGS_SHOWN.iter().map(|s| s.to_string()).collect()));
        let card = t.add("matrix-card", Some(&grid), Kind::Card {
            title: Some("Matrix".into()),
            description: Some("Sent to the board and kept there. Leave a pin empty for the board's default.".into()),
        });
        t.add("matrix-settings", Some(&card), Kind::SettingsForm(MATRIX_SETTINGS.iter().map(|s| s.to_string()).collect()));
        self.render_matrix_test(t, &grid);
    }

    fn render_matrix_test(&self, t: &mut Tree, grid: &str) {
        let card = t.add("matrix-test-card", Some(grid), Kind::Card {
            title: Some("Matrix test".into()),
            description: Some("Shows fixed screens on the matrix. Change the Matrix settings while a screen shows: the board applies them at once.".into()),
        });
        let Some(i) = self.matrix_test else {
            t.button("matrix-test:start", &card, "Start the test", Variant::Primary);
            return;
        };
        let step = &matrix_test::STEPS[i];
        let shot = matrix_test::screen(i);
        t.text("matrix-test-step", &card, format!("Step {} of {}: {}", i + 1, matrix_test::STEPS.len(), step.id), Tone::Normal, false);
        t.add("matrix-test-screen", Some(&card), Kind::PixelGrid {
            width: 8,
            height: 8,
            palette: shot.colors,
            pixels: shot.frames[0].to_vec(),
            editable: false,
        });
        t.text("matrix-test-expect", &card, step.expect, Tone::Normal, false);
        t.text("matrix-test-fix", &card, step.fix, Tone::Muted, false);
        if !self.link.is_open() {
            t.text("matrix-test-closed", &card, "The board is not connected, so the screen shows only here.", Tone::Muted, false);
        }
        let row = t.add("matrix-test-actions", Some(&card), Kind::Stack(Gap::Sm));
        t.add("matrix-test:prev", Some(&row), Kind::Button { label: "Previous".into(), variant: Variant::Secondary, icon: None, disabled: i == 0 });
        let last = i + 1 == matrix_test::STEPS.len();
        t.button("matrix-test:next", &row, if last { "Finish" } else { "Next" }, Variant::Primary);
        t.button("matrix-test:stop", &row, "Stop", Variant::Secondary);
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

    pub fn on_ui_event(&mut self, host: &mut dyn Host, node: &str, event: Event) {
        match (node, event) {
            ("tabs", Event::Selected(tab)) => self.tab = tab,
            ("play-all", Event::Pressed) => self.play_all(),
            (id, Event::Pressed) if id.starts_with("test:") => {
                self.player.play_tests(vec![id["test:".len()..].to_string()], self.now, 2_000);
            }
            ("anim-table", Event::Selected(name)) => {
                if let Some(a) = self.pack.get(&name) {
                    self.editor = Some(Editor { anim: a.clone(), frame: 0 });
                }
            }
            ("editor", Event::Painted { x, y, color }) => {
                if let Some(e) = &mut self.editor {
                    if x < 8 && y < 8 && (color as usize) < e.anim.colors.len() {
                        e.anim.frames[e.frame][y as usize * 8 + x as usize] = color;
                    }
                }
            }
            ("frame-prev", Event::Pressed) => self.with_editor(|e| e.frame = (e.frame + e.anim.frames.len() - 1) % e.anim.frames.len()),
            ("frame-next", Event::Pressed) => self.with_editor(|e| e.frame = (e.frame + 1) % e.anim.frames.len()),
            ("frame-dup", Event::Pressed) => self.with_editor(|e| {
                if e.anim.frames.len() < crate::pack::MAX_FRAMES {
                    let f = e.anim.frames[e.frame];
                    e.anim.frames.insert(e.frame + 1, f);
                    e.frame += 1;
                }
            }),
            ("editor-fps", Event::Number(v)) => self.with_editor(|e| e.anim.fps = (v.round() as u32).clamp(1, crate::pack::MAX_FPS)),
            ("editor-play", Event::Pressed) => {
                if let Some(e) = &self.editor {
                    self.player.play_custom(e.anim.clone(), format!("{} (editing)", e.anim.id), self.now, 3_500);
                }
            }
            ("matrix-test:start", Event::Pressed) => self.show_matrix_test(0),
            ("matrix-test:prev", Event::Pressed) => {
                if let Some(i) = self.matrix_test {
                    self.show_matrix_test(i.saturating_sub(1));
                }
            }
            ("matrix-test:next", Event::Pressed) => match self.matrix_test {
                Some(i) if i + 1 < matrix_test::STEPS.len() => self.show_matrix_test(i + 1),
                Some(_) => self.stop_matrix_test(),
                None => {}
            },
            ("matrix-test:stop", Event::Pressed) => self.stop_matrix_test(),
            ("editor-save", Event::Pressed) => self.save(host),
            _ => {}
        }
        host.invalidate_ui();
    }

    fn with_editor(&mut self, f: impl FnOnce(&mut Editor)) {
        if let Some(e) = &mut self.editor {
            f(e);
        }
    }

    fn save(&mut self, host: &mut dyn Host) {
        let Some(e) = &self.editor else { return };
        let id = e.anim.id.clone();
        let body = serde_json::to_vec_pretty(&e.anim.to_file()).expect("serialisable");
        if let Err(err) = host.content_write_own(&format!("packs/my-pack/animations/{id}.json"), &body) {
            host.set_status(StatusLevel::Warning, &format!("Could not save {id}: {err}"));
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
        match host.content_write_own("packs/my-pack/pack.json", &body) {
            Ok(()) => host.log(LogLevel::Info, &format!("saved {id} to My pack")),
            Err(err) => host.set_status(StatusLevel::Warning, &format!("Could not save My pack: {err}")),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::tests::FakeHost;
    use crate::signal::Flag;
    use pitflag_wire as wire;

    fn running() -> (App, FakeHost) {
        let mut host = FakeHost::default();
        host.settings = r#"{"port":"COM4","brightness":40,"animationPack":"pitcrew.flags.classic","greenHoldS":3}"#.into();
        host.device.borrow_mut().present = true;
        host.device.borrow_mut().hello_on_open = true;
        let mut app = App::init(&mut host).unwrap();
        app.on_session(true, true);
        (app, host)
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
        for tab in ["live", "animations", "device"] {
            app.on_ui_event(&mut host, "tabs", Event::Selected(tab.into()));
            crate::ui::validate(&app.render()).unwrap_or_else(|e| panic!("{tab}: {e}"));
        }
        app.on_ui_event(&mut host, "anim-table", Event::Selected("checkered".into()));
        crate::ui::validate(&app.render()).unwrap();
    }

    #[test]
    fn the_editor_paints_and_saves_a_pack_the_plugin_can_load_again() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "tabs", Event::Selected("animations".into()));
        app.on_ui_event(&mut host, "anim-table", Event::Selected("yellow".into()));
        app.on_ui_event(&mut host, "editor", Event::Painted { x: 0, y: 0, color: 0 });
        app.on_ui_event(&mut host, "editor-save", Event::Pressed);
        let files: Vec<_> = host.written.iter().map(|(p, _)| p.as_str()).collect();
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

    fn last_palette(host: &FakeHost) -> Vec<u8> {
        host.device.borrow().received.iter().rev().find(|p| p.kind == wire::Kind::Palette).map(|p| p.payload.clone()).unwrap_or_default()
    }

    #[test]
    fn the_matrix_test_steps_through_its_screens_and_hands_back_to_the_flag() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Blue), ..Signal::default() });
        app.tick(&mut host, 0);
        app.on_ui_event(&mut host, "tabs", Event::Selected("device".into()));
        app.on_ui_event(&mut host, "matrix-test:start", Event::Pressed);
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
        crate::ui::validate(&app.render()).unwrap();
        app.on_ui_event(&mut host, "matrix-test:prev", Event::Pressed);
        app.tick(&mut host, 139);
        assert_eq!(app.shown_name, "matrix test: white");
        app.on_ui_event(&mut host, "matrix-test:stop", Event::Pressed);
        app.tick(&mut host, 172);
        assert_eq!(app.shown_name, "blue-flash");
        assert_eq!(app.matrix_test, None);
    }

    #[test]
    fn finish_on_the_last_step_ends_the_matrix_test() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "matrix-test:start", Event::Pressed);
        for _ in 1..matrix_test::STEPS.len() {
            app.on_ui_event(&mut host, "matrix-test:next", Event::Pressed);
        }
        assert_eq!(app.matrix_test, Some(matrix_test::STEPS.len() - 1));
        app.on_ui_event(&mut host, "matrix-test:next", Event::Pressed);
        assert_eq!(app.matrix_test, None);
        assert!(!app.player.testing());
    }

    #[test]
    fn a_matrix_test_left_alone_times_out() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "matrix-test:start", Event::Pressed);
        app.tick(&mut host, 40);
        assert_eq!(app.matrix_test, Some(0));
        app.tick(&mut host, MATRIX_TEST_MS + 100);
        assert_eq!(app.matrix_test, None);
        assert_eq!(app.shown_name, STANDBY);
    }

    #[test]
    fn another_test_takes_the_place_of_the_matrix_test() {
        let (mut app, mut host) = running();
        app.on_ui_event(&mut host, "matrix-test:start", Event::Pressed);
        app.tick(&mut host, 40);
        app.on_ui_event(&mut host, "test:checkered", Event::Pressed);
        app.tick(&mut host, 80);
        assert_eq!(app.shown_name, "checkered");
        assert_eq!(app.matrix_test, None);
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
