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

pub trait Host {
    fn log(&mut self, level: LogLevel, text: &str);
    fn set_status(&mut self, level: StatusLevel, text: &str);
    fn invalidate_ui(&mut self);
    fn open_port(&mut self, setting_key: &str, baud: u32) -> Result<Box<dyn Port>, String>;
    fn settings_json(&mut self) -> String;
    fn content_packs(&mut self, of_type: &str) -> Vec<String>;
    fn content_read(&mut self, pack: &str, path: &str) -> Result<Vec<u8>, String>;
    fn content_write_own(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
}
