//! The smallest useful PitCrew plugin: one sidebar screen that shows the flag and a button.
//! Walked through line by line in docs/getting-started.md.

use std::cell::RefCell;

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

// The world's own types (FlagSignal, SessionInfo, UiNode, UiEvent) are already in scope.
use pitcrew::plugin::types::{Flag, StatusLevel};
use pitcrew::plugin::ui::{ButtonProps, ButtonVariant, CardProps, EventKind, NodeKind, PageProps, TextProps, Tone};
use pitcrew::plugin::{host, settings};

#[derive(Default)]
struct State {
    flag: Option<Flag>,
    connected: bool,
    presses: u32,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn node(id: &str, parent: Option<&str>, kind: NodeKind) -> UiNode {
    UiNode { id: id.into(), parent: parent.map(Into::into), kind }
}

fn flag_name(f: Flag) -> &'static str {
    match f {
        Flag::Disqualify => "Disqualified",
        Flag::Black => "Black",
        Flag::Repair => "Repair",
        Flag::Red => "Red",
        Flag::Checkered => "Chequered",
        Flag::White => "White",
        Flag::Yellow => "Yellow",
        Flag::Blue => "Blue",
        Flag::Green => "Green",
    }
}

fn greeting() -> String {
    let json: serde_json::Value = serde_json::from_str(&settings::get()).unwrap_or_default();
    json["greeting"].as_str().unwrap_or("Hello").to_string()
}

struct Hello;

impl Guest for Hello {
    fn init() -> Result<(), String> {
        host::set_status(StatusLevel::Ok, "Ready");
        Ok(())
    }

    fn on_flag(signal: FlagSignal) {
        STATE.with_borrow_mut(|s| s.flag = signal.primary);
        host::invalidate_ui();
    }

    fn on_session(session: SessionInfo) {
        STATE.with_borrow_mut(|s| s.connected = session.connected);
        host::invalidate_ui();
    }

    fn settings_changed() {
        host::invalidate_ui();
    }

    fn tick(_now_ms: u64) {}

    fn on_action(_id: String) {}

    fn render() -> Vec<UiNode> {
        STATE.with_borrow(|s| {
            let flag = match (s.connected, s.flag) {
                (false, _) => "No sim connected".to_string(),
                (true, None) => "No flag".to_string(),
                (true, Some(f)) => flag_name(f).to_string(),
            };
            vec![
                node("page", None, NodeKind::Page(PageProps { title: "Hello".into(), subtitle: Some(greeting()) })),
                node("card", Some("page"), NodeKind::Card(CardProps { title: Some("Flag".into()), description: None })),
                node("flag", Some("card"), NodeKind::Text(TextProps { text: flag, tone: Tone::Normal, mono: true })),
                node(
                    "wave",
                    Some("card"),
                    NodeKind::Button(ButtonProps {
                        label: format!("Wave ({})", s.presses),
                        variant: ButtonVariant::Secondary,
                        icon: Some("hand".into()),
                        disabled: false,
                    }),
                ),
            ]
        })
    }

    fn on_ui_event(event: UiEvent) {
        if event.node == "wave" && matches!(event.kind, EventKind::Pressed) {
            STATE.with_borrow_mut(|s| s.presses += 1);
            host::invalidate_ui();
        }
    }

    fn on_stop() {}
}

export!(Hello);
