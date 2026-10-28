# Changelog: Flags

The release workflow moves `## [Unreleased]` into the new version's section. Add each change there.

## [Unreleased]

## [1.0.0-beta.1] - 2026-10-28

### Added
- The flag on an 8×8 matrix over PitFlag wire v2, for every sim PitCrew reads.
- Matrix settings (type, pins, color order, zigzag, mirror, rotate, current limit). The board keeps
  them, so a turned image is fixed without a new upload.
- The matrix test on the Device tab: six fixed screens that show if the color order, the wiring and
  the brightness are right, and which setting to change.
- LED color correction (on by default): the colors sent to a WS2812B lose some green and blue, so the
  yellow flag is not lime. The screen keeps the pack's colors.
- The standby screen: outside a race (no sim, or not in the car) the matrix plays the pack's
  `standby` animation. "Animate outside a race" turns it off.
- The link sends the whole state again every second, and writes one burst per tick, so a lost
  packet heals.
- A board with PitFlag 1.x firmware is named, with its version, and asks for the new firmware.
- The Classic pack: 15 animations and the flag mapping.
- The Flags screen with the Live, Animations and Device tabs.
- The animation editor, which saves to My pack.
- The PitFlag 2.1.0 firmware for WS2812B and MAX7219 matrices: one prebuilt image each for the
  ATmega328P, ATmega168, ATmega32U4 and ESP32. It shows a boot animation until PitCrew talks, and
  redraws only when the serial line is quiet.
