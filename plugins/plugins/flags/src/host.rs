//! What the plugin needs from PitCrew, as traits, so the logic runs in native tests.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Ok,
    Idle,
    Warning,
    Error,
}

pub trait Port {
    fn write(&mut self, bytes: &[u8]) -> Result<(), String>;
    fn read(&mut self, max: u32) -> Result<Vec<u8>, String>;
}

pub trait Host: Firmware {
    fn log(&mut self, level: LogLevel, text: &str);
    fn set_status(&mut self, level: StatusLevel, text: &str);
    fn invalidate_ui(&mut self);
    fn open_port(&mut self, setting_key: &str, baud: u32) -> Result<Box<dyn Port>, String>;
    fn settings_json(&mut self) -> String;
    fn content_packs(&mut self, of_type: &str) -> Vec<String>;
    fn content_read(&mut self, pack: &str, path: &str) -> Result<Vec<u8>, String>;
    fn content_write_own(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
}

/// What PitCrew says about the board on the device port.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardInfo {
    /// "usb-jtag", "native-usb", "bridge" or "unknown".
    pub transport: String,
    /// A chip id such as "esp32c3", when known.
    pub chip: Option<String>,
    pub usb_name: String,
    pub detail: String,
}

/// A detection or upload PitCrew runs for the plugin.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FirmwareJob {
    pub upload: bool,
    pub running: bool,
    pub progress: f32,
    pub text: String,
    pub error: Option<String>,
    pub board: Option<BoardInfo>,
}

/// PitCrew's board detection and firmware upload (`pitcrew:plugin/firmware`).
pub trait Firmware {
    fn firmware_describe(&mut self, setting_key: &str) -> Result<BoardInfo, String>;
    fn firmware_probe(&mut self, setting_key: &str) -> Result<(), String>;
    fn firmware_upload(&mut self, setting_key: &str, chip: &str, image: &[u8]) -> Result<(), String>;
    fn firmware_job(&mut self) -> Option<FirmwareJob>;
}
