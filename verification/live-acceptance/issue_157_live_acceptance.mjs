// Agent MaxHeadRoom: operator-owned Issue #157 actual binary/stdio acceptance.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { McpClient } from './csharp-compilation-identity/client.mjs';
import { scenarios, write, target, facts } from './issue-157/scenarios.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const output = path.join(repo, 'target/live-acceptance/issue-157', `${new Date().toISOString().replaceAll(/[:.]/g, '-')}-${process.pid}`);
const hash = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const report = { agent: 'Agent MaxHeadRoom', category: 'operator/live MCP; not Rust/CI or Claude field acceptance', status: 'INCOMPLETE', sessions: [], checks: [], errors: [], observations: [], mutations: [] };
fs.mkdirSync(output, { recursive: true });
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', 'utf8');
const controls = new Map();
let active;
let interrupted = false;
function inputs(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isSymbolicLink()) throw new Error(`input symlink: ${entry.name}`);
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? inputs(file) : [file];
  });
}
async function session(binary, scenario) {
  const directory = path.join(output, scenario.label);
  const root = path.join(directory, 'workspace');
  const config = { cbm: { enabled: false }, proxy: { auto_start: false }, default_fidelity: 'high', persistence: { enabled: false, auto_save: false } };
  write(root, { ...scenario.files, '.clean-ctx.json': JSON.stringify(config, null, 2) });
  const ownerHash = hash(fs.readFileSync(path.join(root, 'app.module.ts')));
  const record = { label: scenario.label, root, config, fixture: scenario.files, status: 'INCOMPLETE' };
  report.sessions.push(record); save();
  let client;
  let deadline;
  const started = performance.now();
  const check = (name, pass, detail) => {
    report.checks.push({ label: `${scenario.label}/${name}`, pass: !!pass, ...(pass ? {} : { detail }) }); save();
    if (!pass) throw new Error(`assertion failed: ${scenario.label}/${name}`);
  };
  try {
    client = new McpClient(binary, root, directory); active = client;
    deadline = setTimeout(() => { client.fail(new Error('20-minute session deadline')); client.child.kill(); }, 1200000);
    await client.initialize();
    const list = await client.request('tools/list', {}, 'tools/list');
    check('tools/list', !list.error && Array.isArray(list.result?.tools), list);
    const tool = list.result.tools.find((item) => item.name === 'workspace_query');
    check('workspace-query-schema', !!tool?.outputSchema, tool);
    fs.writeFileSync(path.join(directory, 'workspace-query-schema.json'), JSON.stringify(tool.outputSchema, null, 2), 'utf8');
    let sequence = 0;
    const query = async (args) => {
      const label = `query-${++sequence}`;
      const before = performance.now();
      const response = await client.call('workspace_query', { workspaceRoot: root, ...args }, label);
      const result = response.structuredContent;
      check(`${label}/structured`, !!result && typeof result === 'object', response);
      report.observations.push({ session: scenario.label, label, args, elapsedMs: performance.now() - before, coverage: result.coverage, discovery: result.discovery });
      check(`${label}/target-unchanged`, hash(fs.readFileSync(path.join(root, 'app.module.ts'))) === ownerHash);
      return result;
    };
    const assertFacts = (result, expected) => facts(result, expected, scenario.relation ?? 'DeclaresInModule', check, root, `facts-${sequence}`);
    const oracle = { check, query, facts: assertFacts,
      file: (file) => query({ type: 'entities_in_file', file_path: file, fidelity: 'high' }),
      edges: async (expected) => assertFacts(await query(target()), expected),
      converge: (family, order, signature) => {
        if (order === 'control') controls.set(family, signature);
        else check('nonempty-control-convergence', controls.has(family) && controls.get(family) === signature, { signature, control: controls.get(family) });
      },
      mutate: (name, replacement) => {
        const file = path.resolve(root, name);
        if (!file.startsWith(root + path.sep) || name === 'app.module.ts') throw new Error('invalid fixture mutation');
        report.mutations.push({ session: scenario.label, name, replacement, before: fs.existsSync(file) ? hash(fs.readFileSync(file)) : null }); save();
        if (replacement === null) fs.unlinkSync(file); else write(root, { [name]: replacement });
      },
    };
    await scenario.run(oracle);
    const health = await client.request('tools/list', {}, 'final-health');
    check('final-health', !health.error && Array.isArray(health.result?.tools));
    record.status = 'FINISHED';
  } catch (error) { record.status = 'FAILED'; report.errors.push({ label: scenario.label, message: error.stack }); }
  finally {
    clearTimeout(deadline);
    if (client) try { await client.close(); } catch (error) { record.status = 'FAILED'; report.errors.push({ label: scenario.label, message: `shutdown: ${error.stack}` }); }
    active = undefined; record.elapsedMs = performance.now() - started; save();
  }
}
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  interrupted = true; report.errors.push({ message: `interrupted: ${signal}` }); active?.fail(new Error(signal)); active?.child.kill(); save();
});
try {
  if (process.argv.length !== 3) throw new Error('supply exactly one explicit rebuilt binary path');
  const binary = path.resolve(process.argv[2]);
  const stat = fs.statSync(binary);
  if (!stat.isFile()) throw new Error('binary must be a regular file');
  const build = [...inputs(path.join(repo, 'src')).filter((file) => file.endsWith('.rs')),
    ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((file) => path.join(repo, file)).filter(fs.existsSync)];
  const tracked = [...build, ...inputs(path.join(repo, 'verification/live-acceptance/issue-157')),
    fileURLToPath(import.meta.url), path.join(repo, 'verification/live-acceptance/csharp-compilation-identity/client.mjs')];
  report.binary = { path: binary, sha256: hash(fs.readFileSync(binary)), mtime: stat.mtime.toISOString() };
  report.inputs = tracked.map((file) => ({ file: path.relative(repo, file), sha256: hash(fs.readFileSync(file)) }));
  if (build.some((file) => fs.statSync(file).mtimeMs > stat.mtimeMs)) throw new Error('binary predates Rust/build inputs; rebuild');
  const cases = scenarios(); report.expectedSessions = cases.length; save();
  if (new Set(cases.map((item) => item.label)).size !== cases.length) throw new Error('duplicate session labels');
  for (const scenario of cases) { if (interrupted) throw new Error('interrupted'); await session(binary, scenario); }
  if (hash(fs.readFileSync(binary)) !== report.binary.sha256) throw new Error('binary changed during run');
  for (const item of report.inputs) if (hash(fs.readFileSync(path.join(repo, item.file))) !== item.sha256) throw new Error(`input changed: ${item.file}`);
  report.completed = true;
} catch (error) { report.errors.push({ message: error.stack }); }
finally {
  report.status = report.completed && !interrupted && !report.errors.length
    && report.sessions.length === report.expectedSessions && report.sessions.every((item) => item.status === 'FINISHED')
    && report.checks.length > 0 && report.checks.every((item) => item.pass) ? 'PASS' : 'FAIL';
  report.finished = new Date().toISOString(); save();
  console.log(`${report.status}: ${report.checks.filter((item) => item.pass).length}/${report.checks.length} checks; evidence: ${output}`);
  process.exitCode = report.status === 'PASS' ? 0 : 1;
}
