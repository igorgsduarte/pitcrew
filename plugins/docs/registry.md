# The registry

The registry is the `registry/` folder of this repository. One file per plugin. The app's catalogue
(**Plugins → Discover**) and the website are built from it.

## Add a plugin

Open a pull request that adds `registry/<id>.json`:

```json
{
  "$schema": "../schemas/registry-entry.schema.json",
  "id": "jsmith.pit-timer",
  "repo": "jsmith/pitcrew-pit-timer",
  "releases": [
    {
      "version": "1.0.0",
      "channel": "stable",
      "tag": "v1.0.0",
      "asset": "jsmith.pit-timer-1.0.0.zip",
      "sha256": "…",
      "bytes": 412000,
      "api": "pitcrew:plugin@2"
    }
  ]
}
```

- `pcplugin pack` prints the `releases` entry for you.
- `channel` is `beta` for a `-beta.N` version and `stable` otherwise. See
  [releasing.md](releasing.md#channels).
- Do not add `tier`. The reviewer sets it.
- To publish an update, add a new entry at the top of `releases`. Never change an old entry.
- The official plugins in this repository do not open a pull request: their release workflow adds
  the entry.

## What happens after the pull request

1. **CI checks the entry.** It downloads the asset from
   `https://github.com/<repo>/releases/download/<tag>/<asset>`. It compares the sha256 and size, runs
   the zip checks, and checks that `plugin.json` has the same id, version and API.
2. **CI rebuilds the `.wasm`** from the tagged source for the Community tier and compares it.
3. **A reviewer reads the code** with the [review checklist](review-checklist.md).
4. **After the merge,** CI writes `registry/index.json` and signs it (`index.json.minisig`).

The app trusts only the signed index. It takes the tier, the hash and the catalogue text from it,
never from the package.

## Tiers

| Tier | Who | The driver sees |
| --- | --- | --- |
| Official | PitCrew's own repositories | "Official" |
| Community | Reviewed plugins from other authors | "Reviewed by PitCrew, written by &lt;author&gt;" |
| Content | Reviewed content packs | "Community content" |
| Sideloaded | A local folder or zip, in developer mode only | An amber banner: "Not reviewed by PitCrew" |

## Status of the registry CI

Steps 1, 2 and 4 are not built yet. Until they are, a maintainer runs `pcplugin check` on the asset
and fills `sha256` and `bytes` by hand.
