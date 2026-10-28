# Releasing

A release is one zip on a GitHub release: `<id>-<version>.zip`. The app downloads only that file.

## Channels

Every plugin has two channels:

| Channel | Version | GitHub release |
| --- | --- | --- |
| `beta` | `MAJOR.MINOR.PATCH-beta.N`, e.g. `1.1.0-beta.2` | a pre-release |
| `stable` | `MAJOR.MINOR.PATCH`, e.g. `1.1.0` | a release |

The registry entry of each release has `"channel": "beta"` or `"channel": "stable"`.

## 1. Set the version

Change `version` in `plugin.json`. Use a new version for every release. The registry pins each
version to one sha256, so a version can never change after it is published.

The plugins in this repository do not do this by hand: see [The release workflow](#the-release-workflow).

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

## The release workflow

The plugins in this folder release with **Plugins · Release**
(`.github/workflows/plugins-release.yml` at the root of `igorgsduarte/pitcrew`). It runs only by
hand: **Actions → Plugins · Release → Run workflow**, on `main`.

| Input | Values | What it does |
| --- | --- | --- |
| `plugin` | a folder in `plugins/` | The plugin to release. Add a new plugin to the list in the workflow. |
| `channel` | `beta`, `stable` | The channel. |
| `bump` | `none`, `patch`, `minor`, `major` | `none` releases the version in `plugin.json`, or the next beta of it. A bump starts from the latest stable release. |

Examples, for the Flags plugin with `plugin.json` at `1.0.0`:

| Run | Release |
| --- | --- |
| beta, none (first time) | `1.0.0-beta.1` |
| beta, none (again) | `1.0.0-beta.2` |
| stable, none | `1.0.0` |
| beta, minor | `1.1.0-beta.1` |
| stable, patch | `1.0.1` |

The workflow:

1. Computes the version from the tags `<plugin>-v*` and `plugin.json`
   ([tools/release/release.py](../tools/release/release.py)). It stops if the version exists.
2. Moves `## [Unreleased]` of the plugin's `CHANGELOG.md` into the new version. It stops if that
   section is empty, so add each change there before you release.
3. Runs the tests, builds the `.wasm` and `pcplugin`, validates, packs and checks the zip.
4. For Flags, builds the four PitFlag firmware images on Windows.
5. Commits `plugin.json`, `CHANGELOG.md` and `registry/<id>.json` to `main`, and tags
   `<plugin>-v<version>`.
6. Makes the GitHub release "Plugin · <name> <version>" with the zip (and the firmware images). A
   beta is a pre-release. No plugin release is ever marked latest: `/releases/latest` is the app's.

If `main` moves while the workflow runs, it stops before the commit. Run it again.

**Plugins · CI** (`plugins-ci.yml`) tests every push and pull request that touches `plugins/`. It
never releases.
