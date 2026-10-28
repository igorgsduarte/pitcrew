# Testing

Test a plugin at two levels:

1. **Unit tests, native.** Fast, and they need no sim or device.
2. **`pcplugin run`.** The real `.wasm` in a host with the app's limits.

## 1. Unit tests

Keep your logic away from the generated bindings. The Flags plugin does it like this:

- `src/host.rs` defines a `Host` trait with what the plugin needs: log, status, ports, settings,
  content.
- All logic (`app.rs`, `link.rs`, `player.rs`, …) uses only that trait.
- `src/lib.rs` has a `#[cfg(target_arch = "wasm32")] mod wasm` that implements the trait with the
  generated imports.
- The tests implement the trait with a fake host and a fake device in memory.

Then `cargo test -p your-plugin` runs on your PC in under a second:

```sh
cargo test -p pitcrew-flags
```

Good things to test:

- every flag gives the animation you expect,
- a device that is unplugged and plugged in again,
- every tab renders a tree that the host accepts (copy the checks from `ui::validate`),
- bad content gives a clear error, not a panic.

## 2. `pcplugin run`

```sh
pcplugin run <plugin-dir> --wasm <file.wasm> [options]
```

| Option | Use |
| --- | --- |
| `--script "none:2,ready:1,go:1,yellow+waving:3"` | A session: steps of `tokens:seconds`. It loops. |
| `--flag yellow+waving` | One step that never ends. |
| `--seconds 10` | Stop after 10 s. Use it in CI. |
| `--port COM4` | Use a real device. The default is a fake PitFlag device. |
| `--set brightness=80` | Change a setting. The value is JSON, or text. |
| `--action test@2` | Press a manifest action at 2 s. |
| `--press wave@2` | Click a button on your screen at 2 s. |
| `--select tabs=device@3` | Pick a tab, a table row or an option at 3 s. |
| `--ui` | Print your last screen as an outline. |
| `--plain` | Draw the matrix with `.` and `#`, for logs. |
| `--data <dir>` | Your plugin's data folder. The default is a temp folder. |

**Script tokens:** `none` (no sim), `ready`, `set`, `go`, a flag (`green`, `yellow`, `blue`,
`white`, `red`, `checkered`, `black`, `repair`, `disqualify`), `waving`, `debris`, `one-lap`,
`driver-black`, `driver-repair`, `driver-disqualify`. Join them with `+`.

**What `run` checks, as the app does:**

- each call stays inside its budget (5 ms; `init` 250 ms),
- memory stays under 16 MiB,
- each tree from `render` passes the [ui rules](ui.md#the-tree),
- `serial::open` and `content::write-own` need their permissions.

When a check fails, `run` stops with the reason and exits with code 1.

At the end, `run` prints the number of calls and the slowest one. Keep the slowest call well under
5 ms. A slow PC in a race is slower than your desk.

## 3. On the rig

Before you publish a plugin that drives hardware:

1. Run it with `--port` on the real device for 10 minutes.
2. Unplug the cable while it runs. The status must say so, and it must recover when you plug it in.
3. Close `pcplugin` with Ctrl+C. The device must go dark within 1.5 s.
