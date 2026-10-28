# Review checklist

A reviewer uses this list before a plugin gets the Community tier. The sandbox already stops a
plugin from reaching files, the network or other ports. The review looks at what the sandbox cannot
see: behaviour.

## Package

- [ ] `pcplugin validate` passes.
- [ ] `pcplugin check` passes on the release asset, and its sha256 is the one in the entry.
- [ ] The `.wasm` that CI rebuilt from the tag has the same sha256 as the one in the zip.
- [ ] The `id` is new and does not look like another plugin's id or name.
- [ ] The README says what the plugin does, which hardware it needs and how to set it up.
- [ ] The licence lets PitCrew distribute the zip.

## Permissions

- [ ] Each permission is used by the code. Remove the ones that are not.
- [ ] A serial plugin uses only the port from its `serialPort` setting.

## Behaviour

- [ ] `pcplugin run --seconds 60` passes, with the slowest call under 2.5 ms (half the budget).
- [ ] With no sim connected, the plugin shows nothing on its device.
- [ ] An unplugged device gives a warning status, and the plugin recovers when it comes back.
- [ ] `on-stop` turns the device off.
- [ ] The plugin does not write to its data folder on every tick.
- [ ] The status text is short, in en-US, and says what the driver can do.

## Screen

- [ ] Every tab renders with `--ui` and passes the checks.
- [ ] The screen does not look like a PitCrew core screen or a system message.
- [ ] The screen does not ask for passwords, payment or personal data.
- [ ] Text is en-US.

## Content packs

- [ ] Every file passes the content type's schema.
- [ ] The pack works with the plugin's built-in mapping, or ships its own.
