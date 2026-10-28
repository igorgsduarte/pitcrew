# Your plugin's screen

A plugin does not draw pixels. It describes its screen as a list of nodes, and PitCrew draws them
with its own components (shadcn/ui). So every plugin looks like PitCrew, follows the app's theme and
works with the keyboard and screen readers.

## Where the screen appears

1. Add `nav` to `plugin.json`. See [manifest.md](manifest.md#nav-the-sidebar-item).
2. When the driver turns the plugin on, PitCrew adds your item to the sidebar, in the **Plugins**
   group under the core screens.
3. When the driver opens it, PitCrew draws a frame: your page title, your tier, your status and a
   **Manage** button. Your nodes go inside the frame.

## The tree

WIT has no recursive types, so the tree is a flat list. Each node has:

- `id`: unique in the tree, 1 to 64 characters. Events name the node by its id.
- `parent`: the id of its parent. Only the `page` has none.
- `kind`: what it is, with its properties.

Rules that PitCrew checks on every `render`:

1. There is exactly one root, and it is a `page`.
2. A node comes after its parent in the list.
3. Ids are unique.
4. At most 500 nodes.

If a tree breaks a rule, PitCrew does not draw it. It shows the error in place of your screen and
writes it to `plugin.log`. `pcplugin run --ui` applies the same checks.

## Node kinds

| Kind | Draws as | Properties | Events |
| --- | --- | --- | --- |
| `page` | The page body (root only) | `title`, `subtitle` | |
| `tabs` | Tabs | `selected`: the id of a `tab` child | `selected(tab-id)` |
| `tab` | One tab; its children are its content | the label | |
| `card` | A card | `title`, `description` | |
| `grid` | Columns that fold to one on a narrow window | `columns` 1 to 3, `gap` | |
| `stack` | A row that wraps | `gap` | |
| `separator` | A line | | |
| `text` | Text | `text`, `tone` (normal, muted, warning, error), `mono` | |
| `badge` | A status badge | `text`, `level` (ok, idle, warning, error) | |
| `alert` | An alert box | `title`, `text`, `level` | |
| `table` | A table | `columns`, `rows` (at most 200), `selectable` | `selected(first cell of the row)` |
| `progress` | A progress bar | `value` 0.0 to 1.0, `label` | |
| `button` | A button | `label`, `variant` (primary, secondary, destructive), `icon` (lucide), `disabled` | `pressed` |
| `switch` | A switch | `label`, `checked`, `disabled` | `toggled(bool)` |
| `slider` | A slider | `label`, `min`, `max`, `step`, `value` | `changed-number(f64)` |
| `select` | A drop-down | `label`, `options`, `value` | `selected(value)` |
| `input` | A text field | `label`, `value`, `max-length` | `changed-text(string)` |
| `pixel-grid` | A grid of LEDs | `width`, `height` (1 to 32), `palette` (≤ 16 colours, index 0 is off), `pixels`, `editable` | `painted(x, y, color)` |
| `settings-form` | The manifest's settings form | the setting keys to show | handled by PitCrew |

**The primary button.** PitCrew draws your `primary` variant in its neutral style. The brand
colour is for the app's own actions only.

**Colours.** You choose colours only in a `pixel-grid` palette, because those are data (an LED's
colour). Everything else uses the app's theme.

## Events and state

The screen has no state of its own. When the driver clicks:

1. PitCrew calls `on-ui-event` with the node id and the event.
2. You change your state.
3. You call `invalidate-ui`.
4. PitCrew calls `render` and draws the new tree.

For tabs, keep the selected tab in your state:

```rust
("tabs", Event::Selected(tab)) => self.tab = tab,
```

## Performance

- `render` runs only while your screen is open.
- PitCrew calls it at most 5 times a second, however often you call `invalidate-ui`.
- Build the tree from your state. Do not read files or ports in `render`.

## Example

The Live tab of the Flags plugin:

```
page "Flags"
  tabs (selected: live)
    tab "Live"
      grid 2 columns
        card "Matrix"
          pixel-grid 8×8
          text "showing yellow-waving"
        card "Now"
          table
      card "Test without a sim"
        stack
          button "lights-ready"  …
        button "Play every flag"
    tab "Animations"
    tab "Device"
```

Print your own tree with `pcplugin run <dir> --ui`.
