//! The boards PitFlag runs on and the firmware images this plugin carries. PitCrew recognises the
//! board and writes the image (`pitcrew:plugin/firmware`); the plugin only hands it over.

use pitflag_wire as wire;

/// The firmware version the images in `firmware/dist/` carry. It must match `firmware/pitflag/config.h`.
pub const VERSION: [u8; 3] = [2, 2, 0];

/// The ESP images are merged images for flash address 0, built by `firmware/build.ps1`.
static ESP32C3_IMAGE: &[u8] = include_bytes!("../firmware/dist/pitflag-esp32c3.bin");
static ESP32_IMAGE: &[u8] = include_bytes!("../firmware/dist/pitflag-esp32.bin");
static ATMEGA328P_IMAGE: &[u8] = include_bytes!("../firmware/dist/pitflag-atmega328p.hex");
static ATMEGA168_IMAGE: &[u8] = include_bytes!("../firmware/dist/pitflag-atmega168.hex");
static ATMEGA32U4_IMAGE: &[u8] = include_bytes!("../firmware/dist/pitflag-atmega32u4.hex");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Board {
    Esp32c3,
    Atmega328p,
    Atmega168,
    Atmega32u4,
    Esp32,
}

impl Board {
    pub const ALL: [Board; 5] = [Board::Esp32c3, Board::Atmega328p, Board::Atmega168, Board::Atmega32u4, Board::Esp32];

    /// The `board` setting. An unknown value is the ATmega328P, the setting's default.
    pub fn from_id(id: &str) -> Board {
        Board::ALL.into_iter().find(|b| b.id() == id).unwrap_or(Board::Atmega328p)
    }

    /// Both the `board` setting's value and PitCrew's chip id.
    pub fn id(self) -> &'static str {
        match self {
            Board::Esp32c3 => "esp32c3",
            Board::Atmega328p => "atmega328p",
            Board::Atmega168 => "atmega168",
            Board::Atmega32u4 => "atmega32u4",
            Board::Esp32 => "esp32",
        }
    }

    pub fn chip(self) -> wire::Chip {
        match self {
            Board::Esp32c3 => wire::Chip::Esp32c3,
            Board::Atmega328p => wire::Chip::Atmega328p,
            Board::Atmega168 => wire::Chip::Atmega168,
            Board::Atmega32u4 => wire::Chip::Atmega32u4,
            Board::Esp32 => wire::Chip::Esp32,
        }
    }

    pub fn image_name(self) -> &'static str {
        match self {
            Board::Esp32c3 => "pitflag-esp32c3.bin",
            Board::Atmega328p => "pitflag-atmega328p.hex",
            Board::Atmega168 => "pitflag-atmega168.hex",
            Board::Atmega32u4 => "pitflag-atmega32u4.hex",
            Board::Esp32 => "pitflag-esp32.bin",
        }
    }

    pub fn image(self) -> &'static [u8] {
        match self {
            Board::Esp32c3 => ESP32C3_IMAGE,
            Board::Atmega328p => ATMEGA328P_IMAGE,
            Board::Atmega168 => ATMEGA168_IMAGE,
            Board::Atmega32u4 => ATMEGA32U4_IMAGE,
            Board::Esp32 => ESP32_IMAGE,
        }
    }

    /// What the driver does before pressing Upload.
    pub fn hint(self) -> &'static str {
        match self {
            Board::Atmega32u4 => "PitCrew restarts the board into its bootloader, which shows up as another port for a few seconds. If the upload cannot find it, press the board's reset button twice quickly and upload again.",
            _ => "PitCrew restarts the board into its bootloader and writes the firmware. Keep the board plugged in until the upload ends.",
        }
    }
}

/// A chip id PitCrew reported, as a board.
pub fn board_for_chip(id: &str) -> Option<Board> {
    Board::ALL.into_iter().find(|b| b.id() == id)
}

pub fn version_text(v: [u8; 3]) -> String {
    format!("{}.{}.{}", v[0], v[1], v[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_esp_images_are_merged_images_for_address_zero() {
        assert_eq!(Board::Esp32c3.image()[0], 0xE9, "the C3 bootloader sits at 0x0");
        assert_eq!(Board::Esp32.image()[0x1000], 0xE9, "the ESP32 bootloader sits at 0x1000");
        for b in [Board::Esp32c3, Board::Esp32] {
            let img = b.image();
            assert_eq!(img.len() % 4096, 0, "whole flash sectors");
            assert_eq!(img[0x10000], 0xE9, "the app at 0x10000 is included");
            assert!(img.len() < 1 << 20, "the 0xFF padding is trimmed");
        }
    }

    #[test]
    fn the_avr_images_are_intel_hex() {
        for b in [Board::Atmega328p, Board::Atmega168, Board::Atmega32u4] {
            let text = std::str::from_utf8(b.image()).unwrap();
            assert!(text.starts_with(':'), "{}", b.image_name());
            assert!(text.trim_end().ends_with(":00000001FF"), "{}", b.image_name());
        }
    }

    #[test]
    fn the_version_matches_the_firmware_source() {
        let h = include_str!("../firmware/pitflag/config.h");
        let get = |name: &str| -> u8 {
            h.lines().find_map(|l| l.strip_prefix(&format!("#define {name} "))).expect(name).trim().parse().expect(name)
        };
        assert_eq!(VERSION, [get("FIRMWARE_MAJOR"), get("FIRMWARE_MINOR"), get("FIRMWARE_PATCH")]);
    }

    #[test]
    fn every_board_is_a_pitcrew_chip_id_and_an_unknown_id_is_the_default() {
        for b in Board::ALL {
            assert_eq!(Board::from_id(b.id()), b);
            assert_eq!(board_for_chip(b.id()), Some(b));
        }
        assert_eq!(Board::from_id("nano"), Board::Atmega328p);
        assert_eq!(board_for_chip("esp32s3"), None);
    }
}
