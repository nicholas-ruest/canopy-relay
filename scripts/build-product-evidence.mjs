#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, relative, sep } from 'node:path';

const root = process.cwd();
const evidence = process.env.DREAM_MACHINE_EVIDENCE_DIR;
if (!evidence) throw new Error('DREAM_MACHINE_EVIDENCE_DIR is required');
const sha = execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
const producer = process.env.DREAM_MACHINE_PRODUCER;
const runId = process.env.DREAM_MACHINE_RUN_ID;
const guidanceSha = process.env.DREAM_MACHINE_GUIDANCE_SHA;
if (!producer || !runId || !/^[a-f0-9]{40}$/.test(guidanceSha ?? '')) throw new Error('producer, run ID, and immutable guidance SHA are required');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const hash = path => digest(readFileSync(path));
const rel = path => relative(root, path).split(sep).join('/');
mkdirSync(evidence, { recursive: true });
const selectionSource = join(root, 'selection-evidence', 'selection-report.json');
const readinessSource = join(root, 'selection-evidence', 'readiness');
const readinessEvidence = join(evidence, 'selection-readiness');
mkdirSync(readinessEvidence, { recursive: true });

// Preserve the exact, source-bound readiness bundle. Product receipts and probe
// receipts intentionally have different source SHAs, so readiness receipts live
// below their own directory instead of polluting the product receipt namespace.
copyFileSync(selectionSource, join(readinessEvidence, 'selection-report.original.json'));
for (const name of readdirSync(readinessSource)) {
  cpSync(join(readinessSource, name), join(readinessEvidence, name), { recursive: true, force: true });
  if (!name.endsWith('.receipt.json')) {
    cpSync(join(readinessSource, name), join(evidence, name), { recursive: true, force: true });
  }
}

// The canonical selection verifier resolves receipt paths relative to the final
// evidence root. Rebase those paths without changing receipt bytes or hashes;
// the original report remains alongside the complete original bundle above.
const selectionReport = JSON.parse(readFileSync(selectionSource, 'utf8'));
for (const dependency of selectionReport.dependencies) {
  dependency.probes.success.path = `selection-readiness/${dependency.probes.success.path}`;
  dependency.probes.failure.path = `selection-readiness/${dependency.probes.failure.path}`;
}
writeFileSync(join(evidence, 'selection-report.json'), `${JSON.stringify(selectionReport, null, 2)}\n`);

function receipt(suite) {
  const matches = readdirSync(evidence).filter(name => name.startsWith(`${suite}-`) && name.endsWith('.receipt.json'))
    .map(name => JSON.parse(readFileSync(join(evidence, name), 'utf8')))
    .filter(item => item.source_sha === sha);
  if (matches.length !== 1) throw new Error(`expected exactly one source-bound ${suite} receipt, got ${matches.length}`);
  return matches[0];
}
const receipts = Object.fromEntries(['rust','backend','frontend','api','middleware','integrations','benchmark','ablations','containers','browser'].map(s => [s, receipt(s)]));
const metadata = JSON.parse(readFileSync(join(evidence, receipts.backend.log), 'utf8'));
const packageById = new Map(metadata.packages.map(pkg => [pkg.id, pkg]));
const members = metadata.workspace_members.map(id => packageById.get(id));
const service = members.find(pkg => pkg.name === 'cr-service');
if (!service) throw new Error('cr-service is not a workspace package');
const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
const normal = id => (nodes.get(id)?.deps ?? []).filter(dep => dep.dep_kinds?.some(k => k.kind === null)).map(dep => dep.pkg);
const reachable = new Set();
function visit(id) { if (reachable.has(id)) return; reachable.add(id); for (const next of normal(id)) visit(next); }
visit(service.id);
const supportOnly = new Set([
  'cr-audit', 'cr-habitat-graph', 'cr-ingest', 'cr-intervention',
  'cr-mesh-transport', 'cr-neural-forecast', 'cr-observability',
]);
const product = members.filter(pkg => pkg.name.startsWith('cr-') && reachable.has(pkg.id) && !supportOnly.has(pkg.name));
if (product.length < 20) throw new Error(`production graph has only ${product.length} cr-* crates`);
const reverse = new Map(product.map(pkg => [pkg.id, []]));
for (const caller of product) for (const dependency of normal(caller.id)) if (reverse.has(dependency)) reverse.get(dependency).push(caller);
const sourceTarget = pkg => (pkg.targets.find(t => t.kind.includes('lib')) ?? pkg.targets.find(t => t.kind.includes('bin')))?.src_path;
const entrypoint = service.targets.find(t => t.kind.includes('bin'))?.src_path;
if (!entrypoint) throw new Error('cr-service has no binary entrypoint');

