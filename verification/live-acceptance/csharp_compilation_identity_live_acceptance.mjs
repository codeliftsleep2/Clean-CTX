// Agent MaxHeadRoom: operator-run #153/#154 binary/stdio acceptance.
// Usage: node verification/live-acceptance/csharp_compilation_identity_live_acceptance.mjs <binary>
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { McpClient } from './csharp-compilation-identity/client.mjs';
import { makeFixtures } from './csharp-compilation-identity/fixtures.mjs';
import {
  negative, observe, publishAll, query, refresh, snapshot,
} from './csharp-compilation-identity/oracles.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const output = path.join(repo, 'target', 'live-acceptance', 'csharp-compilation-identity',
  new Date().toISOString().replaceAll(/[:.]/g, '-') + `-${process.pid}`);
fs.mkdirSync(output, { recursive: true });
const report = {
  agent: 'Agent MaxHeadRoom', category: 'operator/live MCP verification; not Rust/CI coverage',
  started: new Date().toISOString(), status: 'INCOMPLETE', checks: [], sessions: [], errors: [],
};
let activeClient;
let interrupted = false;
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', 'utf8');
const check = (label, pass, detail = null) => {
  report.checks.push({ label, pass: Boolean(pass), ...(pass ? {} : { detail }) });
  if (!pass) console.error(`FAIL: ${label}`);
};

function sourceMarkers(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isSymbolicLink()) throw new Error(`refusing source symlink: ${entry.name}`);
    const file = path.join(dir, entry.name);
    return entry.isDirectory() ? sourceMarkers(file) : entry.name.endsWith('.rs') ? [file] : [];
  });
}

function verifyBinary(binary) {
  if (!fs.statSync(binary).isFile()) throw new Error('binary argument must name a regular file');
  const stat = fs.statSync(binary);
  const markers = [...sourceMarkers(path.join(repo, 'src')),
    ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((name) => path.join(repo, name)).filter(fs.existsSync)];
  const stale = markers.filter((file) => fs.statSync(file).mtimeMs > stat.mtimeMs);
  report.binary = { path: binary, sha256: sha(fs.readFileSync(binary)), mtime: stat.mtime.toISOString() };
  report.sources = markers.map((file) => ({
    file: path.relative(repo, file), sha256: sha(fs.readFileSync(file)),
  }));
  if (stale.length) throw new Error(`binary predates source/build inputs; rebuild first: ${stale.slice(0, 5).join(', ')}`);
}

function writeSources(root, files) {
  for (const [file, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, file), source, 'utf8');
  }
}

async function assertSourceSurvives(client, root, item, label) {
  const result = await query(client, root, {
    type: 'find_entities', domain: 'builtin', entity_type: 'Class', name: item.name,
  }, `${label}/source`);
  if (!Array.isArray(result.entities)) throw new Error(`${label}: missing source entities`);
  check(`${label}/source-preserved`, result.entities.some((entity) =>
    entity.domain === 'builtin' && entity.entity_type === 'Class' && entity.name === item.name), result);
}

async function lifecycle(client, root, fixtures, baseline) {
  console.log('  lifecycle: action edit, ancestry retraction, restoration');
  const item = fixtures.editCase;
  const original = fixtures.files[item.file];
  const edited = original.replace('ActionE()', 'ActionZ()');
  if (edited === original) throw new Error('action mutation did not change fixture');
  report.actionMutation = { file: item.file, source: edited, sha256: sha(edited) };
  save();
  fs.writeFileSync(path.join(root, item.file), edited, 'utf8');
  await refresh(client, root, 'action-edit/refresh');
  await publishAll(client, root, fixtures.files, false, 'action-edit');
  const changed = { ...item, targets: item.targets.map((name) => name === 'ActionE' ? 'ActionZ' : name) };
  await observe(client, root, changed, check, 'action-edit');
  fs.writeFileSync(path.join(root, item.file), original, 'utf8');
  await refresh(client, root, 'action-restore/refresh');
  await publishAll(client, root, fixtures.files, false, 'action-restore');
  check('action-restore/exact-facts', JSON.stringify(await observe(client, root, item, check,
    'action-restore')) === JSON.stringify(baseline[item.name]));

  // Mutate one cross-file partial base per framework together, then re-admit
  // every known file. This tests reconciliation, not dependency acquisition.
  for (const mutation of fixtures.lifecycle) {
    fs.writeFileSync(path.join(root, mutation.file), mutation.replacement, 'utf8');
  }
  await refresh(client, root, 'ancestry-retract/refresh');
  await publishAll(client, root, fixtures.files, true, 'ancestry-retract');
  for (const mutation of fixtures.lifecycle) {
    await observe(client, root, mutation.item, check, 'ancestry-retract', false);
    await assertSourceSurvives(client, root, mutation.item, 'ancestry-retract');
  }
  // Unrelated identities, including genuine generic classes and MSTest, survive.
  const affected = new Set(fixtures.lifecycle.map((entry) => entry.item.name));
  const unaffected = fixtures.cases.filter((entry) => !affected.has(entry.name));
  const remaining = await snapshot(client, root, unaffected, check, 'unaffected');
  for (const entry of unaffected) check(`unaffected/${entry.name}/stable`,
    JSON.stringify(remaining[entry.name]) === JSON.stringify(baseline[entry.name]));
  writeSources(root, fixtures.files);
  await refresh(client, root, 'ancestry-restore/refresh');
  await publishAll(client, root, fixtures.files, false, 'ancestry-restore');
  check('ancestry-restore/exact-snapshot', JSON.stringify(await snapshot(client, root,
    fixtures.cases, check, 'ancestry-restore')) === JSON.stringify(baseline));
}

