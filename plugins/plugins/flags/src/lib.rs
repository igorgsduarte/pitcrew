//! PitCrew Flags: race flags on an 8×8 LED matrix over the PitFlag serial protocol.
//!
//! Everything but the `wasm` module is plain Rust behind the `host::Host` trait, so `cargo test`
//! runs it natively against a fake device.

pub mod app;
pub mod firmware;
pub mod host;
pub mod link;
pub mod matrix_test;
pub mod pack;
pub mod player;
pub mod setup;
pub mod signal;
pub mod ui;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;

    use crate::app::App;
    use crate::host::{self as h, Host};
    use crate::signal as s;
    use crate::ui as u;

    wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

    use pitcrew::plugin::types as wt;
    use pitcrew::plugin::ui as wu;
    use pitcrew::plugin::{content, firmware, host, serial, settings};

    thread_local! {
        static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    }

    struct WitHost;

    struct WitPort(serial::Port);

    impl h::Port for WitPort {
        fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
            self.0.write(bytes)
        }
        fn read(&mut self, max: u32) -> Result<Vec<u8>, String> {
            self.0.read(max)
        }
    }

    fn board_info(b: firmware::Board) -> h::BoardInfo {
        h::BoardInfo { transport: b.transport, chip: b.chip, usb_name: b.usb_name, detail: b.detail }
    }

    impl h::Firmware for WitHost {
        fn firmware_describe(&mut self, setting_key: &str) -> Result<h::BoardInfo, String> {
            firmware::describe(setting_key).map(board_info)
        }
        fn firmware_probe(&mut self, setting_key: &str) -> Result<(), String> {
            firmware::start_probe(setting_key)
        }
        fn firmware_upload(&mut self, setting_key: &str, chip: &str, image: &[u8]) -> Result<(), String> {
            firmware::start_upload(setting_key, chip, image)
        }
        fn firmware_job(&mut self) -> Option<h::FirmwareJob> {
            firmware::job().map(|j| h::FirmwareJob {
                upload: j.kind == "upload",
                running: j.running,
                progress: j.progress,
                text: j.text,
                error: j.error,
                board: j.board.map(board_info),
            })
        }
    }

    impl Host for WitHost {
        fn log(&mut self, level: h::LogLevel, text: &str) {
            let level = match level {
                h::LogLevel::Debug => wt::LogLevel::Debug,
                h::LogLevel::Info => wt::LogLevel::Info,
                h::LogLevel::Warn => wt::LogLevel::Warn,
                h::LogLevel::Error => wt::LogLevel::Error,
            };
            host::log(level, text);
        }
        fn set_status(&mut self, level: h::StatusLevel, text: &str) {
            host::set_status(status(level), text);
        }
        fn invalidate_ui(&mut self) {
            host::invalidate_ui();
        }
        fn open_port(&mut self, setting_key: &str, baud: u32) -> Result<Box<dyn h::Port>, String> {
            serial::open(setting_key, baud).map(|p| Box::new(WitPort(p)) as Box<dyn h::Port>)
        }
        fn settings_json(&mut self) -> String {
            settings::get()
        }
        fn content_packs(&mut self, of_type: &str) -> Vec<String> {
            content::packs(of_type)
        }
        fn content_read(&mut self, pack: &str, path: &str) -> Result<Vec<u8>, String> {
            content::read(pack, path)
        }
        fn content_write_own(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
            content::write_own(path, bytes)
        }
        fn content_read_own(&mut self, path: &str) -> Result<Vec<u8>, String> {
            content::read_own(path)
        }
    }

    fn with_app(f: impl FnOnce(&mut App, &mut WitHost)) {
        APP.with_borrow_mut(|app| {
            if let Some(app) = app {
                f(app, &mut WitHost);
            }
        });
    }

    fn status(level: h::StatusLevel) -> wt::StatusLevel {
        match level {
            h::StatusLevel::Ok => wt::StatusLevel::Ok,
            h::StatusLevel::Idle => wt::StatusLevel::Idle,
            h::StatusLevel::Warning => wt::StatusLevel::Warning,
            h::StatusLevel::Error => wt::StatusLevel::Error,
        }
    }

    fn signal(f: wt::FlagSignal) -> s::Signal {
        s::Signal {
            primary: f.primary.map(|p| match p {
                wt::Flag::Disqualify => s::Flag::Disqualify,
                wt::Flag::Black => s::Flag::Black,
                wt::Flag::Repair => s::Flag::Repair,
                wt::Flag::Red => s::Flag::Red,
                wt::Flag::Checkered => s::Flag::Checkered,
                wt::Flag::White => s::Flag::White,
                wt::Flag::Yellow => s::Flag::Yellow,
                wt::Flag::Blue => s::Flag::Blue,
                wt::Flag::Green => s::Flag::Green,
            }),
            waving: f.waving,
            debris: f.debris,
            one_lap_to_green: f.one_lap_to_green,
            start: f.start.map(|l| match l {
                wt::StartLight::Ready => s::StartLight::Ready,
                wt::StartLight::Set => s::StartLight::Set,
                wt::StartLight::Go => s::StartLight::Go,
            }),
            driver: f
                .driver
                .into_iter()
                .map(|d| match d {
                    wt::DriverFlag::Black => s::DriverFlag::Black,
                    wt::DriverFlag::Repair => s::DriverFlag::Repair,
                    wt::DriverFlag::Disqualify => s::DriverFlag::Disqualify,
                })
                .collect(),
        }
    }

    fn gap(g: u::Gap) -> wu::Gap {
        match g {
            u::Gap::Sm => wu::Gap::Sm,
            u::Gap::Md => wu::Gap::Md,
            u::Gap::Lg => wu::Gap::Lg,
        }
    }

    fn node(n: u::Node) -> wu::UiNode {
        let kind = match n.kind {
            u::Kind::Page { title, subtitle } => wu::NodeKind::Page(wu::PageProps { title, subtitle }),
            u::Kind::Tabs { selected } => wu::NodeKind::Tabs(wu::TabsProps { selected }),
            u::Kind::Tab(label) => wu::NodeKind::Tab(label),
            u::Kind::Card { title, description } => wu::NodeKind::Card(wu::CardProps { title, description }),
            u::Kind::Grid { columns, gap: g } => wu::NodeKind::Grid(wu::GridProps { columns, gap: gap(g) }),
            u::Kind::Stack(g) => wu::NodeKind::Stack(gap(g)),
            u::Kind::Separator => wu::NodeKind::Separator,
            u::Kind::Text { text, tone, mono } => wu::NodeKind::Text(wu::TextProps {
                text,
                tone: match tone {
                    u::Tone::Normal => wu::Tone::Normal,
                    u::Tone::Muted => wu::Tone::Muted,
                    u::Tone::Warning => wu::Tone::Warning,
                    u::Tone::Error => wu::Tone::Error,
                },
                mono,
            }),
            u::Kind::Badge { text, level } => wu::NodeKind::Badge(wu::BadgeProps { text, level: status(level) }),
            u::Kind::Alert { title, text, level } => wu::NodeKind::Alert(wu::AlertProps { title, text, level: status(level) }),
            u::Kind::Table { columns, rows, selectable } => wu::NodeKind::Table(wu::TableProps { columns, rows, selectable }),
            u::Kind::Progress { value, label } => wu::NodeKind::Progress(wu::ProgressProps { value, label }),
            u::Kind::Button { label, variant, icon, disabled } => wu::NodeKind::Button(wu::ButtonProps {
                label,
                variant: match variant {
                    u::Variant::Primary => wu::ButtonVariant::Primary,
                    u::Variant::Secondary => wu::ButtonVariant::Secondary,
                    u::Variant::Destructive => wu::ButtonVariant::Destructive,
                },
                icon,
                disabled,
            }),
            u::Kind::Switch { label, checked, disabled } => wu::NodeKind::Switch(wu::SwitchProps { label, checked, disabled }),
            u::Kind::Slider { label, min, max, step, value } => wu::NodeKind::Slider(wu::SliderProps { label, min, max, step, value }),
            u::Kind::Select { label, options, value } => wu::NodeKind::Select(wu::SelectProps {
                label,
                options: options.into_iter().map(|(value, label)| wu::SelectOption { value, label }).collect(),
                value,
            }),
            u::Kind::Input { label, value, max_length } => wu::NodeKind::Input(wu::InputProps { label, value, max_length }),
            u::Kind::PixelGrid { width, height, palette, pixels, editable } => {
                wu::NodeKind::PixelGrid(wu::PixelGridProps { width, height, palette, pixels, editable })
            }
            u::Kind::SettingsForm(keys) => wu::NodeKind::SettingsForm(keys),
        };
        wu::UiNode { id: n.id, parent: n.parent, kind }
    }

    fn event(k: wu::EventKind) -> u::Event {
        match k {
            wu::EventKind::Pressed => u::Event::Pressed,
            wu::EventKind::Toggled(b) => u::Event::Toggled(b),
            wu::EventKind::ChangedNumber(v) => u::Event::Number(v),
            wu::EventKind::ChangedText(t) => u::Event::Text(t),
            wu::EventKind::Selected(id) => u::Event::Selected(id),
            wu::EventKind::Painted(p) => u::Event::Painted { x: p.x, y: p.y, color: p.color },
            wu::EventKind::Filled(c) => u::Event::Filled(c),
            wu::EventKind::PaletteChanged(colors) => u::Event::Palette(colors),
        }
    }

    struct Flags;

    impl Guest for Flags {
        fn init() -> Result<(), String> {
            let app = App::init(&mut WitHost)?;
            APP.with_borrow_mut(|a| *a = Some(app));
            Ok(())
        }

        fn on_flag(f: wt::FlagSignal) {
            with_app(|app, _| app.on_flag(signal(f)));
        }

        fn on_session(session: wt::SessionInfo) {
            with_app(|app, _| app.on_session(session.connected, session.in_car));
        }

        fn settings_changed() {
            with_app(|app, host| app.settings_changed(host));
        }

        fn tick(now_ms: u64) {
            with_app(|app, host| app.tick(host, now_ms));
        }

        fn on_action(id: String) {
            with_app(|app, _| app.on_action(&id));
        }

        fn render() -> Vec<wu::UiNode> {
            APP.with_borrow(|app| app.as_ref().map(|a| a.render().into_iter().map(node).collect()).unwrap_or_default())
        }

        fn on_ui_event(e: wu::UiEvent) {
            with_app(|app, host| app.on_ui_event(host, &e.node, event(e.kind)));
        }

        fn on_stop() {
            with_app(|app, _| app.shutdown());
            APP.with_borrow_mut(|a| *a = None);
        }
    }

    export!(Flags);
}
