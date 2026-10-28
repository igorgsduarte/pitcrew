"""Tests for release.py. Run: python3 -m unittest discover -s tools/release"""

import json
import tempfile
import unittest
from pathlib import Path

from release import ReleaseError, cut_changelog, next_version, registry_entry, set_manifest_version


class NextVersion(unittest.TestCase):
    def test_the_first_release_uses_the_manifest_version(self):
        self.assertEqual(next_version("1.0.0", [], "beta", "none"), "1.0.0-beta.1")
        self.assertEqual(next_version("1.0.0", [], "stable", "none"), "1.0.0")

    def test_betas_count_up_from_the_tags(self):
        self.assertEqual(next_version("1.0.0-beta.1", ["1.0.0-beta.1"], "beta", "none"), "1.0.0-beta.2")
        self.assertEqual(next_version("1.0.0-beta.2", ["1.0.0-beta.1", "1.0.0-beta.2"], "beta", "none"), "1.0.0-beta.3")

    def test_stable_promotes_the_beta(self):
        self.assertEqual(next_version("1.0.0-beta.2", ["1.0.0-beta.1", "1.0.0-beta.2"], "stable", "none"), "1.0.0")

    def test_a_bump_starts_from_the_latest_stable(self):
        tags = ["1.0.0-beta.1", "1.0.0", "1.0.1"]
        self.assertEqual(next_version("1.0.1", tags, "beta", "minor"), "1.1.0-beta.1")
        self.assertEqual(next_version("1.0.1", tags, "stable", "patch"), "1.0.2")
        self.assertEqual(next_version("1.0.1", tags, "stable", "major"), "2.0.0")

    def test_a_bump_continues_the_betas_of_its_target(self):
        tags = ["1.0.0", "1.1.0-beta.1"]
        self.assertEqual(next_version("1.1.0-beta.1", tags, "beta", "minor"), "1.1.0-beta.2")

    def test_a_released_stable_cannot_be_released_again(self):
        with self.assertRaises(ReleaseError):
            next_version("1.0.0", ["1.0.0"], "stable", "none")
        with self.assertRaises(ReleaseError):
            next_version("1.0.0", ["1.0.0"], "beta", "none")

    def test_a_version_below_the_latest_stable_is_refused(self):
        with self.assertRaises(ReleaseError):
            next_version("1.0.0", ["1.2.0"], "stable", "none")

    def test_other_tags_are_ignored(self):
        self.assertEqual(next_version("1.0.0", ["junk", "1.0"], "beta", "none"), "1.0.0-beta.1")


class Manifest(unittest.TestCase):
    def test_only_the_top_level_version_changes_and_the_layout_stays(self):
        text = '{\n  "id": "a.b",\n  "version": "1.0.0",\n  "settings": [ { "key": "x" } ]\n}\n'
        out = set_manifest_version(text, "1.0.0-beta.1")
        self.assertEqual(out, text.replace('"1.0.0"', '"1.0.0-beta.1"'))


CHANGELOG = """# Changelog: Flags

The release workflow moves `## [Unreleased]` into the new version's section.

## [Unreleased]

### Added
- The matrix test.

## [0.9.0] - 2026-01-01

### Added
- Old.
"""


class Changelog(unittest.TestCase):
    def test_unreleased_becomes_the_version_and_a_new_unreleased_opens(self):
        out, notes = cut_changelog(CHANGELOG, "1.0.0-beta.1", "2026-10-28")
        self.assertIn("## [Unreleased]\n\n## [1.0.0-beta.1] - 2026-10-28\n\n### Added\n- The matrix test.\n\n## [0.9.0]", out)
        self.assertEqual(notes, "### Added\n- The matrix test.\n")
        self.assertTrue(out.startswith("# Changelog: Flags\n\nThe release workflow moves `## [Unreleased]` into"))
        self.assertEqual(out.count("- The matrix test."), 1)

    def test_an_empty_unreleased_is_refused(self):
        with self.assertRaises(ReleaseError):
            cut_changelog("# C\n\n## [Unreleased]\n\n## [0.9.0] - x\n", "1.0.0", "2026-10-28")

    def test_a_stable_after_betas_carries_their_notes(self):
        cl, _ = cut_changelog(CHANGELOG, "1.0.0-beta.1", "2026-10-28")
        out, notes = cut_changelog(cl, "1.0.0", "2026-10-30")
        self.assertIn("## [1.0.0] - 2026-10-30\n\nIncludes 1.0.0-beta.1.\n\n## [1.0.0-beta.1]", out)
        self.assertIn("- The matrix test.", notes)


class Registry(unittest.TestCase):
    def test_the_entry_has_the_channel_hash_and_size(self):
        with tempfile.TemporaryDirectory() as d:
            z = Path(d) / "a.b-1.0.0-beta.1.zip"
            z.write_bytes(b"abc")
            e = registry_entry({"version": "1.0.0-beta.1", "api": "pitcrew:plugin@1"}, z, "x-v1.0.0-beta.1")
        self.assertEqual(e["channel"], "beta")
        self.assertEqual(e["bytes"], 3)
        self.assertEqual(e["sha256"], "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        self.assertEqual(json.loads(json.dumps(e))["asset"], "a.b-1.0.0-beta.1.zip")


if __name__ == "__main__":
    unittest.main()
