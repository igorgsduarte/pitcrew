# Content packs

A content pack is data that a plugin reads: animations, mappings, colour sets. Drivers can install
other packs and choose one in the plugin's settings. Nothing in a pack runs.

## Who does what

| Role | Does |
| --- | --- |
| The plugin | Names the content type in `provides`, e.g. `flags.animations`. Reads packs with `content::read`. Validates what it reads. |
| A pack author | Makes a package with `"kind": "content"` and `"extends": "flags.animations"`. |
| PitCrew | Installs packs, lists them with `content::packs`, checks the package limits. |

The plugin owns the format of its content type. Document it in the plugin's README, and give a
JSON schema if you can.

## A pack on disk

```
content/classic/
  pack.json
  mapping.json
  animations/
    yellow.json
    yellow-waving.json
    …
```

`pack.json` needs four fields. The rest belongs to the content type.

```json
{
  "id": "pitcrew.flags.classic",
  "name": "Classic",
  "type": "flags.animations",
  "version": "1.0.0",
  "animations": ["idle", "yellow", "yellow-waving"],
  "hasMapping": true
}
```

## Bundled packs

A plugin can ship packs: list their folders in `packs` in `plugin.json`. A plugin should ship one
pack that works alone, so it works with no other download.

## A content-only package

```json
{
  "id": "jsmith.neon-flags",
  "name": "Neon flags",
  "version": "1.0.0",
  "publisher": "J. Smith",
  "kind": "content",
  "extends": "flags.animations",
  "packs": ["neon"]
}
```

It has no `entry`, `api`, `permissions`, `tickHz` or `nav`. `pcplugin validate` refuses them.

## Packs that a plugin writes

With `content:write-own`, a plugin can write `packs/<name>/pack.json` and its files in its own data
folder. That pack then appears in `content::packs`. The Flags editor saves **My pack** this way.

## The Flags format: `flags.animations`

**An animation** is one file in `animations/`. Its name is the file name without `.json`, and `id`
must be the same.

```json
{
  "id": "yellow-waving",
  "palette": { ".": "000000", "Y": "FFD400" },
  "fps": 8,
  "loop": true,
  "frames": [
    ["..YYYYYY", ".YYYYYY.", "YYYYYY..", "YYYYY..Y", "YYYY..YY", "YYY..YYY", "YY..YYYY", "Y..YYYYY"]
  ]
}
```

- `palette`: one character to `RRGGBB`. `.` is required and means off. At most 16 entries.
- `fps`: 1 to 30.
- `loop`: `true` repeats; `false` holds the last frame.
- `frames`: 1 to 120 frames. Each frame is 8 rows of 8 palette characters, top row first.

**The mapping** (`mapping.json`) picks the animation for a flag. The first rule that matches wins.

```json
[
  { "when": { "driver": "disqualify" }, "show": "black-x" },
  { "when": { "start": "go" }, "show": "green-flash", "holdS": "setting:greenHoldS", "then": "idle" },
  { "when": { "primary": "yellow", "waving": true }, "show": "yellow-waving" },
  { "when": {}, "show": "idle" }
]
```

- `when` fields: `primary`, `waving`, `debris`, `oneLapToGreen`, `start`, `driver`. They match the
  [flag signal](api.md#the-flag-flag-signal). An empty `when` matches everything; put it last.
- `holdS`: show for this many seconds, then show `then` (or `idle`). A number, or
  `"setting:greenHoldS"` for the driver's setting.
- A pack without a mapping (`"hasMapping": false`) uses the mapping of the built-in pack. It can
  replace some animations and keep the rest.

Schemas: [animation.schema.json](../schemas/animation.schema.json),
[mapping.schema.json](../schemas/mapping.schema.json), [pack.schema.json](../schemas/pack.schema.json).
