#!/usr/bin/env node
// Counts committed source bytes without consulting .gitattributes or exclusions.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const EXTENSIONS = new Set((
  'rs js mjs cjs jsx ts mts cts tsx vue svelte astro css scss sass less ' +
  'html htm sh bash zsh fish ps1 psm1 bat cmd py pyi go c h cc hh cpp hpp ' +
  'cxx hxx m mm cs fs fsx vb java kt kts scala swift rb php r R jl lua ' +
  'pl pm ex exs erl hrl hs lhs clj cljs cljc edn dart zig nim nims v ' +
  'sol move wasm wat s S asm sql rego proto tf tfvars hcl cue cmake '
).trim().split(/\s+/));
const FORBIDDEN_DIRECTORY = /(^|\/)(node_modules|target|dist|build|coverage|playwright-report|test-results)(\/|$)/;
const AGENT_STATE = /(^|\/)(\.claude|\.codex)\/(projects|sessions|logs|history(?:\.[^/]*)?|cache|auth\.json|\.?credentials[^/]*)(\/|$)/;
const UNOWNED_RUST = /(^|\/)(vendor|vendored|third_party|third-party|external|generated|gen)(\/|$)|\.generated\.rs$/;

function isSource(path) {
  const name = basename(path);
  return EXTENSIONS.has(extname(path).slice(1)) ||
    /^(Dockerfile|Containerfile)(\.|$)/.test(name) ||
    /^(Makefile|GNUmakefile|CMakeLists\.txt)$/.test(name);
}

export function auditInventory(entries) {
  let rustBytes = 0;
  let sourceBytes = 0;
  let uncreditedRustBytes = 0;
  const failures = [];
  const files = [];
  for (const entry of [...entries].sort((a, b) => a.path.localeCompare(b.path))) {
    const { path, size, mode = '100644', type = 'blob' } = entry;
    if (type !== 'blob') continue;
    if (!Number.isSafeInteger(size) || size < 0) throw new Error(`Invalid size: ${path}`);
    if (FORBIDDEN_DIRECTORY.test(path) || AGENT_STATE.test(path) || /\.tsbuildinfo$/.test(path)) {
      failures.push({ code: 'TRACKED_BUILD_OR_AGENT_STATE', path });
    }
    if (!isSource(path)) continue;
    if (!/^100(644|755)$/.test(mode)) {
      failures.push({ code: 'NON_REGULAR_SOURCE_FILE', path });
      continue;
    }
    sourceBytes += size;
    const rust = path.endsWith('.rs');
    const credited = rust && !UNOWNED_RUST.test(path);
    if (credited) rustBytes += size;
    else if (rust) uncreditedRustBytes += size;
    files.push({ path, bytes: size, credited_rust: credited });
  }
  if (sourceBytes === 0) failures.push({ code: 'NO_SOURCE_CODE' });
  else if (rustBytes * 2 < sourceBytes) failures.push({ code: 'RUST_SHARE_BELOW_50_PERCENT' });
  return {
    schema_version: 1,
    basis: 'committed source bytes; all tests, scripts, frontend and integration code included',
    minimum_rust_percent: 50,
    rust_bytes: rustBytes,
    source_bytes: sourceBytes,
    uncredited_rust_bytes: uncreditedRustBytes,
    rust_percent: sourceBytes ? Number((100 * rustBytes / sourceBytes).toFixed(4)) : 0,
    failures,
    files,
    pass: failures.length === 0,
  };
}

