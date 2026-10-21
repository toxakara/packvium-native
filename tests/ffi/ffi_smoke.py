"""Calls the built `packvium-ffi` shared library through its C ABI.

    python tests/ffi/ffi_smoke.py <path to libpackvium_ffi.{so,dylib} or packvium_ffi.dll>

Loads the library with ctypes, exactly as a C host would, and checks the ownership
contract as well as the answer: every returned string is released with
`packvium_free_string`.
"""

from __future__ import annotations

import ctypes
import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests" / "fixtures" / "five-items.json"


def bind(path: str) -> ctypes.CDLL:
    # Absolute, because Windows does not resolve a relative library path against the cwd.
    library = ctypes.CDLL(str(Path(path).resolve()))
    for name in ("packvium_version", "packvium_solve_json"):
        getattr(library, name).restype = ctypes.c_void_p
    library.packvium_solve_json.argtypes = [ctypes.c_char_p]
    library.packvium_free_string.argtypes = [ctypes.c_void_p]
    library.packvium_free_string.restype = None
    return library


def take(library: ctypes.CDLL, pointer: int | None) -> str:
    """The string a call returned, released back to the library that allocated it."""
    if not pointer:
        raise AssertionError("the library returned a null pointer")
    try:
        return ctypes.string_at(pointer).decode("utf-8")
    finally:
        library.packvium_free_string(pointer)


def without_duration(result: str) -> object:
    """The result minus every `duration_ms`, the one field that measures the clock.

    The primary result and each of its alternatives carry one, so all are removed.
    """
    return _strip_durations(json.loads(result))


def _strip_durations(value: object) -> object:
    if isinstance(value, dict):
        return {key: _strip_durations(item) for key, item in value.items() if key != "duration_ms"}
    if isinstance(value, list):
        return [_strip_durations(item) for item in value]
    return value


def main(path: str) -> int:
    library = bind(path)
    declared = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    reported = take(library, library.packvium_version())
    if reported != declared:
        raise AssertionError(f"library reports {reported!r}, the workspace declares {declared!r}")

    request = FIXTURE.read_bytes()
    first = take(library, library.packvium_solve_json(request))
    result = json.loads(first)
    if not result["complete"] or result["summary"]["packed_item_count"] != 5:
        raise AssertionError(f"unexpected result: {first[:200]}")
    if without_duration(take(library, library.packvium_solve_json(request))) != without_duration(first):
        raise AssertionError("the same request gave a different answer")

    refused = json.loads(take(library, library.packvium_solve_json(None)))
    if refused.get("status") != "error":
        raise AssertionError(f"a null request was not refused: {refused}")

    print(f"C ABI {reported} passed against {path}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    raise SystemExit(main(sys.argv[1]))
