# `@packvium/native`

The compiled Rust engine for [`@packvium/engine`](https://www.npmjs.com/package/@packvium/engine),
as a prebuilt N-API addon. It is not meant to be installed on its own.

npm installs exactly one of its optional platform packages, the one matching the host:

| Package | Platform |
| --- | --- |
| `@packvium/native-darwin-arm64` | macOS, Apple silicon |
| `@packvium/native-darwin-x64` | macOS, Intel |
| `@packvium/native-linux-x64-gnu` | Linux x64, glibc |
| `@packvium/native-win32-x64-msvc` | Windows x64 |

The loader in `index.js` names every binary it can load by a literal specifier, and refuses
a binary built for a different release than its own.

Every package is published by the tagged release workflow of
[packvium-native](https://github.com/toxakara/packvium-native) with npm provenance.

Documentation: [packvium.com](https://packvium.com). Licence: MIT.
