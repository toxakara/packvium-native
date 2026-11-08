# Changelog

What this repository publishes, release by release: the `packvium-native-rust` wheels on
PyPI, `@packvium/native` and its platform packages on npm, and the C library on GitHub
Releases. The format follows [Keep a Changelog](https://keepachangelog.com/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.10.0]

The compiled engines rebuilt on the 1.10.0 core: a pallet load may overhang its deck,
`gusseted_bag` joins the flat bag, a convex hull turns 24 ways, every placement carries a
`placement_id`, a request may declare its `currency`, and `maximum_value`,
`open_dimension_height` and `maximum_volume` state a lower bound. `units` now refuses every key
but `length`, and the grid lattice and block loader number containers per type (`large#1`, not
`large#10`). The Node addon is built with napi 3.16. The wheel and addon tests mask
`duration_ms` in every alternative, not only the answer.

## [1.9.0]

The compiled engines rebuilt on the 1.9.0 core: the centre of gravity is checked under a support
ratio, a length past `2^53 - 1` ticks or an empty side is refused with a pointer, nesting under a
clearance agrees with the other engines, and `exact_small` is 9-28% faster where it runs.

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
