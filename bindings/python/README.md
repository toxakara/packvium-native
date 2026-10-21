# packvium-native-rust

The compiled Rust engine behind [`packvium-native`](https://pypi.org/project/packvium-native/),
built as one `abi3` wheel per platform for CPython 3.9 and newer.

It is not meant to be installed on its own. Install the selector instead, which picks this
wheel when it is present and falls back to the pure-Python
[`packvium`](https://pypi.org/project/packvium/) package when it is not:

```bash
pip install 'packvium-native[native]' packvium
```

Wheels are published for Linux (manylinux_2_28, x86_64 and aarch64), macOS (arm64 and
x86_64) and Windows (x86_64). Each wheel is built by the tagged release workflow of
[packvium-native](https://github.com/toxakara/packvium-native) and carries a build
provenance attestation.

The module answers the same JSON request contract as every Packvium engine:

```python
import json
import packvium_native_rust

result = json.loads(packvium_native_rust.pack_json(json.dumps(request)))
issues = packvium_native_rust.validate_json(json.dumps(request), json.dumps(result))
```

Documentation: [packvium.com](https://packvium.com). Licence: MIT.
