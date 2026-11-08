#!/usr/bin/env node
// Necessary readiness/rotation checks, never a substitute for source review.
import { createHash } from 'node:crypto';
import { readFileSync, realpathSync, statSync } from 'node:fs';
import { isAbsolute, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const shaPattern = /^[a-f0-9]{40}$/;
const fail = (message) => { throw new Error(`SELECTION_BLOCKED: ${message}`); };
const text = (value, label) => {
  if (typeof value !== 'string' || !value.trim()) fail(`Missing ${label}`);
  return value;
};
function items(value, label, minimum = 1) {
  if (!Array.isArray(value) || value.length < minimum) fail(`Missing ${label}`);
  return value;
}
function canonical(item) {
  const repo = text(item.repo, 'canonical upstream').toLowerCase();
  if (!/^[a-z0-9_.-]+\/[a-z0-9_.-]+$/.test(repo) || repo.endsWith('.git')) fail('Invalid canonical upstream');
  if (!['ruvnet', 'enterprise'].includes(item.category) ||
      (item.category === 'ruvnet') !== repo.startsWith('ruvnet/')) fail(`Invalid category: ${repo}`);
  return repo;
}
function upstreams(value, label) {
  const map = new Map();
  for (const item of items(value, label)) {
    const repo = canonical(item);
    if (map.has(repo)) fail(`Duplicate upstream: ${repo}`);
    map.set(repo, item);
  }
  return map;
}
function artifact(evidence, item) {
  const path = text(item?.path, 'evidence path');
  if (isAbsolute(path)) fail('Absolute evidence path');
  const target = realpathSync(resolve(evidence, path));
  const rel = relative(realpathSync(evidence), target);
  if (rel === '..' || rel.startsWith(`..${sep}`) || isAbsolute(rel) || !statSync(target).isFile()) {
    fail('Evidence path escapes directory');
  }
  const bytes = readFileSync(target);
  if (!bytes.length || !/^[a-f0-9]{64}$/.test(item.sha256 ?? '') || hash(bytes) !== item.sha256) {
    fail(`Missing or altered evidence: ${path}`);
  }
  return bytes;
}
const signature = (map) => [...map.keys()].sort().join(',');

export function verifySelection(report, evidence, now = Date.now()) {
  if (report.schema_version !== 1) fail('Unsupported selection schema');
  text(report.attempt_id, 'attempt ID');
  if (!/^\d{4}-\d{2}-\d{2}$/.test(report.date ?? '') ||
      !shaPattern.test(report.probe_source_sha ?? '')) fail('Invalid selection date or probe source');
  const selectedAt = Date.parse(report.selected_at);
  if (!Number.isFinite(selectedAt) || selectedAt > now || now - selectedAt > 72 * 3600000) {
    fail('Selection must be current, at most 72 hours old');
  }
  const torontoDate = new Intl.DateTimeFormat('en-CA', { timeZone: 'America/Toronto',
    year: 'numeric', month: '2-digit', day: '2-digit' }).format(selectedAt);
  if (report.date !== torontoDate) fail('Selection date must be the Toronto calendar date');
  for (const field of ['users', 'capability']) text(report.product?.[field], `product ${field}`);
  const contexts = items(report.product?.bounded_contexts, 'three bounded contexts', 3);
  if (new Set(contexts.map((x) => text(x, 'context'))).size !== contexts.length) fail('Duplicate context');
  for (const field of ['frontend', 'backend', 'middleware', 'persistence']) {
    text(report.product?.topology?.[field], `topology ${field}`);
  }
  const workflows = items(report.product?.workflows, 'three product workflows', 3);
  const names = new Set(workflows.map((flow) => {
    text(flow.user_outcome, 'workflow user outcome');
    return flow.name;
  }));
  if (!['primary', 'denial-recovery', 'durable-retrieval'].every((name) => names.has(name))) {
    fail('Missing full-stack workflow plan');
  }
  const shortlist = upstreams(report.shortlist, 'researched shortlist');
  for (const [category, minimum] of [['ruvnet', 10], ['enterprise', 5]]) {
    if ([...shortlist.values()].filter((x) => x.category === category).length < minimum) {
      fail(`Research at least ${minimum} ${category} candidates`);
    }
  }
  for (const [repo, item] of shortlist) {
    if (!shaPattern.test(item.revision ?? '') ||
        !['selected', 'rejected'].includes(item.decision)) fail(`Unpinned/unassessed candidate: ${repo}`);
    for (const key of ['license', 'capability', 'rationale']) text(item[key], `${repo} ${key}`);
    if (item.source_url !== `https://github.com/${item.repo}/tree/${item.revision}`) {
      fail(`Candidate source must identify the inspected revision: ${repo}`);
    }
  }
  const selected = upstreams(report.dependencies, 'selected dependencies');
  for (const [category, minimum] of [['ruvnet', 5], ['enterprise', 2]]) {
    if ([...selected.values()].filter((x) => x.category === category).length < minimum) {
      fail(`Need ${minimum} executed ${category} readiness probes`);
    }
  }
  const picked = [...shortlist].filter(([, x]) => x.decision === 'selected').map(([repo]) => repo).sort();
  if (picked.join(',') !== signature(selected)) fail('Shortlist selection differs from dependencies');
  // Bind the actual probe code archive plus recorder receipts/raw logs, not just PASS booleans.
  artifact(evidence, report.probe_source_archive);
  const usedReceipts = new Set();
  for (const [repo, dependency] of selected) {
    const candidate = shortlist.get(repo);
    if (!candidate || candidate.decision !== 'selected' || candidate.revision !== dependency.revision ||
        candidate.license !== dependency.license) fail(`Readiness provenance differs: ${repo}`);
    for (const key of ['responsibility', 'binding', 'test_path']) text(dependency[key], `${repo} ${key}`);
    for (const mode of ['success', 'failure']) {
      const ref = dependency.probes?.[mode];
      const receipt = JSON.parse(artifact(evidence, ref).toString('utf8'));
      if (usedReceipts.has(ref.path)) fail('A readiness receipt cannot prove two boundaries');
      usedReceipts.add(ref.path);
      if (receipt.schema_version !== 2 || receipt.suite !== 'integrations' ||
          receipt.source_sha !== report.probe_source_sha || receipt.exit_code !== 0 ||
          receipt.signal || receipt.error || receipt.integrity_error ||
          !Array.isArray(receipt.argv) || receipt.argv.length === 0) fail(`Failed readiness: ${repo}/${mode}`);
      const start = Date.parse(receipt.started_at); const end = Date.parse(receipt.ended_at);
      if (!Number.isFinite(start) || !Number.isFinite(end) || end < start || end > selectedAt ||
          selectedAt - start > 72 * 3600000) fail(`Stale readiness: ${repo}/${mode}`);
      artifact(evidence, { path: receipt.log, sha256: receipt.log_sha256 });
    }
  }
  if (report.history_status !== 'VERIFIED_ATTEMPTS') fail('SELECTION_HISTORY_UNVERIFIED');
  const history = items(report.history, 'attributable attempted-build history');
  if (history.length > 10) fail('Use the last ten attempted builds');
  const seen = new Set([report.attempt_id]);
  let previousTime = selectedAt;
  const previous = history.map((build) => {
    const time = Date.parse(build.attempted_at);
    if (!Number.isFinite(time) || time > previousTime || seen.has(build.attempt_id) ||
        !shaPattern.test(build.source_sha ?? '')) fail('Invalid, unordered or duplicate attempt history');
    previousTime = time; seen.add(text(build.attempt_id, 'historical attempt ID'));
    if (!['BUILT', 'PARTIAL_FAILURE', 'NONCOMPLIANT_BUILD', 'INTEGRATION_BLOCKED',
      'PRECHECK_BLOCKED', 'BLOCKED_GIST_PUBLISH'].includes(build.status)) fail('Missing historical outcome');
    if (!/^https:\/\/github\.com\/[\w.-]+\/[\w.-]+\//.test(build.evidence_url ?? '') ||
        !build.evidence_url.includes(build.source_sha)) fail('History needs immutable source evidence');
    // Include selected AND executed ingredients; failed builds cannot reset the rotation window.
    const planned = upstreams(build.selected_dependencies, 'historical selection');
    const executed = Array.isArray(build.executed_dependencies) ? build.executed_dependencies : null;
    if (!executed) fail('Missing historical executed dependency evidence');
    for (const item of executed) {
      const repo = canonical(item);
      if (!planned.has(repo)) fail('Executed dependency absent from historical selection');
    }
    return planned;
  });
  const lastThree = new Set(previous.slice(0, 3).flatMap((map) => [...map.keys()]));
  for (const [category, minimum] of [['ruvnet', 2], ['enterprise', 1]]) {
    const fresh = [...selected].filter(([repo, item]) => item.category === category && !lastThree.has(repo));
    if (fresh.length < minimum) fail(`Rotate ${minimum} ${category} projects outside the last three attempts`);
  }
  if (previous.some((prior) => signature(prior) === signature(selected))) fail('Repeated full ingredient set');
  return { status: 'SELECTION_CHECKS_PASS', attempt_id: report.attempt_id,
    dependency_repos: [...selected.keys()].sort(),
    dependency_revisions: [...selected].map(([repo, item]) => `${repo}@${item.revision}`).sort(),
    limitation: 'Inspect probe code, real upstream calls, canonical aliases, history completeness and product substance independently. Not BUILT.' };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [path, evidence] = process.argv.slice(2);
    if (!path || !evidence) fail('Usage: selection-gate SELECTION_JSON EVIDENCE_DIRECTORY');
    console.log(JSON.stringify(verifySelection(JSON.parse(readFileSync(path, 'utf8')), evidence), null, 2));
  } catch (error) {
    console.error(JSON.stringify({ status: 'SELECTION_BLOCKED', error: error.message }));
    process.exitCode = 1;
  }
}
