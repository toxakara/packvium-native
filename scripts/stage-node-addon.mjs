#!/usr/bin/env node
// Put a cargo-built N-API library where the loader and its platform package expect it.
//
//   node scripts/stage-node-addon.mjs <rust-target> [library]
//
// `library` defaults to cargo's output for `--target <rust-target>`. The binary is written
// beside `bindings/node/index.js`, which a source checkout loads, and into the matching
// `bindings/node/npm/<platform>/` package, which is what npm publishes.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BINDING = path.join(ROOT, 'bindings', 'node');

/** npm platform suffix and cargo's file name for the library, per declared target. */
const TARGETS = {
  'aarch64-apple-darwin': ['darwin-arm64', 'libpackvium_node_native.dylib'],
  'x86_64-apple-darwin': ['darwin-x64', 'libpackvium_node_native.dylib'],
  'x86_64-pc-windows-msvc': ['win32-x64-msvc', 'packvium_node_native.dll'],
  'x86_64-unknown-linux-gnu': ['linux-x64-gnu', 'libpackvium_node_native.so'],
};

const [target, library] = process.argv.slice(2);
const declared = JSON.parse(fs.readFileSync(path.join(BINDING, 'package.json'), 'utf8')).napi.targets;
if (JSON.stringify([...declared].sort()) !== JSON.stringify(Object.keys(TARGETS).sort())) {
  throw new Error(`package.json napi.targets ${declared} differ from the targets this script stages`);
}
if (!TARGETS[target]) {
  throw new Error(`usage: stage-node-addon.mjs <${Object.keys(TARGETS).join('|')}> [library]`);
}

const [platform, fileName] = TARGETS[target];
const source = path.resolve(library ?? path.join(ROOT, 'target', target, 'release', fileName));
const binary = `packvium-native.${platform}.node`;
for (const destination of [path.join(BINDING, binary), path.join(BINDING, 'npm', platform, binary)]) {
  fs.copyFileSync(source, destination);
  process.stdout.write(`${path.relative(ROOT, source)} -> ${path.relative(ROOT, destination)}\n`);
}
