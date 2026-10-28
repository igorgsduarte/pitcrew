# Getting started

This guide makes a plugin that shows the race flag on its own screen. It takes about 15 minutes.

## 1. Install the tools

1. Install Rust from <https://rustup.rs>.
2. Add the WebAssembly target:

   ```sh
   rustup target add wasm32-wasip2
   ```

3. Clone this repository and build the developer tool:

   ```sh
   git clone https://github.com/igorgsduarte/pitcrew
   cd pitcrew/plugins
   cargo build --release -p pcplugin
   ```

   The tool is now at `target/release/pcplugin` (`pcplugin.exe` on Windows).

## 2. Copy the example

Copy `examples/hello` to a new folder, for example `my-plugin`. Then change these values:

| File | Field | Change it to |
| --- | --- | --- |
| `Cargo.toml` | `name` | your crate name, e.g. `my-plugin` |
| `plugin.json` | `id` | `<you>.<plugin>`, lowercase, e.g. `jsmith.pit-timer` |
| `plugin.json` | `name`, `publisher`, `description` | your text |
| `plugin.json` | `entry` | the file name of your `.wasm`, e.g. `pit-timer.wasm` |

If your plugin is in this repository, add its folder to `members` in the root `Cargo.toml`.

If your plugin is in its own repository:
1. Copy `wit/` next to it, and change `path: "../../wit"` in `src/lib.rs` to `"wit"`.
2. In `Cargo.toml`, replace each `.workspace = true` with a value: `edition = "2024"`,
   `license = "MIT"`, `serde_json = "1"`.

## 3. Read the example

`src/lib.rs` has four parts.

**The bindings.** This line reads the WIT and makes Rust types and functions from it:

```rust
wit_bindgen::generate!({ world: "plugin", path: "../../wit" });
```

**The state.** A plugin runs on one thread. Keep your state in a `thread_local!`:

```rust
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}
```

**The exports.** PitCrew calls the functions of the `Guest` trait. The example uses five of them:

| Function | When PitCrew calls it |
| --- | --- |
| `init` | Once, after it loads the plugin. Return `Err` to stop. |
| `on_flag` | When the flag changes, and once after `init`. |
| `on_session` | When a sim connects or disconnects. |
| `render` | When the driver has the plugin's screen open and you asked for a redraw. |
| `on_ui_event` | When the driver clicks something on your screen. |

[api.md](api.md) lists all of them.

**The screen.** `render` returns a flat list of nodes. Each node names its parent:

```rust
node("page", None, NodeKind::Page(PageProps { title: "Hello".into(), subtitle: None })),
node("card", Some("page"), NodeKind::Card(CardProps { title: Some("Flag".into()), description: None })),
node("flag", Some("card"), NodeKind::Text(TextProps { text: flag, tone: Tone::Normal, mono: true })),
```

When your data changes, call `host::invalidate_ui()`. PitCrew then calls `render` again, at most 5
times a second. [ui.md](ui.md) lists all node kinds.

## 4. Build

```sh
cargo build --release --target wasm32-wasip2 -p my-plugin
```

The component is in `target/wasm32-wasip2/release/my_plugin.wasm`.

## 5. Validate

```sh
pcplugin validate my-plugin
```

The tool checks `plugin.json` with the same rules as the app. It lists every problem at once.

## 6. Run

```sh
pcplugin run my-plugin --wasm target/wasm32-wasip2/release/my_plugin.wasm --ui --seconds 10
```

`pcplugin run` loads your plugin with the app's limits. It plays a scripted session: no sim, start
lights, green, yellow, and more. You see:

- each step of the script,
- your plugin's log lines and status,
- your screen as an outline (with `--ui`),
- the slowest call and its time against the 5 ms budget.

To test one flag, use `--flag`:

```sh
pcplugin run my-plugin --wasm … --flag yellow+waving
```

To click your button at 2 seconds, use `--press wave@2`. [testing.md](testing.md) has all options.

## 7. Publish

1. Put your plugin in a public GitHub repository.
2. Make a release with `pcplugin pack`. See [releasing.md](releasing.md).
3. Open a pull request on this repository that adds `registry/<id>.json`. See [registry.md](registry.md).

## Next

- [manifest.md](manifest.md): every field of `plugin.json`.
- [permissions.md](permissions.md): what each permission allows.
- [serial-devices.md](serial-devices.md): drive an Arduino or ESP32.
- [content-packs.md](content-packs.md): ship data that drivers can swap, such as animations.
