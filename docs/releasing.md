# Releasing

A release is one zip on a GitHub release: `<id>-<version>.zip`. The app downloads only that file.

## 1. Set the version

Change `version` in `plugin.json` (and in `Cargo.toml`). Use a new version for every release. The
registry pins each version to one sha256, so a version can never change after it is published.

## 2. Build and pack

```sh
cargo build --release --target wasm32-wasip2 -p my-plugin
pcplugin pack my-plugin --wasm target/wasm32-wasip2/release/my_plugin.wasm --tag v1.2.0
```

`pack` writes `dist/<id>-<version>.zip` and prints:

- the sha256 and the size,
- the release entry for `registry/<id>.json`.

The zip holds `plugin.json`, the `.wasm` (named as `entry`), the bundled packs, and `README.md`,
`CHANGELOG.md`, `LICENSE` (as `LICENSE.txt`) and `icon.png` when they exist.

`pack` makes the same bytes from the same inputs: it sorts the entries and gives them a fixed date.

## 3. What the app refuses

The app checks the zip before it extracts anything. `pcplugin check <zip>` runs the same checks:

- a sha256 or size that is not the registry's,
- an entry with an absolute path, `..`, a drive letter, a backslash or a symlink,
- more than 500 entries,
- more than 64 MiB unpacked,
- a file that is not `.wasm .json .md .png .txt`,
- no `plugin.json` at the root.

After the checks, the app validates `plugin.json` and installs the plugin **off**.

## 4. Publish

1. Make a tag, e.g. `v1.2.0`.
2. Make a GitHub release for the tag and attach the zip as the only asset.
3. Add the release to `registry/<id>.json` with a pull request. See [registry.md](registry.md).

Do not delete or replace a published asset. The registry would refuse the new file, and drivers
would see "This download does not match the registry. Nothing was installed."

## Automate it

[.github/workflows/ci.yml](../.github/workflows/ci.yml) in this repository builds, tests, packs and
releases the Flags plugin when a tag `flags-v*` is pushed. Copy it for your plugin.
