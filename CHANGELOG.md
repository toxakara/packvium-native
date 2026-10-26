# Changelog

What this repository publishes, release by release: the `packvium-native-rust` wheels on
PyPI, `@packvium/native` and its platform packages on npm, and the C library on GitHub
Releases. The format follows [Keep a Changelog](https://keepachangelog.com/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.8.0]

The first release of the compiled engines. Each binary is built from the Rust source in this
repository at the tagged commit, tested on the platform it names, and attested. The engine
inside is the 1.8.0 core.

### Added

- **The C library** — `libpackvium_ffi` for Linux (x86_64, aarch64) and macOS (arm64,
  x86_64), `packvium_ffi.dll` for Windows (x86_64) — on the GitHub Release with `packvium.h`
  and `SHA256SUMS.txt`.
- **`packvium-native-rust` abi3-py39 wheels** for the same five targets, each installed into a
  clean CPython 3.9 and 3.14 and run before upload.
- **`@packvium/native`** with four platform packages (`darwin-arm64`, `darwin-x64`,
  `linux-x64-gnu`, `win32-x64-msvc`); its loader names every binary by a literal specifier and
  refuses one from another release.

The wheels and npm packages reach PyPI and npm only once their Trusted Publishers exist; until
then the release carries the C library alone.
