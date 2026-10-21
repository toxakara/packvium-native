// Runs the staged N-API addon through the loader npm consumers get.
//
// A plain script rather than `node --test`, so the same file runs on every Node release the
// addon supports, Node 16 included. Stage the binary first:
//   node scripts/stage-node-addon.mjs <rust-target> [library]

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const BINDING = path.join(ROOT, 'bindings', 'node');
const require = createRequire(import.meta.url);

const checks = [];
const check = (name, body) => checks.push([name, body]);

check('the loader names every module it can load by a literal specifier', () => {
  const loader = fs.readFileSync(path.join(BINDING, 'index.js'), 'utf8');
  const specifiers = [...loader.matchAll(/require\(([^)]*)\)/g)].map((match) => match[1]);
  assert.ok(specifiers.length > 0);
  for (const specifier of specifiers) {
    assert.match(specifier, /^'[^']+'$/, `non-literal require(${specifier})`);
  }
  assert.doesNotMatch(loader, /child_process|process\.env|import\(/);
});

const native = require(BINDING);
const manifest = JSON.parse(fs.readFileSync(path.join(BINDING, 'package.json'), 'utf8'));
const request = fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'five-items.json'), 'utf8');

check('the binary reports the release its package names', () => {
  assert.equal(native.version(), manifest.version);
});

check('it packs every item and the answer passes the reference validator', () => {
  const result = native.packJson(request);
  const decoded = JSON.parse(result);
  assert.equal(decoded.complete, true);
  assert.equal(decoded.summary.packed_item_count, 5);
  assert.deepEqual(JSON.parse(native.validateJson(request, result)), { issues: [] });
});

/** The result minus `algorithm.duration_ms`, the one field that measures the clock. */
const withoutDuration = (result) => {
  const decoded = JSON.parse(result);
  delete decoded.algorithm.duration_ms;
  return decoded;
};

check('the same request gives the same answer', () => {
  assert.deepEqual(withoutDuration(native.packJson(request)), withoutDuration(native.packJson(request)));
});

check('a malformed request throws an error carrying its code', () => {
  assert.throws(() => native.packJson('{"items": "not a list"}'),
    (error) => error instanceof Error && typeof error.code === 'string');
});

for (const [name, body] of checks) {
  body();
  process.stdout.write(`ok - ${name}\n`);
}
process.stdout.write(
  `native addon ${native.version()} passed on ${process.platform}/${process.arch} `
  + `with Node ${process.versions.node}\n`,
);
