// Source-bound structural checks; semantic and runtime review remain mandatory.
import { createHash } from 'node:crypto';
import { readFileSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve, sep } from 'node:path';

export function verifyBackend(report, root, evidence, manifest, receipts, sha) {
  const fail = (message) => { throw new Error(`NONCOMPLIANT_BACKEND: ${message}`); };
  const required = (value, label) => {
    if (typeof value !== 'string' || !value.trim()) fail(`Missing ${label}`);
    return value;
  };
  const list = (value, label) => {
    if (!Array.isArray(value) || !value.length) fail(`Missing ${label}`);
    return value;
  };
  const inside = (base, path) => {
    const rel = relative(base, path);
    return rel === '' || (!isAbsolute(rel) && rel !== '..' && !rel.startsWith(`..${sep}`));
  };
  const source = (path) => {
    required(path, 'source path');
    if (isAbsolute(path)) fail('Source paths must be relative');
    const target = realpathSync(resolve(root, path));
    if (!inside(realpathSync(root), target)) fail('Source path escapes checkout');
    if (!readFileSync(target).length) fail(`Empty source file: ${path}`);
    return target;
  };
  const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
  const metadataArtifact = manifest.artifacts.find((item) => item.kind === 'backend_metadata');
  if (!metadataArtifact) fail('Missing Cargo backend_metadata artifact');
  const metadataFile = realpathSync(resolve(evidence, required(metadataArtifact.path, 'metadata path')));
  if (isAbsolute(metadataArtifact.path) || !inside(realpathSync(evidence), metadataFile)) {
    fail('Metadata escapes evidence directory');
  }
  const bytes = readFileSync(metadataFile);
  if (digest(bytes) !== metadataArtifact.sha256) fail('Cargo metadata hash mismatch');
  const metadataReceipt = receipts.find((item) => item.suite === 'backend' &&
    item.source_sha === sha && item.log === metadataArtifact.path &&
    item.log_sha256 === metadataArtifact.sha256 &&
    typeof item.stderr_log === 'string' && /^[a-f0-9]{64}$/.test(item.stderr_sha256 ?? '') &&
    JSON.stringify(item.argv) === JSON.stringify([
      'cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
    ]));
  if (!metadataReceipt) fail('Cargo metadata needs its actual source-bound backend recorder receipt');
  const metadata = JSON.parse(bytes.toString('utf8'));
  if (metadata.version !== 1 || !isAbsolute(metadata.workspace_root ?? '') || !metadata.resolve?.nodes) {
    fail('Cargo metadata must contain the resolved dependency graph; --no-deps is insufficient');
  }
  const localPath = (path) => {
    if (!isAbsolute(path ?? '') || !inside(metadata.workspace_root, path)) {
      fail('Cargo workspace path is invalid');
    }
    return source(relative(metadata.workspace_root, path));
  };
  const packages = new Map(list(metadata.packages, 'Cargo packages').map((pkg) => [pkg.id, pkg]));
  const members = list(metadata.workspace_members, 'Cargo workspace members').map((id) => {
    const pkg = packages.get(id);
    if (!pkg) fail('Unknown Cargo workspace member');
    const path = localPath(pkg.manifest_path);
    // Match the Cargo package name to the immutable source manifest, not just a filename.
    const text = readFileSync(path, 'utf8');
    const section = text.match(/^\s*\[package\]\s*\r?\n([\s\S]*?)(?=^\s*\[|$(?![\s\S]))/m)?.[1];
    const name = section?.match(/^\s*name\s*=\s*["']([^"']+)["']/m)?.[1];
    if (name !== pkg.name) fail('Cargo package identity differs from source manifest');
    return { ...pkg, local_manifest: path };
  });
  const byManifest = new Map(members.map((pkg) => [pkg.local_manifest, pkg]));
  if (byManifest.size !== members.length) fail('Duplicate Cargo workspace manifest');
  const entrypoint = source(report.backend?.entrypoint);
  const service = members.find((pkg) => (pkg.targets ?? []).some((target) =>
    target.kind?.includes('bin') && localPath(target.src_path) === entrypoint));
  if (!service) fail('Backend entrypoint must be an actual workspace binary target');
  const edges = new Map();
  for (const node of metadata.resolve.nodes) {
    edges.set(node.id, new Set((node.deps ?? []).filter((dep) =>
      dep.dep_kinds?.some((kind) => kind.kind === null)).map((dep) => dep.pkg)));
  }
  const reachable = new Set();
  function visit(id) {
    if (reachable.has(id)) return;
    reachable.add(id);
    for (const dependency of edges.get(id) ?? []) visit(dependency);
  }
  visit(service.id);
  const owner = (path) => members.filter((pkg) => inside(dirname(pkg.local_manifest), path))
    .sort((a, b) => b.local_manifest.length - a.local_manifest.length)[0];
  const crates = list(report.backend?.crates, 'backend crates');
  const counted = new Set();
  for (const crate of crates) {
    const path = source(crate.manifest_path);
    const pkg = byManifest.get(path);
    if (!pkg || !reachable.has(pkg.id)) fail(`Crate is outside the backend production graph: ${crate.manifest_path}`);
    if (counted.has(path)) fail('Duplicate counted backend crate');
    counted.add(path);
    if (owner(source(crate.implementation_path))?.id !== pkg.id) fail('Crate implementation belongs to another package');
    const caller = owner(source(crate.caller_path));
    if (!caller || !reachable.has(caller.id) ||
        (caller.id === pkg.id ? pkg.id !== service.id : !edges.get(caller.id)?.has(pkg.id))) {
      fail(`Caller has no production dependency edge to ${pkg.name}`);
    }
    required(crate.responsibility, 'crate responsibility');
    required(crate.public_api, 'crate public API');
    source(crate.test_path); source(crate.failure_test_path);
  }
  if (counted.size < 20) fail('Need at least 20 backend-connected workspace crates');
  const designs = new Map();
  for (const [kind, floor] of [['adrs', 25], ['ddds', 12]]) {
    const docs = list(report.design?.[kind], kind);
    const bodies = new Set(); const paths = new Set();
    for (const doc of docs) {
      const path = source(doc.path);
      if (!/\.md$/i.test(path) || paths.has(path) || designs.has(path)) fail('Design document paths must be unique Markdown files');
      const text = readFileSync(path, 'utf8');
      const body = text.split('\n').filter((line) => !/^\s*#/.test(line)).join(' ').replace(/\s+/g, ' ').trim();
      if (!body || bodies.has(body)) fail(`Empty or duplicate ${kind} document body`);
      bodies.add(body); paths.add(path);
      if (digest(Buffer.from(text)) !== doc.sha256) fail('Design document hash differs from tested source');
      required(doc.title, 'design title');
      for (const target of list(doc.crate_manifest_paths, 'design crate mappings')) {
        if (!counted.has(source(target))) fail('Design references an uncounted crate');
      }
      for (const key of ['implementation_paths', 'test_paths']) {
        for (const target of list(doc[key], `design ${key}`)) source(target);
      }
      designs.set(path, { kind, doc });
    }
    if (paths.size < floor) fail(`Need at least ${floor} substantive ${kind} documents`);
    source(report.design?.[`${kind}_index_path`]);
  }
  for (const crate of crates) {
    for (const [key, kind] of [['adr_paths', 'adrs'], ['ddd_paths', 'ddds']]) {
      for (const path of list(crate[key], `crate ${key}`)) {
        const entry = designs.get(source(path));
        if (entry?.kind !== kind || !entry.doc.crate_manifest_paths.includes(crate.manifest_path)) {
          fail('Crate/design mapping must be reciprocal');
        }
      }
    }
  }
  for (const key of ['schema_path', 'client_path', 'contract_test_path']) source(report.api?.[key]);
  if (!/\.tsx?$/.test(report.api.client_path)) fail('API client must be TypeScript');
  source(report.middleware?.registration_path);
  const order = list(report.middleware?.order, 'middleware registration order');
  if (new Set(order).size !== order.length) fail('Duplicate middleware order entries');
  const controls = list(report.middleware?.controls, 'middleware controls');
  const found = new Set();
  for (const control of controls) {
    if (found.has(control.name) || !order.includes(control.name)) fail('Invalid middleware control registration');
    found.add(control.name);
    source(control.implementation_path); source(control.test_path);
  }
  for (const name of ['authn', 'authz', 'policy', 'validation', 'errors', 'correlation', 'timeouts', 'observability']) {
    if (!found.has(name)) fail(`Missing middleware control: ${name}`);
  }
  return { crates: counted.size, adrs: report.design.adrs.length, ddds: report.design.ddds.length };
}
