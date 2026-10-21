// Agent MaxHeadRoom: operator-owned #156 EF model/configuration MCP acceptance.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { McpClient } from './csharp-compilation-identity/client.mjs';
import { Oracles, fixture, models, target, write } from './issue-156/scenarios.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const output = path.join(repo, 'target/live-acceptance/issue-156',
  `${new Date().toISOString().replaceAll(/[:.]/g, '-')}-${process.pid}`);
fs.mkdirSync(output, { recursive: true });
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const report = { agent: 'Agent MaxHeadRoom', category: 'operator/live MCP; not Rust/CI or Claude field acceptance',
  status: 'INCOMPLETE', started: new Date().toISOString(), sessions: [], checks: [], mutations: [], observations: [], errors: [] };
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', 'utf8');
let active;
let interrupted = false;
let binary;

function inputs(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isSymbolicLink()) throw new Error(`input symlink: ${entry.name}`);
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? inputs(file) : [file];
  });
}
function prepare() {
  if (process.argv.length !== 3) throw new Error('supply exactly one explicit rebuilt binary path');
  binary = path.resolve(process.argv[2]);
  const stat = fs.statSync(binary);
  if (!stat.isFile()) throw new Error('binary must be a regular file');
  const build = [...inputs(path.join(repo, 'src')).filter((file) => file.endsWith('.rs')),
    ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((file) => path.join(repo, file)).filter(fs.existsSync)];
  report.binary = { path: binary, sha256: sha(fs.readFileSync(binary)), mtime: stat.mtime.toISOString() };
  report.inputs = [...new Set([...build, ...inputs(path.join(repo, 'verification/live-acceptance/issue-156')),
    fileURLToPath(import.meta.url), path.join(repo, 'verification/live-acceptance/csharp-compilation-identity/client.mjs')])]
    .map((file) => ({ file: path.relative(repo, file), sha256: sha(fs.readFileSync(file)) }));
  if (build.some((file) => fs.statSync(file).mtimeMs > stat.mtimeMs)) throw new Error('binary predates Rust/build inputs; rebuild');
}

async function session(label, files, run, rootOrder) {
  if (interrupted) throw new Error('interrupted');
  const directory = path.join(output, label);
  const root = path.join(directory, 'workspace');
  fs.mkdirSync(root, { recursive: true });
  write(root, files);
  let additionalRoots = [];
  if (rootOrder !== undefined) {
    additionalRoots = ['foreign-a', 'foreign-b'].map((name, index) => {
      const foreign = path.join(directory, name);
      write(foreign, { 'Foreign.csproj': '<Project />',
        'Rows.cs': `namespace Foreign${index}; public class First {} public class Mapping {}` });
      return foreign;
    });
    if (rootOrder) additionalRoots.reverse();
  }
  const config = { cbm: { enabled: false }, proxy: { auto_start: false }, default_fidelity: 'high',
    additional_roots: additionalRoots, persistence: { enabled: false, auto_save: false } };
  write(root, { '.clean-ctx.json': JSON.stringify(config, null, 2) + '\n' });
  const started = performance.now();
  const record = { label, root, fixture: files, config, status: 'INCOMPLETE', started: new Date().toISOString() };
  report.sessions.push(record);
  save();
  let client;
  let deadline;
  const check = (name, pass, detail) => {
    report.checks.push({ label: `${label}/${name}`, pass: !!pass, ...(pass ? {} : { detail }) });
    save();
    if (!pass) throw new Error(`assertion failed: ${label}/${name}`);
  };
  try {
    client = new McpClient(binary, root, directory);
    active = client;
    deadline = setTimeout(() => { client.fail(new Error('20-minute session deadline')); client.child.kill(); }, 1200000);
    await client.initialize();
    const list = await client.request('tools/list', {}, 'tools/list');
    check('tools/list', !list.error && Array.isArray(list.result?.tools), list.error);
    const tool = list.result.tools.find((item) => item.name === 'workspace_query');
    check('workspace-query-tool', !!tool);
    const coverage = tool.outputSchema?.properties?.coverage;
    check('dependency-proof-schema', !!coverage?.properties?.dependency_proof);
    fs.writeFileSync(path.join(directory, 'workspace-query-schema.json'), JSON.stringify(tool.outputSchema, null, 2) + '\n', 'utf8');
    const mutate = (name, replacement) => {
      const file = path.resolve(root, name);
      if (!file.startsWith(root + path.sep)) throw new Error(`mutation outside fixture: ${name}`);
      report.mutations.push({ label, name, replacement, before: fs.existsSync(file) ? sha(fs.readFileSync(file)) : null });
      save();
      if (replacement === null) fs.unlinkSync(file);
      else write(root, { [name]: replacement });
    };
    const observe = (item) => { report.observations.push({ session: label, ...item }); save(); };
    const oracle = new Oracles(client, root, coverage, check, mutate, observe);
    const value = await run(oracle, additionalRoots);
    const health = await client.request('tools/list', {}, 'final-health');
    check('final-health', !health.error && Array.isArray(health.result?.tools));
    record.status = 'FINISHED';
    return value;
  } catch (error) {
    record.status = 'FAILED';
    report.errors.push({ label, message: error.stack });
    return undefined;
  } finally {
    clearTimeout(deadline);
    if (client) try { await client.close(); } catch (error) {
      record.status = 'FAILED'; report.errors.push({ label, message: `shutdown: ${error.stack}` });
    }
    active = undefined;
    record.finished = new Date().toISOString();
    record.elapsedMs = performance.now() - started;
    save();
  }
}

