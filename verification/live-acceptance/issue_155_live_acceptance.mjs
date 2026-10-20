// Agent MaxHeadRoom: operator-owned #155 real binary/stdio acceptance.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { McpClient } from './csharp-compilation-identity/client.mjs';
import { Oracles, roles, fixture, write, target, attributeFixture, attributes } from './issue-155/scenarios.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const output = path.join(repo, 'target/live-acceptance/issue-155',
  `${new Date().toISOString().replaceAll(/[:.]/g, '-')}-${process.pid}`);
fs.mkdirSync(output, { recursive: true });
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const report = { agent: 'Agent MaxHeadRoom', category: 'operator/live MCP; not Rust/CI coverage',
  status: 'INCOMPLETE', started: new Date().toISOString(), checks: [], sessions: [], errors: [] };
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', 'utf8');
let active;
let interrupted = false;
let binary;

function inputs(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isSymbolicLink()) throw new Error(`input symlink: ${entry.name}`);
    const file = path.join(dir, entry.name);
    return entry.isDirectory() ? inputs(file) : [file];
  });
}
function prepare() {
  if (process.argv.length !== 3) throw new Error('supply exactly one explicit rebuilt binary path');
  binary = path.resolve(process.argv[2]);
  const stat = fs.statSync(binary);
  if (!stat.isFile()) throw new Error('binary must be a regular file');
  const build = [...inputs(path.join(repo, 'src')).filter((file) => file.endsWith('.rs')),
    ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((name) => path.join(repo, name)).filter(fs.existsSync)];
  report.binary = { path: binary, sha256: sha(fs.readFileSync(binary)), mtime: stat.mtime.toISOString() };
  report.inputs = [...new Set([...build, ...inputs(path.join(repo, 'verification/live-acceptance/issue-155')),
    fileURLToPath(import.meta.url), path.join(repo, 'verification/live-acceptance/csharp-compilation-identity/client.mjs')])]
    .map((file) => ({ file: path.relative(repo, file), sha256: sha(fs.readFileSync(file)) }));
  if (build.some((file) => fs.statSync(file).mtimeMs > stat.mtimeMs)) throw new Error('binary predates source/build inputs; rebuild');
}
function workspace(label, files, durable = false) {
  const root = path.join(output, label, 'workspace');
  fs.mkdirSync(root, { recursive: true });
  write(root, files);
  const config = { cbm: { enabled: false }, proxy: { auto_start: false }, default_fidelity: 'high',
    persistence: { enabled: durable, auto_save: durable, db_path: path.join(root, 'evidence.db') } };
  write(root, { '.clean-ctx.json': JSON.stringify(config, null, 2) + '\n' });
  report.sessions.push({ label, root, fixture: files, config, status: 'PREPARED' });
  return root;
}
async function session(label, root, run) {
  if (interrupted) throw new Error('interrupted');
  const dir = path.join(output, label);
  fs.mkdirSync(dir, { recursive: true });
  const record = { label, root, status: 'INCOMPLETE' };
  report.sessions.push(record);
  const client = new McpClient(binary, root, dir);
  active = client;
  const deadline = setTimeout(() => { client.fail(new Error('20-minute session deadline')); client.child.kill(); }, 1200000);
  const check = (name, pass, detail) => {
    report.checks.push({ label: `${label}/${name}`, pass: !!pass, ...(pass ? {} : { detail }) });
    save();
    if (!pass) throw new Error(`assertion failed: ${label}/${name}`);
  };
  try {
    await client.initialize();
    const list = await client.request('tools/list', {}, 'tools/list');
    if (list.error || !Array.isArray(list.result?.tools)) throw new Error('tools/list failed');
    const tools = list.result.tools;
    for (const name of ['workspace_query', 'provide_code_context']) check(`tool/${name}`, tools.some((tool) => tool.name === name));
    const coverage = tools.find((tool) => tool.name === 'workspace_query').outputSchema?.properties?.coverage;
    if (!coverage?.properties?.dependency_proof) throw new Error('registered dependency proof schema absent');
    await run(new Oracles(client, root, coverage, check));
    const health = await client.request('tools/list', {}, 'final-health');
    check('final-health', !health.error && Array.isArray(health.result?.tools));
    record.status = 'FINISHED';
  } finally {
    clearTimeout(deadline);
    try { await client.close(); } finally { active = undefined; save(); }
  }
}
async function main() {
  prepare();
  save();
  // Each of the eight first observations has its own empty process and index.
  for (const role of Object.keys(roles)) for (const surface of ['entities_in_file', 'forward_edges']) {
    const label = `${role}-${surface}`;
    await session(label, workspace(label, fixture(role)), (o) => o.convergence(role, surface));
  }
  for (const kind of ['method-shadow', 'class-shadow', 'ambiguity']) {
    await session(kind, workspace(kind, attributeFixture(kind)), (o) => attributes(o, kind));
  }
  for (const reverse of [false, true]) {
    const mixed = `mixed-${reverse ? 'reverse' : 'forward'}`;
    await session(mixed, workspace(mixed, fixture()), (o) => o.mixed(reverse));
    const context = `context-${reverse ? 'reverse' : 'forward'}`;
    await session(context, workspace(context, fixture()), (o) => o.provide('context-batch', true, reverse));
  }
  const files = fixture();
  const root = workspace('durable', files, true);
  // Context alone must earn its proof before any query observes the owner.
  await session('durable/cold', root, async (o) => {
    await o.provide('provide/cold');
    await o.provide('provide/warm');
    const result = await o.query(target('Hub', 'forward_edges'), 'after-provide');
    o.facts(result, 'forward_edges', 'Hub', 'after-provide');
  });
  if (!fs.existsSync(path.join(root, 'evidence.db')) || fs.statSync(path.join(root, 'evidence.db')).size === 0) {
    throw new Error('durable SQLite evidence file missing or empty');
  }
  await session('durable/reopen', root, (o) => o.provide('provide/reopened'));
  write(root, { 'Foundation.cs': 'public class Foundation {}' });
  report.mutations = [{ file: 'Foundation.cs', replacement: 'public class Foundation {}', phase: 'retract' },
    { file: 'Foundation.cs', replacement: files['Foundation.cs'], phase: 'restore' }];
  save();
  await session('durable/stale-on-disk', root, async (o) => {
    await o.provide('provide/stale-durable');
    const result = await o.query(target('Hub', 'entities_in_file'), 'retracted');
    o.facts(result, 'entities_in_file', 'Hub', 'retracted', false);
    // Restoration and retraction inside one process must work without refresh.
    write(root, { 'Foundation.cs': files['Foundation.cs'] });
    await o.provide('provide/restored');
    o.facts(await o.query(target('Hub', 'forward_edges'), 'restored'), 'forward_edges', 'Hub', 'restored');
    write(root, { 'Foundation.cs': 'public class Foundation {}' });
    await o.provide('provide/retracted-again');
    o.facts(await o.query(target('Hub', 'entities_in_file'), 'retracted-again'), 'entities_in_file', 'Hub', 'retracted-again', false);
    write(root, { 'Foundation.cs': files['Foundation.cs'] });
    await o.provide('provide/restored-again');
  });
  await session('durable/final-reopen', root, async (o) => {
    await o.provide('provide/final-reopen');
    await o.convergence('Hub', 'forward_edges');
  });
  if (sha(fs.readFileSync(binary)) !== report.binary.sha256) throw new Error('binary changed during run');
  for (const item of report.inputs) if (sha(fs.readFileSync(path.join(repo, item.file))) !== item.sha256) {
    throw new Error(`input changed during run: ${item.file}`);
  }
  report.completed = true;
}
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  interrupted = true;
  report.errors.push({ message: `interrupted: ${signal}` });
  active?.fail(new Error(signal));
  active?.child.kill();
  save();
});
try { await main(); } catch (error) { report.errors.push({ message: error.stack }); console.error(error.message); }
finally {
  report.finished = new Date().toISOString();
  report.status = report.completed && !interrupted && !report.errors.length
    && report.checks.length > 0 && report.checks.every((item) => item.pass) ? 'PASS' : 'FAIL';
  save();
  console.log(`${report.status}: ${report.checks.filter((item) => item.pass).length}/${report.checks.length} checks; evidence: ${output}`);
  process.exitCode = report.status === 'PASS' ? 0 : 1;
}
