# pitcrew-plugins

Plugins add features to [PitCrew](https://pitcrew.nouss.dev), the voice race engineer. A plugin can
drive a device on a serial port, show its own screen in the app sidebar and react to the race flag.

This repository holds:

| Folder | What it is |
| --- | --- |
| [`wit/`](wit/) | The plugin API, `pitcrew:plugin@1`, as a WIT world. |
| [`docs/`](docs/) | The guide for plugin authors. Start with [getting-started.md](docs/getting-started.md). |
| [`tools/pcplugin/`](tools/pcplugin/) | The developer tool: validate, package and run a plugin on your PC. |
| [`examples/hello/`](examples/hello/) | The smallest useful plugin. Copy it to start. |
| [`plugins/flags/`](plugins/flags/) | **Flags**, the official plugin for an 8×8 LED flag matrix, with its firmware. |
| [`crates/pitflag-wire/`](crates/pitflag-wire/) | The PitFlag serial protocol, shared by the plugin and the tests. |
| [`schemas/`](schemas/) | JSON schemas for `plugin.json`, content packs and registry entries. |
| [`registry/`](registry/) | One file per published plugin. The app's catalogue is built from it. |

## How a plugin works

1. **You write the plugin** in Rust or another language that builds a WASI 0.2 component.
2. **PitCrew runs it in a sandbox** in its engine. The plugin can only use the permissions in its
   `plugin.json`. It cannot open files, the network or a port by name.
3. **The plugin declares its screen** as a list of nodes (page, card, table, button, pixel grid…).
   The app draws them with its own components, so every plugin looks like PitCrew.
4. **You publish a release** on GitHub with one zip, `<id>-<version>.zip`.
5. **You register it** with a pull request that adds `registry/<id>.json`. After review, drivers find
   it in the app under **Plugins → Discover** and install it with **Download**.

## Quick start

```sh
rustup target add wasm32-wasip2
cargo build --release -p pcplugin
cargo build --release -p hello-plugin --target wasm32-wasip2
target/release/pcplugin run examples/hello --wasm target/wasm32-wasip2/release/hello_plugin.wasm --ui
```

Read [docs/getting-started.md](docs/getting-started.md) for the full walk-through.

## Status

The API is `pitcrew:plugin@1`, version 1.0.0. The PitCrew app does not load plugins yet. Until it
does, `pcplugin run` is the host: it applies the same imports, permission checks and limits.

## Licence

MIT. See [LICENSE](LICENSE).
