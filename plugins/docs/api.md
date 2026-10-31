# The plugin API: `pitcrew:plugin@2`

The API is the WIT world in [wit/pitcrew-plugin.wit](../wit/pitcrew-plugin.wit). This page explains
it. The WIT file is the reference.

## What PitCrew calls (exports)

| Export | When | Budget |
| --- | --- | --- |
| `init() -> result<_, string>` | Once, after load. Load your data here. Return `err` to stop; the driver sees the text. | 250 ms |
| `on-flag(signal)` | When the flag changes, and once after `init`. | 5 ms |
| `on-session(session)` | When a sim connects or disconnects, and once after `init`. | 5 ms |
| `settings-changed()` | After the driver saves the settings form. Read them again with `settings::get`. | 5 ms |
| `tick(now-ms)` | At the manifest's `tickHz`. `now-ms` is a monotonic clock in milliseconds. | 5 ms |
| `on-action(id)` | The driver pressed one of the manifest's `actions`. | 5 ms |
| `render() -> list<ui-node>` | Only while your screen is open, after you call `invalidate-ui`. | 5 ms |
| `on-ui-event(event)` | The driver clicked or changed a node on your screen. | 5 ms |
| `on-stop()` | The driver turned the plugin off, or the engine is stopping. Turn your device off. | 5 ms |

PitCrew calls them one at a time, on one thread. You do not need locks.

**Budget.** The budget counts the CPU time your call uses, not the time spent waiting in PitCrew's imports or while Windows runs other programs. If a call uses more than its budget, PitCrew stops it. Then it restarts the plugin after
1 s, 5 s and 30 s. After 3 failures in 10 minutes, it turns the plugin off and shows a red dot on its
sidebar item. Do slow work in small parts across `tick` calls.

## What PitCrew gives you (imports)

### `host`

| Function | Use |
| --- | --- |
| `log(level, text)` | Writes to the plugin's `plugin.log`. Use `debug` for detail. |
| `set-status(level, text)` | The status line on your sidebar item and in **Installed**. Call it when the status changes, not every tick. |
| `invalidate-ui()` | Ask for a new `render`. PitCrew draws at most 5 times a second, so you can call it freely. |

### `settings`

`get() -> string` returns a JSON object of setting key to value. Missing values have their `default`.

### `serial` (needs `devices:serial`)

`open(setting-key, baud) -> result<port, string>` opens the port that the driver picked in the
`serialPort` setting with that key. You cannot open a port by name.

A `port` has:
- `write(bytes) -> result<_, string>`
- `read(max) -> result<list<u8>, string>`: returns what is waiting, maybe nothing. It never blocks.

When the device is unplugged, `write` or `read` returns `err`. Drop the port, set a warning status
and try `open` again later. See [serial-devices.md](serial-devices.md).

### `firmware` (needs `devices:firmware`) — since 1.1.0

PitCrew recognises boards and writes firmware, so a plugin never speaks a bootloader protocol. It
works on the port the driver picked in a `serialPort` setting.

| Function | Use |
| --- | --- |
| `chips() -> list<string>` | The chip ids PitCrew can write: `esp32c3`, `esp32`, `atmega328p`, `atmega168`, `atmega32u4`. |
| `describe(setting-key) -> result<board, string>` | What the port's USB ids say. Instant; the board is not touched. |
| `start-probe(setting-key) -> result<_, string>` | Asks the board's bootloader which chip it is. Takes up to a few seconds. |
| `start-upload(setting-key, chip, image) -> result<_, string>` | Writes `image` and checks it. An ESP image is a merged `.bin` for flash address 0; an AVR image is the Intel HEX text. At most 8 MiB. |
| `job() -> option<job-status>` | The plugin's last probe or upload: `running`, `progress` (0 to 1), `text`, `error`, and for a probe the `board`. |

A probe or an upload runs in the background. **PitCrew takes the port for the length of the job**:
your open `port` returns `err` until it ends, and `serial::open` refuses the port. Drop your port when
you start a job, call `job()` on each `tick`, and open the port again when `running` is false.

PitCrew restarts the board into its bootloader itself (DTR and RTS, or the 1200-baud touch of a
Leonardo). The driver presses no button.


| Function | Permission | Use |
| --- | --- | --- |
| `packs(of-type) -> list<string>` | none | The installed pack ids of a type you `provide`. Your bundled packs come first. |
| `read(pack, path) -> result<list<u8>, string>` | none | A file from a pack, e.g. `animations/red.json`. |
| `write-own(path, bytes) -> result<_, string>` | `content:write-own` | Writes into your plugin's data folder. At most 1 MiB a file. A pack written to `packs/<name>/` appears in `packs`. |
| `read-own(path) -> result<list<u8>, string>` | none | Reads a file you wrote with `write-own`. Since API 1.2.0. |

Paths are relative, with `/`. A path with `..`, a drive or a backslash is refused.

## The flag: `flag-signal`

PitCrew reads the flag from the sim and decides what is out. Your plugin does not need to know the
sim.

| Field | Meaning |
| --- | --- |
| `primary: option<flag>` | The most important flag out: `disqualify`, `black`, `repair`, `red`, `checkered`, `white`, `yellow`, `blue`, `green`. `none` when nothing is out. |
| `waving: bool` | A full-course yellow. With `primary: yellow` and `waving: false`, it is a local yellow. |
| `debris: bool` | The surface flag (yellow with red stripes). |
| `one-lap-to-green: bool` | The restart is at the end of this lap. |
| `start: option<start-light>` | `ready`, `set`, `go`. Only from sims that publish start lights. |
| `driver: list<driver-flag>` | Flags for the player's own car: `black`, `repair`, `disqualify`. |

A sim that does not publish a field gives `none` or `false`. PitCrew never guesses a flag.

## The session: `session-info`

| Field | Meaning |
| --- | --- |
| `sim` | `iracing`, `assetto-corsa`, `assetto-corsa-evo` or `replay`. |
| `connected` | A sim is running and sending data. When `false`, show nothing on a flag device. |
| `in-car` | The driver is in the car, not in the garage or the menus. |
| `session-kind`, `track` | When the sim gives them. |

## The sandbox

A plugin is a WASI 0.2 component. PitCrew gives it:

- stderr, which goes to `plugin.log`. A Rust panic message goes there too.
- clocks and random numbers.
- at most 16 MiB of memory.

It does **not** give environment variables, arguments, files, sockets or other processes. A plugin
gets the outside world only through the imports above, and only with its permissions.

Plugins never speak. Voice is PitCrew's own.

## Versions

`@2` changes only by addition: new functions, new node kinds, new fields in new records. A plugin
built for `pitcrew:plugin@2.0.0` runs on every later `@2`. A removal or a change is `@3`.

`@2` added two events for an editable pixel grid, `filled` and `palette-changed`. A new case in an
event is a change for a plugin built before it, so `@1` plugins must be rebuilt against `@2`: change
`api` in `plugin.json` to `pitcrew:plugin@2` and build again. PitCrew does not load `@1` plugins.

## Other languages

Any language that builds a WASI 0.2 component can make a plugin. Generate bindings from `wit/`:

| Language | Tool |
| --- | --- |
| Rust | `wit-bindgen` (this repository uses 0.63) |
| C / C++ | `wit-bindgen c`, then `wasi-sdk` |
| Go | TinyGo with `wit-bindgen-go` |
| JavaScript | `componentize-js` (the component is large, a few MB) |

`pcplugin run` works with all of them.
