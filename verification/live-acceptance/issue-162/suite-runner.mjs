// Agent MaxHeadRoom: operator-run binary/stdio evidence, separate from Rust tests.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { McpClient } from '../csharp-compilation-identity/client.mjs';
import { injectFixture, injectAssertions, constructorAssertions } from './inject-scenario.mjs';
import { stringFixture, stringAssertions } from './string-scenario.mjs';
import { memberFixture, memberAssertions } from './member-scenario.mjs';
import { functionalFixture, functionalAssertions, functionalSource as directFunctionalSource, positionOrdinary as directPositionOrdinary, positionView as directPositionView, compositionSource, compositionOrdinary, compositionView } from './functional-scenario.mjs';

import { runLifecycle } from './lifecycle.mjs';

export async function runSuite(suite) {
const functionalSource = suite.group === 'composition' ? compositionSource() : directFunctionalSource;
const positionOrdinary = suite.group === 'composition' ? compositionOrdinary : directPositionOrdinary;
const positionView = suite.group === 'composition' ? compositionView : directPositionView;
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const output = process.env.ISSUE162_SUITE_OUTPUT ?? path.join(repo, 'target/live-acceptance/issue-162', `${new Date().toISOString().replaceAll(/[:.]/g, '-')}-${suite.name}-${process.pid}`);
const root = path.join(output, 'workspace');
const checkpointFiles = suite.files;
const hash = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const report = { agent: 'Agent MaxHeadRoom', category: 'operator binary/stdio; not Rust/CI or Claude field acceptance', suite: suite.name, maxChecks: 500, status: 'INCOMPLETE', sessions: [], checks: [], errors: [], mutations: [] };
fs.mkdirSync(root, { recursive: true });
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', 'utf8');
const write = (name, text) => {
  const file = path.resolve(root, name);
  if (!file.startsWith(root + path.sep)) throw new Error('fixture path outside workspace');
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text, 'utf8');
};
const normalize = (file) => {
  const normalized = file.replace(/^\\\\\?\\/, '').replaceAll('\\', '/');
  return process.platform === 'win32' ? normalized.toLowerCase() : normalized;
};
const leaf = "import { Directive as Dir } from '@angular/core'; @Dir({selector:'[leaf]'}) export class Leaf {}";
const dashboard = "import { Component as View } from '@angular/core'; import { Leaf as Left } from './left'; import { Leaf as Right } from './right'; import { PublicFeature as Feature } from './barrel'; const ALL = [Left, Right, Feature]; @View({selector:'x-dashboard',imports:[...ALL]}) export class Dashboard {}";
const app = "import { NgModule as Mod } from '@angular/core'; import { Leaf as Left } from './left'; import { Leaf as Right } from './right'; import { PublicFeature as Feature } from './barrel'; const ALL = [Left, Right]; @Mod({declarations:[...ALL],exports:[...ALL],imports:[Feature]}) export class AppModule {}";
let active;
let interrupted = false;
function check(name, pass, detail) {
  if (report.checks.length >= 500) throw new Error('suite exceeds 500 checks; split this suite before acceptance');
  report.checks.push({ name, pass: !!pass, ...(pass ? {} : { detail }) }); save();
  if (!pass) throw new Error(`assertion failed: ${name}`);
}
function inputs(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`input symlink: ${file}`);
    return entry.isDirectory() ? inputs(file) : [file];
  });
}
function mutate(name, replacement) {
  const file = path.join(root, name);
  report.mutations.push({ name, before: fs.existsSync(file) ? hash(fs.readFileSync(file)) : null, replacement }); save();
  if (replacement === null) fs.unlinkSync(file); else write(name, replacement);
}
async function session(binary, name, action) {
  const directory = path.join(output, name);
  fs.mkdirSync(directory, { recursive: true });
  const record = { name, status: 'INCOMPLETE' };
  report.sessions.push(record); save();
  const client = new McpClient(binary, root, directory);
  active = client;
  try {
    await client.initialize();
    const listing = await client.request('tools/list', {}, 'schema-preflight');
    const tools = listing.result?.tools;
    check(`${name}/tools-list`, !listing.error && Array.isArray(tools), listing);
    for (const required of ['workspace_query', 'compress_code_context', 'save_context', 'restore_context']) {
      check(`${name}/tool-${required}`, tools.some((tool) => tool.name === required));
    }
    const schema = tools.find((tool) => tool.name === 'workspace_query')?.outputSchema;
    check(`${name}/workspace-query-schema`, !!schema);
    fs.writeFileSync(path.join(directory, 'workspace-query-schema.json'), JSON.stringify(schema, null, 2), 'utf8');
    let sequence = 0;
    const call = (tool, args) => client.call(tool, { workspaceRoot: root, ...args }, `${name}/${++sequence}/${tool}`);
    const moduleFacts = async (expected, standaloneExpected = expected, standaloneOwner = true) => {
      const response = await call('workspace_query', { type: 'forward_edges', domain: 'angular', entity_type: 'Module', name: 'AppModule' });
      const result = response.structuredContent;
      check(`${name}/${sequence}/structured-edges`, Array.isArray(result?.edges), response);
      for (const relation of ['DeclaresInModule', 'ExportsFromModule']) {
        const edges = result.edges.filter((edge) => edge.relation === relation);
        const actual = edges.map((edge) => [edge.object.name, edge.object.entity_type, normalize(edge.object.file)]).sort();
        const wanted = expected.map((file) => ['Leaf', 'Directive', normalize(path.join(root, file))]).sort();
        check(`${name}/${sequence}/${relation}`, JSON.stringify(actual) === JSON.stringify(wanted), { actual, wanted });
        check(`${name}/${sequence}/canonical-private-${relation}`, edges.every((edge) => !('canonical' in edge.subject) && !('canonical' in edge.object)));
        check(`${name}/${sequence}/owner-${relation}`, edges.every((edge) => edge.subject.domain === 'angular' && edge.object.domain === 'angular'
          && edge.subject.name === 'AppModule' && normalize(edge.subject.file) === normalize(path.join(root, 'app.ts'))));
      }
      const imports = result.edges.filter((edge) => edge.relation === 'ImportsModule');
      check(`${name}/${sequence}/aliased-module-import`, imports.length === 1 && imports[0].object.name === 'FeatureModule'
        && normalize(imports[0].object.file) === normalize(path.join(root, 'feature.ts')), imports);
      check(`${name}/${sequence}/initiating-source-unchanged`, hash(fs.readFileSync(path.join(root, 'app.ts'))) === report.ownerHash);
      const standaloneResponse = await call('workspace_query', { type: 'forward_edges', domain: 'angular', entity_type: 'Component', name: 'Dashboard' });
      const standaloneEdges = standaloneResponse.structuredContent?.edges;
      check(`${name}/${sequence}/standalone-structured`, Array.isArray(standaloneEdges), standaloneResponse);
      const selected = standaloneEdges.filter((edge) => edge.relation === 'ImportsModule');
      const actualStandalone = selected.map((edge) => [edge.object.name, edge.object.entity_type, normalize(edge.object.file)]).sort();
      const wantedStandalone = standaloneOwner ? [...standaloneExpected.map((file) => ['Leaf', 'Directive', normalize(path.join(root, file))]),
        ['FeatureModule', 'Module', normalize(path.join(root, 'feature.ts'))]].sort() : [];
      check(`${name}/${sequence}/standalone-exact-imports`, JSON.stringify(actualStandalone) === JSON.stringify(wantedStandalone), { actualStandalone, wantedStandalone });
      check(`${name}/${sequence}/standalone-owner`, selected.every((edge) => edge.subject.domain === 'angular' && edge.subject.entity_type === 'Component'
        && edge.subject.name === 'Dashboard' && normalize(edge.subject.file) === normalize(path.join(root, 'dashboard.ts'))));
      check(`${name}/${sequence}/standalone-canonical-private`, selected.every((edge) => !('canonical' in edge.subject) && !('canonical' in edge.object)));
      check(`${name}/${sequence}/standalone-source-unchanged`, hash(fs.readFileSync(path.join(root, 'dashboard.ts'))) === report.dashboardHash);
    };
    const facts = suite.family === 'modules' ? moduleFacts : async () => {};
    const injectFacts = suite.family === 'inject' ? injectAssertions({ call, check, root, normalize, hash, fs, name }) : async () => {};
    const constructorFacts = suite.family === 'constructor' ? constructorAssertions({ call, check, root, normalize, hash, fs, name }) : async () => {};
    const stringFacts = suite.family === 'strings' ? stringAssertions({ call, check, root, normalize, hash, fs, name }) : async () => {};
    const memberFacts = suite.family === 'members' ? memberAssertions({ call, check, root, normalize, name }) : async () => {};
    const functionalFacts = suite.family === 'functional' ? functionalAssertions({ call, check, root, normalize, name, group: suite.group }) : async () => {};
    await action({ call, facts, injectFacts, constructorFacts, stringFacts, memberFacts, functionalFacts, client });
    const health = await client.request('tools/list', {}, 'final-health');
    check(`${name}/final-health`, !health.error && Array.isArray(health.result?.tools));
    record.status = 'FINISHED';
  } catch (error) { record.status = 'FAILED'; throw error; }
  finally {
    try { await client.close(); }
    catch (error) { record.status = 'FAILED'; report.errors.push({ message: `shutdown: ${error.stack}` }); }
    active = undefined; save();
  }
}
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  interrupted = true; report.errors.push({ message: `interrupted: ${signal}` });
  active?.fail(new Error(signal)); active?.child.kill(); save();
});
try {
  if (process.argv.length !== 3) throw new Error('supply exactly one explicit rebuilt binary path');
  const binary = path.resolve(process.argv[2]);
  const stat = fs.statSync(binary);
  if (!stat.isFile()) throw new Error('binary must be a regular file');
  const build = [...inputs(path.join(repo, 'src')).filter((file) => file.endsWith('.rs')),
    ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((file) => path.join(repo, file)).filter(fs.existsSync)];
  const tracked = [...build, ...inputs(path.join(repo, 'verification/live-acceptance/issue-162')).filter(file => file.endsWith('.mjs')), path.resolve(process.argv[1]), path.join(repo, 'verification/live-acceptance/issue_162_live_acceptance.mjs'), path.join(repo, 'verification/live-acceptance/issue-162/inject-scenario.mjs'), path.join(repo, 'verification/live-acceptance/issue-162/string-scenario.mjs'), path.join(repo, 'verification/live-acceptance/csharp-compilation-identity/client.mjs')];
  tracked.push(path.join(repo, 'verification/live-acceptance/issue-162/member-scenario.mjs'));
  tracked.push(path.join(repo, 'verification/live-acceptance/issue-162/functional-scenario.mjs'));
  report.binary = { path: binary, sha256: hash(fs.readFileSync(binary)), mtime: stat.mtime.toISOString() };
  report.inputs = [...new Set(tracked)].map((file) => ({ file: path.relative(repo, file), sha256: hash(fs.readFileSync(file)) }));
  if (build.some((file) => fs.statSync(file).mtimeMs > stat.mtimeMs)) throw new Error('binary predates Rust/build inputs; rebuild');
  report.config = { cbm: { enabled: false }, proxy: { auto_start: false }, default_fidelity: 'high',
    persistence: { enabled: true, auto_save: false, db_path: path.join(root, 'authority.db') } };
  const fixture = {
    '.clean-ctx.json': JSON.stringify(report.config, null, 2),
    'tsconfig.json': JSON.stringify({ compilerOptions: { module: 'esnext', moduleResolution: 'bundler', moduleDetection: 'legacy', noLib: true, types: [] } }),
    'node_modules/@angular/core/package.json': JSON.stringify({ name: '@angular/core', version: '20.0.0', types: 'index.d.ts' }),
    'node_modules/@angular/core/index.d.ts': 'export declare function NgModule(metadata: unknown): any; export declare function Directive(metadata: unknown): any; export declare function Component(metadata: unknown): any; export declare function Injectable(metadata?: unknown): any; export declare function inject(token: unknown): unknown; export declare function Inject(token: unknown): any;',
    'left.ts': leaf, 'right.ts': leaf,
    'feature.ts': "import { NgModule as Mod } from '@angular/core'; @Mod({}) export class FeatureModule {}",
    'barrel.ts': "export { FeatureModule as PublicFeature } from './feature';",
    'app.ts': app, 'dashboard.ts': dashboard, ...injectFixture, ...stringFixture, ...memberFixture, ...functionalFixture,
    'functional.ts': functionalSource, 'functional-neighbor.ts': functionalSource,
    ...suite.fixture,
  };
  fixture['node_modules/@angular/core/index.d.ts'] += ' export declare class InjectionToken<T> { constructor(description: string); }';
  fixture['node_modules/@angular/core/index.d.ts'] += ' export declare function input<T>(value:T):unknown; export declare function output<T>():unknown;';
  fixture['node_modules/@angular/core/index.d.ts'] += ' export declare const model: {required:<T>()=>unknown;};';
  for (const [name, text] of Object.entries(fixture)) write(name, text);
  report.fixture = fixture; report.ownerHash = hash(Buffer.from(app)); report.dashboardHash = hash(Buffer.from(dashboard)); save();
  await (suite.lifecycle ?? runLifecycle)({session,binary,checkpointFiles,root,fixture,mutate,leaf,check,functionalSource,positionOrdinary,positionView}, suite);
  check('binary-unchanged', hash(fs.readFileSync(binary)) === report.binary.sha256);
  for (const item of report.inputs) {
    if (hash(fs.readFileSync(path.join(repo, item.file))) !== item.sha256) throw new Error(`input changed: ${item.file}`);
  }
  check("repository-inputs-unchanged", true);
  report.completed = true;
} catch (error) { report.errors.push({ message: error.stack }); }
finally {
  report.status = report.completed && !interrupted && !report.errors.length && report.sessions.length === 3
    && report.sessions.every((item) => item.status === 'FINISHED') && report.checks.every((item) => item.pass) ? 'PASS' : 'FAIL';
  report.finished = new Date().toISOString(); save();
  console.log(`${report.status}: ${report.checks.filter((item) => item.pass).length}/${report.checks.length} checks; evidence: ${output}`);
  process.exitCode = report.status === 'PASS' ? 0 : 1;
}

}
