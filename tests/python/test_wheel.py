"""Runs the installed `packvium-native-rust` wheel the way the selector does.

Run against an installed wheel, never the source tree: `python -m unittest discover -s
tests/python`. The interpreter is the point -- the release runs this on the Python 3.9 floor
the `abi3-py39` tag promises.
"""

from __future__ import annotations

import json
import unittest
from importlib.metadata import version
from pathlib import Path

import packvium_native_rust

FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "five-items.json"


def without_duration(result: str) -> dict[str, object]:
    """The result minus `algorithm.duration_ms`, the one field that measures the clock."""
    decoded = json.loads(result)
    del decoded["algorithm"]["duration_ms"]
    return decoded


class InstalledWheel(unittest.TestCase):
    def setUp(self) -> None:
        self.request = FIXTURE.read_text()

    def test_the_module_comes_from_an_installed_wheel(self) -> None:
        self.assertIn("site-packages", Path(packvium_native_rust.__file__).parts)

    def test_the_compiled_version_is_the_distribution_version(self) -> None:
        self.assertEqual(packvium_native_rust.version(), version("packvium-native-rust"))

    def test_it_packs_every_item_and_the_answer_passes_the_reference_validator(self) -> None:
        result = packvium_native_rust.pack_json(self.request)
        decoded = json.loads(result)
        self.assertTrue(decoded["complete"])
        self.assertEqual(decoded["summary"]["packed_item_count"], 5)
        self.assertEqual(packvium_native_rust.validate_json(self.request, result), [])

    def test_the_same_request_gives_the_same_answer(self) -> None:
        self.assertEqual(without_duration(packvium_native_rust.pack_json(self.request)),
                         without_duration(packvium_native_rust.pack_json(self.request)))

    def test_a_malformed_request_raises_the_packvium_error(self) -> None:
        with self.assertRaises(packvium_native_rust.PackviumError):
            packvium_native_rust.pack_json('{"items": "not a list"}')


if __name__ == "__main__":
    unittest.main()
