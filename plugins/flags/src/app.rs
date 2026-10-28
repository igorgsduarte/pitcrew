//! The plugin itself: settings, the pack, the player, the link and the screen.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::host::{Host, LogLevel, StatusLevel};
use crate::link::{Link, Want};
use crate::pack::{Animation, BUILTIN_PACK, CONTENT_TYPE, Pack, PackFile};
use crate::player::{Player, Source};
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
pub const SETTINGS_SHOWN: [&str; 4] = ["port", "brightness", "animationPack", "greenHoldS"];
pub const MY_PACK_ID: &str = "local.my-pack";
const UI_EVERY_MS: u64 = 200;

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
}

fn default_brightness() -> u8 {
    40
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
    player: Player,
    link: Link,
    blank: Animation,
    shown_name: String,
    shown_source: Source,
    shown_frame: [u8; 64],
    shown_colors: Vec<u32>,
    tab: String,
    editor: Option<Editor>,
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
            player: Player::default(),
            link: Link::default(),
            blank: Animation::blank(),
            shown_name: "idle".into(),
            shown_source: Source::NoSim,
            shown_frame: [0; 64],
            shown_colors: vec![0],
            tab: "live".into(),
            editor: None,
            saved: BTreeSet::new(),
            now: 0,
            last_invalidate: 0,
        })
    }

    pub fn on_flag(&mut self, signal: Signal) {
        self.signal = Some(signal);
    }

    pub fn on_session(&mut self, connected: bool) {
        self.connected = connected;
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
        let signal = if self.connected { self.signal.as_ref() } else { None };
        let choice = self.player.choose(now, signal, &self.pack, self.settings.green_hold_s, &self.blank);
        let frame = *choice.anim.frame_at(now.saturating_sub(choice.started));
        let colors = choice.anim.colors.clone();
        let (name, source) = (choice.name, choice.source);
        self.link.tick(host, now, &Want { colors: &colors, frame: &frame, brightness: self.settings.brightness });
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
        let signal = match (&self.signal, self.connected) {
            (_, false) => "No sim connected".to_string(),
            (Some(s), true) => s.describe(),
            (None, true) => "Waiting for the first flag".to_string(),
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
            t.text("fw", &card, format!("PitFlag wire v{} · firmware {a}.{b}.{c} · {}×{} {} · up to {} fps", h.wire, h.width, h.height, if h.rgb { "RGB" } else { "mono" }, h.max_fps), Tone::Muted, true);
        }
        let bps = self.link.bytes_per_second;
        t.add("usage", Some(&card), Kind::Progress {
            value: (bps as f32 / 11_520.0).min(1.0),
            label: Some(format!("Serial link in use: {bps} of 11 520 B/s")),
        });
        let card = t.add("settings-card", Some(&grid), Kind::Card { title: Some("Settings".into()), description: None });
        t.add("settings", Some(&card), Kind::SettingsForm(SETTINGS_SHOWN.iter().map(|s| s.to_string()).collect()));
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
                    self.player.play_custom(e.anim.clone(), self.now, 3_500);
                }
            }
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
        app.on_session(true);
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
    fn with_no_sim_the_device_gets_a_blank_frame() {
        let (mut app, mut host) = running();
        app.on_flag(Signal { primary: Some(Flag::Red), ..Signal::default() });
        app.on_session(false);
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