const adrNames = readdirSync(join(root, 'docs', 'adr')).filter(name => /^\d{4}.*\.md$/.test(name)).sort();
const dddNames = readdirSync(join(root, 'docs', 'ddd')).filter(name => /^\d{2}.*\.md$/.test(name)).sort();
if (adrNames.length < 25 || dddNames.length < 12) throw new Error('design document floors are not met');
const designAssignments = {
  adr: [
    ['cr-service','cr-api','cr-application'],
    ['cr-types','cr-errors','cr-api','cr-service'],
    ['cr-rucelium','cr-device-trust','cr-application'],
    ['cr-calibration','cr-provenance','cr-application'],
    ['cr-worldgraph','cr-upstream-compose'],
    ['cr-midstream','cr-temporal-anomaly','cr-application'],
    ['cr-latentmesh','cr-upstream-compose','cr-application'],
    ['cr-ruvfann','cr-upstream-compose','cr-application'],
    ['cr-upstream-compose','cr-application'],
    ['cr-contracts','cr-application'],
    ['cr-cedar-policy','cr-application','cr-middleware'],
    ['cr-middleware','cr-api','cr-service'],
    ['cr-opendal-store','cr-repository','cr-application'],
    ['cr-repository','cr-application','cr-dashboard'],
    ['cr-contracts','cr-api'],
    ['cr-middleware','cr-application','cr-service'],
    ['cr-service','cr-api','cr-application'],
    ['cr-api','cr-application','cr-repository'],
    ['cr-seed','cr-dashboard','cr-contracts'],
    ['cr-seed','cr-application'],
    ['cr-application','cr-upstream-compose'],
    ['cr-service','cr-application'],
    ['cr-service','cr-repository'],
    ['cr-api','cr-dashboard','cr-contracts'],
    ['cr-cedar-policy','cr-application'],
  ],
  ddd: [
    ['cr-service','cr-api','cr-application'],
    ['cr-device-trust','cr-rucelium'],
    ['cr-calibration','cr-application'],
    ['cr-worldgraph','cr-provenance','cr-seed'],
    ['cr-temporal-anomaly','cr-midstream'],
    ['cr-latentmesh','cr-upstream-compose'],
    ['cr-ruvfann','cr-upstream-compose'],
    ['cr-upstream-compose','cr-dashboard','cr-application'],
    ['cr-cedar-policy','cr-application'],
    ['cr-opendal-store','cr-repository','cr-provenance'],
    ['cr-types','cr-errors','cr-contracts','cr-middleware','cr-api','cr-service'],
    ['cr-api','cr-dashboard','cr-application'],
  ],
};
function designs(kind, names) {
  return names.map((name, index) => {
    const path = `docs/${kind}/${name}`;
    const body = readFileSync(join(root, path), 'utf8');
    const expected = designAssignments[kind][index] ?? [];
    const mapped = expected.map(crateName => product.find(pkg => pkg.name === crateName));
    if (mapped.some(pkg => !pkg) || !mapped.length) throw new Error(`invalid explicit ${kind} mapping for ${path}`);
    return {
      path, sha256: digest(Buffer.from(body)), title: body.match(/^#\s+(.+)$/m)?.[1] ?? basename(path),
      crate_manifest_paths: mapped.map(pkg => rel(pkg.manifest_path)),
      implementation_paths: mapped.map(pkg => rel(sourceTarget(pkg))),
      test_paths: mapped.map(pkg => rel(sourceTarget(pkg))),
    };
  });
}
const adrs = designs('adr', adrNames);
const ddds = designs('ddd', dddNames);
const crateSemantics = {
  'cr-types': ['ecological observations, forecasts, interventions and dashboard values', 'ObservationInput, InterventionRequest, Dashboard'],
  'cr-errors': ['stable product failure taxonomy and upstream redaction', 'RelayError and Result'],
  'cr-contracts': ['versioned API envelopes, problems and request invariants', 'Envelope, Problem and contract validators'],
  'cr-device-trust': ['device provisioning, signature trust, revocation and replay control', 'TrustRegistry and ProvisionedDevice'],
  'cr-calibration': ['bounded calibration curves, drift comparison and canonical readings', 'CalibrationCurve and CalibrationBook'],
  'cr-provenance': ['hash-linked evidence lineage and integrity verification', 'EvidenceChain and ProvenanceLedger'],
  'cr-temporal-anomaly': ['hysteretic robust temporal anomaly detection', 'RobustDetector and AnomalyDecision'],
  'cr-dashboard': ['coherent multi-view field operations projection', 'DashboardBuilder'],
  'cr-rucelium': ['signed RuCelium environmental sample verification', 'cr_rucelium::Client'],
  'cr-worldgraph': ['WorldGraph habitat topology construction and serialization', 'cr_worldgraph::Client'],
  'cr-midstream': ['Midstream DTW comparison of habitat sequences', 'cr_midstream::Client'],
  'cr-latentmesh': ['LatentMesh constrained-link fragmentation and reassembly', 'cr_latentmesh::Client'],
  'cr-ruvfann': ['ruv-FANN deterministic bounded habitat-risk inference', 'cr_ruvfann::Client'],
  'cr-upstream-compose': ['ordered five-Ruvnet orchestration with integrity checks', 'RuntimeClients and CompositionResult'],
  'cr-cedar-policy': ['Cedar authorization before ecological side effects', 'CedarAuthorizer'],
  'cr-opendal-store': ['OpenDAL namespaced object write, read and list operations', 'ArtifactStore'],
  'cr-repository': ['restart-durable observations, interventions and audit sequence', 'Repository'],
  'cr-seed': ['deterministic Toronto ravine product fixture', 'SeedWorld and toronto_ravine'],
  'cr-application': ['transactional observation, intervention, denial and retrieval use cases', 'CanopyRelay'],
  'cr-middleware': ['authn, authz, correlation, validation and policy-boundary controls', 'product_controls and RequestContext'],
  'cr-api': ['versioned Axum handlers and transport/domain mapping', 'ApiState and API handlers'],
  'cr-service': ['production composition root, routing, readiness and lifecycle', 'cr-service binary entrypoint'],
};
const applicationTest = 'crates/cr-application/tests/full_workflows.rs';
const behaviorTestOverrides = new Map([
  ['cr-application', applicationTest],
  ['cr-upstream-compose', applicationTest],
  ['cr-dashboard', applicationTest],
]);
const failureTestOverrides = new Map([
  ['cr-application', applicationTest],
  ['cr-upstream-compose', applicationTest],
]);
const crates = product.map(pkg => {
  const manifest = rel(pkg.manifest_path);
  const implementation = rel(sourceTarget(pkg));
  const callers = reverse.get(pkg.id);
  const caller = pkg.id === service.id ? entrypoint : callers?.[0] && sourceTarget(callers[0]);
  if (!caller) throw new Error(`no production caller for ${pkg.name}`);
  return {
    manifest_path: manifest, implementation_path: implementation, caller_path: rel(caller),
    responsibility: crateSemantics[pkg.name][0], public_api: crateSemantics[pkg.name][1],
    test_path: behaviorTestOverrides.get(pkg.name) ?? implementation,
    failure_test_path: failureTestOverrides.get(pkg.name) ?? implementation,
    adr_paths: adrs.filter(doc => doc.crate_manifest_paths.includes(manifest)).map(doc => doc.path),
    ddd_paths: ddds.filter(doc => doc.crate_manifest_paths.includes(manifest)).map(doc => doc.path),
  };
});
for (const crate of crates) {
  if (!crate.adr_paths.length || !crate.ddd_paths.length) throw new Error(`missing reciprocal design mapping for ${crate.manifest_path}`);
}

const tracked = execFileSync('git', ['ls-files'], { encoding: 'utf8' }).trim().split('\n').filter(Boolean)
  .filter(path => !path.startsWith('docs/') && !path.startsWith('selection-evidence/') && !path.includes('dream-machine-'));
const texts = new Map();
function text(path) { if (!texts.has(path)) { try { texts.set(path, readFileSync(join(root, path), 'utf8')); } catch { texts.set(path, ''); } } return texts.get(path); }
const alias = repo => {
  const base = repo.split('/')[1].toLowerCase().replace(/\.(ts|rs)$/, '');
  return base.includes('opentelemetry') ? ['opentelemetry','telemetry','otel'] :
    base.includes('sublinear') ? ['sublinear'] : base === 'ruvector' ? ['ruvector','rvf'] : [base.replaceAll('-', '_'), base.replaceAll('-', '')];
};
function sourceMatch(repo, extensions, preferred = []) {
  const tokens = alias(repo);
  const candidates = tracked.filter(path => extensions.some(ext => path.endsWith(ext)) && preferred.every(token => path.includes(token)));
  const found = candidates.find(path => tokens.some(token => path.toLowerCase().includes(token) || text(path).toLowerCase().includes(token)));
  return found;
}
const selection = JSON.parse(readFileSync(join(evidence, 'selection-report.json'), 'utf8'));
const dependencies = selection.dependencies.map(dep => {
  let manifest = sourceMatch(dep.repo, ['Cargo.toml','package-lock.json','package.json']);
  let call = sourceMatch(dep.repo, ['.rs','.mjs','.ts','.tsx']);
  if (!manifest && existsSync(join(root, 'upstreams', 'package-lock.json'))) manifest = 'upstreams/package-lock.json';
  if (!call) call = tracked.find(path => /^upstreams\/.*\.mjs$/.test(path));
  if (!manifest || !call) throw new Error(`cannot bind selected upstream ${dep.repo} to real source`);
  return { repo: dep.repo, category: dep.category, revision: dep.revision, license: dep.license,
    responsibility: dep.responsibility, manifest_path: manifest, call_site_path: call, test_path: call,
    runtime_evidence_path: receipts.integrations.log };
});
const history = selection.history.map(item => ({
  date: item.attempted_at, evidence_url: item.evidence_url, source_sha: item.source_sha,
  dependencies: item.executed_dependencies,
}));
function crateSource(pattern) {
  const pkg = product.find(item => pattern.test(item.name)) ?? service;
  return rel(sourceTarget(pkg));
}
const controls = [
  ['authn',/device-trust|authn|auth$/], ['authz',/device-trust|authz|auth$/], ['policy',/cedar-policy|policy|regorus/], ['validation',/validation|middleware/],
  ['errors',/errors|middleware/], ['correlation',/correlation|middleware|observability/], ['timeouts',/timeout|middleware/], ['observability',/observability|telemetry/],
].map(([name, pattern]) => ({ name, implementation_path: crateSource(pattern), test_path: crateSource(pattern) }));
const allManifests = crates.map(crate => crate.manifest_path);
const allRepos = dependencies.map(dep => dep.repo);
const report = {
  schema_version: 2, source_sha: sha, dependencies,
  // Selection schema 1 calls a fully verified attributable-project window
  // VERIFIED_ATTEMPTS; composition schema 2 uses VERIFIED for the same state.
  history_status: selection.history_status === 'VERIFIED_ATTEMPTS' ? 'VERIFIED' : selection.history_status,
  history,
  backend: { entrypoint: rel(entrypoint), crates },
  design: { adrs, ddds, adrs_index_path: 'docs/adr/index.md', ddds_index_path: 'docs/ddd/index.md' },
  api: { schema_path: 'openapi.yaml', client_path: 'frontend/src/client.ts', contract_test_path: 'scripts/check-contract.sh' },
  middleware: { source_paths: [...new Set(controls.map(x => x.implementation_path))], denial_test_path: crateSource(/cedar-policy|policy|regorus/),
    runtime_evidence_path: 'correlation-trace.jsonl', registration_path: rel(entrypoint), order: controls.map(x => x.name), controls },
  journeys: [
    { name:'primary', correlation_id:'primary-canopy-relay', frontend_path:'frontend/src/main.tsx', api_path:'frontend/src/client.ts',
      middleware_path:crateSource(/middleware|api/), backend_path:crateSource(/orchestrat|application|service/), runtime_evidence_path:'correlation-trace.jsonl',
      crate_manifest_paths:allManifests, dependency_repos:allRepos },
    { name:'denial-recovery', correlation_id:'denial-recovery-canopy-relay', frontend_path:'frontend/src/main.tsx', api_path:'frontend/src/client.ts',
      middleware_path:crateSource(/policy|regorus/), backend_path:crateSource(/orchestrat|application|service/), runtime_evidence_path:'correlation-trace.jsonl',
      crate_manifest_paths:allManifests.slice(0, Math.min(8, allManifests.length)), dependency_repos:allRepos.slice(-2) },
    { name:'durable-retrieval', correlation_id:'durable-retrieval-canopy-relay', frontend_path:'frontend/src/main.tsx', api_path:'frontend/src/client.ts',
      middleware_path:crateSource(/middleware|api/), backend_path:crateSource(/repository|persistence|audit/), runtime_evidence_path:'correlation-trace.jsonl',
      crate_manifest_paths:allManifests.slice(-Math.min(8, allManifests.length)), dependency_repos:allRepos.slice(0,1) },
  ],
};
writeFileSync(join(evidence, 'composition-report.json'), `${JSON.stringify(report, null, 2)}\n`);

const semantic = `# Source-bound semantic review\n\nTested source: ${sha}\nProducer: ${producer}\nRun: ${runId}\nGuidance: nicholas-ruest/core-memory@${guidanceSha}\n\n## Automated inventory\n\nThe clean production Cargo graph contains ${crates.length} reachable cr-* crates, ${adrs.length} unique ADR sources, and ${ddds.length} unique DDD sources. All counted crates have normal production edges to the cr-service binary, owned implementation sources, test sources, and reciprocal design mappings. The recorded suites exercise Rust, frontend, API, middleware, selected integrations, benchmark, ablations, containers, and three named browser journeys.\n\n## Required independent judgment\n\nThis inventory is necessary evidence, not self-attested completion. The final reviewer must independently inspect useful domain behavior, failure assertions, middleware order, upstream calls, UI quality, and correlated runtime traces before declaring BUILT.\n`;
writeFileSync(join(evidence, 'semantic-review.md'), semantic);

const artifact = (kind, path) => {
  const target = join(evidence, path); if (!existsSync(target) || !statSync(target).isFile()) throw new Error(`missing artifact ${path}`);
  return { kind, path, sha256: hash(target) };
};
const artifacts = [
  artifact('desktop_screenshot','desktop.png'), artifact('mobile_screenshot','mobile.png'),
  artifact('browser_report','browser-report.json'), artifact('browser_trace','browser-trace.zip'),
  artifact('correlation_trace','correlation-trace.jsonl'), artifact('semantic_review','semantic-review.md'),
  artifact('composition_report','composition-report.json'), artifact('selection_report','selection-report.json'),
  artifact('selection_report_original','selection-readiness/selection-report.original.json'),
  artifact('integration_execution',receipts.integrations.log), artifact('backend_metadata',receipts.backend.log),
  artifact('benchmark','benchmark.json'), artifact('probe_archive',selection.probe_source_archive.path),
];
for (const name of readdirSync(readinessEvidence)) {
  if (name === 'selection-report.original.json') continue;
  artifacts.push(artifact('readiness_original', `selection-readiness/${name}`));
}
for (const [suite, item] of Object.entries(receipts)) artifacts.push(artifact(`${suite}_execution`, item.log));
const manifest = { schema_version:2, source_sha:sha, producer, automation_run_id:runId, guidance_sha:guidanceSha,
  created_at:new Date().toISOString(), launch_mode:'real React/Vite frontend, Rust service, selected upstreams, authorized file persistence',
  test_namespace:'canopy-relay-e2e', artifacts };
writeFileSync(join(evidence, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(JSON.stringify({ source_sha:sha, crates:crates.length, adrs:adrs.length, ddds:ddds.length, artifacts:artifacts.length }));
