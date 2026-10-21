# Contributing

Thanks for looking. This repository turns one Rust engine into the binaries other packages
load — please keep it to that.

## Getting set up

You need Rust 1.97.1 (`rust-toolchain.toml` selects it), Python 3.9 or newer with
[maturin](https://www.maturin.rs/), and Node.js 16 or newer.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Then build and run each artifact the way its consumer does — the commands are in the
README's *Building from source* section. CI runs exactly those on every platform in the
README's table.

## The rules that matter here

**No engine changes here.** `crates/packvium-core` is the published engine's source,
carried so that every binary is traceable to the commit it was built from. Solver
behaviour, constraints and the request format change in
[packvium-rust](https://github.com/toxakara/packvium-rust), where their tests live, and
reach this repository from there.

**A binding forwards; it does not decide.** `bindings/python`, `bindings/node` and
`crates/packvium-ffi` translate between a host's types and the engine's JSON contract. A
result that depends on which binding produced it is a bug.

**Every binary is run where it was built.** A platform is supported when CI builds its
binary on a native runner and runs the matching test there. Cross-compiled artifacts are
not published.

**The loader stays readable.** `bindings/node/index.js` names every module it can load as
a literal `require('...')`. No specifier from a variable, no environment override, no
child processes — `tests/node/addon.test.mjs` checks this.

**`Cargo.lock` is part of the release.** Builds use `--locked`, so a dependency moves only
in a change that says so.

## Pull requests

- One logical change per pull request.
- Commit messages in imperative mood, under 72 characters:
  `type(scope): description` with `feat`, `fix`, `refactor`, `chore`, `docs` or `test`.
- Do not bump versions; releases are cut separately.

## Reporting a bug

Include the platform, the package and version you installed, and the request that
reproduces it. For anything with security implications, follow [SECURITY.md](SECURITY.md)
rather than opening a public issue.
