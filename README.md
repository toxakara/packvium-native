# packvium-native

The compiled distributions of the Packvium engine, built from the Rust source in this
repository by its tagged release workflow:

| Artifact | Where it is published | Used by |
| --- | --- | --- |
| `packvium-native-rust` — one `abi3` wheel per platform, CPython 3.9+ | [PyPI](https://pypi.org/project/packvium-native-rust/) | [`packvium-native`](https://pypi.org/project/packvium-native/), the Python backend selector |
| `@packvium/native` and its four per-platform packages | [npm](https://www.npmjs.com/package/@packvium/native) | [`@packvium/engine`](https://www.npmjs.com/package/@packvium/engine) |
| `packvium-ffi` — the C ABI as a shared library, with [`include/packvium.h`](include/packvium.h) | this repository's GitHub Releases | the [PHP native bridge](https://github.com/toxakara/packvium-php-bridge), and any C-compatible host |

None of them is meant to be the first thing you install. Each is the fast path behind a
package that also works without it, and gives the same answers either way.

Full documentation, the constraint reference and benchmarks live at
[packvium.com](https://packvium.com).

## Platforms

| Platform | Wheel | Node addon | C library |
| --- | --- | --- | --- |
| Linux x86_64, glibc | `manylinux_2_28` | `@packvium/native-linux-x64-gnu` | `libpackvium_ffi.so` |
| Linux aarch64, glibc | `manylinux_2_28` | — | `libpackvium_ffi.so` |
| macOS arm64 | yes | `@packvium/native-darwin-arm64` | `libpackvium_ffi.dylib` |
| macOS x86_64 | yes | `@packvium/native-darwin-x64` | `libpackvium_ffi.dylib` |
| Windows x86_64 | yes | `@packvium/native-win32-x64-msvc` | `packvium_ffi.dll` |

Every binary is built on a native runner of the platform it names, never cross-compiled,
and is loaded and run on that runner before it is collected.

## Install

```bash
pip install packvium-native-rust    # Python
npm install @packvium/native        # Node.js
```

npm installs only the platform package that matches the host. `@packvium/engine` does not load
`@packvium/native`: it uses the addon bundled with it where there is one and its JavaScript
engine otherwise. The C library is a release asset: download the file for your platform from
the release, verify it (below) and point your host at it.

## Verify what you downloaded

Every wheel, npm tarball and shared library is attested by the workflow that built it. With
the GitHub CLI:

```bash
gh attestation verify libpackvium_ffi-x86_64-unknown-linux-gnu.so --repo toxakara/packvium-native
```

npm shows the same provenance on each package page, and `npm audit signatures` checks it
for an installed tree. Each release also carries `SHA256SUMS.txt` for its C libraries.

## The C ABI

JSON in, JSON out, and every returned string is owned by the caller:

```c
#include "packvium.h"

char *result = packvium_solve_json(request_json);
/* use result */
packvium_free_string(result);
```

Release a returned pointer exactly once with `packvium_free_string`, never with the host
allocator. Invalid input and caught panics come back as JSON errors rather than crashes.
The request and result format is the one every Packvium engine uses:
[docs/PUBLIC-API.md](docs/PUBLIC-API.md), with exact units in
[docs/UNITS-AND-NUMERICS.md](docs/UNITS-AND-NUMERICS.md).

## Building from source

Rust 1.97.1 is pinned in `rust-toolchain.toml`.

```bash
cargo build --release --locked -p packvium-ffi                     # the C library
cd bindings/python && maturin build --release --locked --out dist  # a wheel for this host
cargo build --release --locked -p packvium-node-native             # the Node addon
node scripts/stage-node-addon.mjs aarch64-apple-darwin target/release/libpackvium_node_native.dylib
```

The tests in `tests/` run each artifact the way its consumer does:

```bash
python -m unittest discover -s tests/python -v         # against an installed wheel
node tests/node/addon.test.mjs                         # against the staged addon
python tests/ffi/ffi_smoke.py target/release/libpackvium_ffi.dylib
```

## Releases

A `v*` tag builds every artifact, tests it on its own platform, attests it, publishes the
wheels to PyPI and the npm packages (platform packages first, then the root that depends
on them) through trusted publishing, and cuts the GitHub Release with the C libraries
attached. No registry token is stored in this repository. Changes are listed in
[CHANGELOG.md](CHANGELOG.md).

The engine itself is developed in the open in
[packvium-rust](https://github.com/toxakara/packvium-rust); its correctness suite runs
there. This repository holds the same source so that every binary can be traced to the
commit it was built from.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). Licence: MIT,
see [LICENSE](LICENSE).
