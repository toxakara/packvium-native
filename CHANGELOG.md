# Changelog

What this repository publishes, release by release: the `packvium-native-rust` wheels on
PyPI, `@packvium/native` and its platform packages on npm, and the C library on GitHub
Releases. The format follows [Keep a Changelog](https://keepachangelog.com/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.8.0]

The first published release of the compiled engines. Each binary is built from the Rust
source in this repository at the tagged commit, run on the platform it names, and attested.
The engine inside is the 1.8.0 core.

### Added

- **`packvium-native-rust` wheels**, one `abi3-py39` wheel each for Linux (manylinux_2_28,
  x86_64 and aarch64), macOS (arm64 and x86_64) and Windows (x86_64). Every wheel is
  installed into a clean CPython 3.9 and 3.14 and solves a request before it is published.
  It is the backend the `packvium-native` selector's `native` extra installs; that selector is
  not on PyPI yet.
- **`@packvium/native`** with four platform packages: `darwin-arm64`, `darwin-x64`,
  `linux-x64-gnu` and `win32-x64-msvc`. npm installs the one matching the host as an
  optional dependency. The loader names every binary it can load by a literal specifier and
  refuses a binary from another release.
- **The C library** — `libpackvium_ffi` for Linux (x86_64, aarch64) and macOS (arm64,
  x86_64), `packvium_ffi.dll` for Windows (x86_64) — attached to the GitHub Release with
  `packvium.h` and `SHA256SUMS.txt`.

## Before 1.8.0

No compiled engine was published. The Python selector and `@packvium/engine` answered from
their pure implementations, which give the same results.
