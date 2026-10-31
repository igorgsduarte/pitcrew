# Changelog

Every notable change to PitCrew is recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**This file is the source of the release notes the driver reads.** It is mirrored verbatim into the
public repository ([igorgsduarte/pitcrew](https://github.com/igorgsduarte/pitcrew)) on every
release, and the section for a version becomes that release's body. Write entries for the driver,
not for the compiler: *"the spotter no longer calls a car that has already pitted"* rather than
*"guard `CarIdxOnPitRoad` in `spotter::alongside`"*.

Every pull request adds at least one entry under `## [Unreleased]`, in one of the four subsections
below — the CI gate rejects a PR that does not. The next version number lives in
[`.release/next.json`](.release/next.json), not here; cutting a release moves this section into a
dated one. See `docs/release-process.md`.

## [Unreleased]

### Added
- **The engine can run plugins.** A plugin adds something PitCrew does not do itself, such as a
  flag display on an LED matrix. It runs apart from the engineer: a plugin that crashes, hangs or
  uses too much memory is stopped and started again, and after three failures in ten minutes it stays
  off. The engineer keeps talking in every case. For plugin authors,
  `pitcrew-engine --headless --plugin-dev <folder>` runs a plugin and reloads it when it changes, and
  `--safe-mode` starts with no plugins.
- **Plugins can use USB serial devices.** A plugin opens only the port you pick in its settings, and
  two plugins never share one. PitCrew closes the port when the plugin stops or fails, so a
  device such as an LED matrix goes dark instead of freezing on the last frame.
- **Plugins remember their own choices.** A plugin can read back the files it saved, so the Flags
  plugin keeps your finished device setup and the flags you turned off between restarts.
- **Plugin settings are easier to undo and to pick.** Empty a number field to put the setting back
  to its default (for the Flags plugin, the board's own pin). An animation pack is now picked from
  a list of the installed packs, by name, so a pack you saved yourself can be chosen.
- **A device plugged in late shows up.** A plugin's port list looks again when you come back to the
  window, and has a button to look again now. With no device plugged in, it says so.
- **Plugin settings use sliders and button rows.** A plugin can ask for a slider, a percent slider or
  a row of buttons, add a unit and a one-line description, and offer a pin field for a board's
  GPIO pins.
- **A plugin's pixel editor has a palette.** Pick a color and paint with it, drag across cells,
  right-click to turn a cell off, fill or clear the frame, and change a color or add one with the
  system picker or a hex code.
- **A plugin's pixel editor works from the keyboard.** Tab into it, move with the arrow keys and
  paint with Enter or Space; a screen reader names each cell's row, column and color by name.
- **Plugin screens keep your place with the keyboard.** A button a plugin turns off while it works
  keeps keyboard focus, and the next step's heading takes focus when a setup moves on. The step
  forward always sits at the right of its row, a success reads as a success, and a progress bar
  tells a screen reader only each step and quarter instead of every update.
- **A plugin's pixel editor announces the brush.** The colors are one group you move through with
  the arrow keys, the eraser is called the eraser, and a read-only grid names its corner colors.
- **A firmware upload survives a plugin restart.** If a plugin restarts while PitCrew writes a
  board for it, the plugin finds the upload still running and shows its progress again, instead
  of losing track of it while the port stays busy.
- **A plugin's screen shows which button moves you on.** Its main action is drawn in the app's own
  accent, so a setup's Next or Upload stands out from Back; a step that is not open yet stays plain. A
  message's heading sits on its own line. A screen reader hears a plugin's progress
  and messages as they change, and a read-only pixel grid says what it shows.
- **A Plugins screen.** The new Plugins item in the rail lists the plugins on this PC. Turn each one
  on or off and change its settings there; the port setting lists the USB devices plugged in. A
  plugin that is on and has a screen of its own gets an item in the rail under Plugins, with a dot
  when it needs your attention. Plugins still arrive by hand for now: the catalogue comes later.
- **PitCrew uploads firmware to Arduino and ESP32 boards for a plugin.** A plugin that drives a
  device, such as the Flags plugin, can ask PitCrew to find out which board is on the port and to
  write the plugin's firmware to it. PitCrew restarts the board into its bootloader by itself, so
  there is no button to hold and no Arduino IDE to install. It works with the ESP32-C3, the ESP32,
  the Arduino Uno and Nano, and the Leonardo and Micro. While it writes, it holds the port, and the
  plugin gets it back when the upload ends.
- A plugin is no longer stopped for being slow when Windows only paused it to run something else.
  Only the time the plugin really computes counts against its limit.
- **A field-of-view calculator for iRacing, Assetto Corsa and Assetto Corsa EVO.** Settings › Display
  › Field of view works out the correct field of view from your screen size, the distance from your
  eyes and, on triple screens, the side angle and bezels. Curved screens count their real curve.
  iRacing gets the horizontal view across all your screens, and PitCrew can write it with the
  matching monitor values, after you review them and with a backup of each file. Both Assetto Corsa
  games get the vertical view of one screen, to enter in the game. It also suggests the side angle
  at which your side screens face you, and draws your rig from above.
- **A new PitCrew is announced in a window of its own.** When a new version of PitCrew is ready and
  you are not driving, a dialog shows the version, the download size and a link to what changed.
  Download, then Restart to update, are still two separate answers, and "Later" puts it back in the
  corner panel until you next open PitCrew. It never appears while iRacing is running. New
  engineers, phrases and screens still arrive in the corner panel.
- **`ire-replay decide --language`.** The replay tool can now speak a recording in any catalog
  language, numbers included, instead of only in Portuguese. The parity corpus keeps an English
  record of one race, which the website's radio log reads.

### Changed
- **Plugins are built for plugin API 2.** A plugin made for the first plugin API does not load until
  its author rebuilds it; the Plugins screen says why. The Flags plugin is already rebuilt.
- **Plugin screens hold up when something goes wrong.** A plugin screen PitCrew can't draw no
  longer blanks the window: it says so and keeps the plugin's settings usable. A screen that never
  arrives gets a Try again button instead of waiting forever, and a plugin restarting after an error
  counts down to the restart. A double click on a plugin's button or action sends it once, and the
  port list picks up a device as soon as you plug it in.
- **Plugin screens are easier to scan.** A plugin's tabs sit under its title, its buttons line up in
  rows, its status lines and switch lists group together, and its tables are denser. On the Plugins
  screen the status, Open and the switch line up from row to row, and a long plugin name keeps its
  place at narrow window sizes.
- **Clearer messages on the Plugins screen and on plugin screens.** Errors say what failed and what
  to do next, a port in use names the plugin that holds it, and the Plugins screen drops its
  one-tab bar. It now mentions the rail only when more plugins have a screen than the rail can show.
- **The curved-monitor correction moved to Settings › Display.** It sits next to the new
  field-of-view calculator. It works as before; only the place changed.
- **PitCrew installs for your Windows user, and updates without asking for administrator rights.**
  The download is now a `-setup.exe` instead of an `.msi`, and it installs into your own user folder.
  When you accept a new PitCrew from the update panel, it closes, shows a small progress window and
  opens again. The first time, if your PitCrew came from the old `.msi`, Windows asks your permission
  once to remove it. Your settings, profiles and voices stay where they are.
- **Plugins are published next to PitCrew.** The releases page now also lists plugin releases, named
  "Plugin · …". The app's installer is in the newest release named "PitCrew v…". The plugin source and
  the guide for plugin authors are in the `plugins/` folder of the public repository.

### Fixed
- **A plugin with damaged files now says it can't start, instead of saying it isn't installed.**
  When no serial port is found, the settings now say to plug the device in, and the list picks it up
  on its own.
- **Plugin screens work better with a keyboard and a screen reader.** A plugin's button keeps focus
  after you press it, and the color editor puts focus back on the color you edited. A screen reader
  now says when a plugin fails, restarts or comes back, and reads a setting's error with that
  setting. Error text in plugin settings is easier to read, unlit pixels on a matrix show their
  outline, and with no device plugged in the port row says so in a place the keyboard reaches.
- **The phone and tablet show the Companion and PIT screens of the version you installed.** An
  installer could carry the screens from an earlier build, so the race screen on the phone kept its
  old layout after an update.
- **PitCrew opens again after it updates itself.** When you accepted a new PitCrew from the update
  panel, the window closed for the installer and did not come back. It now reopens when the install
  is done.

### Removed

## [0.8.6] - 2026-10-28

### Added

- **A briefing before you get in the car.** With a session loaded and you out of the car, the
  Companion shows the weekend's sessions and which one is running, the weather and track
  temperature, the grid from qualifying (or the field by iRating before there is one) around your
  slot, your car and setup, your best lap there and your saved pit plan. It works for practice and
  qualifying too, not only races. With the sim closed it shows your last race and the ones before it.
  The waiting-screen options in the Companion's settings are now the clock, how many recent races to
  list and how much of the grid to show.
- **Assetto Corsa EVO.** Open Assetto Corsa EVO and the engineer is listening, with nothing to
  install or set up: fuel, laps and lap times, your position, flags, pit lane, the session clock,
  and tyre temperatures and pressures. The spotter calls cars alongside from where every car is on
  track. After your first clean lap at a track, which PitCrew remembers for next time, the gaps to the
  cars ahead and behind are heard too. EVO does not share the other cars' names, positions or lap
  times, so calls that need those stay silent. Settings → Connections has an Assetto Corsa EVO card
  that says whether the game was found and is coming through, and whether the clean lap is still
  needed. With iRacing and both Assetto Corsas on one rig, PitCrew follows whichever one you are
  driving.
- **The other cars on Assetto Corsa.** PitCrew now puts a small app in Assetto Corsa's folder and
  switches it on, so the engineer hears the rest of the field: gaps, traffic, rivals and the
  undercut, as on iRacing. Settings → Connections has an Assetto Corsa card that says whether the
  app is installed, switched on and sending, and what to do when it is not, such as updating it or
  restarting the session. An Assetto Corsa started before the app was installed needs one restart.
- **Visual checks of the desktop app through the agent MCP.** A remote development agent can
  inspect the real PitCrew window, operate its controls through Orca, and receive screenshots
  directly to help verify interface changes.
- **Live pit plan (beta).** Your saved pit plan now follows the race. Every lap it works out again,
  from the fuel you are actually using, how many stops you need, where they fall and how much fuel
  each one takes. Each plan keeps a strategy profile, conservative, balanced or aggressive, that you
  pick on the PIT page or on the phone. The fuel of a stop that follows the plan adjusts on its own.
  Anything bigger is only proposed, on the PIT page and on the phone, and nothing changes until you
  accept it: one stop more, one stop less, a stop that moves, or a stop you can drop by saving a
  little fuel each lap. The plan also warns when a fuel amount you set by hand is no longer enough.
  It recovers the stops already made if PitCrew restarts mid-race. **Settings → Pit stops → Let the
  live plan act** is off by default. Turned on, the radio, the PIT page and the phone all read the
  same fuel figure, a stop that follows the plan is armed with the litres worked out at that moment,
  and the strategist tells you when the plan changes or an accepted fuel saving is not happening.
  **Settings → Data** shows how much of your driving history is kept for each car and track, and
  deletes it.
- **The Debrief reads like a timing sheet.** Pick a race at the top and step to the one before or
  after. A summary gives where you started and finished, your best and ideal lap, how many laps were
  within half a second of your best, your incidents and your time on pit road. A strip under it
  shows your lap times and your position through the race, with the stops and the cautions marked;
  click it to jump to that lap. The lap table colours each sector against your best and, in races
  recorded from now on, shows the gap to the car ahead, the car behind and the leader, and marks the
  caution laps. Stops lists the stops where the car stood in its box, each with the laps before it,
  the places won or lost and the time it cost; a drive-through, or leaving the pits before the
  start, is named for what it is. Sectors, Fuel and At this track complete it. Older debriefs still
  open, without the columns they never recorded.
- **Pit strategy is one sheet per plan.** The fuel calculator and the planner are now one set of
  race figures: race length, lap time, fuel per lap, starting fuel, tank, reserve, tyre life and the
  series' rules. Each says where its number came from, and what the race needs is highlighted. Under
  them are a chart of the fuel in the tank lap by lap and the plan as a table, with a row per stint
  (laps, fuel in, fuel used, fuel left, tyre age against tyre life) and a row per stop. Adding or
  editing a stop opens one window. You pick the tyres on a drawing of the car seen from above, see
  the tread left at your last stop at that track, and see how the stint splits and what the stop
  does to the fuel. Your saved plans sit in a column on the right, and pit presets have their own
  tab. When iRacing runs that car and track, the live plan sits above the sheet.
- **Generic plans.** A plan can now fit any car and track, such as "fuel only, tyres kept, 60
  minutes". Generic plans are listed under each car and track's own plans. **Use for this car and
  track** copies one there, sized to that race, and makes the copy the active plan, so the engineer
  arms its stops as usual. A stop with no window is called when the fuel runs short. Changing the
  generic plan later leaves the copies alone; using it again updates the copy.
- **Curved monitor correction (beta).** Settings → iRacing → Curved monitor works out iRacing's
  monitor values from your monitor's size, curve, eye distance and bezel gap. Your rig is on the
  left, with a status list right under it that says whether the correction is in this build,
  installed in the iRacing folder, blocked by another program's dxgi.dll (ReShade, for example),
  when it last ran in iRacing and with which values, and whether iRacing's settings match. On the
  right are the numbers iRacing needs, a view from above of your screens next to the ones iRacing
  is set up for now, the picture the correction sends to the screens, and iRacing's monitor
  values. If iRacing is set up for a different monitor, a warning next to those values gives both
  fields of view, because lines cannot line up until they match. One button, Apply to iRacing,
  saves the settings, installs or updates the small add-on in the iRacing install folder, and
  opens iRacing's monitor values for the usual review and backup; what it saved and installed is
  still shown while you review. The on/off switch, the install folder and Uninstall sit in a
  folded Install section under that button. Test mode covers all your monitors with two patterns.
  Horizon lines every 5° show the curve: when they look straight, the radius is right. A 3D room
  shows the seams: when its lines carry on across the gaps, the bezel gap is right. A small panel
  in the middle shows the radius, the eye distance and what to look for; H opens the full panel
  with buttons. The arrow keys adjust the radius and eye distance, Tab switches the pattern, and
  Ctrl+Shift+F10 compares with and without the correction, in test mode and while driving. If another program's dxgi.dll was there, it is kept as a backup and put back
  when you uninstall. The add-on is unsigned and has not been cleared for iRacing's anti-cheat, so
  use it in offline sessions until that is confirmed.

- **The garage briefing suggests your cold tyre pressures.** Once you have at least three similar
  runs in a car at a track, the briefing suggests a cold pressure for the front and rear, or for each
  tyre when the sides differ, and says how many runs it is based on. A similar run has the same
  compound, the same wet or dry track, and a track temperature close to today's. The suggestion
  aims for the hot pressures of your fastest of those runs. A run with a yellow, a reconnect or a
  missing lap is not counted. With too few runs, or on a fixed setup, no number is given. Nothing
  is changed in the car. The thresholds are in `tuning.json`'s `pressureEstimate` section.
- **Follow a rival from the Companion.** Tap a driver on the Companion's leaderboard and the engineer
  confirms it, then calls that car's lap times and the gap to it each lap, and tells you when it
  enters and leaves the pits. Tap the same row again to stop. The followed row is marked on every
  connected screen, and the choice carries over from practice to the race at the same event. Set
  `rivals.watchLapEvery` in `tuning.json` to hear the lap calls less often.
- **Gap calls say whether the rival has pitted.** Once the stops begin, the lines about the car ahead
  and behind end with *"esse carro já parou"* or *"esse carro ainda tem parada pra fazer"* when
  that car has made a different number of stops than you.
- **Who you'll come out behind.** When your stop is called for the next lap and the time you lose in
  the pits is known from a previous stop, the engineer estimates which car in your class you will
  rejoin behind, and by how much. With no measured pit loss it says nothing.
  `rivals.rejoinMaxGapS` in `tuning.json` sets how far back a car may be and still be named.
- **The engineer learns where to stay quiet.** After a complete clean lap in a car at a track,
  routine calls wait through braking and corners for the next straight. Spotter calls and urgent
  alerts still come through immediately. The learned zones are kept between sessions, and the
  waiting time and straight length are adjustable in `tuning.json`.
- **Tyre warnings compare with your own runs.** A fresh reading after a run is compared with your
  earlier readings in that car at that track, per tyre, compound and wet or dry conditions. The
  engineer says when temperatures are unusually high or low, or a hot pressure is low against your
  history, and stays silent until enough runs are recorded. These calls describe the last run,
  because iRacing does not continuously report hot tyre readings on track.
- **The engineer tells you where your lap time is going.** In practice and in a race, when one sector
  of a lap cost you clearly more than the others against your own best lap in that car at that
  track, you hear which sector and by how much (*"setor dois, cinco décimos acima da sua
  referência"*). It waits for a straight with nobody alongside, never speaks on consecutive laps, and
  stays quiet about a sector you spent behind another car, a lap with a yellow or a pit stop in it,
  and a best lap that is old, on another setup or in the other weather. Your best clean lap is kept
  between sessions, so from the second session at a track it can speak from the first lap. The
  thresholds are in `tuning.json`'s `sectorCoaching` section.
- **Prepare your iRacing settings before entering the simulator.** Settings → iRacing edits the
  values in your existing INI files, loads selected values from an imported INI or from your own
  named presets, and shows every change for review before saving. A complete backup of each file is
  kept before it is replaced, and History restores selected values or a whole file. Editing is
  blocked while iRacing is running, and starting iRacing discards unsaved changes.
- **The spotter tells you a car is still there.** A car that stays alongside is called again every
  three seconds (*"continua na esquerda"*) until it clears, so you are never left wondering whether it
  has gone. Set `spotter.stillIntervalMs` in `tuning.json` to change how often, or to `0` to turn it
  off.
- **Inside and outside on ovals.** On an oval the spotter says *"carro por dentro"* and *"carro por
  fora"* instead of left and right. Road courses keep left and right.
- **Three wide tells you where you are.** With a car on each side you hear *"você no meio"*, and with
  two cars on one side you hear which side you are on, instead of where the two other cars are.
- **Change your pit stop by voice.** Say *"coloca vinte litros"*, *"abastece até o fim"*, *"troca os
  quatro pneus"*, *"só os dianteiros"*, *"sem pneus"*, *"reparo rápido"* or *"tear off"* and the engineer
  sets it in the car, then tells you what the car now shows. If iRacing caps the fuel to what fits
  in the tank, you hear the capped amount. Each command changes only what you asked for, and an
  automatic arm from your pit plan never overwrites it: the plan waits until you have made the stop.
  A command is only obeyed when it is heard in full, and never when you said *"não"* in the middle.
- **Fifteen more questions you can ask by voice.** Ask for the gap to the leader, the last lap of the
  car ahead, the car behind or the leader, your best lap, the time left, how much fuel you need to
  finish, how many laps you have done since your stop, your tyres, your repair time, your incidents
  and how many are left before the limit, your iRating, your licence and safety rating, the strength
  of field, and the track and air temperature. Tyre wear is the reading from your last stop, because
  iRacing only reports it in the pit box. *"Quem está na frente?"* and *"quem tá atrás?"* now
  work too, and answer with the name and the gap. The questions you could already ask are understood
  exactly as before.
- **A briefing before every session.** When you sit in the garage, or in your pit box before you pull
  away, the engineer goes over the track and air temperature and whether the track is wet. From the
  second session on, you hear only what changed: a warmer or cooler track, more rubber down, a wetter
  or drier surface, and which way to move your starting pressures. If nothing changed, the engineer
  says so in one line. The briefing stops as soon as the car moves, and it is never repeated for the
  same session, even after a restart. Turn it off under Settings → Race engineer.
- **The weather is followed over time.** The engineer now tells you when the rain is getting heavier,
  when the track is getting wetter, when race control declares the track wet, and when the track has
  warmed or cooled by several degrees since the start of the race, which changes your tyre pressures.
- **Every run's tyre pressures are kept.** After each run PitCrew saves the cold and hot pressures,
  tyre temperatures and tread with the track conditions, next to your lap history. This is the data
  a future pressure recommendation will learn from. Nothing is sent anywhere.
- **High oil temperature is called.** When the car's oil temperature warning comes on, the engineer
  now tells you, the same way it already did for the water temperature.
- **An early warning when the engine runs hot.** The engineer learns what water and oil temperature
  are normal for your car in the current session, then tells you when either climbs well above that,
  before iRacing's own warning light comes on. The classic case is a long stint in another car's
  tow: you hear the call in time to pull out and let the engine breathe. It stays quiet for the first
  15 minutes in the car while the engine warms up and it learns your normal running.
- **Ask "how is the engine?"** Press push-to-talk and ask about the engine, the water or the oil, and
  the engineer answers with the water and oil temperatures and the oil pressure, in your sim's units.
- **Where you line up behind the pace car.** On the formation lap and when a caution forms, the
  engineer tells you whether it is single or double file and where you are: your place in the line,
  or your row and which side of the grid. At one lap to green, that call includes your current row.
  You also hear when you get the free pass, when you are waved around, and when you are sent to the
  end of the line. Races only.
- **Telemetry health on the Companion.** The Companion's settings drawer now shows how many
  telemetry frames were thrown away, how many times PitCrew lost and found the sim again, and how many
  times it re-read the session details. If a session sounded wrong, include these numbers when you
  report it.
- **"Are you OK?" after a big hit.** When the car takes a heavy impact, the engineer checks on you
  straight away, then tells you about any damage once the car reports it. Kerbs and hard braking do
  not set it off.
- **DRS and push-to-pass calls.** In a race, on cars that have them, the engineer tells you when DRS
  is enabled for the next zone, and after each push-to-pass how many you have left. Cars without
  them hear nothing new.
- **The race briefing names the start.** Before a race with a standing start or a short parade lap,
  the engineer says so right after the briefing opens. A normal rolling start is not mentioned.
- **The pit planner follows a mandatory compound change.** When the race requires a change of tyre
  compound, the Pit strategy screen turns the rule on by itself, the proposal puts all four tyres on
  the first stop and says that is where the compound changes, and a plan where no stop picks a
  compound gets a warning. You can still switch the rule on or off by hand.
- **A warning when the planned stop doesn't take in the car.** If the engineer sets your stop and
  the car's pit menu still shows something else a few seconds later, you hear it and can check.
- **The engineer tells you when the car ahead retires**, by name, once.
- **The overtake call works from the first laps of a race.** Until there are three race laps to
  compare, the engineer compares your qualifying lap with the car ahead's.
- **Re-running one of iRacing's own `.ibt` telemetry files now includes the gap calls.** Those files
  do not record the other cars, so the engineer says "the car ahead" or "the car behind" instead
  of a name when you are catching someone or being caught.
- **A warning before the pit-road line when you are coming in too fast.** Approaching the pits,
  the engineer works out the speed you will cross the line at from how hard you are braking, and
  tells you to brake while there is still room. It learns where pit road starts from your first
  stop of the event, so it speaks from the second one on.

### Changed

- **The Companion opens a race on a pit wall.** The phone and tablet screen for a race now looks
  like the pit walls drivers already know: a black screen with outlined panels, the session along
  the top (position and places gained, strength of field, time left, lap, incidents, session best),
  the two cars ahead and the two behind with their gap, last and best laps and whether the gap is
  growing or shrinking each lap, and your car and lap times along the bottom, with traction
  control, ABS and brake bias each in its own colour. Down the right is the next stop: the fuel
  going in as the biggest number on the screen, the lap to box, the pit window, which tyres and
  repairs are armed, what the stop costs and the two cars you will come out between. Your race
  screen and font switch to the new ones once after the update; practice, qualifying and your
  other settings stay as you left them, and the old arrangement is still in the settings drawer.
- **On Assetto Corsa, sector times start on the first lap at a track you have driven before.** The
  sector lines PitCrew works out on your first lap at a track are now remembered, so the next session
  there has sector times and sector calls from the start instead of after a full lap. If the game's
  sector lines have moved, the ones from the new lap replace them.
- **Assetto Corsa started as `acs_pro.exe` is recognised.** PitCrew now listens to that build of the
  game the same way it listens to `acs.exe`. Competizione is still never read as Assetto Corsa.
- **Pit-stop timing uses your car's real refuelling speed.** The engineer times how fast the crew
  fills the tank at your stops and uses it for the pit loss and the stop calls, and remembers it for
  the next time you race this car at this track. The debrief shows the rate for each stop.

- **The phone Companion reads like a timing sheet.** Qualifying now opens on the lap in progress
  drawn the way an engineer's timing sheet is: one cell per sector that fills in as you cross it,
  the delta to your best owning the top of the sheet, then one ruled line per time of the session
  (predicted, last, best, session best, ideal, personal best) with the class order beside it. A
  sector or lap lost to track limits stays on the sheet, struck through and stamped OFF TRACK,
  instead of vanishing. The whole Companion takes the sheet's palette, bone ink on a carbon copy at
  night and on paper by day, its condensed capitals over tabular figures as the default font pair,
  and squarer corners. Your font pair, accent and arrangements are kept as you set them.
- **The engineer's voice now matches the moment.** Every call is said in one of five tones. Urgent
  calls, like the spotter, "box, box" or a waved yellow, come faster and tighter. Alerts are a little
  quicker. Briefings and debriefs are calmer, and a personal best or a strong finish sounds pleased.
  Routine information is unchanged. With the studio voice the tone changes its expressiveness too.
  Lines that should be called out now end that way, like "Box, box!" and "Manda tudo!". Routine
  engineer lines no longer arrive on a fixed beat. Each waits a random moment of up to a quarter of a
  second. Each tone, and that wait, can be adjusted in `tuning.json`. Toned lines are new audio, so
  they are synthesized again when first said or with *Warm cache*. With the studio voice, that is billed.
- **The engineer speaks each language like a native.** The lines in Portuguese, English, Spanish,
  French and German were rewritten in the words racers of that language use on the radio, instead of
  reading like a translation. Spanish is neutral Latin American Spanish, English uses iRacing's own
  words (pit road, caution, checkered), and no line assumes another driver is a man: rivals are called
  by name or as *"esse carro"*. Sentences around numbers and positions now read correctly at every
  value, such as one car or one lap. The start-of-race line about the field no
  longer calls every grid strong; it gives the average iRating. Changed lines are new audio, so with
  the studio voice they are said in the local voice until their studio audio is fetched in the
  background, or all at once with *Warm cache*, which is billed.
- **The engineer talks like a Brazilian on the radio.** The Portuguese lines now sound like a friend in
  your pit box rather than a polite announcement: short sentences, more exclamations and questions,
  *"pô"*, *"eita"*, *"bora"* and *"hein"*, and *gasolina* instead of *combustível*. Nothing crude, no
  regional slang, and still no line that assumes another driver is a man. Answers to the questions
  you ask by voice stay direct, and the other languages are unchanged. Changed lines are new audio:
  the local voices pick them up on their own, and with the studio voice they are said in the local
  voice until their studio audio is fetched in the background, or all at once with *Warm cache*,
  which is billed.
- **The iRacing tab has controls instead of text boxes.** It now opens on *Essentials*: the
  settings most drivers change, such as the frame-rate limit, anti-aliasing, cars drawn, detail
  levels, mirrors, monitor setup, field of view, volumes, the iRacing spotter and autofuel. They use
  iRacing's names, with switches, lists and sliders that show units. *Advanced* still lists every
  value in every INI. Each value there now gets a switch, list, slider or number field based on
  iRacing's own note for that line, so a typo can no longer be saved. Text boxes remain only for text,
  such as chat macros. Values iRacing manages itself, such as audio device IDs, are read only.
  The review before saving reads the way you set each value ("Field of view 141° → 143°"). Each
  changed setting has its own Revert, and discarding everything asks first. Turning off Live
  telemetry, which silences your race engineer, asks first too. Backup copies of the INIs no longer
  appear as files to edit. Settings are grouped in tabs down the left side and laid out in two
  columns, so a whole group, such as Performance, fits on one screen.
- Preparatory work for driver-specific adaptive strategy history. Current fuel advice and pit
  commands only change when you let the live pit plan act (above). The recording backend now
  supports explicit engine-side start/stop, bounded storage, protected references and confirmed
  deletion by driver/combo. Your laps are now recorded automatically whenever you drive in a live
  iRacing session; only clean laps close to your usual consumption are remembered, and
  `adaptiveRecording.autoStart` in `tuning.json` turns recording off. Remembered laps only count
  for the same setup file and a track within 10 °C of today's.
  An SDK diagnostic (`ire-dump --adaptive`) checks driver identity and data freshness. In a team
  race only your own stints are recorded, never a teammate's. While recording, the engineer also
  notes, lap by lap, what its current fuel figure and the new history-based estimate predicted, so
  the two can be compared before the new one is trusted, including how much fuel each expected
  to be left at your stops and at the flag; neither changes what it says or does.

- The spotter now watches every frame iRacing sends, not one in three, so a car is called as soon as
  it has really been alongside long enough, and a car that only brushed alongside for an instant is
  no longer mistaken for one that stayed.
- Temperatures and pressures are spoken in the units your sim is set to, and numbers above 99 are
  now spoken as words in every language.
- The engineer can now be tested against the `.ibt` telemetry files iRacing saves for your own
  sessions, not only against PitCrew's own recordings, so a problem you report can be checked on the
  session where it happened. No visible change in the app.
- The garage briefing can now be checked against a recorded session like every other call, and
  checking the same recording twice gives the same result. No visible change in the app.
- Every change to what the engineer says is now checked against a set of recorded race situations
  (a formation lap, a caution, a multiclass race, a driver disconnecting, a timed finish and a reset
  practice) before it ships. No visible change in the app.
- The closing rate the dash server sends with each leaderboard row is measured per second of
  session time, so it stays right at any dash update rate. Above five updates a second it used to
  read several times too fast.
- With several phones and tablets connected, the dash server builds each update once and sends
  the same one to every screen, instead of building it again for each screen, so another screen
  adds almost no work on the PC.
- Behind the scenes, the race engineer's code was tidied so it is easier to maintain: its tests sit
  beside the code they check, the part that reads iRacing's memory explains what it relies on, one
  shared way of recovering after an internal error replaces dozens of copies, `tuning.json` is read
  straight into the settings of each part of the engineer, the pit stops the engineer, the debrief
  and the pit history keep are all worked out in one place, the whole field's positions are read
  once per update instead of once per thing that looks at them, and when each line may be said
  (before the start, under a caution, in the pits) is now written beside the line in the phrase
  catalog. Nothing you hear or see changes.
- **Groundwork for new calls from telemetry PitCrew does not read yet.** A developer tool now
  measures which channels of an iRacing telemetry file change while you drive and which change only in
  the pits, so a future call is built only on data that is up to date on track. Nothing you hear or
  see changes.
- **The engineer warns you before an incident penalty, not only before disqualification.** When a
  series gives a penalty at a set incident count (17x, for example), you hear that it is close and
  that one more point brings it. The debrief's lap sheet now names what each incident was: a car
  contact, a contact with the wall, an off-track or a loss of control.
- **A second hit on an already damaged car counts as new damage** in your driving history, so the
  laps after it no longer pass for the car you had before.
- **A timed race has a length from the green flag.** Before your first timed lap, the strategist and
  a new pit plan use iRacing's own estimate of a lap for this car and track, so the fuel to finish
  and the stop count no longer wait for a lap. Your own laps replace it as soon as there are some.
  A lap far shorter than that estimate, such as a spin across the line, no longer counts toward the
  fuel average or your driving history.
- **Your driving history tells setups apart by their settings, not their file name.** Two setup
  files with the same values now share their history, and a setup changed by even one click starts
  its own. The tyre wear, temperatures, fuel load and in-car dials a race changes do not count. Laps
  recorded before this update were kept under the file name, so each car and track's history starts
  again once.
- **Your driving history tells car builds and balance of performance apart.** Laps recorded after
  iRacing updates the car, or after the series changes its weight or power, no longer count as the
  same car as the laps before when PitCrew learns your fuel use.

### Fixed

- **A new pit plan no longer replaces the active one.** Saving a plan you had just created on the Pit
  strategy screen wrote its stops over the active plan for that car and track, under the active
  plan's name. It is now saved beside it.
- **The fuel per lap no longer drops after you spin across the start line.** iRacing can count a
  lap, and sometimes take it back, while the car spins over the line. That "lap" of a few seconds
  went into the fuel average and made the laps left in the tank look much better than they were,
  for several laps. A lap now counts only once the car has been seen past halfway.
- **The live pit plan learns your fuel use again after a small contact.** A knock that left
  optional repairs pending marked every lap after it as damaged until the flag. The plan then
  ignored the whole race. Now only mandatory repairs, the meatball flag, or new damage on that lap
  keep a lap out.
- **The live pit plan no longer loses laps at random.** iRacing sometimes repeats its session clock
  for a frame, and PitCrew threw away the lap in progress each time. In a two-hour race that was
  about one lap in five.
- **In a series that limits how full the tank may be, the live pit strategy now plans with the tank
  the series allows.** The radio, the PIT screen and the Companion counted the car's whole tank, so
  the laps per tank, the stop window and the top-up could disagree with the plan you made on the
  desktop.
- **Pit history keeps counting stints after PitCrew restarts in the middle of a session.** Laps
  recorded after a restart were filed as the first stint again, which mixed up the stint and tyre
  figures for the rest of that session. A stop made while PitCrew was closed is still not seen.
- Three of `tuning.json`'s error messages (for `traffic.clearS`, `traffic.imminentS` and
  `engineTemps.baselineMinS`) no longer have a long run of spaces in the middle.
- **After installing an update, PitCrew now switches to the new race engineer without a restart.**
  Before, the window kept talking to the previous version's engineer when it was still running, so
  newer screens such as the iRacing settings tab said the engine did not support them until PitCrew
  was restarted. If a session is live when PitCrew opens, the switch waits until you leave the car.
  The same fix lets a downloaded engine update take effect straight away. Before, it usually waited
  until the next launch.
- **Clicking Install on an update no longer freezes the window.** PitCrew checks whether you are in a
  session before it installs, and the window used to stop responding until the answer came back.
- **Kokoro and Piper work on a fresh install.** Both voices need espeak-ng's pronunciation data,
  and the installer did not include it, so on most PCs they could not say a line. PitCrew now
  installs that data with the app and points the engineer at it.
- **The local voices speak in sentences, not in pieces.** Kokoro and Piper no longer stop at
  every comma: each sentence is said in one breath, with a short pause between sentences, so
  `Car left, careful.` no longer has a hole in the middle. Driver names are said as the surname
  (`Howard`), or the initial and surname when two drivers share it, instead of `Howard, J`. Local
  lines are synthesized again the first time they are said after the update.
- **The local voice no longer goes silent when Kokoro or Piper cannot start.** If one local voice
  fails, the line is now tried in each of your other installed local voices, and then in the
  studio voice's saved audio, instead of being dropped. The log says once why Kokoro and Piper
  stopped.
- **The spotter no longer waits for the engineer to finish.** When a car comes alongside while the
  engineer is talking, the engineer stops mid-sentence, the spotter calls the car, and the engineer
  repeats the line once the car has cleared. The pause between two lines now starts when a line
  ends, so long lines are no longer followed straight away by the next one.
- **The engineer talks again during the race.** The green and white flags are called the moment
  they are shown, and the race debrief starts seconds after the finish line instead of half a minute
  later. Overtake calls are never held either. Lifting off, part throttle and oval banking no longer
  count as a corner, so routine calls find a straight again. They wait only for braking and
  cornering, and only while racing, never under caution or on the formation lap. A routine call
  waits for a straight long enough to say it. While a car is alongside, everything except the
  spotter waits until it is clear, flags included. The zones you learned before are learned again
  on your next clean lap.
- PitCrew now logs a warning once per connection when iRacing's telemetry layout differs from the
  expected version or Assetto Corsa's layout is unverified, helping diagnose incompatible game updates.
- "Track drying" is said once while the track dries, instead of at every step of the drying.
- A stalled engine is called once, as a stall, instead of also as an oil-pressure and a fuel-pressure
  warning. Nothing is called any more when you switch the engine off in your pit box, when you are out
  of the car, or when the car is placed on track at the start of a session before its engine fires.
- The end-of-race debrief now waits until you cross the finish line yourself, instead of starting
  as soon as the leader takes the flag while you are still on your last lap. Places gained or lost on
  that last lap now count in the finishing position you hear.
- Damage is called on track, as soon as iRacing says the car needs repairs. Before, the damage call
  could only come once you were already stopped in your pit box. Light damage you can choose to repair
  at the stop and damage that needs a mandatory repair get different calls. If race control shows you
  the orange-disc repair flag, you hear that call alone, not both.
- Resetting an offline practice without leaving the car now starts a fresh session for the engineer.
  Before, calls it had already made, and their waiting times, carried over from the session you
  reset.
- The engineer no longer names a driver who has disconnected, for example calling that you are
  lapping someone who already left the race.
- A place gained or lost on your last lap, after the leader has already taken the flag, is now called.
- PitCrew's window no longer freezes when you change a pit preset key on the Pit strategy screen.
- When an update was installed at the same moment PitCrew replaced an older race engineer it had
  found running, it could end up attached to the engineer it was replacing. The two now wait for
  each other.
- Switching between live telemetry and a replay no longer freezes the dash, the overlay and the
  engineer for up to a second.
- Kokoro now speaks from the folder you point it at in Settings. Before, the Voices screen said it
  was installed while the engineer kept looking for it in the default folder, and the voice list,
  voice downloads and the setup check used the default folder too.
- Supertonic (beta) now uses two processor threads per model, like the other local voices, instead
  of one per core, which leaves more of the processor to iRacing while it speaks.
- Asking how much fuel you need to finish now gives the same number as the dash and as *"abastece
  até o fim"*: it counts where you are on the lap and keeps the reserve in the tank, and it also
  answers in timed races.
- In timed races the engineer now warns you when the fuel will not reach the flag. That warning
  used to come only in races with a lap count.
- In multi-class races, a faster car coming up to lap you is called again after it has already
  lapped you once. Cars pitting or leaving the pits near you are now found by where they are on the
  road, whatever lap they are on.

### Removed

- Removed six diagnostic-only commands (session recording, replay switching, and a debug speech test)
  that had no caller since the Diagnostics screen came out. No visible change for drivers.

## [0.5.0] - 2026-10-16
### Added

- **Your tyres at every pit stop, in the race debrief.** iRacing only measures the tyres while the car
  stands in its pit box, so PitCrew now reads them at each stop and keeps them with the race: tread
  left across each tyre, carcass temperatures, hot pressure and the cold pressure you left with, which
  corners were changed, the compound, the time in the box and the fuel added. The debrief's new Pit
  stops section shows them in the units your sim is set to. Races recorded before this update have no
  stop data.

- **A fuel calculator beside your pit strategy.** Type the race length in laps or minutes and the Pit
  strategy screen tells you straight away how much fuel the race takes, reserve included, using the lap
  time and fuel per lap from your own clean laps in that car at that track. With the tank known it also
  says whether the race fits in one tank or how many stops the fuel forces. When PitCrew hasn't seen
  enough clean laps yet, it asks you for the figure instead of guessing.

- **Pit stop presets you switch between from the wheel.** Save as many as you like on the Pit strategy
  screen — say “autofuel, no tyres” or “fronts only” — and bind one button or key to step through them
  and another to clear. Each press arms the preset in the car's black box straight away and shows it
  over the sim for ten seconds, even with radio messages off. While a preset is armed it takes the
  place of your plan: the engineer won't arm the planned stop over it, and once that stop is done the
  plan picks up again from the next one. Clear hands the stop back to the plan and leaves the black box
  as it is.

- **A pit strategy planner built from your own laps.** Pick a car and track on the Pit strategy screen
  and see what you actually ran there — fuel per lap, lap pace and how much pace you lose across a
  stint — each with how many clean laps it rests on. Tell it the race length, the fuel you start with,
  the tank, a fuel reserve in laps and any series rules, and it proposes the fewest stops the race
  needs with the stints split evenly, and the lap window for each. Refine every stop by hand before
  you save.

- **Several plans for each car and track.** Keep a “sprint” and a “wet” plan side by side and pick the
  active one; with none picked, the one you used last is active. Your existing plan carries over as the
  first entry, nothing lost.

- **Your engineer arms the next stop for you.** In a race, when the active plan's stop comes round —
  on the first lap of its window, or earlier if the fuel won't last — PitCrew sets it in the car's
  black box and says “parada armada” once iRacing shows it. A change you make in the car afterwards is
  kept, a stop is never sent twice (not even after a reconnect), and you can turn it off in Settings or
  for a single plan. The phone shows what was armed.

- **PitCrew now keeps a history of every lap you run, per car and track.** Practice, qualifying and
  race alike, with only clean laps — green flag, off pit road, not an out-lap — counted toward the
  planner. Laps from your older race debriefs are brought in once and kept apart, because they can't be
  checked as clean. Tyre wear is noted at each stop, where iRacing reports it.

- **Your strategist now calls a planned pit stop in three beats.** One lap before the recommended
  stop, it says the next lap is the one; around thirty seconds before pit entry it calls “box, box”; and
  once you are in, it reads back the fuel, tyres and service iRacing actually has armed.

- **A welcome screen for your first launch.** A fresh install now opens on a short welcome that says
  what PitCrew needs before your first session, downloads the voice model right there with its
  progress in view, and then takes you to the audio setup. A failed download can be tried again or
  skipped, and so can the whole welcome — it never comes back once you finish or skip it, and if you
  already have the voice model you will not see it at all. The Sound tab now also warns you when no
  output device is found, and confirms which device a test call played on.

- **PitCrew now listens to Assetto Corsa too.** Open Assetto Corsa instead of iRacing and the same
  engineer is on the radio, with nothing to set up: PitCrew notices which sim is running and moves
  between the two by itself, without a restart. This first step covers your own car — fuel, laps,
  position, pit lane and flags. The spotter, gaps and traffic around you are still iRacing-only for
  now, and it has not yet been driven against the real game, so treat it as a beta.

- **Your phone can show the lap you can still find.** The lap card now keeps your achieved session best
  in view beside an ideal lap once every sector has a clean best, and your personal best for this car and
  track. Until the rig has either target, it stays blank instead of pretending a zero is a lap time.

- **Your pit plan comes with you on the phone.** Choose the stop the rig has planned, see the fuel,
  compound and tyres it calls for, then arm that one stop from the phone. The dash reads back what
  iRacing actually has armed, so it never calls a sent request a completed change.

- **The phone dash is going on the Play Store.** PitCrew Dash gets its listing — what it does, what it
  needs, what it collects — and a signed build that can be published from one button instead of by hand.
  Nothing about the app on your phone changes yet; this is the road it takes to get there. Its version
  now matches the PitCrew you run on the rig, so a phone and a PC that say different numbers actually
  mean it.

- **The iPhone gets its own road in, through TestFlight.** The dash can now be built, signed and sent to
  Apple from one button, so an iPhone can run it as a beta rather than only an Android. Nothing on your
  phone changes yet, and one thing is still Apple's to decide: finding the rig by itself, with no address
  typed in, needs a permission Apple grants by hand. Until that comes through, an iPhone build asks you
  for the rig's address once on first launch and remembers it after.

- **Spanish, French and German are ready to speak.** Every radio call and every number now has its
  own wording in all five languages, and the Voice screen only offers a language when the selected
  engine can produce it. German needs Piper, Supertonic, Azure or ElevenLabs because Kokoro has no
  German voice.

- **Settings has a Models section.** One list of everything PitCrew downloads to speak and to listen —
  each voice engine and the recogniser behind voice commands — with what is already on this PC, what
  it would cost to fetch, and a download button for the ones that are missing. Each engine also has a
  switch: turn one off and it stops appearing when you pick an engine for an engineer, so a PC that
  will only ever use one of them stops offering you the other two. Nothing is deleted and nothing
  stops working — an engineer already speaking with an engine keeps it.

- **Clicking a race on Home opens it.** The rows under *Last races* now take you straight to that
  race's debrief, lap by lap, instead of only reporting it. Arrow keys walk the list and Enter opens
  the one you are on.

- **The whole crew checks in when you get to the track.** Your chief calls the spotter, the spotter
  answers, he calls the engineer, the engineer answers, and then he turns to you by name and asks if
  you are ready — three real voices, one after the other, instead of the single line PitCrew used to
  open with. If any of it cannot be said at the moment it belongs, the rest is dropped rather than
  arriving late: half a roll call is worse than none. **Qualifying gets the chief on his own instead**,
  with one line about the lap you are going out to do, because you come and go from a qualifying
  session all afternoon.

- **Your engineer tells you when you are in the tow.** One short call as you close onto the car in
  front on track, and one only — it never arrives instead of a spotter call, and never while you are
  in the pits, under yellow or after the flag.

- **A single word as you cross the line, and then quiet.** Where you finished, where you started,
  your best lap and your incident count, in one sentence, once — said the moment you take the flag,
  not a beat later. A number your engineer did not actually see is left out rather than guessed at,
  and a race that ended in the wall or with a disqualification is said plainly and kindly instead of
  being congratulated. After that the radio is silent for the rest of the session.

- **The Voice screen shows the last call you heard.** Who said it, how long ago, the words themselves,
  and a button to hear it again — in the same voice that said it the first time.

- **Reload voices**, on the Voice screen, for when you have installed a model or pasted a key somewhere
  else and want PitCrew to look again without restarting.

- **Home has a Radio check button.** Press it and your engineer reads a real line out of the phrase
  catalog, over the radio, in the language and the voice you have set — so you can hear whether they
  will come through before you go out, without leaving the first screen.

- **Test versions are now actually published, and they arrive on their own.** Turning *Get test
  versions* on used to buy you the same builds as everybody else, a little earlier at best — nothing
  was ever released to that channel by itself. Now a change to PitCrew's screens reaches you within
  minutes of it being written, and the bigger changes are published to you ahead of everyone as they
  are ready.

- **You can ask for test versions, in Settings.** Off by default. Turn it on under *Startup* and
  PitCrew starts offering you new things before everyone else gets them — they are less tested, and
  something may not work, which is the deal. Turn it off and you are back to released versions at the
  next check.

- **PitCrew's screens can now update on their own, without reinstalling anything.** When only the
  interface has changed, the panel offers it like any other update — usually a few hundred kilobytes,
  because PitCrew reuses everything that did not change. Installing it redraws the window and your
  engineer keeps talking through it. If the new screens do not come up, PitCrew notices and puts the
  ones you had back by itself; there is also a *Use the original interface* item in the tray menu if
  you ever want to go back to the screens your PitCrew was installed with.

- **PitCrew can now update itself.** When a new version is out, the panel in the corner offers it the
  same way it already offers a new race engineer: it tells you the size, you decide whether to
  download it, and then — separately — whether to install it. Installing closes PitCrew and hands
  over to the Windows installer, which the panel says before you press. Nothing is ever downloaded or
  installed without you pressing, and never while iRacing is running. Beta versions are not offered
  this way.

- **Each voice engine can now generate its own audio, and you choose which model to pay for.** Open
  any engine on the Voices screen and there is a *Generated audio* section at the bottom: one press
  fills that engine's cache with every fixed line the engineers on it say, all of them in one run.
  On ElevenLabs each of the three models gets its own button with its price beside it, so you can
  stock the expensive model for a race weekend while your engineers keep speaking on the cheap one —
  the audio is waiting the moment you switch. Nothing is generated until you press, and the button
  says when it is billed.

- **Bind your push-to-talk key by pressing it.** The key was the one thing on that screen you still
  had to *spell* — and PitCrew's spelling, at that, with the modifiers first, no spaces, and
  `NumpadAdd` for the numpad plus. Press *Press the keys* on the Voice commands screen, hold the
  combination you want, and it is bound. Your wheel button has always worked this way; the key does
  now too. You can still type one in, and that is deliberate rather than left over: a chord Windows
  hands to something else before it reaches PitCrew never arrives at that screen to be captured, so
  typing is how you reach one you cannot press there.

- **The screen now tells you which key is actually working.** Changing the key takes a restart —
  PitCrew asks Windows for it once, when it starts — and until now nothing said so at the moment it
  mattered. Rebind the key and the screen names the one still in use until you restart, and the
  microphone test names the key it is genuinely listening for instead of saying "press your key".
  Rebinding and testing straight away used to look exactly like a broken microphone.

- **Your microphone's Windows level, on the same screen as everything else about it.** A microphone
  Windows has turned down is the most ordinary reason a driver cannot be heard, and until now fixing
  it meant leaving PitCrew, opening Sound Control Panel and picking the right device out of a list.
  There is now a *Level in Windows* slider under the meter on the Voice commands screen — drag it and
  watch the bar follow. It is **Windows' own level and not a setting of PitCrew's**: it is the same
  number that control panel shows, it stays where you put it after PitCrew is closed, and anything
  else recording through that microphone — your voice chat included — hears the change too. The
  slider says so. Change it in Windows while the screen is open and it follows along.

- **A test that tells you whether your microphone and your key actually work.** Until now the only
  way to find out was to ask your engineer something on track and see whether anything happened. On
  the Voice commands screen there is now a *Start the test* button under your push-to-talk key: press
  your real key or wheel button, say anything, and let go. You watch the level move while you speak,
  and when you stop you get your own take **played back to you** with the words PitCrew heard written
  underneath — so a dead headset, a key that does nothing and a microphone that is simply too quiet
  stop looking the same. It is a test and nothing more: nothing is saved, nothing is sent anywhere,
  and the microphone closes when you leave.

- **You can now talk to your engineer with a button on your wheel.** Push-to-talk was a keyboard key,
  which meant letting go of the wheel to ask a question — at exactly the moment the answer was worth
  having. Bind any button on your wheel, button box or gamepad instead: `Settings → Voice commands`,
  press *Press a button*, then press the one you want and it is bound. Your key still works, so you can
  keep it for the desk and use the button on track. And you can now choose how it behaves — **hold it
  while you speak**, as before, or **tap it once to start and once more when you are done**, for a
  button you cannot comfortably hold through a corner. If you tap it open and forget, it closes itself
  after twenty seconds. The button takes effect the moment you bind it; the key still needs a restart.
  Pedals and most rotaries cannot be bound — your rig reports those as dials rather than buttons, and
  the screen tells you so instead of leaving you pressing something that will never register.

- **A debrief screen, so PitCrew is worth opening after the race and not only during it.** Every
  race your engineer finishes is now written down, and there is a new screen that reads it back:
  your stints and where the stops fell, your fuel lap by lap — what the engineer *expected* each lap
  to take drawn against what it actually took, which is the only way to see whether the strategy was
  right — how consistent your lap times were, where your best lap left time on the table sector by
  sector, and how your position moved through the race. The last twenty races are kept; the
  twenty-first pushes the oldest out.

- **Your engineer now calls the undercut and the overcut.** When the car directly ahead still has a
  stop to make and it would drop them behind you, you hear it — so you can hold station instead of
  burning the tyres chasing a place that is coming to you anyway. And when the car directly behind
  has already stopped and yours would hand them the place, you hear that too, while there is still
  time to build a gap. Both are estimates and both say so: your engineer says *"if he stops now"*
  and *"by our maths"*, because nobody can see another car's fuel. When the numbers are not there —
  you have not made a stop yet, so the pit loss has never been measured, or the car's fuel tank size
  is not in the session data — the call is simply not made rather than guessed at. One call about
  one car, then several laps of quiet about it, so a mid-pack fight does not fill the whole radio.

- **You can teach your engineer how *you* ask for things.** The phrasings PitCrew ships are ours, and
  yours are better — on a real session, changing how a question was asked took it from six right out
  of nine to eight, while a bigger recogniser changed nothing. `Settings → Voice commands → Teach it
  how you ask` opens a screen with every answer your engineer can give on one side and the ways you
  can ask for it on the other. You **record** a phrasing rather than typing one: hold your key, say
  it the way you would say it on track, see exactly what was heard, and keep it or say it again. A
  typed phrase is a guess that your voice produces that exact sentence; a recorded one is something
  your microphone has already produced once. It keeps what it *heard*, so if it hears "quanta gasosa"
  when you said "quanta gasolina", keeping the odd-looking version is the right move — it will
  mishear you the same way next time, and now the answer will be waiting for it. Ours and yours sit
  side by side, each labelled and each removable on its own, and a PitCrew update that improves the
  built-in phrasings still reaches you. It refuses a phrasing that would reach a different answer,
  and tells you which one, rather than leaving you with a question that never works.

- **You can now see your microphone working, and hear yourself on it, before you record anything.**
  The Voice commands screen has a level meter with a switch: flip it, say something, and the bar
  moves — or it does not, which is the fastest way to find out the wrong microphone is selected. It
  tells you when you are too loud as well as too quiet. Turn on *Hear yourself* and it plays you back
  through the same output your engineer speaks from, so you can judge how you actually sound —
  **wear headphones for that one**, or your speakers will feed straight back into the microphone.
  It opens the microphone only while the switch is on, closes it the moment you switch it off or
  leave the screen, and steps out of the way if you press push-to-talk while it is running.

- **You can now ask in one language and be answered in another.** There is a language selector on the
  Voice commands screen for the language *you speak* — which decides what PitCrew listens for and how
  it writes down what you said — and it is no longer tied to the language your engineer answers in.
  So you can have a Brazilian Portuguese engineer and ask your questions in English, or the other way
  round. All five languages are offered here, including the ones your engineer cannot yet speak.
  Anything you have recorded stays filed under the language you recorded it in: switching shows you
  that language's phrasings, and switching back brings the others straight back. Nothing changes for
  an existing setup — it keeps asking in whatever language it was already using.

- **You can talk back.** Hold a key — `Alt+V` by default, and you can change it — and ask your
  engineer a question out loud: *"combustível?"*, *"qual a minha posição?"*, *"quantas voltas
  faltam?"*, *"quanto pro carro da frente?"*, *"quanto foi minha última volta?"*. It answers in the
  same voice, from the same phrase catalog it always uses. *"Cala a boca"* silences it and *"pode
  falar"* brings it back.

  It listens **only while you hold the key** — there is no wake word and the microphone is never left
  open — and it recognises what you said **on your own PC**: no key, no account, no network, and
  nothing you say ever leaves the machine. Turn it on in Settings, where a one-off ~43 MB download
  gets the recogniser; it is off until you do, and PitCrew never fetches it on its own.

  It understands a fixed list of questions rather than anything you care to say, and that is
  deliberate: anything it is not sure of gets *"não entendi"* instead of a guess, because a wrong
  answer mid-race is worse than no answer. The list works in all five languages PitCrew speaks.

  Say the whole phrase rather than one word — *"quanto combustível eu tenho?"* rather than
  *"combustível?"*. Both are on the list, but a single word gives it nothing to go on and is the one
  thing that reliably fails.

- **The button tells you it is listening.** You now hear a radio click when the microphone opens and
  a lower one when it closes — the same click your engineer's own voice is bracketed with, so it
  sounds like the radio you are already on rather than like a notification.

  It is worth more than knowing the button worked. Wait for the click before you speak and your first
  word stops being cut off, because PitCrew does not start listening until it has finished. That was
  the one thing you previously had to be told to work around. You can make it a plain tone instead,
  or switch it off, in Settings.

- **You can hear a voice before you choose it.** Every engine that runs on your own PC — Kokoro, Piper
  and Supertonic — now lists its voices instead of hiding them in a dropdown, and every row has a play
  button. Press it and that voice reads you a line in *this* engineer's language, at *this* engineer's
  speed, so you can hold two of them against each other without putting your engineer on either. The
  row also ticks the voice they are already on, and offers the download if that voice is not on your PC
  yet — so you fetch what you are about to use rather than everything the engine has. Picking a voice
  used to mean choosing a name off a list and finding out afterwards. If a voice will not play or a
  download does not finish, the list says so underneath rather than leaving you pressing a button that
  makes no sound.

- **A fourth voice engine you can try, Piper, and it is the quickest of the lot to speak a new line.**
  One pass through one model, where the others are four or more — by a wide margin the least your PC has
  to do per callout. It runs on your own PC, needs no key and bills nothing. It is beta: nothing routes
  to it until you pick it, and how it sounds is yours to judge. **It speaks all five languages, and has
  fifteen voices to choose between** — nine in English, three in Brazilian Portuguese and one each in
  Spanish, French and German. On Piper a voice *is* a language, so the language you set for an engineer
  decides which voices they can be given. **Each voice is a model of its own and downloads on its own,
  about 63 MB**: Piper is ready for an engineer the moment that engineer's voice is on your PC, rather
  than after the whole set has arrived. One thing to know before you go looking for it — **none of
  Piper's Portuguese voices is a woman's**, at any quality. That is true of the voices published for the
  engine rather than a choice PitCrew made; Kokoro and Supertonic each have one.

- **A new voice you can run on your own PC: Supertonic, with ten voices and no key.** It is a beta and it
  is off until you pick it. Ten preset voices come with the download — five female, five male — and each
  of your engineers can read in a different one at their own pace. There are two versions of the model
  to choose between, and they are a real trade rather than old and new: version 3 is the steadier
  reader, version 2 sounds closer to Brazilian Portuguese and is noticeably quicker to speak a line it
  has not spoken before. They share no files, so switching downloads the other one in full — the panel
  says so, and the size, before you start. Nothing about your install changes until you choose it.

- **A second online voice you can put an engineer on: Azure AI Speech.** It is a beta and it is off
  until you pick it. Paste a subscription key and the region it belongs to, and the pt-BR voices your
  account can use appear in the picker — each engineer can have their own, at their own pace. Nothing
  changes for an install that does not touch it: without a credential your engineers still say every
  word, in the local voice, offline. If audio is refused after you save, check the region as well as
  the key — Azure answers the same way for both, and the panel says so.

- **Picking an engine now starts with one question: on this PC, or online.** Every engineer's panel asks
  that first, then which engine of that kind, and then everything that engine has — its voice, its
  weights or its account, and how it reads. The old row of engine tiles put a voice that costs nothing
  next to one that bills your card and left you to know the difference; the two are separated now, with
  the trade written under each side.

- **Everything about an engine is inside the engineer who uses it.** The Voice screen had a tab per
  engine — one each for `Kokoro` and `ElevenLabs` — and now has two tabs, `Engineers` and `Sound`. The
  voice list, the key, the account and the weights are all still there, opened from the engineer they
  belong to rather than from a strip along the top. Picking a voice no longer sends you to another tab
  and leaves you to find your way back.

- **A control that is shared between your engineers now says so.** A key, a download and a model graph
  belong to your whole install, not to one engineer, so each is marked **All engineers** wherever it
  appears. Pasting a key under the spotter's name is the same key the chief engineer uses, and the panel
  no longer leaves you guessing.

- **Each of your engineers can now speak their own language, and there are five to choose from.** PitCrew
  spoke Portuguese or English; it now also speaks Spanish, French and German, and the language is set
  per engineer rather than once for the whole app — so the spotter can call cars in English while the
  race engineer talks to you in Portuguese. The list you are offered for a given engineer is what the
  engine behind them can actually pronounce: the voice model that ships with PitCrew has no German
  speaker, so German is offered on every other engine and not on that one. Numbers are spoken
  properly in every one of the five — a lap time, a gap and a position are read the way that language
  says them, not translated word by word. Nothing changes for an install that does not touch it: every
  engineer keeps the language you already had, and none of the audio PitCrew has already generated is
  thrown away.

- **Everything about an engineer is now behind their own name.** Each engineer's card opens a panel
  holding every choice about them — which engine makes their voice, which voice, which language, how
  fast they read, and a button that generates their lines. The Voices screen used to be organised by
  *engine*, which meant setting up one engineer took you across three tabs and back. Those tabs are
  still there for the things that belong to your PC rather than to a person: downloading the voice
  models, pasting your studio key, fetching the beta engine's files.

- **Each engineer has their own cache warm.** Generating the fixed lines used to be one button for the
  whole app, and that stopped making sense once every engineer could have their own voice and their
  own language — two of them can be perfectly ready while the third has nothing on disk. The button
  now sits inside the engineer it is about and generates only their lines, so changing one engineer's
  voice no longer means regenerating everyone's. **The single "Warm cache" button on the Phrases
  screen is gone**; that screen still tells you how much is generated and what finishing it would
  cost.

- **The wait between qualifying and the race is now a screen worth looking at.** Climb out after your
  qualifying run and your phone used to say your car had not loaded into the session, which was true and
  no use. It now shows you the grid: where you start, the cars either side of you, their licence and
  iRating, their last lap and how far off you they were — over the track, your class, the strength of
  field and the time left. It is the order the sim has the field in, not one PitCrew worked out, and it
  goes back to your normal dash the moment you load onto the track. Your accent, your fonts and your idle
  background all come with it.

- **Your phone's dash now works out your pit stop.** A new block — on the practice and race screens out
  of the box, at the top of the column, and off for qualifying — that answers the three questions a stop
  actually raises: the lap your fuel window
  closes on, how much to put in, and where the stop leaves you. The first two arrive as soon as you have
  a clean lap to average. **The third needs one stop to learn from**: PitCrew times your own stop at your
  own pit lane, so from your second stop onwards it can say which lap of the window comes out best, and
  until then it says so rather than showing you a number it guessed. It reads the cars in your class that
  have already stopped and the ones that still owe one, and it tells you it is a projection — it does not
  know about tyre wear, traffic on your out-lap, or when the drivers who have not stopped are going to.

- **The dash on your phone now switches between practice, qualifying and race by itself.** The three
  arrangements you configure separately are now picked automatically from the session you are actually
  in, decided on the rig rather than guessed from the session's name — so a race a league called
  something unusual still gets your race screen. A session PitCrew cannot place falls back to the race
  screen, which is the one carrying every block.

- **Qualifying now gets a screen built for one lap.** FOCUS puts a single column down the middle of your
  phone with nothing either side of it, and that is what the qualifying screen now starts as: your
  predicted lap, your last, your best and the session's, at the size you can read them at from the corner
  of your eye, with your TC, ABS and brake bias on a strip underneath. No leaderboard, because on a hot
  lap you are not racing the car ahead — and the time left is already on the bar across the top. It is
  still yours to change: every block on it has a switch, the leaderboard included. It is the one
  arrangement the picker does not offer, though — it is where qualifying *starts*, and picking either of
  the two panels instead is a one-way door, so the drawer shows you FOCUS selected for as long as you are
  on it.

- **You can reorder the cards down the side of the race dash by dragging them.** Press and hold one of
  the three — pit strategy, the vehicle strip, fuel — and slide it up or down the column to where you
  want it. They swap places among themselves and nowhere else: there is no drag that lands a card on the
  leaderboard, in the bar, or across in the other column, so a stray thumb cannot rearrange your dash.
  Let go without moving and nothing changes. It is the race screen only, and the one thing it costs is
  that a long press on one of those three cards now picks the card up instead of opening the settings —
  press and hold the board, the bar or the space between the cards for that, or use the button on the bar.

- **The leaderboard now has a switch, like every other block.** It was the one block you could never
  turn off. You can now leave it off on any screen — what PitCrew still refuses is an empty panel, so
  the last block left on a screen keeps its switch on until you turn another one on.

- **Each of your engineers can now have their own voice.** Putting the race engineer, the strategist and
  the spotter all on the same local engine used to give you one voice saying everything — a voice
  belonged to the engine, not to the person speaking. Every engineer's card on the Voices screen now has
  its own voice, and says which one they fall back to until you give them one. Nothing changes for an
  install that does not touch it: every engineer keeps the shared voice they already had, and none of
  the audio PitCrew has already generated is thrown away. Giving one engineer their own voice does mean
  their lines are generated again in it — they are genuinely different recordings — and this applies to
  both engines that run on your own PC. It applies to the studio voice too, with the cost that comes
  with it — see *Each engineer now reads at their own pace* further down.

- **PitCrew now tells you when there is a new engineer or new phrases, and never installs either on its
  own.** A small panel in the bottom-right corner says what is available and how big it is. You accept
  the **download**; later, and separately, you accept the **install** — two answers, because a download
  spends your connection while you are doing something else and an install stops the engineer for a few
  seconds to start it again. Neither is ever offered or carried out while iRacing is running. Everything
  PitCrew fetches is signed, and a file that is not exactly the one that was signed for is thrown away
  rather than installed. If an update does not start, the engineer you had comes back by itself. New
  phrases arrive this way too, without a new installer — so a better call, or a fix to something the
  engineer says, no longer waits for a release you have to download and run.

- **Your phone's dash is now yours to shape.** Settings moved into a drawer that slides in over the
  board, so you can see what you are changing while you change it — every choice lands on the panel
  behind it as you tap. Inside it: **two arrangements** to pick from, drawn as little diagrams rather
  than described — the board on the left with the lap-times strip under it and three cards stacked
  right, and the same panel mirrored for a phone cradled to your right — and **seven blocks** you place
  into them: the board, the session bar, a big session clock, lap times, fuel, car adjustments and pit
  strategy. A block goes where you put it and the slot's size *is* its size, so
  there is no second control to argue with the layout. If a block will not fit, the drawer says so
  before you commit rather than shrinking a number you were reading.

- **Practice, qualifying and the race each get their own dash.** Three arrangements, configured
  separately, so the panel you want for a qualifying lap is not the one you want at lap 40 of a race —
  a block can be on for the race and off for qualifying. Out of the box practice and the race share the
  panel PitCrew has always drawn, and qualifying gets a screen of its own — see above.

- **A preview, so you can set all of this up with iRacing closed.** It draws the real panel from
  made-up numbers — a full multi-class field, a tank that is short of the finish, a lap in progress —
  and it is labelled `PREVIEW · MADE-UP DATA` the whole time it is open, in a bar above the panel, so
  it can never be mistaken for your own session. You can flip between the three screens inside it. Reached
  from the idle screen and from the drawer.

- **Five accent colours and three font pairs.** The accent recolours *your* row on the board and nothing
  else — every other colour on the dash keeps its one meaning, so purple is still a session best and
  amber is still a warning. The fonts are three pairs of a text face and a number face, each previewed
  in itself with a real lap time, and every number face keeps its digits the same width so a lap time
  cannot jitter as it counts.

- **The idle screen can be your own.** Your car number and name at the top, off the session's roster,
  with a switch to hide them; a choice of what sits underneath — your last race, or a clock; and a plain
  or graded background. Two options are visible and switched off with a note saying why: `NEXT RACE`,
  because PitCrew cannot read your schedule yet, and a video background, which needs clips that are not
  cleared.

- **PitCrew now states plainly what it collects and the terms it's free to use under, in the app and
  in the installer.** `Settings → About` has two new buttons — Privacy and License — that open the
  full text: what is sent to Google Analytics and Sentry, what is never sent, and the free,
  proprietary license PitCrew ships under, with the list of third-party components underneath it. The
  installer now shows a short version of the same license, with a privacy notice, on the one screen it
  can — none of this changes what PitCrew already collected, it makes it easy to actually read.

- **Your engineer has three or four ways to say every call now, instead of one or two.** Two ways of
  saying a thing is not variety — it is alternation, and after a handful of laps you could hear the app
  taking turns. Every callout the driver hears repeatedly inside one race — the spotter, the gaps, the
  positions, the traffic — now has at least four, and everything else at least three, in all five
  languages. The spotter is the careful case: its calls still open with the same words in the same
  order, because *"carro na esquerda"* has to be recognised before it is understood, and only the tail
  of the call changes. The countdown to the green is untouched where it is counting — *"cinco",
  *"quatro"* — because a number has no synonym.

- **Qualifying finally has an engineer.** Until now a qualifying session got the race's phrases or
  nothing at all. It now has its own: the session opening, the out-lap (*"essa volta não é a volta"* —
  attacking on cold tyres is the most common way to waste a qualifying run), five minutes, two minutes,
  and the call the app has never made — **one minute, and if you do not cross the line before the clock
  hits zero the lap does not count**. It tells you when a lap improved and what slot that puts you in,
  when it did not and to cool the tyres and go again, and where you start from when the session closes.

- **A red flag is now called on the radio.** It was the most serious thing the app said nothing about.
  So is a full-course yellow, which now has its own call — *"amarela geral, reduz e não ultrapassa"* —
  told apart from a local yellow for the first time.

- **The engineer talks in the window before the green, where a real one talks most.** The formation lap
  (warm everything, hold formation), your grid slot, one lap to the green, a held start, crossed flags
  at half distance, and the standing-start light sequence down to *"vai!"*. The app used to be silent
  from "get in the car" until the green flag dropped.

- **You can now choose which ElevenLabs model reads your engineer's lines**, on `Voice → Studio
  voice`. Three of them, with what each one costs per character shown next to it: the fastest and
  cheapest, a steadier one at the same price, and the most expressive one at double. Before this, the
  app always used the expensive one and never asked.

- **PitCrew now checks what your ElevenLabs key is *allowed* to do, not just that it works.** ElevenLabs
  lets you restrict a key to certain parts of its API, and a restricted key passes an ordinary check and
  then refuses the thing you wanted — the first time you try to hear the studio voice, or when the voice
  list turns up empty. Pasting a key now has three answers instead of two: rejected, works, or works but
  is missing a permission. The third names the permission, says which part of PitCrew is unavailable
  until you enable it, and links to the page where you do. It is told apart from "your plan doesn't
  include this", which looks identical and is not fixed the same way. Checking still costs no
  characters, a key missing one permission keeps working for everything else, and no call is ever lost:
  what the studio voice cannot say is read by the local voice.

- **The spotter is its own voice now, separate from the race strategist.** *"Carro na esquerda"*,
  *"livre"*, *"dois carros na direita"* — the six calls you hear most often mid-corner used to share a
  voice with fuel warnings and penalties. The spotter has its own card on the Voices screen now, with
  its own engine and voice picker, so you can put it on a different voice from the rest of the
  strategist's calls if you want to. Nothing about when or how often it speaks changed.

- **A warning in the header of every screen when phrases are missing from the cache**, saying how many
  and roughly what generating them would cost. Clicking it opens the phrase catalog, where the button
  that does it is. It goes away by itself once they are generated, and it says nothing at all while a
  generation is running.

- **The relative table on your phone now shows every class.** It used to list only the cars in your own
  class, which meant the one car you most needed warning about — the prototype coming up behind you —
  was never on it. The board now lists the cars physically around you on the track whoever they are, in
  road order: furthest ahead at the top, you in the middle, furthest behind at the bottom. So you can
  see the faster car arriving, and which direction a blue flag is about to come from, before you are
  shown one.

- **The interval column follows the track, not the standings.** In the new relative it counts the
  seconds around the lap between you and each car, which is the number that tells you when they arrive
  — a car a lap ahead of you sitting fifty metres back reads as two seconds away, not as a lap. The
  column is labelled `REL` when it means that and `GAP` when it means seconds behind you in the race,
  so the two can never be mistaken for one another.

- **Your phone has its own dash settings.** Long-press the top bar, or use the button on the waiting
  screen, and you can set the size, the theme, the units and which blocks are drawn on *that phone* —
  useful because a phone in a cradle is not the same distance from your eyes as a tablet clamped to the
  rig. Nothing here is sent to the rig, and nothing is changed for anyone else connected to it.

- **The board's compact size really lists nineteen cars now.** All three sizes ask for a number of cars
  — 19, 13 or 9 — and the two larger lists were quietly short, because the rig only ever sent eleven
  cars to choose from. It now sends enough.

- **Your phone shows your last race on the screen you meet before the next one.** The waiting screen —
  the one your phone sits on with the sim shut — now carries where you finished, how many incidents it
  cost you, the laps you completed, your best lap of that race, and whether the race took anything off
  your record for that car and track. A race you left early keeps its position and says *did not
  finish* beside it, because both of those are facts about your night. A rig that has not logged a race
  yet shows exactly what it always did.

- **What your iRating and safety rating did is not shown, and the screen does not pretend otherwise.**
  iRacing tells the sim what your rating is when a session starts and never what a race did to it, so
  PitCrew genuinely does not know. Rather than draw two empty boxes you would keep checking, they are
  not there at all. The same goes for the race you are signed up for next.

- **Your engineer now warns you about a car entering or leaving the pits nearby.** A car diving into
  the pit lane directly ahead or behind you, or one rejoining the track as you come past the pit exit,
  now gets its own call — the same kind of attention a spotter gives a car about to change what is
  around you.

- **Your phone's board shows the session's fastest lap, not just your own.** A fourth line under BEST
  LAP on the lap-times card — the quickest lap set by anyone in the field, whoever that is, updated
  live.

- **A settings button now sits on your phone's live screen, next to the PitCrew mark** — the same
  screen as before still opens with a long press, this is a second, faster way in for a driver who
  knows where to look.

- **The Dash settings screen now shows the panel it configures**, not just the controls for it. A live
  preview above the settings draws the real board, session bar and cards — the exact layout your
  tablet shows — over a sample race, and it redraws the moment you change a size, a theme, the units or
  a block, so you see the result before it ever reaches the rig.

- **Each engineer now reads at their own pace, and has their own studio voice and model.** These were
  the last three things about an engineer that were really about the *engine* behind them: moving the
  speech-rate slider in one engineer's panel changed how fast every engineer on that engine read, and
  all your engineers on the studio voice were locked to one voice between them. Now the panel means what
  it says — set the spotter to read fast and clipped while the race engineer takes their time, or give
  each of them a different studio voice. Anyone you have not given their own keeps following the shared
  setting, which still lives on the `Sound` and `ElevenLabs` tabs and is what a fresh install uses
  throughout.

  Two things worth knowing before you use it. A different voice, model or pace means that engineer's
  fixed lines have to be generated again — free on your own PC, and **billed per character on the studio
  voice**, so three engineers on three studio voices is three times the generation to pay for. And
  nothing is ever generated until you press the button for it. Every screen where one of these choices
  is made now says whose voice you are changing, and the tabs say plainly when you are editing the
  shared one instead.

- **A safeguard against settings showing up in two places at once.** Each setting now belongs to
  exactly one screen, enforced at build time — a future duplicate control, like the radio filter
  that once lived on two screens, fails before it can ship.

- **`Settings → About` now shows your engineer's version as well as PitCrew's.** They are the same
  number on every install today, because both arrive in one installer. They will not always be: your
  engineer is its own program now, and the point of it being one is that it can be improved without you
  reinstalling PitCrew. When that happens, "which version" is two questions — and a bug report that
  answers only one of them is a bug report that guesses. The row reads `Not running` when no engineer is
  running, which is a thing Home already tells you about.

- **And if the two ever cannot talk to each other, PitCrew says which one to update.** It will not
  happen to an install that came from one installer — this is the safeguard for the day your engineer can
  update on its own. Rather than the app going quiet with "the race engineer is not running" while an
  engineer is plainly running, every screen says which of the two is the older half and what to do about
  it, and PitCrew refuses to drive an engineer it does not understand instead of half-working.

- **A car on your left is now called from your left.** The spotter's callouts are placed in the stereo
  field on the side the car is actually on — left for "car left", right for "car right", and the same
  for two cars — so you know the side from the first instant of the call instead of waiting for the
  word. Nothing else moves: "clear" and "cars both sides" stay in front of you, and so does everything
  the race engineer and the strategist say. The words still name the side either way, so nothing is
  lost on a mono headset or a single speaker — and there is a slider for how far the call is pushed
  (including off) beside the volume, on the Sound tab. It changes only how a line is played, so no
  audio is generated again and nothing is billed.

### Changed

- **No more pill-shaped fields and buttons in the window.** Text fields, dropdowns, search boxes and
  every button — the small round icon buttons included — now share one squared-off corner, so a form
  reads as one set of controls instead of a mix of lozenges and boxes. Switches, sliders, radio choices,
  status badges and the segmented choices keep their round shapes, and the phone's Companion and PIT
  pages are unchanged.

- **Voice commands now use Parakeet instead of Whisper.** After a one-time download of about 670 MB,
  you can ask in Portuguese or English without switching languages, while your engineer answers in
  the voice and language you chose. Your recorded phrasings stay in place. The old Whisper model is no
  longer used at all, so you can delete its folder. An incomplete or damaged download is shown as not
  ready, with a way to retry.

- **Your phone keeps the easy way to find your rig while its Companion and PIT move to pages the rig
  serves.** Pairing, QR scanning and the saved connection stay native; once the transition reaches your
  app, the same Companion and PIT will also open directly from any browser on your home network. The
  rig's Cockpit dash window shows one code for each, so scanning one opens that page directly. The
  phone app now runs full screen, with the status and navigation bars hidden until you swipe from the
  edge, and while Companion or PIT is open the screen stays on: it no longer dims or locks.

- **The PIT page always shows what iRacing's pit box is set to right now**: fuel to add, whether
  autofuel is on, and which tyres will be changed. You see it even with no plan for the car and track
  and no stop needed yet, and next to the planned stop when there is one.

- **The phone's Race screen is redrawn: the leaderboard moves to the right, and the clock, the pit
  window and the tank now share one card on the left.** Race Control replaces the three separate
  cards that used to carry them, and the vehicle card's TC/ABS/brake-bias readings are now rings
  instead of a strip. Practice draws the same panel; Qualify and the leaderboard's own columns are
  unchanged. Preview mock details now stay behind one compact menu, so they do not cover the dash.

- **Hold-or-tap and the microphone's cue are now switches you can see both sides of.** On Voice
  commands, the two drop-downs have become rows of buttons: both answers are on screen and choosing
  one is a single press instead of opening a list.

- **The push-to-talk key and the button that records it now sit on one line**, and the microphone
  card puts *Check the level* and *Level in Windows* side by side — the two halves of "is anything
  arriving, and is it loud enough" are read together. The *Model* box on that card is gone: PitCrew
  ships one recogniser and its name is on its own row in the new Models section on Settings.

- **The banner over the sim has been redrawn.** It is PitCrew's own green now, on a flat near-black
  panel with a thin edge — the violet-and-blue chip with the dotted background is gone. The light that
  says your engineer is on the radio is green instead of red, the name beside it is easier to catch at
  a glance, and the little sound meter's bars stand on a line together rather than floating, so you can
  read them out of the corner of your eye. All four shapes changed together and every one of them
  still says exactly what it said before.

- **The low-profile strip is now the shape you get if you have never chosen one.** It is the one the
  new design was drawn for: a single line the height of a caption — the light, your engineer's name,
  the meter — and it costs the least of your screen of the four. **If you have already picked a shape
  on the Overlay screen, nothing changes for you.** If you never opened that picker, your banner will
  look different after this update; the other three shapes are still there and one click away.
  safety warnings now have their own queue: they win over anything else, and if they cannot be said
  while they are still true they are dropped rather than played late. Everything else — strategy,
  status, celebrations — waits for a quiet moment and is thrown away if the fact behind it has
  changed.

- **The ten-second countdown is one continuous call.** *"Ten, nine, eight…"* is spoken as a single
  phrase now instead of ten separate ones, so the numbers can no longer arrive out of order, repeat,
  or be interrupted halfway. A countdown your engineer notices too late is not started at all.

- **Your race ends when you cross the line, not when the winner does.** The chequered flag comes out
  for the whole field at once, and PitCrew used to go quiet then — while you might still have half a
  lap to run, in traffic. Your engineer now keeps working until you have taken the flag yourself, and
  goes quiet the moment you do.

- **The tow is called once for a given car.** Sitting behind the same one for a stint used to earn a
  reminder every thirty seconds, because the gap breathes across any threshold you pick. A new car in
  front is still a new call.

- **Reconnecting mid-race no longer sounds like the start of a session.** *"Driver, engineer on the
  radio"* is for when your engineer really has just come on the radio.

- **One yellow is one call.** A full-course yellow, a waved yellow and a yellow sector describe the
  same incident and used to be able to arrive one after another; the first one that is said wins,
  and a detail can only follow it a few seconds later if it is still true.

- **Position calls are about your class, and about racing.** Gaining or losing a place is announced
  when the change has held on track — a pit stop, a caution or the sim reshuffling its own scoring
  no longer announces itself as places won or lost.

- **A car in the pits is never called a lapped car.** *"He is lapping you"* now needs both cars on
  track, out of the pits, and a lap difference that agrees with itself across more than one moment.

- **Your grid slot is only read out when the session has actually been scored.** It used to say you
  were starting first whoever you were, because before the sim scores a session every car reports
  the same number. Your engineer now says nothing rather than something false.

- **The sidebar is your seven screens and nothing else, and every key is listed in Settings.** The
  keyboard panel that opened from the foot of the sidebar has gone, and your initials with it. Every
  key PitCrew answers to is on one card now — Settings, General, Hotkeys — including the number that
  jumps to each screen, which used to be in that panel and nowhere else. `Alt+K` no longer opens
  anything; your name is still yours to set under Voice.

- **What PitCrew is short of, and the one thing it ever asks you for, are now a strip along the foot
  of the window.** The warning about phrases that have not been generated used to sit beside each
  screen's own buttons, and pressing it took you to a screen that could only tell you to go somewhere
  else. It now says how many lines are missing and takes you straight to Voice, where each engineer is
  warmed. Beside it, in the same strip and on every screen, is the app's only ask — three amounts and
  a button that opens PayPal in your own browser. Home ends with your race log again.

- **The Debrief list tells you how each race went.** Every race down the side now carries where you
  finished — `P1`, `P2`, `P13`, `DNF` — in the same colours the race log on Home uses, so you can find
  the race you are thinking of instead of picking between three entries for the same track. The result
  beside the report you are reading is coloured the same way now too; it used to draw a P13 as though
  it were a P6.

- **Every screen now reaches the edge of the window.** The scrollbar used to float twenty pixels
  inside it on all eight, as if the page were a card sitting on the app rather than the app itself.
  On the Debrief screen the list of your races is now divided from the report by a line down the
  full height of the window, and the Language screen's two families lists the same — which is how
  they were drawn and what the old gutter had been in the way of.

- **The Voice screen no longer counts your engines twice, differently.** Its heading said you had
  five engines, the panel underneath said two, and neither number agreed with the tiles between
  them. The count with no honest denominator is gone; each engine still says for itself whether it
  is ready, and the panel now states in words what the amber pill only implied — a cloud key changes
  the *voice*, never what gets said.

- **Debrief is redrawn around the race you picked.** Your recorded races are a list down the side
  instead of a narrow strip, the race you are reading opens with its track, its car and where you
  finished, and the four figures that answer *how did that go* — best lap, consistency, your record
  here and how this race compared — sit above everything else. The stints read as a table with a line
  per run, and the four charts are two to a row instead of one under another, so a whole race fits on
  a screen. The screen also picks up the setup notice and the phrase-cache warning it was the only
  screen missing, and *Re-check* asks for your races again without leaving it.

- **Settings is three tabs instead of one long page.** *General* is what you change — how PitCrew
  starts, the keys that reach it from inside the sim, the window size and the dash rate. *Data &
  storage* is what is on disk and which build you are running. *Connections* is the cockpit dash:
  the address, the code to scan and what is connected, now on a screen of its own rather than only
  behind a button. Each card's settings are separated by a fine line, so a card reads as a list of
  things you can change instead of a run of paragraphs, and *Clear audio cache* is now drawn in red
  — the one button on that screen that deletes anything.

- **The Overlay screen is redrawn, and it finally shows you the banner.** The preview used to appear
  only if you shrank the interface, so on a normal window there was nothing to look at while you set
  the size and the transparency; now it is on screen at every window size, with the four banner
  shapes directly under it and everything you can change down the side.

- **The Phrase catalog screen is now Language, and it has both halves of what gets said.** *It says* is
  the catalog you already had — every line your engineer can speak, what voice makes it, and whether it
  is on disk yet, with the parts a race fills in now marked inside the sentence. *You ask* is the phrase
  book, moved here from Voice commands: every question you can ask, and the wording you recorded for
  each one. Nothing on either tab is made up as you drive — which is the point of putting them on one
  screen.

- **Voice commands is now only about being heard.** The phrase book has moved to Language; what is left
  is your key, your microphone and the test that tells you which of the two is at fault.

- **The Voice commands screen is redrawn.** What you press and what hears you sit side by side, and
  the test — press your key, say anything, read it back — spans the foot of both, where it belongs:
  an empty take is as often the microphone as the key. The promise is now at the top of the screen
  where you cannot miss it: the mic opens when you press your key and closes when you are done.

- **The warning about iRacing bindings is shown before you pick a button**, not after. PitCrew cannot
  see what iRacing has bound, and the moment that matters is while you are choosing.

- **The Voice screen is redrawn.** Your three engineers sit across the top as three cards — engine,
  voice and language, one line each, with a play button to hear them without opening anything — and
  down the side is what your install can actually do: which engines are on this PC and how much they
  take up, whether the studio voice has a key, and what was last said over the radio.

- **Deciding who says what is now two boxes you drag between.** Every kind of call sits in the box of
  the engineer who reads it; drag one across to hand it over, or press it, whichever you prefer. Under
  them, PitCrew names whatever you have moved away from the built-in split.

- **Names have their own tab on the Voice screen** — yours, your race engineer's and your strategist's
  — instead of sitting at the bottom of the radio settings, where they had never really belonged. The
  three boxes now line up across the tab, with the line explaining each one underneath it rather than
  above, and an engineer you have not named yet says *Not set* instead of sitting empty.

- **The first screen is redrawn.** Your races are a proper table now — finishing position, track and
  car, best lap, laps, and how long ago — under four figures that say how it has been going: races,
  laps, average finish, and your quickest lap of the last seven days. Down the side: what your
  engineer remembers about the last track you raced, who is on your crew and which voice each of
  them speaks with, and which version of PitCrew you are running.

- **Cockpit dash is a button on the first screen**, instead of an unlabelled icon in the strip down
  the left. Same panel, now named rather than guessed at, on the screen PitCrew opens on — and the
  whole of it is also a tab of its own under Settings, if that is where you go looking.

- **Show overlay is gone from the first screen.** The switch on the Overlay screen turns the banner
  on and off *and* tells you which it is, which that button never could — it could only ever switch
  it on. `Alt+H` still works while iRacing has the screen.

- **The window has a new look, starting with the frame around every screen.** PitCrew is drawn in a
  new typeface, on a darker window with a soft lime and blue glow behind it, and it now has a proper
  title bar: the screen you are on is named at the top left, and whether iRacing has been found is
  said at the top right — on every screen, not just the first one. The icons down the left are
  narrower, and the one you are on is lit in lime.

- **The icons down the left are in a new order, so the `Alt` keys have moved with them.** It now runs
  Home, Debrief, Voice, Voice commands, Language, Overlay, Settings — `Alt+1` through `Alt+7` in that
  order. `Alt+K` still opens the full list of keys. Sorry about the muscle memory.

- **Nothing on screen has moved — but everything behind it has.** PitCrew's screens were rearranged so
  that each one keeps its own pieces together instead of sharing one pile. You will not see a
  difference in this release; it is the groundwork for the redesign of the whole window that follows.

- **PitCrew now remembers which installation it is beside your settings, instead of inside the
  window.** Nothing looks different, and nothing new is collected — this is the same anonymous
  install id and the same *"which races have already been counted"* note as before, moved somewhere
  that survives the interface being replaced. It is the first step towards updating PitCrew's
  screens without reinstalling the whole app. Your existing id and race history are carried over, so
  no race is counted twice.

- **Every box you type in, pick from or press is now the same height across the whole app.** Text
  fields, dropdowns, search boxes and buttons each used to work out their own size, so a row that
  should have read as one line — a field with a button beside it — was three slightly different
  heights. They all measure the same now, and nothing in the app is taller than that.

- **One corner, everywhere.** The cards on the screens and the ones in dialogs were rounded by
  different amounts, which showed the moment a dialog opened over a screen. They are drawn with the
  same corner as each other and as the window itself, which is a touch rounder than before.

- **The window's corners no longer show a seam against the desktop.** PitCrew draws its own frame,
  and Windows was rounding the same window a second time a few pixels tighter — so at each corner
  there was a sliver of Windows' shadow and border sitting outside PitCrew's own edge. The window's
  corner is now PitCrew's alone. The trade is that the main window no longer casts the Windows drop
  shadow: there is no way to have the system draw a shadow around a corner it is not drawing.

- **A long device name no longer breaks a dropdown onto two lines.** *Microphone (2- Realtek(R)
  Audio)* and names like it now end in an ellipsis inside the closed dropdown, which stays one line
  tall. The full name is still there in the list when you open it.

- **The Voice commands screen is now two tabs instead of three columns.** Setting the microphone up
  and teaching PitCrew how you ask are two different jobs, and they were sharing one screen — with
  everything about your microphone squeezed into a narrow strip down the right-hand side. *Push to
  talk* now has the whole width for your key, your wheel button, the test and the microphone itself,
  side by side; *Input catalog* has the whole width for the list of what you can ask and how you
  phrase it. The switch that turns the whole thing on stays at the top of both, and the screen opens
  on *Push to talk* — because if you came here in a hurry, that is usually why.

- **The buttons on the Voice commands screen now match the rest of PitCrew.** That screen had kept
  square, uppercase buttons from an older part of the app while every other screen moved to the
  rounded sentence-case ones — so *RECORD A PHRASING* sat next to *Start the test* and neither looked
  like it belonged. They are all one shape now.

- **The push-to-talk panel is grouped instead of being one list of six rows.** What you press — the
  key and the wheel button — is now separated from what happens once you have: hold or tap, and the
  sound when the microphone opens.

- **The Voice commands screen reads properly with a screen reader or with Voice Control.** Several
  rows were captioned one way on screen and named another way to assistive technology — the field
  under *Key* announced itself as "push-to-talk binding", so saying "click Key" to Windows Voice
  Control reached nothing — and the sentence under each caption, including the one explaining that
  the sound when the microphone opens is what stops your first word being cut off, was never read out
  at all. Each control is now called what the screen calls it, and its explanation is announced with
  it.

- **The microphone panel no longer leaves a gap below it.** It was still sized as though it were the
  whole right-hand column, which stretched it to the full height of the window and could put a second
  scrollbar inside the first.

- **Groundwork for better speech recognition.** PitCrew can now be pointed at a second, higher
  precision copy of the recogniser, so the two can be measured against the same recordings rather
  than against an impression. **Nothing changes for you yet** — same model, same download size, same
  everything a driver sees. It is how the next improvement to *"não entendi"* gets chosen on
  evidence.

- **Voice commands is its own screen now, and it has everything about it on it.** It used to be
  split: a card on Settings held the switch, the push-to-talk key, the wheel button and the sound the
  mic makes, while the microphone, the level meter, the language you ask in and the recogniser lived
  somewhere else entirely. If your engineer could not hear you, the two things worth checking were on
  two different screens. They are all on one screen now, reached straight from the bar down the left
  side — **`Alt+6`** — with what you press on one side and what hears you beside it. Settings no
  longer has a Voice commands card at all, and **Settings has moved to `Alt+7`**.

- **Everything about your microphone is now on the Voice commands screen, beside the phrasings.**
  Picking the microphone, downloading the recogniser and pointing PitCrew at a copy you already have
  used to live under `Settings → Voice commands`, two clicks from the screen that shows you what was
  actually heard — so a wrong microphone looked like a phrasing problem. They are now a column down
  the right-hand side of the Voice commands screen, next to the answers and the recorder they belong
  to. Settings keeps what it is good at: your push-to-talk key, your wheel button, whether you hold or
  tap it, and the sound it makes. There is one row there taking you across, and on a fresh install it
  is the first thing on the card.

- **The Voice commands screen no longer refuses to open before you have downloaded the recogniser.**
  It used to be a single paragraph pointing you back at Settings. Now it draws normally — you can read
  through every answer your engineer can give and every way of asking for it — with the download in
  the right-hand column and only the *Record a phrasing* button waiting on it.

- **The Settings screen now tells you when a push-to-talk key will not work, instead of letting you
  find out at the next race.** The key you type is checked as you type it: a key PitCrew cannot bind,
  a letter with no `Alt` or `Ctrl` in front of it — which would take that key away from iRacing and
  everything else on the machine — or a chord PitCrew already uses for something else, which would
  stop the app starting at all. Each one says so under the field, along with what happens instead.
  The keycap beside it shows *your* key rather than the one PitCrew ships with, and once you have
  changed it there is a **Restore Alt+V** button to put it back.

- **`Settings → Voice commands` is sorted into what you press and what does the listening**, so the
  microphone and the wheel button are no longer mixed in with the recogniser's download and its model
  folder. On an install that has not downloaded the recogniser yet, the download comes first. The
  Hotkeys card above it leads with the keys that work while iRacing has focus, and no longer repeats
  the screen numbers the rail already lists under `Alt+K`. On `Data & storage`, a Clear button with
  nothing to clear now says **Nothing cached** rather than sitting there greyed out for no stated
  reason.

- **Your engineer is better at judging who still owes a pit stop.** It used to treat any car it had
  once seen in the pit lane as done, and every other car as still owing a stop — which is wrong in a
  race short enough that nobody has to stop, and wrong about a car that started on a small fill. It
  now works out how far into its own stint each car is, so the pit-window advice on your dash lines
  up better with what the cars around you are actually going to do.

- **When it does not understand you, it now asks you to say it again.** It used to just say it had
  not understood, which left you wondering whether to repeat yourself or give up on the question.

- **Your engineer now knows which of its lines are answers, and it will only ever speak one of those
  back to you.** Before, a question was matched to a line by name and spoken; anything that name
  pointed at would have been said out loud, including the warnings it gives you on its own — and a
  warning is only true at the moment it fires. *"Combustível pra cinco voltas"* is the right call
  when you have five laps left and simply a false one when you ask with twenty. Every line is now
  marked either as something you can ask for or as something the engineer says on its own, and a
  question that lands on the second kind gets *"não tenho esse dado"* instead. Nothing you can ask
  today has changed; this is what lets the list of questions grow without any of them being able to
  lie to you.

- **Generating an engineer's lines now shows you how it is going, in the panel you started it from.**
  Pressing "Warm this engineer" used to do nothing you could see: the button changed its label and
  the progress bar it was reporting to sits at the bottom of the window, behind the open panel. So the
  one button in PitCrew that can spend your money was also the one that gave you no sign it had
  worked. The panel now reports the run itself, right under the button — how many lines are done, a
  bar for each voice doing the work, and "Stop using credits" where you can actually press it. Close
  the panel and the run carries on, reported by the bar at the bottom of the window as before. One
  thing it deliberately does not claim: a warm-up carries no engineer's name, so if you started one
  under a different engineer, this says it may be that run rather than pretending it is this
  engineer's.

- **The model an engine runs on is now a card you can download from.** It was a dropdown that let you
  choose a model and then gave you no way to fetch it: pick the Supertonic generation that is not on
  your PC and you had picked something you could not use, with the download button one row above
  describing the *other* one. Whether a model is on your machine — the single thing that decides
  whether that engine can speak — was the last three words of an option inside a closed list. Models
  are now cards, like the engines above them: the name, the size, whether it is on this PC, and a
  Download button on the one that is not, with the progress shown on that card while it runs. It
  reads the same way for every engine that runs on your PC, including Piper, where each *voice* is a
  model and the card says so and points you at the voice list.

- **The panel where you set an engineer up now fits on the screen.** It was one long scroll: the
  engineer's name went past the top as you worked down it, and "Warm this engineer" — the button you
  press *after* changing anything, and the only one that can spend money — sat at the very bottom
  where you could never see it while you were making the changes. The name now stays at the top, the
  middle scrolls on its own and fades where it is cut so you can tell there is more, and the warm
  button is fixed at the bottom, always in reach. It also says which it is before you press it: free
  when that engineer is on a voice from your own PC, and billed to your account when they are on an
  online one.

- **The engineer's panel opens from the bottom of the window now, instead of floating in the middle
  of it.** It slides up from the edge and spans the width, so it grows upward when an engineer has
  more to set up and the buttons never move: the name stays at the top of the panel and "Warm this
  engineer" stays at the bottom, in the same place every time you open it.

- **Choosing between a voice made on your PC and an online one now says what each costs, before you
  choose.** That first choice was two words in a strip — "On this PC" and "Online" — with the trade
  written underneath for whichever side you had already picked. It is now the same pair of cards the
  engines themselves are picked from, so both sides state their price at once: *downloads once, works
  offline, costs nothing* against *needs an account, billed per character*.

- **The engineer's language is the first thing you set now, instead of the last.** It decides which
  voices exist to be chosen at all, so setting it at the bottom of the panel meant re-opening the
  question you had just answered. It sits at the top, in a field the size of a language name rather
  than the width of the panel.

- **When an engineer's voice cannot speak, the panel now takes you to the fix.** It used to tell you
  what was missing — "Needs a key and a region", "Needs a voice" — and leave you to find the control
  yourself, further down a panel you could not see the bottom of. That line is now the button: press
  it and the right panel opens, whether that is where you paste a key, the voice list, or a download.

- **PitCrew no longer throws away your settings when it meets one it does not recognise.** A single
  unreadable entry — an engine that is no longer offered, say — used to fail the whole settings file,
  and a settings file that fails to load is silently replaced with a fresh one: your key, your engine
  choices and every voice you had picked, gone, with your own name disappearing from the app as the
  only sign anything had happened. Now the one entry it cannot read is dropped, that engineer goes
  back to the engine PitCrew would have given them, and everything else in the file is exactly where
  you left it.

- **Speech rate now belongs to each voice engine instead of to the app.** One slider used to set the
  pace of everything, and it could not be honest about it: the local engine takes 50% to 200% and the
  studio voice only 70% to 120% — so past the marks the slider was moving one voice and pretending to
  move the other. Each engine now has its own rate, in its
  own range, next to the rest of that engine's settings. **The pace you already set is kept**, on both
  engines, and none of the audio PitCrew has already generated is thrown away.

- **The dash on your phone is redrawn to the design it was meant to have.** The board goes back to the
  type size it was cut for, so a driver's name reads whole instead of trailing off after two letters,
  every column holds its number without clipping, and the leaderboard fits more of the field on the same
  screen. A car a lap ahead or a lap behind — and your own car — now carries a coloured bar down the edge
  of its row, so you find yourself and the traffic without reading a single cell. The interval column
  reads `+1.4` and `−6.4` rather than a bare number, which is the sign you used to have to get from the
  colour. The bar across the top is calmer: the numbers are back at the size the rest of the panel is set
  at, `TIME LEFT` is yellow and your incident count amber so the two you glance at still stand out, and a
  position or a lap now reads as `9` with a dimmed `/25` beside it rather than one run-on figure.

- **The dash you get out of the box is the one the design was drawn around.** Practice and the race now
  start on `TWO PANELS`: the board over a full-width lap-times strip — predicted, last, best and the
  session's fastest, side by side under the leaderboard instead of stacked in the column — with pit
  strategy, the vehicle strip and fuel down the side. That is six readings on the panel where the old
  default fitted five. The lap-times block takes whichever of its three shapes the place you put it in
  can actually hold, so moving it into the column still gives you the stacked card.

- **The `CAR ADJUSTMENTS` card is now a slim `VEHICLE` strip.** Traction control, ABS and brake bias sat
  in three boxed tiles under a heading, taking a third of the column for three numbers that change a
  handful of times a stint. They are one line now — the reading, its name beside it, each still in its
  own colour — and the height that frees up goes to the fuel card below, which is the one down there
  that actually wants the room.

- **The cards down the side of your phone's dash are in the order the dash was designed in**: pit
  strategy at the top, the vehicle strip under it, fuel at the bottom. They used to come out in the
  order PitCrew ranks them by importance, which is the order it still drops them in when you pick an
  arrangement with fewer places than you have blocks — those are two different questions and they were
  sharing one answer.

- **The whole panel was being drawn at three quarters of its intended size on a 7" tablet.** Every card,
  every label and every figure — the dash was solving its sizes against a 10" tablet's proportions and
  then fitting the result to a smaller screen. It draws at full size now, which is the single largest
  change to how the dash looks on a phone or a small tablet.

- **Your licence rating is shown the way iRacing spells it.** The board rounded `B 3.41` to `B 3.1`,
  which was a column that had run out of width rather than a decision about the number — and the last
  digit is the one that actually moves after a race.

- **The lap-times block reads by colour again.** Your last lap is plain white, your own best is violet —
  the same violet the board already puts on a driver's personal best — and the session's fastest is
  plain. Your last lap used to be yellow, which is the colour the bar above uses for a clock running
  out, and one colour cannot mean two things on a panel read at speed.

- **The board lists far fewer cars, and every one of them is readable.** `COMPACT`, `NORMAL` and `LARGE`
  listed 19, 13 and 9 — counts taken from the desktop dash, which draws its board in a window twice the
  height. On a phone that meant solving nineteen rows out of about a third of the screen and rendering a
  driver's name at six pixels. The steps are **9, 7 and 5** now: `NORMAL` is the board the dash was
  designed around, and `LARGE` is for reading at a glance with two cars each side of you.

- **The leaderboard runs edge to edge.** It was inset twelve points on every side like the cards beside
  it; a card holding three figures wants that air and a leaderboard wants the width, so the rows are
  wider — the driver-name column gets the difference — and taller. Everything else on the dash keeps a
  twelve-point inset, which is now the same number as the space between two blocks rather than a
  second one.

- **The places-gained reading is off the top bar.** It was one of six segments competing for a row that
  fits five comfortably. Where you are running and which way you are moving are both on the board
  already — your row, and every row above and below it.

- **Your engineer now really does run as its own program, and the PitCrew window is the screen that
  talks to it.** Opening PitCrew starts your engineer if it is not already running, and finds it if it
  is — so closing the window to the tray, or the window itself going wrong, no longer takes your
  engineer off the air mid-race. Everything you press still does the same thing: the voices, the
  previews, the cache warm-up, the settings, the tablet dash, Alt+M and Alt+R. Quitting from the tray
  icon still shuts the engineer down with the app.

- **Your engineer is now a separate piece from the window it lives in — nothing changes about the way
  PitCrew drives today.** Everything that decides what to say and then says it (the telemetry reading,
  the phrase catalog, the voices, the audio, the tablet dash) moved into its own component, and the
  app around it is now just the screen you click on. Same calls, same voices, same settings, same
  files on disk. It is groundwork: keeping the two apart is what will later let PitCrew improve your
  engineer without you reinstalling the whole app.

- **Your engineer can now run on its own, without the PitCrew window open.** It is the next step of the
  same groundwork and there is nothing for you to do with it yet — the app still starts your engineer
  and drives it exactly as before, and the installer is unchanged. What it means is that the part that
  reads the telemetry, decides what to say, speaks it and serves the tablet dash no longer needs a
  window to exist at all. Two small things come with it: a crash in your engineer is now told apart
  from a crash in the app around it, which is the difference between a report that can be fixed and one
  that cannot, and none of this changes what PitCrew sends or when it sends it.

- **The app and your engineer can now talk to each other over a private channel on your own PC.** Third
  step of the same groundwork, and again nothing for you to do: PitCrew works exactly as it did. It
  matters because of what it is *not* — that channel is reachable only from your own machine, never from
  your network, and it needs a password that PitCrew generates fresh every time your engineer starts.
  The tablet dash is unchanged and stays open to your Wi-Fi, because all it ever does is show you
  numbers; the new channel is the one that will carry things like your settings and your ElevenLabs key,
  so it is locked down instead. Nothing new leaves your PC, and nothing changes about what PitCrew
  sends.

- **Every screen in PitCrew is now answered by your engineer rather than by the window.** Fourth step of
  the same groundwork: your settings, your race log, the disk figures, your recordings, the catalog, the
  voice lists, the radio check, the cache warm and everything on the ElevenLabs side — the key check, the
  credit readout, the auditions and the voice designer — all moved. Same screens, same values, same
  files, and nothing new is bought or sent. Two things did get better: the "this install is incomplete"
  banner now tells you which file is missing even when the voice engine itself failed to start, which is
  exactly the case it was drawn for and the one case it used to stay blank in; and `settings.json` now
  has exactly one writer for good, which is what keeps that promise once your engineer runs as its own
  program. The one thing that stayed in the window is the file browser for importing a voice clip,
  because picking a file needs a window to pick it in.

- **Your engineer now answers all of those screens over that private channel too.** Same groundwork, and
  still nothing to do: the app keeps asking for everything the way it always has. What changed is that
  every question now has a second way in — the one the app will use once your engineer is its own
  program — and one detail that will matter to you on a slow day: a long job like designing a voice or
  downloading a voice model no longer holds up everything else your engineer is being asked. It answers
  each question as it finishes, so PitCrew can keep telling you your engineer is alive while a
  two-minute download is running instead of looking frozen until it ends.

- **Two engineers can no longer start on one PC, and one that loses its window keeps the race going.**
  The last piece of groundwork before the app and your engineer become two programs, and it is the one
  that decides when your engineer stops. Only one can run at a time now — two of them reading iRacing's
  data would be two voices talking over each other — and the second one to start says so and stops
  instead of joining in. It costs nothing if a previous one crashed or the power went out: the claim is
  released by the machine itself, so there is no leftover file to delete and nothing to repair. And when
  PitCrew closes normally your engineer is told, and goes; when the *window* dies without warning your
  engineer keeps working for a minute so relaunching the app picks up the same one — which matters
  mid-race, because starting a fresh engineer means loading the voice again and a few seconds of silence
  is exactly what you cannot afford there.

- **The installer now carries your engineer, and PitCrew installs it into your own user folder the
  first time it starts.** The last piece of groundwork before your engineer can be updated on its own.
  Nothing to do and nothing to see: install PitCrew on a PC with no internet and you get a working
  engineer, exactly as before. What changed is where it lives. The copy in the installer is a seed —
  PitCrew copies it into your user folder on the first start and runs that copy from then on, keeping
  the one before it beside it. That is what will let a future engineer arrive without a new installer
  and without Windows asking for an administrator, and it is what keeps updating PitCrew from being
  blocked by an engineer that happens to be running while the installer works.

- **Your phone keeps its own dash settings, and no longer follows the desktop's.** Size, theme and units
  used to arrive from the rig unless you had overridden them on the phone, with a `FOLLOW THE RIG` button
  on each one to hand it back. The phone owns all of it now, with sensible defaults, because the desktop
  no longer draws or configures the phone's dash — it sends telemetry. Nothing on your phone looks
  different until you open the drawer: the settings you had set are kept, and the arrangement you get is
  the panel you already had. The one thing you lose is that units could previously follow whatever
  iRacing was set to; on the phone you now pick litres or gallons yourself.

- **The settings screen is now a drawer**, and it opens the same three ways as before — the labelled
  button on the idle screen, a long press on the live panel, and now from the preview. The long press
  used to have to land on the session bar; it works anywhere on the panel now, because the bar is a block
  you can switch off. A single tap on the live panel still does nothing at all.

- **The second voice is now the race strategist.** It speaks the calls whose message is fixed text —
  the spotter, fuel, pit stops, penalties and session — and "chief engineer" never said it also does
  the spotter, so the label promised less than the app delivers. The name you gave that voice in the
  settings keeps working; only the built-in label changed.

- **The settings button on your phone's waiting screen now says "Settings".** It was an unlabelled icon
  in a corner. The panel you drive by still hides settings behind a long press — a tap target on a
  board you read at speed is a setting changed by a knuckle — but a parked phone is where you actually
  go looking for the switch.

- **The connection thread across the top of your phone's dash is gone.** The two-pixel line that swept
  once per frame has been removed from every screen. Nothing about how the panel tells you it has lost
  the feed has changed: it still dims when the sim stops publishing — whether that is a dropped
  connection or a sim sitting frozen with the socket still open — and the bar still says which. What
  you lose is the first few seconds of that warning, and what you get back is a panel with nothing
  moving on it that is not a reading. (The dash you open in a browser is untouched and still has it.)

- **The waiting screen is no longer dimmed.** Dimming means "these numbers may be out of date", and
  that screen has no such numbers — a race that is over and a panel that is waiting do not go stale.
  The live board still dims exactly as before.

- **PitCrew never generates phrases in bulk on its own any more.** It used to start a run whenever you
  changed something that invalidates the cache — a voice, the speech rate, the language — which meant
  the app could spend ElevenLabs credits as a side effect of you adjusting a setting. Now it only ever
  happens when you press a button: *Warm cache* on the phrase catalog, *Regenerate them now* on the
  voice screen, or the last step of setup. Nothing about what gets generated changed; what changed is
  who decides when.

- **Generation progress is now a bar at the bottom of the window instead of a panel over the screen.**
  It shows one bar per voice that is working — the one on your PC and the studio voice, separately,
  because they run at the same time and only one of them costs money — and it stays visible while you
  move around the app, so you can leave the phrase catalog without losing sight of a run you started.
  Stopping the paid half is still one click, on the bar itself.

- **The studio voice now uses ElevenLabs' cheap model by default, which halves what generating your
  catalog costs.** If you had already generated phrases, they are still on disk but were made with the
  old model, so PitCrew asks you to regenerate them — until you do, those calls fall back to the local
  voice. If you preferred the old sound, pick `eleven_multilingual_v2` on `Voice → Studio voice` and
  nothing changes.

- **Scrollbars are PitCrew's own now, not Windows'.** Every list that scrolls used to get the pale
  system scrollbar, which was the only light-coloured thing in a dark app and sat hard against the edge
  of whatever card it was next to. It is a thin dark bar that brightens when you point at it.

- **The window zoom now resizes the window once, a second after you let go of the slider.** It used to
  redraw the whole app on every 5% step, so dragging the slider looked like the app was struggling. The
  percentage still follows your finger; the window catches up when you stop.

- **The Voice screen's four tabs are laid out to one measure.** Cards on *Engineers*, *Voice packs*,
  *Studio voice* and *Sound* were built at four different sizes over four rounds of work, so switching
  tabs moved every edge and the voice designer's two columns were spaced differently from each other.
  Same content, one rhythm.

- **The engineer now tells you when the car next to you is not in your race.** A car that is a lap
  ahead or a lap behind is not somebody to fight, and until now PitCrew said nothing about it at all.
  It now names the driver and which way round it is — you are being lapped, or you are the one doing
  the lapping — and that there is nothing there to defend or chase. It only speaks when that car is
  genuinely close on track, and how close, how many laps of difference count, and how often it
  repeats are all yours to change in `tuning.json` and the phrase catalog.

- **Brighter colours on the phone, so the numbers that matter stand out further.** The backgrounds went
  darker and the signal colours louder — your own row, a session best, the predicted lap. The day theme
  gained depth of colour rather than brightness, because a lighter colour on a light screen is harder to
  read in sun, not easier. Nothing changed about what a colour *means*: your row is still the same
  green, a session best is still violet, on the phone and on the desktop alike.

- **The ready-made voices in the voice designer are a list you can read.** `Voice → Studio voice` has
  always had six engineers to start from — a calm veteran, one tense under pressure, four others — but
  they were drawn as six small tags under the description box, and what each one actually asks for only
  appeared if you happened to hover over it. Each is a row now, showing the description it writes for
  you and the seed that makes picking it twice give you the same voice. The list also says which file it
  is read from, `phrases/voice_presets.json`, which you can edit to change the six or add your own.

- **The tabs on the Voice screen are named after the engines they configure.** `Voice packs` is now
  `Kokoro` and `Studio voice` is now `ElevenLabs` — so the strip reads as one tab per place a voice
  can come from. Pressing `Change` next to an engineer takes you to the tab for the engine they are
  actually on, which used to mean landing on the Kokoro catalogue whichever engine they were on.

- **The lap card on your phone reads label-then-value now, right-aligned, instead of side by side.**
  PREDICTED, LAST LAP, BEST LAP and SESSION BEST all sit the same way: the name above the number,
  both flush to the right edge of the card, so the figures line up with each other instead of with
  whatever their label happened to be that row.

- **The fuel card on your phone dropped its gauge.** The ring around the laps-left figure is gone —
  the number was always the answer, the ring was decoration around it. What is left is the figure,
  "LAPS", and the pill underneath it that already told you whether that number is comfortable or
  tight.

- **The relative table on your phone breathes more.** The gap between columns is wider, and LAST LAP
  has more room, so a slower driver's name and the times either side of it stop crowding together.
  The interval column is also colour-only now — no leading `+` or `−`, just red for behind and green
  for ahead, since the colour already said which one it is.

- **The bar across the top of your phone changed shape.** Strength of Field now leads the row instead
  of Position, and the settings icon is half again as big — a target you actually hit at a glance while
  the car is moving.

- **Your phone's dash is set on your phone now, and nowhere else.** PitCrew used to keep the panel's
  size, theme and which blocks it shows in the desktop app, and send them to the tablet on every frame —
  so after your phone got settings of its own, the same three things could be set in two places. The
  desktop's copy is gone: open the dash's own settings on the tablet to change how it looks. **Units are
  the one exception and still come from PitCrew**, because whether you see litres or gallons follows
  what iRacing itself is set to, and only PitCrew can read that. The Dash screen in the app now says
  where the controls went; connecting a tablet is unchanged and still one click from the rail.

### Fixed

- **Companion and PIT now reach your tablet from a fresh install.** The desktop installer includes both
  phone pages with the race engineer, instead of leaving the tablet at a missing-bundle message.

- **Your engineer stops speaking while you ask a question.** Holding push-to-talk now cuts the current
  call before it reaches the microphone, so the recogniser hears your question rather than competing
  with the engineer's voice.

- **Arming a stop from your phone now actually arms it — and no longer risks your replay.** The phone
  was sending its pit commands on the wrong channel, so iRacing never ticked a thing, and the first
  message of every arm could erase the session's replay tape. Arming now reaches the black box, and it
  touches only what the stop asks for: your fast repair and tear-off stay ticked, tyres you left
  "as set" stay as they are, and a stop that picks tyres clears the corners first and ticks only the
  ones chosen.

- **Your strategist calls the stop in a one-stop race, on the right lap, once.** Before your first stop
  of a session it used to say nothing at all; now it calls "next lap" and "box, box" for the last lap
  the fuel allows — keeping at least half a lap in the tank, which it used to count one lap too late.
  Ignore it and you get one last call on the lap the tank can no longer finish, then silence, instead
  of "box, box" every lap. A stop you planned is called at the start of its window; if the fuel will
  not reach that window, the engineer says the stop has moved earlier and calls it there.

- **Timed races get a pit strategy too**, worked out from your own lap times with a lap of margin — on
  the phone and on the radio, which now agree on how many laps are left.

- **The pit-entry service brief is said only when it means something**: in a race, under green, and
  never while you serve a black flag or are being towed.

- **The pit lane's time loss is remembered for your car and track**, so the strategy and the undercut
  calls have it from the start of your next race there instead of waiting for your first stop.

- **Your plan can pick the car's own tyre compound, and arming sends it.** The editor offers the
  compounds iRacing lists for your car (get in the car once and PitCrew learns them), a stop can use
  iRacing's autofuel, and the phone shows whether autofuel is actually on and confirms only what the
  car really has armed. Plans made before this keep their fuel and tyres; a generic compound from an
  older plan is shown for you to pick again rather than guessed.

- **The PIT page never shows a pit window backwards.** When one stop cannot reach the flag it says so,
  instead of a window like *L16–9*.

- **Your engineers' names are no longer cut short on the Voice screen.** On a fresh install — before a
  voice pack has been downloaded — each of the three cards had to fit both a name and a longer
  readiness note on one line, and the name was what gave way: *Race engineer* read *Ra…*. The note
  now moves under the name when the two will not share a line, so the name is always whole. The
  Engine, Voice and Language lines on those cards do the same, so *Portuguese (Brazil)* is readable
  rather than clipped to *Portuguese …*, and hovering any of them still shows the full text.

- **Three groups of calls were listed by their internal names.** Traffic, Track and Race appeared as
  `traffic`, `track` and `race` in the Language screen's list of families and among the categories you
  drag between your engineers on the Voice screen — the only three of the eighteen that had never been
  given a name to show. The spotter's own family is now called *Car alongside* there, rather than
  repeating the heading above it.

- **The voice and cache column on the Language screen no longer cuts its own text in half.** A line
  waiting to be warmed said "Supertonic · not warmed" over two lines in a space only wide enough for
  one, so the second was clipped and a perfectly normal state read as a fault.

- **An interface update no longer leaves people on test versions without one.** Publishing a fix to
  PitCrew's screens removed the test channel's list of what was available, so anyone with *Get test
  versions* on stopped being offered anything at all — quietly, since PitCrew read it as "nothing new"
  — until the next full release put it back.

- **The four figures across the top of Home line up again.** *Best lap this week* is the only one of
  the four whose heading needs two lines, and its time was sitting lower than the other three because
  of it — on the one tile the screen is trying to draw your eye to. The four now sit on the same line
  whatever their headings do.

- **The phrase list shows half again as many lines at once.** Every row on the Language screen was
  taller than it was drawn to be, so the longest list in PitCrew showed five phrases where it was
  meant to show eight. The play button beside each line is the same size to hit as before.

- **How you open the microphone no longer reads as half a sentence.** The setting on the Voice
  commands screen was too narrow for its own answer and showed *Hold while speaki…*. The two choices
  are now **Hold to speak** and **Tap on, tap off** — the same two things, said in the room the
  setting has — and they stack under one another instead of clipping when the window is too narrow
  for both.

- **The radio banner came back on its own once a session started.** Turning *Radio messages* off — on
  the Overlay screen or with Alt+H — held only until you got in the car: PitCrew put the banner back
  up the moment the session went live, and the switch quietly moved itself back on to match. The
  switch is now the one thing that decides. With it off the banner stays off whatever iRacing is
  doing; with it on the banner comes and goes with the session as before. It starts off every time
  you open PitCrew.

- **While recording a phrasing, PitCrew told you to hold the wrong key.** If you had changed your
  push-to-talk key, the recorder still said *"Hold Alt+V and say it"* — the key PitCrew ships with
  rather than yours — at the one moment it was asking you to press something. It now names your own
  key, and so does the microphone test.

- **The microphone level bar was invisible when nothing was arriving.** It was drawn in the same
  colour as the panel behind it, so a silent microphone showed no bar at all — on the screen you open
  precisely because your microphone is silent. The empty bar is now visible, and a level loud enough
  to clip still shows red.

- **A dead *Play it back* button on a take with no sound.** When a test take came back without
  playable audio the button stayed on screen greyed out, which reads as a broken player. It is simply
  not there now.

- **PitCrew no longer disappears while it is listing your microphones or speakers.** Reading the
  audio devices off Windows could take the whole app down with it — no message, no crash report, just
  gone — and which of them did it depended on invisible timing, so it could happen while you opened
  the microphone picker, unplugged a headset, or did nothing at all. Every device list PitCrew reads
  now goes through one place that is safe to read it from.

- **PitCrew now starts even when your push-to-talk key belongs to something else.** If you set
  push-to-talk to a key Discord, ShadowPlay, Steam or any other program on your PC had already
  claimed, PitCrew stopped opening at all — the next time you started it, with no message, and
  hand-editing a settings file as the only way back in. It now starts normally: the key is left
  unbound, `Settings → Voice commands` says Windows would not give PitCrew that key and that
  something else on the PC has it, and your wheel button still works while you pick another one.

- **The message about missing recogniser weights now spells the folder the way Windows does.** When
  voice commands had nothing to listen with, the sentence naming the folder it had looked in wrapped
  it in quotes and doubled every backslash — `"C:\\Users\\..."` — which is not a path you can paste
  into Explorer or into the model-folder field it was telling you about. The same sentence on the
  Voices screen, about a speaking voice that is not downloaded, had it too.

- **Downloading the voice-command recogniser no longer ends in a red error.** The download itself was
  finishing, but the screen misread the engineer's "done" and showed *"`voice.download` was answered
  with something this build could not read"* — so the weights were on your PC and the switch still said
  *Not loaded*, with nothing to press but Download again. It now reports the download it actually did
  and loads the recogniser straight away, no restart.

- **Every call your engineer makes now ends when the words do.** A full second of silence was being
  added to the end of every line — 42% of a short one like *"Box, box!"* — on both of the voices that
  run on your own PC. You will notice it most on the spotter, whose whole job is timing: the call
  finishes and the next one can start, instead of the line holding the channel for another second with
  nothing left to say. The pause between two halves of a longer sentence is untouched; that one is
  deliberate. Audio already saved on your PC keeps its old ending until it is generated again.

- **Switching Supertonic's generation no longer claims the other one is on your PC.** Picking
  Supertonic 2 while Supertonic 3 was the one installed left the panel saying "On this PC", in green,
  about files that were not there — and the two generations share nothing, so acting on it would have
  meant an engineer that could not speak. The panel now re-checks your disk when you switch, and it is
  honest in the meantime.

- **The window has its rounded corners back.** They were lost in an earlier release and nothing else
  changed with them; the panel that slides up from the bottom of the window follows the same curve
  again.

- **The licence you read inside PitCrew, and the one the installer shows you, now credit the voices
  you can download.** Two of them — Siwis in French and Sam in English — are published under terms
  that ask for an attribution, and only the copy of the licence on the website was carrying it. All
  four copies say the same thing now, including that a voice you download is licensed to you by
  whoever published it rather than by PitCrew.

- **Looking at the other kind of engine no longer looks like you switched to it.** Flipping an
  engineer's panel from "On this PC" to "Online" — or back — showed the first engine in that group
  ticked and highlighted, as though your engineer had just been moved onto it. They had not, and
  nothing had been saved; the panel was only showing you what is there. Nothing is ticked now until
  you pick one.

- **"Warm cache" now tells you when it could not actually save anything.** It used to fill its bar and
  report success even when nothing reached your disk — so a full drive, or a folder it could not write
  to, looked exactly like a finished warm-up until the next race. It now says so instead of claiming it
  finished. This also closed a hole in the two new voices below, Piper and Supertonic, which prepare
  their audio at a different quality to the others: nothing they made could be stored at all. Nothing
  shipped with that; it is fixed before the voices that would have hit it arrive.

- **French works. It never did.** An engineer you set to French went silent on their first call — the
  part of PitCrew that turns writing into sounds was being asked for a French it does not answer to, on
  both of the voices that run on your own PC. Nobody had heard it, because nothing had asked it to
  speak until now. Spanish and German were always fine, and none of the audio PitCrew has already
  generated is thrown away.

- **The Cancel button on a cache warm would have stopped only one of the paid voices.** PitCrew is being
  prepared for more than one, and the stop button was written when there was exactly one — so a second paid
  engine's queue would have kept generating, and kept billing, after you pressed it. Nothing shipped with
  that bug; it is fixed before the engines that would have hit it arrive.

- **An engineer on a paid voice you have not set up is read by the local voice, as it always should be.**
  With one paid engine this was already right. With more than one PitCrew would have checked the *wrong*
  account before deciding — so an engineer set to a voice you had never configured could have waited on the
  network mid-lap and then said nothing at all. It now asks about the engine that engineer is actually on.

- **The English lines have been read end to end, and six of them were wrong.** English has been in
  PitCrew's vocabulary for a while but had never been read as a whole, and reading it turned up
  mistakes that no single line looks like on its own. The one that mattered most: with a lap of fuel
  left, your engineer said *"we're out of fuel"* — a different thing from what the Portuguese line
  says, on the one call you might act on straight away. A warning about the driver ahead read *"holds
  a A licence"* whenever that licence was an A or an R. A closing gap was reported as *"one second to
  go"*, which sounds like laps left in the race rather than the gap to the car. A blue flag was shown
  *"for us"* instead of for you. And the spotter's own six calls were not written in one voice — *"Car
  left."* next to *"Two cars on the left."* — which is audible when they land seconds apart. All six
  are fixed, and English is still awaiting its listening test before it is offered as a finished
  language.

- **An update no longer leaves the previous race engineer installed.** PitCrew decided whether the
  engineer that came with an update was new by looking at its version number alone, so a release that
  reissued the same number — which every prerelease does — was installed and then ignored, and PitCrew
  went on running the engineer it already had. It now notices that the file itself has changed and
  replaces it. If the engineer that is running cannot be replaced at that moment, PitCrew says so in its
  log and tries again the next time you open it, rather than quietly keeping the old one forever.

- **PitCrew no longer sits on "Starting the race engineer" without saying why.** When the app and the
  race engineer on your PC disagree about a version, PitCrew will not drive that engineer — and it had
  written the reason to its log, shown it on screens you could not reach, and left you watching a
  pulsing dot for as long as you were willing to. The starting screen now carries the sentence that
  names which of the two to update, and PitCrew stops starting an engineer it has already refused twice
  instead of starting a fresh one every half minute. It keeps watching, so an engineer it *can* drive is
  still picked up without restarting the app.

- **The session bar's readings now sit against the right-hand edge.** Strength of Field, position, lap,
  time left and incidents were right-aligned inside their own share of the bar rather than against the
  bar itself, so they floated with an empty strip of card after the incident count and no clear gap
  between them and the session's name. The five readings hug the trailing edge now, with the space
  falling where it should — between the race you are in and the numbers about it.

- **The fuel card's tank gauge was never actually drawn.** The bar showing how full the tank is has been
  missing since the card was built — the `TANK 45%` line underneath it was sitting on empty space. It is
  there now, and it fills from the left in proportion to what is in the tank.

- **`RACE NEEDS` beside that bar was reading back the fuel already in your tank.** Any time you had
  enough on board to reach the flag, the figure said the race needed exactly what you were carrying —
  and it went up every time you put more in. It is what the race actually needs from here now: your own
  consumption times the laps left, so it is a number you can compare against the tank rather than one
  that agrees with it. It reads as blank in a timed session, which has no lap count to work from.

- **The small print on your phone's dash is readable again.** The labels on the lap-times card — the
  ones naming your predicted lap and the delta beside it — were being drawn at about half their intended
  size, and the three figures under the fuel gauge at three quarters. Every card now gets the height its
  contents actually need, so nothing on the panel is shrunk to fit a box that was too small for it. The
  session bar's six readings are at full size too, on every screen the panel is built for.

- **Your licence rating is no longer cut off on the leaderboard.** Every row showed `A 5…` instead of
  `A 5.0` — the safety rating gone from the whole board — and the column header itself was clipped
  mid-word. The columns are sized against what actually goes in them now, including a three-digit car
  number and an interval over a hundred seconds.

- **The largest board size no longer reduces every driver to their initials.** At `LARGE · 9` on a 7"
  tablet the names came out as `D. …` and `R. D…` on all nine rows, which is not a leaderboard. The
  board now gives the names room and takes the difference out of the type size instead.

- **The board's rows are easier to tell apart, and the faint text on the panel is legible.** The
  alternating row shading was a twentieth of a step — invisible at arm's length, which is the one thing
  it exists to avoid — and the hairlines under the column headers were fainter still. The quietest ink
  on the panel, which is what every card's small print and every label on the session bar is set in, was
  under the contrast a small reading needs on both the cockpit and the day theme.

- **The preview no longer covers the session bar it is previewing.** The `PREVIEW · MADE-UP DATA` label
  sat on top of the panel and hid the lap, the time left and the incident count on a 7" tablet. It has
  its own bar above the panel now.

- **PitCrew no longer wipes your settings when it starts.** The window used to open a couple of seconds
  before the engineer's own program was ready, and in that gap it filled the screen in with blanks —
  empty names, no studio-voice key, the language back at English — because it had no way to say "I
  haven't read your settings yet". Everything then looked normal, so the next thing you changed saved
  those blanks over the real file. PitCrew now waits for the engineer, says **"Starting the race
  engineer"** while it does, and will not write anything until it has actually read what you had. It
  also only ever sends the setting you just changed, so nothing else in the file can be overwritten by
  it — and if the engineer restarts, the window notices and picks up the current values instead of
  holding a stale copy.

- **The readiness screen no longer reports installed files as missing.** During that same startup gap
  it showed the Kokoro voice model as not installed, next to a button to download it — on machines where
  it was installed all along. One driver re-downloaded a voice pack because of it. Nothing on that
  screen now claims a file is absent unless it has actually looked, and the same applies to the Voices
  screen, which briefly emptied its voice list, and to Home's race-log column, which briefly read as an
  install with no races yet.

- **The "phrases still to generate" badge no longer says everything is up to date when it doesn't
  know.** If PitCrew could not read the cache state, the badge in every screen's header quietly
  disappeared, which reads as "nothing missing". It now says nothing at all in that case, which is the
  truth. The studio-voice audition had the same problem in a worse place — it claimed you had no
  catalog on disk for that language, right beside the button that spends credits.

- **The session bar on your phone no longer cuts off its last reading on a 4:3 tablet.** Six readings did
  not fit the bar's width there, and the one on the end was silently clipped. They now shrink together
  when the bar is short, so every one of them stays on screen and readable.

- **The leaderboard on your phone no longer risks clipping its right-hand column** on a very wide screen
  at the largest size. It now sizes its columns against the width it actually has as well as the height.

- **Your engineer no longer calls a personal best a session record.** A quicker lap of your own was
  announced as *"novo recorde da sessão"*, which may have belonged to somebody two seconds up the road.
  It says *your* best now, and the app already had a separate call that names whoever actually holds the
  session's fastest lap.

- **A local yellow no longer gets full-course instructions.** *"Reduz o ritmo e não ultrapassa"* on a
  single-corner yellow is wrong — it is caution in that sector, not a lap-wide speed limit and not a
  blanket ban on passing. The two are separate calls now, and each says what is actually true.

- **Reference lap times are read to the tenth.** *"O líder tá girando um minuto e quarenta e dois
  segundos"* was rounded to the whole second, which is useless as a reference when the difference you
  are chasing is hundredths. Your engineer now says *"um minuto, quarenta e dois e três"*, the way the
  number is actually said on the radio — for the car ahead, the session best, the leader's pace and your
  own best on a track you have run before.

- **Race advice no longer arrives during qualifying.** *"Faltam cinco minutos, controla o ritmo"* used
  to fire in every session there is, and in qualifying it means the opposite: five minutes left is the
  last window to put a lap in. Every call that gives advice about a situation now knows which kind of
  session it is in, so a race line cannot reach a qualifying session and the other way round.

- **The engineer stops calling you *motorista*.** That is what you call someone driving a bus. You are
  a *piloto*, and the radio check at the start of a session no longer says *"pronto pra corrida"* in the
  middle of a practice session either.

- **The spotter says *"livre"* instead of *"limpo"*.** *"Limpo"* reads as a clean lap; *"livre"* is what
  a Brazilian spotter actually says when the car beside you has gone.

- **Nobody speaks a colon.** *"iRating médio do grid na sua classe: 2340"* is a screen being read out,
  not a person talking — the field-strength call is a briefing line now, and the same fix applies
  everywhere it had crept in. The track-limits warning also stops saying *"preta enrolada"*, which is
  *furled black* translated word for word and something no one says out loud.

- **Coming out of the pits, the engineer says what is useful.** It used to narrate what you had just
  done — *"entrando no pit"* — and now it says *"limitador ligado"*. And when a stop left the old tyres
  on, it says so as a fact rather than only as a consequence.

- **Buttons, toggles and tabs show the hand cursor again when you point at them.** Every clickable
  thing in the app — the engineer toggles, the engine tiles on `Voice → Engineers`, the tab strips, the
  routing tags — was showing the ordinary arrow, so nothing on screen looked pressable until you pressed
  it. Controls the app cannot honour yet still show that they are unavailable.

- **The gap calls no longer chase a car that is a lap away.** "You are catching the car ahead" and
  "the car behind is coming" are about the people you are racing. On a short track — where a whole
  lap can take less time than the window those calls look at — a car a lap down running up the road
  could land inside that window and be announced as a threat closing on you. You would defend, or
  attack, and lose time to a car that was never in the fight. How many laps apart you are is now part
  of the decision instead of a side effect of how long a lap happens to be.

- **When ElevenLabs refuses your key, PitCrew now tells you which of four reasons it was.** Some of
  those refusals used to arrive on screen as "the request was refused" and a number, which reads like a
  bug in PitCrew and is not one. There are four causes behind it, they look identical from the outside,
  and each needs something different from you: your **plan** does not cover the feature; your **key** is
  missing a permission; ElevenLabs has **flagged the account**, which no other key on that account gets
  past and only they can lift; or the refusal **never reached ElevenLabs** at all, because something
  between your PC and their API answered for it — a filtering proxy, a company or school network, or an
  antivirus that inspects HTTPS traffic. Each of the four now says so in its own words, and says what to
  do next. Two of them also stopped being reported as "that key was not accepted", which had drivers
  re-pasting a key that was never the problem.

- **A refusal from ElevenLabs is now always written to the log file**, with the reason ElevenLabs gave
  for it. Some of them left no record at all, which made "I pasted my key and got nothing" impossible
  to answer from a log a driver sent in. The log never contains your key.

- **The board no longer marks the car two seconds behind you as a lap down.** It counted laps by
  comparing lap numbers, and a driver right behind you has not crossed the line yet — so his lap number
  is one lower than yours while he is on exactly the same piece of track. The board called him lapped
  traffic, which was already misleading and became a whole row of the wrong colour once lapped cars
  started being tinted. It now counts the way the engineer does, so the two can no longer tell you
  different things about the same car.

- **A car a lap ahead of you is blue and a car a lap behind is red — they were the wrong way round.**
  The phone was tinting the cars you had lapped in blue and the cars that had lapped you in red, which
  is exactly backwards: blue is the colour of the flag you are shown when a faster car is coming
  through. It is also the whole row that is tinted now rather than a small badge on it, so you can read
  it without looking straight at it.

- **During qualifying, your engineer only talks about your lap.** Fuel, gaps, traffic and the rest of
  the race radio used to keep talking through a qualifying session as if it were a race; now that
  content stays quiet and only what qualifying is actually about — your lap, the session itself — gets
  through.

- **After the checkered flag, the radio winds down instead of carrying on.** Your engineer used to
  keep reporting fuel, the car ahead, the car behind — all race information about a race that just
  ended. Now the radio says only what closes out a session: a word about how it went, and to bring it
  to the pits.

- **The radio goes quiet when you are not on track.** Sitting in the garage or a menu used to still get
  you spotter and engineer calls meant for a car that is out there; now nothing is said until you
  actually are.

- **Cars sitting in their pit box no longer show up on your phone's board.** A car in the pits is not
  someone you are racing at that moment, and it was taking up a row that could have shown one who is.

- **Your phone's predicted lap time stopped getting stuck empty.** A brief reconnect — a Wi-Fi blip, a
  moment in the background — used to throw away your reference lap entirely, so on a phone that
  reconnects more than a desktop ever does, PREDICTED could sit on "no reference lap" for the whole
  session. A short reconnect no longer costs you it.

- **Leaving your session for the garage or a menu now takes your phone back to the waiting screen.**
  With iRacing still running but no car of yours on track, the panel used to keep showing whatever it
  had last drawn instead of admitting there was no longer a race to report.

- **The lap marker no longer pushes a driver's name off the row.** On smaller screens at the largest
  size the `+1L` badge could overflow the name beside it. It now gives way to the name, since the row's
  own colour already says the same thing.

- **Pressing play twice no longer leaves two voices talking over each other.** Every preview in the app
  — a voice on `Voice → Voice packs`, "hear the whole chain", a line in the phrase catalog, a studio
  voice audition, a designed candidate — used to get a player of its own, so pressing play again while
  one was still speaking gave you both sentences at once, and a third press gave you three. There is one
  player now: pressing play ends whatever it was saying and says the new one instead, wherever in the app
  the two presses were. Previews after the first are also quicker to start, because the audio device now
  stays open between them. Nothing changes about what the engineer says during a race — those were
  always one at a time.

- **Your predicted lap time has an actual reference to compare against now.** The timer that builds
  PREDICTED was watching the same thread that speaks to you, and that thread waits for a phrase to
  finish playing before it does anything else — so any lap with a callout on it (which is most laps)
  looked like a gap the timer couldn't explain, and it gave up on that lap rather than guess. It now
  watches the telemetry feed directly, which never waits on the radio, so a lap finishing in silence
  is no longer the only kind that counts.

- **The Windows build no longer prints a spurious linker warning about a missing debug-symbol file
  for the audio codec.** Nothing the driver sees changed — this is build hygiene, so the next release
  logs stay readable.

### Removed

- **The desktop app no longer has its own Dash screen.** The cockpit panel is a phone thing now:
  choosing the theme and the blocks it draws happens on the phone app, and the desktop page that used
  to hold those settings was left pointing at the phone with nothing to point at. The quick "Cockpit
  dash" switch in the rail — the QR code, the address and the server's on/off — is still there, one
  click from any screen.

- **Home is now just your race history.** The "Your engineer is ready" checklist, the "Radio last
  race" strip and the "Your engineer" summary card are gone — engineer health is the app's own
  problem, not a checklist to read before the green flag. Home keeps the race log (your results, laps
  and what the engineer spent the radio on), your track memory and the list of recent sessions. The
  keyboard shortcuts shifted by one to match: Debrief is now Alt+5 and Settings Alt+6.

- **The spotter no longer counts down "dez, nove, oito... um" before the green flag.** The reminders
  further out — five minutes down to thirty seconds before the start — are unchanged.

- **Removed the first-run setup wizard.** PitCrew now opens straight to the app — set your name,
  voice, and language from the Voice screen whenever you like, and the header tells you what's still
  missing.

## [0.1.0] - 2026-08-05
### Added

- **The first public beta.** PitCrew speaks to you during an iRacing session: the spotter (a car
  alongside, which side, and when you are clear), fuel (consumption per lap, whether it reaches the
  end, and warnings as the tank runs down), gaps and position (filtered by class, so another class's
  leader is not reported as if it were your race), your lap times and personal bests, session flags
  and pit and penalty calls, and a memory of your best known lap for each car/track combination.
- **Brazilian Portuguese or English**, chosen independently of the interface, which is English.
  Spanish, German and French are written but not released yet.
- **A local voice that needs no account and no network.** Speech is synthesized on your own machine;
  an ElevenLabs API key is an optional upgrade to the studio voice, never a requirement, and an
  install without one gives up no information at all.
- **An overlay** over the sim — proximity bars and the last message as text — transparent,
  always-on-top and click-through, so it cannot steal a click mid-race. It keeps working with
  iRacing in exclusive fullscreen.
- **Three global shortcuts:** `Alt+H` shows or hides the overlay, `Alt+R` repeats the last thing
  said, `Alt+M` mutes the engineer.
- **Timing you can tune yourself** without waiting for a release — spotter debounce, fuel warning
  thresholds, the gap window and the opening silence, all in an optional `tuning.json`.
- Windows installer builds and GitHub releases are now produced by CI, and published here together
  with these notes.
- The website uses Google Analytics to count visits and how many people click through to download.
- **PitCrew now measures how the app itself is used, and there is a page listing exactly what that
  means** — [what PitCrew sends](docs/what-pitcrew-sends.md). The short version: it reports what
  happens in the app, never who you are or how you drive. Whether the first-run voice download
  succeeded or failed, whether setup finished, whether PitCrew found iRacing, that a race was logged
  and how much the engineer said in it. Never your name, your engineers' names, your API key, your lap
  times or your telemetry, and nothing you typed. An earlier version of these notes said the app made
  no analytics requests at all; that is no longer true, and this replaces it rather than sitting next
  to it. There is no switch in the app — the page explains how to block it at the network, and PitCrew
  is built so that blocking it cannot break anything.
- **A way to support PitCrew, at the bottom of the home screen.** There is no paid version and
  donating unlocks nothing — every feature is already yours. It is there because the app has no paid
  tier, no ads and no account, and is written and tested by one person after work. Pick $5, $10, $25
  or your own amount; the button opens PayPal in your own browser, where you confirm before anything
  is charged. Your card details never reach PitCrew.
- **The same way to support PitCrew, on the website**, under the download button rather than in front
  of it. It offers the same $5, $10 and $25 as the app, or an amount you choose, and it says plainly
  what the money is for: the installer is not code-signed, so Windows warns you twice, and a signing
  certificate is the next thing that fixes for everyone who installs. Donating still unlocks nothing.

### Changed

- The website now says what the first launch actually does: PitCrew downloads a 310 MB voice model
  the first time you open it, and runs offline from then on. It previously promised that a clean
  install worked offline, which was never true — and a driver on a metered connection would have
  found that out after installing.
- The website lists the two languages PitCrew speaks today, Brazilian Portuguese and English, rather
  than the five it has phrases written for. Spanish, German and French still need a voice before they
  can be spoken, and the site now says so instead of counting them.
- Every quoted radio call on the website is shown in Portuguese *and* English. The page's own
  interface is English and its readers are not only Portuguese speakers, so a column of untranslated
  quotes was decoration where it was meant to be the evidence.
- The website's "who speaks what" table can now be read in any of the five languages PitCrew has
  phrases written for, and the lines in it are the real ones out of the phrase catalog rather than
  examples written for the page. Picking Spanish, German or French says plainly that the words exist
  but the voice does not yet.
- The website's sections have more room to breathe, and the closing invitation to download gets some
  runway before the ask.
- The website no longer assumes your engineer is a man. It says "them" where it said "him", and the
  two example engineers now carry voice names that match the names they are given — the local one was
  labelled with a female voice under a man's name.
- The website's links and buttons are big enough to hit with a thumb. On a phone the four section
  links at the top were about a third of the size they should have been, and they are the only way to
  skip a section on a page that runs to some nine thousand pixels.
- The website is served with a strict content-security policy and the usual browser protection
  headers, so nothing it did not ship can run on it, and it cannot be framed by another site.

### Fixed

- **The website's download button now downloads PitCrew.** Every link on the page pointed back into
  the page itself, so the button at the bottom — the one every other button leads to — did nothing at
  all when clicked. It, and the two above it, now go to the latest release.
- The website's footer links go where they say: the install guide, getting started, voices and
  languages, troubleshooting, the release notes and the issue tracker. All twelve previously scrolled
  back to the top of the page.
- The four numbered items in the website's header now scroll to the sections they name, and the
  header's own menu lists them in the order the page presents them. Section 03 was missing from the
  menu entirely, and the menu disappeared completely on a phone.
- Picking a voice in the website's voice-designer panel no longer has the panel change it back a few
  seconds later.
- The website no longer renders as a blank page when the browser is not running animations — a
  restored background tab, low-power mode, or an in-app browser. All of its text was hidden until an
  animation revealed it.
- Sharing a link to the website now produces a preview card with an image, instead of an empty one.

### Removed

- The website's hero is much quieter: eight things were moving continuously behind the headline, and
  now two are.
