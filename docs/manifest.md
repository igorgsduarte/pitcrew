# plugin.json

Every package has a `plugin.json` at its root. The app reads it before it runs any code. The schema
is [schemas/plugin.schema.json](../schemas/plugin.schema.json). `pcplugin validate` applies the
same rules as the app.

## Example

```json
{
  "$schema": "../../schemas/plugin.schema.json",
  "id": "pitcrew.flags",
  "name": "Flags",
  "version": "1.0.0",
  "publisher": "PitCrew",
  "description": "Shows the race flags on an 8×8 LED matrix.",
  "license": "MIT",
  "repository": "https://github.com/jsmith/pitcrew-pit-timer",
  "kind": "plugin",
  "api": "pitcrew:plugin@1",
  "entry": "flags.wasm",
  "permissions": ["topic:flag", "topic:session", "devices:serial", "content:write-own"],
  "tickHz": 30,
  "settings": [
    { "key": "port", "type": "serialPort", "label": "Device port" },
    { "key": "brightness", "type": "integer", "label": "Brightness", "min": 1, "max": 255, "default": 40 }
  ],
  "actions": [{ "id": "test", "label": "Play every flag" }],
  "nav": { "label": "Flags", "icon": "flag" },
  "provides": ["flags.animations"],
  "packs": ["content/classic"]
}
```

## Fields

| Field | Required | Rule |
| --- | --- | --- |
| `id` | yes | Lowercase letters, digits and dashes, in two or more parts with dots: `<you>.<plugin>`. At most 64 characters. It never changes after the first release. |
| `name` | yes | 1 to 40 characters. Shown in the catalogue. |
| `version` | yes | `MAJOR.MINOR.PATCH`, digits only. |
| `publisher` | yes | Your name or team. The **tier** (Official, Community) comes from the registry, never from this field. |
| `description` | no | At most 300 characters. |
| `license` | no | An SPDX id, e.g. `MIT`. |
| `repository` | no | `https://github.com/<owner>/<repo>`: the repository with the releases. |
| `kind` | yes | `plugin` (has code) or `content` (data only). |
| `api` | plugin | `pitcrew:plugin@1`. |
| `entry` | plugin | The `.wasm` file name, at the package root. |
| `extends` | content | The content type the package adds to, e.g. `flags.animations`. |
| `permissions` | no | See [permissions.md](permissions.md). Ask only for what you use. |
| `tickHz` | no | How often `tick` runs: 1 to 60 a second. Leave it out if you do not need `tick`. |
| `settings` | no | The settings form. See below. |
| `actions` | no | Buttons on the plugin's row in **Plugins → Installed**. Each has an `id` and a `label` of at most 24 characters. A press calls `on-action(id)`. |
| `nav` | no | One sidebar item that opens your screen. See below. |
| `provides` | no | Content types that your plugin reads, e.g. `flags.animations`. |
| `packs` | no | Folders with content packs that ship with the plugin. See [content-packs.md](content-packs.md). |

## Settings

The app draws the settings form from `plugin.json` and saves the values. So the form works before
the plugin runs, after it stops with an error, and in safe mode. A driver can always fix a wrong
port.

| `type` | Value | Extra fields |
| --- | --- | --- |
| `serialPort` | A port the driver picks, e.g. `COM4`. Needs `devices:serial`. | |
| `integer` | A whole number | `min`, `max`, `default` |
| `number` | A number | `min`, `max`, `default` |
| `boolean` | `true` or `false` | `default` |
| `string` | Text | `default` |
| `select` | One of the options | `options: [{ "value", "label" }]`, `default` |
| `contentRef` | The id of a content pack | `of`: the content type; `default` |

Each setting has a `key` (camelCase, at most 32 characters) and a `label` (at most 40 characters).

Read the values with `settings::get()`. It returns a JSON object of key to value. When the driver
changes a value, PitCrew calls `settings-changed`.

To show the form on your own screen, add a `settings-form` node with the keys. See [ui.md](ui.md).

## nav: the sidebar item

```json
"nav": { "label": "Flags", "icon": "flag" }
```

- `label`: 1 to 16 characters.
- `icon`: a [lucide](https://lucide.dev/icons) icon name, e.g. `flag`, `gauge`, `timer`. A plugin
  cannot ship its own icon image for the sidebar.

The item appears in the sidebar when the driver turns the plugin on. It shows a status dot. A
plugin without `nav` has no screen; its settings are in **Plugins → Installed**.

The sidebar holds at most 8 plugin items. Other plugins open from **Plugins → Installed → Open**.
