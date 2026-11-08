#!/usr/bin/env node
// Verify repository bytes before any gate execution. This is not product evidence.
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const requiredPaths = [
  'scripts/dream-machine-product-gate.mjs',
  'scripts/dream-machine-selection-gate.mjs',
  'scripts/dream-machine-backend-gate.mjs',
].sort();
const hash = (algorithm, bytes) => createHash(algorithm).update(bytes).digest('hex');
const fail = (message) => { throw new Error(`GATE_BUNDLE_MISMATCH: ${message}`); };

export function verifyGateBundle(root) {
  const manifest = JSON.parse(readFileSync(resolve(root, 'config/dream-machine-gate-bundle.json'), 'utf8'));
  if (manifest.schema_version !== 1 || !Array.isArray(manifest.modules)) fail('Invalid manifest schema');
  const paths = manifest.modules.map((item) => item.path).sort();
  if (JSON.stringify(paths) !== JSON.stringify(requiredPaths)) fail('Require exactly all three gate modules');
  for (const item of manifest.modules) {
    const path = resolve(root, item.path);
    if (!lstatSync(path).isFile()) fail(`Module must be a regular file: ${item.path}`);
    const bytes = readFileSync(path);
    if (!Number.isInteger(item.bytes) || item.bytes !== bytes.length) fail(`Byte count differs: ${item.path}`);
    if (!/^[a-f0-9]{64}$/.test(item.sha256 ?? '') || hash('sha256', bytes) !== item.sha256) {
      fail(`SHA-256 differs: ${item.path}`);
    }
    // Git object identity confirms these are the bytes advertised by the pinned tree.
    const object = Buffer.concat([Buffer.from(`blob ${bytes.length}\0`), bytes]);
    if (!/^[a-f0-9]{40}$/.test(item.git_blob_sha1 ?? '') || hash('sha1', object) !== item.git_blob_sha1) {
      fail(`Git blob identity differs: ${item.path}`);
    }
  }
  return { status: 'GATE_BUNDLE_BYTES_VERIFIED', modules: manifest.modules,
    limitation: 'Byte integrity only; execute readiness/product gates and independently review the running product.' };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const root = process.argv[2] ?? resolve(dirname(fileURLToPath(import.meta.url)), '..');
    console.log(JSON.stringify(verifyGateBundle(root), null, 2));
  } catch (error) {
    console.error(JSON.stringify({ status: 'GATE_BUNDLE_MISMATCH', error: error.message }));
    process.exitCode = 1;
  }
}
