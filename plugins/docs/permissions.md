# Permissions

A plugin asks for permissions in `plugin.json`. The driver sees them as plain sentences before they
turn the plugin on. The app writes those sentences, not you, so they always mean the same thing.

| Permission | The driver reads | What it allows |
| --- | --- | --- |
| `topic:flag` | "Sees the race flags." | `on-flag` calls. |
| `topic:session` | "Knows when a sim is running." | `on-session` calls with the sim, the track and the session kind. |
| `devices:serial` | "Uses the USB serial port you choose." | `serial::open`, only for a port the driver picked in a `serialPort` setting. |
| `devices:firmware` | "Writes firmware to the device you choose." | The `firmware` interface: board detection and firmware upload, only on a port the driver picked in a `serialPort` setting. |
| `content:write-own` | "Saves files in its own folder." | `content::write-own`, inside the plugin's data folder, at most 1 MiB a file. |

What no permission allows:

- reading or writing other files on the PC,
- the network,
- opening a port by name, or a port that the driver did not pick,
- starting a program,
- speaking with PitCrew's voice.

## Rules

- Ask only for what you use. The review refuses a permission that the code does not need.
- A `serialPort` setting needs `devices:serial`. `pcplugin validate` checks it.
- A new permission in an update asks the driver again before the update turns on.
