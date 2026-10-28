#!/usr/bin/env python3
"""Version, changelog and registry steps of a plugin release. See docs/releasing.md.

  release.py prepare <plugin-dir> --channel beta|stable --bump none|patch|minor|major --out <dir>
  release.py register <plugin-dir> --zip <file> --tag <tag>

`prepare` computes the next version from the git tags `<folder>-v*` and plugin.json, writes the
version into plugin.json, cuts the `## [Unreleased]` section of the plugin's CHANGELOG.md, writes
<out>/notes.md and prints `key=value` lines for $GITHUB_OUTPUT.

`register` adds the release at the top of registry/<id>.json.

Standard library only, so it runs on any CI runner.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

VERSION_RE = re.compile(r"(\d{1,6})\.(\d{1,6})\.(\d{1,6})(?:-beta\.(\d{1,6}))?")
UNRELEASED = "## [Unreleased]"


class ReleaseError(Exception):
    pass


def parse(version: str) -> tuple[tuple[int, int, int], int | None]:
    m = VERSION_RE.fullmatch(version)
    if not m:
        raise ReleaseError(f'"{version}" is not MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-beta.N')
    base = (int(m[1]), int(m[2]), int(m[3]))
    return base, int(m[4]) if m[4] else None


def fmt(base: tuple[int, int, int]) -> str:
    return ".".join(str(n) for n in base)


def bumped(base: tuple[int, int, int], bump: str) -> tuple[int, int, int]:
    major, minor, patch = base
    if bump == "major":
        return (major + 1, 0, 0)
    if bump == "minor":
        return (major, minor + 1, 0)
    if bump == "patch":
        return (major, minor, patch + 1)
    raise ReleaseError(f"unknown bump {bump!r}")


def next_version(current: str, released: list[str], channel: str, bump: str) -> str:
    """The version to release. `released` holds the versions that already have a tag."""
    cur_base, _ = parse(current)
    parsed = [parse(v) for v in released if VERSION_RE.fullmatch(v)]
    stables = sorted(b for b, beta in parsed if beta is None)
    latest = stables[-1] if stables else None

    if bump == "none":
        target = cur_base
    else:
        target = bumped(latest or cur_base, bump)

    if target in stables:
        raise ReleaseError(f"{fmt(target)} is already released as stable. Choose a bump.")
    if latest and target < latest:
        raise ReleaseError(f"{fmt(target)} is below the latest stable {fmt(latest)}. Choose a bump.")

    if channel == "stable":
        return fmt(target)
    if channel != "beta":
        raise ReleaseError(f"unknown channel {channel!r}")
    n = max((beta for b, beta in parsed if b == target and beta is not None), default=0) + 1
    return f"{fmt(target)}-beta.{n}"


def set_manifest_version(text: str, version: str) -> str:
    out, count = re.subn(r'("version"\s*:\s*")[^"]*(")', lambda m: m[1] + version + m[2], text, count=1)
    if count != 1 or json.loads(out).get("version") != version:
        raise ReleaseError("could not set the top-level version in plugin.json")
    return out


def sections(changelog: str) -> list[tuple[str, str]]:
    """(heading, body) for every `## ` section, in file order."""
    parts = re.split(r"(?m)^(## .*)$", changelog)
    return [(parts[i].strip(), parts[i + 1]) for i in range(1, len(parts), 2)]


def cut_changelog(changelog: str, version: str, date: str) -> tuple[str, str]:
    """Moves `## [Unreleased]` into `## [version] - date`. Returns the new file and the notes.

    A stable release that follows betas of the same version also gets the betas' notes.
    """
    heading = re.search(r"(?m)^## \[Unreleased\][ \t]*$", changelog)
    if not heading:
        raise ReleaseError(f"CHANGELOG.md has no {UNRELEASED} section")
    secs = dict(sections(changelog))
    body = secs.get(UNRELEASED, "").strip()

    notes = body
    base, beta = parse(version)
    if beta is None:
        betas = [
            (h, b.strip())
            for h, b in sections(changelog)
            if (m := re.match(r"## \[(.+?)\]", h)) and VERSION_RE.fullmatch(m[1]) and parse(m[1])[0] == base and parse(m[1])[1] is not None
        ]
        if betas:
            included = ", ".join(re.match(r"## \[(.+?)\]", h)[1] for h, _ in reversed(betas))
            notes = "\n\n".join(x for x in [body, f"Includes {included}:"] + [b for _, b in betas] if x)
            body = "\n\n".join(x for x in [body, f"Includes {included}."] if x)
    if not notes:
        raise ReleaseError(f"{UNRELEASED} in CHANGELOG.md is empty. Add what changed.")

    new_section = f"{UNRELEASED}\n\n## [{version}] - {date}\n\n{body}\n\n"
    head, rest = changelog[: heading.start()], changelog[heading.end() :]
    after = re.split(r"(?m)^(?=## )", rest, maxsplit=1)
    tail = after[1] if len(after) > 1 else ""
    return (head + new_section + tail).rstrip() + "\n", notes + "\n"


def git_tags(prefix: str, cwd: Path) -> list[str]:
    out = subprocess.run(["git", "tag", "--list", f"{prefix}*"], cwd=cwd, check=True, capture_output=True, text=True).stdout
    return [t[len(prefix):] for t in out.split()]


def prepare(args: argparse.Namespace) -> None:
    plugin = Path(args.plugin_dir)
    manifest_path = plugin / "plugin.json"
    manifest_text = manifest_path.read_text(encoding="utf-8")
    manifest = json.loads(manifest_text)
    prefix = f"{plugin.resolve().name}-v"

    version = next_version(manifest["version"], git_tags(prefix, plugin), args.channel, args.bump)
    date = args.date or dt.date.today().isoformat()

    changelog_path = plugin / "CHANGELOG.md"
    changelog, notes = cut_changelog(changelog_path.read_text(encoding="utf-8"), version, date)

    manifest_path.write_text(set_manifest_version(manifest_text, version), encoding="utf-8", newline="\n")
    changelog_path.write_text(changelog, encoding="utf-8", newline="\n")
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "notes.md").write_text(notes, encoding="utf-8", newline="\n")

    for key, value in {
        "version": version,
        "tag": f"{prefix}{version}",
        "channel": args.channel,
        "id": manifest["id"],
        "name": manifest["name"],
    }.items():
        print(f"{key}={value}")


def registry_entry(manifest: dict, zip_path: Path, tag: str) -> dict:
    data = zip_path.read_bytes()
    _, beta = parse(manifest["version"])
    return {
        "version": manifest["version"],
        "channel": "stable" if beta is None else "beta",
        "tag": tag,
        "asset": zip_path.name,
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
        "api": manifest.get("api", ""),
    }


def register(args: argparse.Namespace) -> None:
    plugin = Path(args.plugin_dir)
    manifest = json.loads((plugin / "plugin.json").read_text(encoding="utf-8"))
    registry_path = Path(args.registry) / f"{manifest['id']}.json"
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    entry = registry_entry(manifest, Path(args.zip), args.tag)
    if any(r["version"] == entry["version"] for r in registry["releases"]):
        raise ReleaseError(f"{registry_path} already has {entry['version']}")
    registry["releases"].insert(0, entry)
    registry_path.write_text(json.dumps(registry, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{registry_path}: added {entry['version']} ({entry['channel']}, {entry['bytes']} bytes)")


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    pr = sub.add_parser("prepare")
    pr.add_argument("plugin_dir")
    pr.add_argument("--channel", required=True, choices=["beta", "stable"])
    pr.add_argument("--bump", required=True, choices=["none", "patch", "minor", "major"])
    pr.add_argument("--out", required=True)
    pr.add_argument("--date")
    rg = sub.add_parser("register")
    rg.add_argument("plugin_dir")
    rg.add_argument("--zip", required=True)
    rg.add_argument("--tag", required=True)
    rg.add_argument("--registry", default="registry")
    args = p.parse_args(argv)
    try:
        {"prepare": prepare, "register": register}[args.cmd](args)
    except ReleaseError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
