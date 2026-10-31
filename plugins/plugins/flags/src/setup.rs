//! The device setup: its steps, what the plugin remembers about a finished setup, and the checks
//! that run on the matrix settings before an upload.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::app::Settings;
use crate::firmware::Board;
use crate::host::StatusLevel;

/// The file in the plugin's data folder that holds `Prefs`.
pub const PREFS_FILE: &str = "prefs.json";

/// The animation the editor holds unsaved, as an animation file. Empty when there is none.
pub const DRAFT_FILE: &str = "editor-draft.json";

/// Above this, a WS2812B matrix needs its own 5 V supply: a USB 2.0 port gives 500 mA.
pub const USB_MILLIAMPS: u16 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Board,
    Matrix,
    Upload,
    Test,
}

impl Step {
    pub const ALL: [Step; 4] = [Step::Board, Step::Matrix, Step::Upload, Step::Test];

    pub fn label(self) -> &'static str {
        match self {
            Step::Board => "Board",
            Step::Matrix => "Matrix",
            Step::Upload => "Upload",
            Step::Test => "Test",
        }
    }

    pub fn index(self) -> usize {
        Step::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn next(self) -> Step {
        Step::ALL[(self.index() + 1).min(Step::ALL.len() - 1)]
    }

    pub fn prev(self) -> Step {
        Step::ALL[self.index().saturating_sub(1)]
    }

    pub fn from_label(label: &str) -> Option<Step> {
        Step::ALL.into_iter().find(|s| s.label() == label)
    }
}

/// The board a finished setup was for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub port: String,
    pub board: String,
}

/// What the plugin keeps between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prefs {
    #[serde(default)]
    pub device: Option<Device>,
    /// The flag animations the driver turned off. The matrix stays dark while one of them is out.
    #[serde(default)]
    pub flags_off: BTreeSet<String>,
    /// The setup step on screen when the plugin stopped, by label. Only while no setup is finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup_step: Option<String>,
}

impl Prefs {
    /// A missing or unreadable file is a fresh install.
    pub fn parse(bytes: Option<&[u8]>) -> Prefs {
        bytes.and_then(|b| serde_json::from_slice(b).ok()).unwrap_or_default()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).expect("serialisable")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub level: StatusLevel,
    pub text: String,
}

impl Check {
    fn new(level: StatusLevel, text: impl Into<String>) -> Check {
        Check { level, text: text.into() }
    }

    pub fn blocks(&self) -> bool {
        self.level == StatusLevel::Error
    }
}

pub fn is_max7219(s: &Settings) -> bool {
    s.matrix == "max7219"
}

/// The matrix settings the Matrix step shows for the picked matrix.
pub fn matrix_keys(s: &Settings) -> Vec<String> {
    let keys: &[&str] = if is_max7219(s) {
        &["matrix", "dataPin", "clockPin", "csPin"]
    } else {
        &["matrix", "dataPin", "maxMilliamps"]
    };
    keys.iter().map(|k| k.to_string()).collect()
}

/// The board's default pins for the picked matrix, as a sentence.
pub fn default_pins_text(s: &Settings) -> String {
    let board = Board::from_id(&s.board);
    let p = board.default_pins();
    if is_max7219(s) {
        format!("Leave a pin empty to use the board's default: DIN {}, CLK {}, CS {}.", p.din, p.clk, p.cs)
    } else {
        format!("Leave the data pin empty to use the board's default: pin {}.", p.ws2812)
    }
}

/// The pins the firmware will drive, each with its name.
fn pins_used(s: &Settings) -> Vec<(&'static str, u8)> {
    let p = Board::from_id(&s.board).default_pins();
    if is_max7219(s) {
        vec![
            ("DIN", s.data_pin.unwrap_or(p.din)),
            ("CLK", s.clock_pin.unwrap_or(p.clk)),
            ("CS", s.cs_pin.unwrap_or(p.cs)),
        ]
    } else {
        vec![("Data", s.data_pin.unwrap_or(p.ws2812))]
    }
}

/// The checks on the matrix settings. An error blocks the upload.
pub fn matrix_checks(s: &Settings) -> Vec<Check> {
    let board = Board::from_id(&s.board);
    let pins = pins_used(s);
    let mut out = Vec::new();
    let mut bad = false;
    for (name, pin) in &pins {
        if let Some(why) = board.pin_problem(*pin) {
            bad = true;
            out.push(Check::new(StatusLevel::Error, format!("{name} pin {pin} {why}. Pick another pin.")));
        }
    }
    let distinct: BTreeSet<u8> = pins.iter().map(|(_, p)| *p).collect();
    if distinct.len() < pins.len() {
        bad = true;
        out.push(Check::new(StatusLevel::Error, "DIN, CLK and CS must be three different pins."));
    }
    if !bad {
        let list: Vec<String> = pins.iter().map(|(n, p)| format!("{n} {p}")).collect();
        out.push(Check::new(StatusLevel::Ok, format!("Pins: {}.", list.join(", "))));
    }
    if !is_max7219(s) {
        if s.max_milliamps > USB_MILLIAMPS {
            out.push(Check::new(
                StatusLevel::Warning,
                format!("{} mA is more than a USB port gives. Power the matrix from its own 5 V supply.", s.max_milliamps),
            ));
        } else {
            out.push(Check::new(StatusLevel::Ok, format!("Current limit: {} mA, safe on USB power.", s.max_milliamps)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(json: &str) -> Settings {
        Settings::parse(json)
    }

    #[test]
    fn the_defaults_pass() {
        assert!(!matrix_checks(&settings("{}")).iter().any(Check::blocks));
        assert!(!matrix_checks(&settings(r#"{"matrix":"max7219"}"#)).iter().any(Check::blocks));
        assert!(!matrix_checks(&settings(r#"{"board":"esp32c3"}"#)).iter().any(Check::blocks));
    }

    #[test]
    fn a_flash_pin_on_an_esp32_c3_blocks_the_upload() {
        let checks = matrix_checks(&settings(r#"{"board":"esp32c3","dataPin":12}"#));
        assert!(checks.iter().any(|c| c.blocks() && c.text.contains("flash")), "{checks:?}");
    }

    #[test]
    fn two_max7219_lines_on_one_pin_block_the_upload() {
        let checks = matrix_checks(&settings(r#"{"matrix":"max7219","clockPin":11}"#));
        assert!(checks.iter().any(|c| c.blocks() && c.text.contains("different")), "{checks:?}");
    }

    #[test]
    fn more_current_than_usb_is_a_warning_not_a_block() {
        let checks = matrix_checks(&settings(r#"{"maxMilliamps":1500}"#));
        assert!(checks.iter().any(|c| c.level == StatusLevel::Warning));
        assert!(!checks.iter().any(Check::blocks));
    }

    #[test]
    fn prefs_survive_a_round_trip_and_a_bad_file_is_a_fresh_install() {
        let mut p = Prefs::default();
        p.device = Some(Device { port: "COM4".into(), board: "esp32c3".into() });
        p.flags_off.insert("debris".into());
        assert_eq!(Prefs::parse(Some(&p.to_bytes())), p);
        assert_eq!(Prefs::parse(Some(b"not json")), Prefs::default());
        assert_eq!(Prefs::parse(None), Prefs::default());
    }
}