function git(root, args) {
  return execFileSync('git', ['-C', root, ...args], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
}

export function auditRepository(root) {
  root = git(root, ['rev-parse', '--show-toplevel']).trim();
  const head = git(root, ['rev-parse', 'HEAD']).trim();
  const dirty = git(root, ['status', '--porcelain=v1', '-z', '--untracked-files=all']).length !== 0;
  const entries = git(root, ['ls-tree', '-r', '-l', '-z', 'HEAD']).split('\0').filter(Boolean).map(record => {
    const tab = record.indexOf('\t');
    const header = record.slice(0, tab).trim().split(/\s+/);
    if (tab < 0 || header.length !== 4) throw new Error('Invalid Git tree entry');
    const [mode, type, , bytes] = header;
    return { path: record.slice(tab + 1), mode, type, size: type === 'blob' ? Number(bytes) : 0 };
  });
  const report = auditInventory(entries);
  report.source_sha = head;
  report.clean_source = !dirty;
  if (dirty) report.failures.push({ code: 'DIRTY_SOURCE' });
  report.pass = report.failures.length === 0;
  return report;
}

function selfTest() {
  const f = (path, size, mode = '100644') => ({ path, size, mode });
  assert.equal(auditInventory([f('src/lib.rs', 100), f('frontend/main.ts', 100)]).pass, true);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('frontend/main.ts', 101)]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 1_000_000), f('main.ts', 1_000_001)]).pass, false);
  assert.equal(auditInventory([]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('Cargo.lock', 1_000_000), f('README.md', 2_000_000)]).rust_percent, 100);
  assert.equal(auditInventory([f('vendor/upstream.rs', 10_000), f('main.ts', 1)]).rust_bytes, 0);
  assert.equal(auditInventory([f('generated/types.rs', 10_000), f('main.ts', 1)]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('scripts/check.mjs', 101), f('.gitattributes', 1000)]).pass, false);
  for (const path of ['frontend/dist/app.js', 'target/debug/app.rs', 'x/node_modules/a.js', 'frontend/tsconfig.tsbuildinfo', '.claude/sessions/a.json', '.codex/auth.json', '.codex/history.jsonl', '.claude/.credentials.json']) {
    assert.equal(auditInventory([f('src/lib.rs', 10_000), f(path, 1)]).pass, false, path);
  }
  assert.equal(auditInventory([f('src/lib.rs', 100, '120000')]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('Dockerfile', 101)]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('policies/policy.rego', 101)]).pass, false);
  assert.equal(auditInventory([f('src/lib.rs', 100), f('run.sh', 101)]).pass, false);
  assert.throws(() => auditInventory([f('src/lib.rs', -1)]));
  const root = mkdtempSync(join(tmpdir(), 'rust-share-test-'));
  try {
    git(root, ['init', '--quiet']);
    mkdirSync(join(root, 'src'));
    writeFileSync(join(root, 'src', 'space\nname.rs'), 'fn main() {}\n');
    writeFileSync(join(root, 'README.md'), 'fixture\n');
    git(root, ['add', '.']);
    git(root, ['-c', 'user.name=Gate Test', '-c', 'user.email=gate-test@example.invalid', 'commit', '--quiet', '-m', 'fixture']);
    const clean = auditRepository(root);
    assert.equal(clean.pass, true);
    assert.equal(clean.source_sha, git(root, ['rev-parse', 'HEAD']).trim());
    assert.equal(clean.files[0].path, 'src/space\nname.rs');
    assert.deepEqual(auditRepository(join(root, 'src')), clean);
    writeFileSync(join(root, 'untracked.ts'), 'export {};');
    assert.equal(auditRepository(root).pass, false);
    rmSync(join(root, 'untracked.ts'));
    writeFileSync(join(root, 'README.md'), 'modified\n');
    assert.equal(auditRepository(root).pass, false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
  console.log('RUST_SHARE_SELF_TEST_PASS');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length === 1 && args[0] === '--self-test') selfTest();
    else {
      if (args.length > 1 || (args[0]?.startsWith('-'))) throw new Error('Usage: node dream-machine-rust-share.mjs [repository] | --self-test');
      const report = auditRepository(resolve(args[0] ?? '.'));
      console.log(JSON.stringify(report, null, 2));
      process.exitCode = report.pass ? 0 : 1;
    }
  } catch (error) {
    console.error(JSON.stringify({ pass: false, error: error.message }));
    process.exitCode = 2;
  }
}