async function session(binary, fixtures, reverse, withLifecycle) {
  const label = reverse ? 'reverse-publication' : 'forward-publication';
  const dir = path.join(output, label);
  const root = path.join(dir, 'workspace');
  fs.mkdirSync(root, { recursive: true });
  writeSources(root, fixtures.files);
  fs.writeFileSync(path.join(root, '.clean-ctx.json'), JSON.stringify({
    cbm: { enabled: false }, proxy: { auto_start: false }, default_fidelity: 'high',
  }, null, 2) + '\n', 'utf8');
  const record = { label, root, status: 'INCOMPLETE' };
  report.sessions.push(record);
  const client = new McpClient(binary, root, dir);
  activeClient = client;
  // Operator harness deadlines only; no production timeout policy is changed.
  const deadline = setTimeout(() => {
    client.fail(new Error('20-minute session deadline exceeded'));
    client.child.kill();
  }, 20 * 60 * 1000);
  try {
    await client.initialize();
    const tools = await client.request('tools/list', {}, 'tools/list');
    const names = tools.result?.tools?.map((tool) => tool.name);
    if (tools.error || !Array.isArray(names)) throw new Error('tools/list failed');
    for (const required of ['provide_code_context', 'workspace_query', 'refresh_workspace']) {
      if (!names.includes(required)) throw new Error(`required MCP tool absent: ${required}`);
    }
    console.log(`${label}: explicit file admission, cold then warm semantic queries`);
    await publishAll(client, root, fixtures.files, reverse, 'initial');
    const cold = await snapshot(client, root, fixtures.cases, check, 'cold');
    await negative(client, root, fixtures.negative, check, 'cold');
    const warm = await snapshot(client, root, fixtures.cases, check, 'warm');
    check(`${label}/cold-warm-equivalent`, JSON.stringify(cold) === JSON.stringify(warm));
    fs.writeFileSync(path.join(dir, 'cold-snapshot.json'), JSON.stringify(cold, null, 2) + '\n', 'utf8');
    if (withLifecycle) await lifecycle(client, root, fixtures, cold);
    const health = await client.request('tools/list', {}, 'final-health');
    if (health.error || !Array.isArray(health.result?.tools)) throw new Error('final health check failed');
    record.status = 'FINISHED';
    return cold;
  } finally {
    clearTimeout(deadline);
    try { await client.close(); }
    finally { activeClient = undefined; save(); }
  }
}

async function main() {
  if (process.argv.length !== 3) throw new Error('pass exactly one explicit binary path; see README');
  const binary = path.resolve(process.argv[2]);
  verifyBinary(binary);
  const fixtures = makeFixtures();
  report.manifest = {
    cases: fixtures.cases, lifecycle: fixtures.lifecycle, negative: fixtures.negative,
    files: Object.entries(fixtures.files).map(([file, source]) => ({ file, sha256: sha(source) })),
  };
  save();
  let first;
  let second;
  for (const reverse of [false, true]) {
    if (interrupted) break;
    try {
      const value = await session(binary, fixtures, reverse, !reverse);
      if (reverse) second = value; else first = value;
    } catch (error) {
      report.errors.push({ session: reverse ? 'reverse' : 'forward', message: error.stack });
      console.error(error.message);
      save();
    }
  }
  if (first && second) check('fresh-session/publication-order-equivalent',
    JSON.stringify(first) === JSON.stringify(second));
  else check('both-sessions-finished', false);
  check('binary-unchanged-during-run', sha(fs.readFileSync(binary)) === report.binary.sha256);
  for (const input of report.sources) check(`source-unchanged/${input.file}`,
    fs.existsSync(path.join(repo, input.file))
      && sha(fs.readFileSync(path.join(repo, input.file))) === input.sha256);
}

for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  interrupted = true;
  report.errors.push({ message: `operator interrupted: ${signal}` });
  activeClient?.fail(new Error(`operator interrupted: ${signal}`));
  activeClient?.child.kill();
  save();
});

try { await main(); }
catch (error) { report.errors.push({ message: error.stack }); console.error(error.message); }
finally {
  report.finished = new Date().toISOString();
  report.status = report.errors.length === 0 && report.checks.length > 0
    && report.checks.every((entry) => entry.pass) ? 'PASS' : 'FAIL';
  save();
  console.log(`${report.status}: ${report.checks.filter((entry) => entry.pass).length}/${report.checks.length} checks`);
  console.log(`evidence: ${output}`);
  process.exitCode = report.status === 'PASS' ? 0 : 1;
}
