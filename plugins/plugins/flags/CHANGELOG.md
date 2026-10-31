# Changelog: Flags

The release workflow moves `## [Unreleased]` into the new version's section. Add each change there.

## [Unreleased]

### Changed
- The Live tab leads with the matrix. Beside it are what the sim shows, what the matrix plays and
  the flag player. The Device card stays under it, whether the board is fine, missing or needs an
  update, so it no longer jumps. The header says only **Connected**; the card names the chip and the
  firmware.
- The Animations tab puts playback and the flags you allow on the left, and the pack and the editor
  on the right. The animation editor groups its controls: painting, frames and speed, then save.
- **Detect board** only says which chip the board has. A matching chip no longer counts as a yes
  for the upload, because a button box can carry the same chip: the Upload step asks with a
  **Yes, COM… is my matrix board** button, and says that Detect board restarts the board.
- A matrix test that paused after two minutes goes on from the screen it was on.
- The Test step says in one line what it is for. Live names the sim PitCrew reads instead of always
  saying iRacing.

### Added
- A safer upload. When nothing on the port answers as PitFlag, the Upload step asks whether that
  port is your matrix board before it writes, so a button box or wheel on the wrong port is not
  overwritten. **Detect board** answers it too. While it writes, the Upload button stays, as
  **Uploading…**, under a notice to keep the board plugged in, with how long it takes.
- The animation editor takes PitCrew's palette: pick a color and paint with it, drag to paint,
  right-click to turn a cell off, fill or clear the frame, and change a color or add one (up to 16).
  These changes count as unsaved edits and come back after a restart like the others.
- Built for plugin API 2 (`pitcrew:plugin@2`).
- When Windows gives the matrix a new COM port and PitFlag answers there on the board you set up,
  the setup offers **Use COM…** to take the port in one click. A set-up matrix can always leave the
  setup with **Cancel**, and only a new port opens the setup again.
- Fill, Clear and Delete frame in the editor can be undone once. **Discard changes** asks before it
  throws the changes away. **Play on the matrix** says why it is off when the board is not
  connected.
- An upload that is still running when the plugin restarts comes back on the Upload step with its
  progress, and the port stays PitCrew's until it ends. Changing the port mid-upload no longer
  takes the old port back early, and the Board step locks the port and board while PitCrew looks at
  the board.
- When the board does not answer 20 seconds after a good upload, the Upload step says so and tells
  you to unplug and replug it, or upload again, instead of waiting without end. A port that just
  took PitFlag is not asked about again.
- After a failed upload, the board's own tip (such as the double reset on a Leonardo) shows again.
- The setup step and the editor's unsaved changes come back after PitCrew restarts.
- **Test the matrix** from the device card is a test of its own, with **Done**, not the setup's last
  step. Changing a setting restarts the test's two-minute timer.
- Finishing the setup plays green, then checkered, on the matrix.
- The Matrix step asks only for the matrix, its pins and the current limit. Color order, LED
  correction and rotation are in Test, where you can see them.
- Brightness is on the device card. After a save to My pack, the pack picker is right under the
  message. Live names the flag and the animation in plain words, and keeps drawing the preview when
  the board is unplugged.
- The Flags screen. Until a device is set up, it shows only the setup: Board (port and board type),
  Matrix (type, pins, current limit), Upload (checks the
  settings, then writes the firmware) and Test (the matrix test). **Finish setup** unlocks only on
  the last test screen, with the board connected.
- After the setup, the screen shows the device's connection, with **Test the matrix** and **Set up
  again**, and two tabs: Live and Animations. A new port opens the setup again.
- The setup blocks a pin the board cannot use (a flash pin on an ESP32, a pin the board does not
  have) and two MAX7219 lines on one pin, and warns above 500 mA on USB power.
- The Live tab is one card: the matrix as it is now, the flag the sim shows and the flag the matrix
  plays. The device card shows the connection only when it needs attention; the screen's header
  shows it the rest of the time.
- Turn each flag on or off on the Animations tab, in four groups: Start, Caution, Race, and Stop and
  penalties. While a flag that is off is out, the matrix stays dark. A test still plays it.
- The animation editor can delete a frame and discard unsaved changes. Buttons that cannot act are
  off: Previous and Next on one frame, Duplicate at 120 frames, Play with no board connected. A save
  note goes away at the next edit, and an animation whose id is not a plain file name is not saved.
- The Animations tab names each animation the way the flag switches do ("Yellow, waving", not
  `yellow-waving`), in the table, the editor and its messages, and lists them in that order.
- The Live matrix refreshes the screen only while the Live tab is open.
- The animation editor, which saves to My pack. A save keeps the animations saved before, the
  matrix picks up the change at once when My pack is in use, and the editor says how to switch to it
  when it is not.
- Picking another animation while the editor holds unsaved changes asks first: save and open, discard,
  or keep editing.
- A failed firmware upload offers **Try the upload again** and says the board is not damaged. After a
  good upload, the step says it waits for the board to restart.
- The Live tab shows a warning when the board is not connected.
- Settings use the right control: brightness, current limit and green time are sliders (brightness in
  percent), matrix type, color order and rotation are button rows, and pins are pin fields that
  fall back to the board's default pin when left empty. Each setup field says what it is for.
- Simpler screens: the orientation test has the Zigzag and Mirror switches, so the whole setup stays
  on the Flags screen; old firmware gets an **Update to PitFlag** button that opens only the upload;
  the upload step shows one line when your settings check out; Live plays a flag from a list with
  one **Play** button; the animation table drops its On/Off column, which the switches already show.
- Setup reads at a glance: a progress bar with the step you are on, Back and the next step always in
  the same place, a clear "PitFlag is running" message after the upload, and no greyed-out button on
  the last test screen.
- The Now card on Live reads as two plain lines instead of a table with empty headers.
- The PitFlag 2.2.0 firmware for WS2812B and MAX7219 matrices: one prebuilt image each for the
  ATmega328P, ATmega168, ATmega32U4, ESP32 and ESP32-C3. It shows a boot animation until PitCrew
  talks, and redraws only when the serial line is quiet.
- ESP32-C3 boards, on the chip's own USB port. The default data pin is GPIO 4.
- Firmware upload in the setup for every board: ESP32-C3, ESP32, Arduino Uno and Nano
  (ATmega328P and ATmega168) and Leonardo or Micro (ATmega32U4). Pick the board and press **Upload**.
  PitCrew restarts the board into its bootloader, writes the firmware the plugin carries and checks
  it. No button to hold, no Arduino IDE.
- **Detect board** in the setup asks the board which chip it is, and says which Board to pick.
  The step also names the port's USB adapter.

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