async function main() {
  prepare();
  save();
  for (const [kind, model] of Object.entries(models)) {
    // First establish a nonempty supported control. Every order uses a fresh
    // process/workspace with identical fixture source bytes, never a warmed index.
    const expected = await session(`${kind}/dependency-files`, fixture(model), async (o) => {
      await o.warm('dependency-files'); return o.edges('target');
    });
    for (const order of ['cold', 'dependency-names', 'unrelated', 'repeat']) {
      await session(`${kind}/${order}`, fixture(model), async (o) => {
        await o.warm(order);
        const actual = await o.edges('target');
        o.check('positive-control-established', expected !== undefined);
        o.check('order-convergence', actual === expected, { expected, actual });
      });
    }
    await session(`${kind}/lifecycle`, fixture(model), (o) => o.lifecycle(kind));
  }
  for (const reverse of [false, true]) {
    await session(`batch-${reverse}`, fixture(), (o) => o.batch(reverse));
    await session(`roots-${reverse}`, fixture(), async (o, roots) => {
      for (const foreign of roots) await o.query({ type: 'entities_in_file', file_path: path.join(foreign, 'Rows.cs') }, 'foreign');
      await o.edges('root-order');
    }, reverse);
  }
  await session('concurrent', fixture(), (o) => o.concurrent());
  await session('reverse-cold', fixture(), async (o) => {
    const result = await o.query({ type: 'reverse_edges', domain: 'dotnet', entity_type: 'Entity', name: 'First' }, 'reverse');
    o.facts(result, ['First'], 'reverse');
  });
  const aliased = fixture('builder.Entity<Row>(); builder.ApplyConfiguration(new Map());');
  aliased['Target.cs'] = `using Row = App.Second; using Map = App.Mapping;
 using Microsoft.EntityFrameworkCore; namespace App; public class Target : Bridge {
 protected override void OnModelCreating(ModelBuilder builder) { builder.Entity<Row>(); builder.ApplyConfiguration(new Map()); } }`;
  await session('aliases', aliased, (o) => o.edges('aliases', ['First', 'Second']));
  const duplicate = fixture(models.direct);
  duplicate['Duplicate.cs'] = 'namespace App; public class First {}';
  await session('ambiguous-entity', duplicate, (o) => o.edges('ambiguous', [], false));
  const missing = fixture(models.direct);
  delete missing['Rows.cs'];
  await session('missing-entity', missing, (o) => o.edges('missing', [], false));
  const marker = fixture('builder.ApplyConfigurationsFromAssembly(typeof(Marker).Assembly);');
  marker['Marker.cs'] = 'namespace App; public class Marker {}';
  await session('marker-project', marker, async (o) => {
    await o.edges('marker-initial');
    o.mutate('Marker.cs', 'namespace Elsewhere; public class Marker {}');
    await o.edges('marker-changed', [], false);
    o.mutate('Marker.cs', marker['Marker.cs']);
    await o.edges('marker-restored');
    o.mutate('Ambiguous.csproj', '<Project />');
    await o.edges('project-ambiguous', [], false);
    o.mutate('Ambiguous.csproj', null);
    await o.edges('project-restored');
  });
  if (sha(fs.readFileSync(binary)) !== report.binary.sha256) throw new Error('binary changed during run');
  for (const item of report.inputs) if (sha(fs.readFileSync(path.join(repo, item.file))) !== item.sha256) {
    throw new Error(`input changed during run: ${item.file}`);
  }
  report.completed = true;
}
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  interrupted = true; report.errors.push({ message: `interrupted: ${signal}` });
  active?.fail(new Error(signal)); active?.child.kill(); save();
});
try { await main(); } catch (error) { report.errors.push({ message: error.stack }); }
finally {
  report.finished = new Date().toISOString();
  report.status = report.completed && !interrupted && !report.errors.length && report.sessions.length === 28
    && report.sessions.every((item) => item.status === 'FINISHED')
    && report.checks.length > 0 && report.checks.every((item) => item.pass) ? 'PASS' : 'FAIL';
  save();
  console.log(`${report.status}: ${report.checks.filter((item) => item.pass).length}/${report.checks.length} checks; evidence: ${output}`);
  process.exitCode = report.status === 'PASS' ? 0 : 1;
}
