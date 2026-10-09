// Tracked live MCP acceptance for v0.9.2 structured C# declaration headers.
// Usage: node verification/live-acceptance/csharp_base_type_live_acceptance.mjs [binary]

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';

const repoRoot = process.cwd();
const failures = [];
const captures = {};

function resolveBinary() {
  const names = process.platform === 'win32' ? ['clean-ctx.exe', 'clean-ctx'] : ['clean-ctx'];
  const candidates = [];
  for (const profile of ['release', 'debug']) for (const name of names) {
    const candidate = path.resolve('target', profile, name);
    if (fs.existsSync(candidate)) candidates.push(candidate);
  }
  if (candidates.length === 0) return path.resolve('target', 'debug', names[0]);
  return candidates.sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs)[0];
}

const binary = process.argv[2] ? path.resolve(process.argv[2]) : resolveBinary();
const markers = [
  'src/queries.rs', 'src/ir/pipeline/core.rs', 'src/ir/pipeline/core/csharp.rs',
  'src/ir/layers/csharp.rs', 'src/ir/semantic_projection.rs',
  'src/layers/meta/builtin.rs', 'src/mcp/tool_handlers/query/coverage.rs',
  'src/mcp/tool_handlers/query/edges.rs',
].map((entry) => path.resolve(entry));

function binaryIsStale() {
  try {
    const time = fs.statSync(binary).mtimeMs;
    return markers.some((marker) => fs.existsSync(marker) && time < fs.statSync(marker).mtimeMs);
  } catch { return false; }
}

class McpClient {
  constructor(cwd) {
    this.child = spawn(binary, [], { cwd, stdio: ['pipe', 'pipe', 'pipe'] });
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = '';
    this.child.on('error', (error) => {
      for (const [id, resolve] of this.pending) {
        this.pending.delete(id);
        resolve({ error: { message: 'spawn failed: ' + error.message } });
      }
    });
    this.child.stderr.on('data', (chunk) => { this.stderr += chunk.toString(); });
    readline.createInterface({ input: this.child.stdout }).on('line', (line) => {
      let message;
      try { message = JSON.parse(line.trim()); } catch { return; }
      const resolve = this.pending.get(message.id);
      if (!resolve) return;
      this.pending.delete(message.id);
      resolve(message);
    });
  }
  request(method, params) {
    const id = this.nextId++;
    const answer = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('timeout waiting for ' + method)), 120000);
      this.pending.set(id, (message) => { clearTimeout(timer); resolve(message); });
    });
    this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n');
    return answer;
  }
  notify(method, params) {
    this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method, params }) + '\n');
  }
  async close() {
    this.child.stdin.end();
    if (this.child.exitCode !== null) return;
    await new Promise((resolve) => {
      const forceKill = setTimeout(() => this.child.kill(), 1000);
      const finish = () => {
        clearTimeout(forceKill);
        resolve();
      };
      this.child.once('exit', finish);
      this.child.once('error', finish);
    });
  }
}

function check(group, description, condition, detail = '') {
  console.log('    [' + (condition ? 'PASS' : 'FAIL') + '] ' + description +
    (detail ? ' — ' + detail : ''));
  if (!condition) failures.push(group + ': ' + description);
}
function structured(response, label) {
  if (response?.error) throw new Error(label + ': ' + JSON.stringify(response.error));
  const value = response?.result?.structuredContent;
  if (!value) throw new Error(label + ': missing structuredContent');
  captures[label] = value;
  return value;
}
function hasEdge(result, relation, subjectType, subjectName, objectType, objectName) {
  return (result.edges ?? []).some((edge) =>
    edge.relation === relation && edge.subject?.domain === 'builtin' &&
    edge.subject?.entity_type === subjectType && edge.subject?.name === subjectName &&
    edge.object?.domain === 'builtin' && edge.object?.entity_type === objectType &&
    edge.object?.name === objectName);
}
function inheritanceEdges(result) {
  return (result.edges ?? []).filter((edge) =>
    ['HasBaseType', 'Extends', 'Implements'].includes(edge.relation));
}
async function initialize(client) {
  await client.request('initialize', {
    protocolVersion: '2024-11-05', capabilities: {},
    clientInfo: { name: 'csharp-base-type-live-acceptance', version: '1.0.0' },
  });
  client.notify('notifications/initialized', {});
}
async function publish(client, root, file) {
  const response = await client.request('tools/call', {
    name: 'provide_code_context',
    arguments: { filePath: file, fidelity: 'high', workspaceRoot: root },
  });
  if (response?.error || !response?.result) {
    throw new Error('publish-' + file + ': ' + JSON.stringify(response));
  }
  captures['publish-' + file] = {
    ok: true,
    meta: response.result._meta ?? null,
  };
  return response.result;
}
async function query(client, root, type, entityType, name, label) {
  return structured(await client.request('tools/call', {
    name: 'workspace_query',
    arguments: { type, domain: 'builtin', entity_type: entityType, name, workspaceRoot: root },
  }), label);
}
async function entitiesInFile(client, root, file) {
  return structured(await client.request('tools/call', {
    name: 'workspace_query',
    arguments: { type: 'entities_in_file', file_path: path.join(root, file), workspaceRoot: root },
  }), 'entities-in-file');
}

async function runScenario() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'clean-ctx-csharp-base-types-'));
  fs.writeFileSync(path.join(root, 'Targets.cs'),
    'public class LiveBase {}\npublic interface ILiveOne {}\npublic interface ILiveTwo {}\n');
  fs.writeFileSync(path.join(root, 'Headers.cs'), [
    'public partial class LivePartial : LiveBase {}',
    'public class LivePrimary(int value) : LiveBase {}',
    'public class LiveMulti',
    '    : LiveBase,',
    '      ILiveOne,',
    '      ILiveTwo',
    '{}',
    'public class LiveConstraintOnly<T> where T : class {}',
    'public class LiveCommentOnly // : ILiveOne',
    '{}',
    'public class LiveGeneric : GenericBase<int> {}',
    'public class LiveQualified : LiveNs.QualifiedBase {}',
    'public struct LiveStruct : ILiveOne {}',
    'public record LiveRecord(int Value) : ExternalRecordBase;',
    '',
  ].join('\n'));

  const client = new McpClient(root);
  try {
    await initialize(client);
    await publish(client, root, 'Headers.cs');

    console.log('\n[Case A] shared structured declaration identity');
    const entityResult = await entitiesInFile(client, root, 'Headers.cs');
    const identities = new Set((entityResult.entities ?? []).map(
      (entity) => entity.entity_type + '/' + entity.name));
    for (const expected of ['Class/LivePartial', 'Class/LivePrimary',
      'Struct/LiveStruct', 'Record/LiveRecord']) {
      check('A', expected + ' is registered exactly', identities.has(expected));
    }
    check('A', 'primary-constructor identity is not truncated',
      [...identities].every((identity) => !identity.includes('LivePrimary(int')));

    console.log('\n[Case B] multiline Class bases and typed classification');
    const multi = await query(client, root, 'forward_edges', 'Class', 'LiveMulti', 'multi');
    check('B', 'neutral first base is preserved',
      hasEdge(multi, 'HasBaseType', 'Class', 'LiveMulti', 'TypeRef', 'LiveBase'));
    check('B', 'class target derives Extends',
      hasEdge(multi, 'Extends', 'Class', 'LiveMulti', 'Class', 'LiveBase'));
    for (const name of ['ILiveOne', 'ILiveTwo']) check('B', name + ' derives Implements',
      hasEdge(multi, 'Implements', 'Class', 'LiveMulti', 'Interface', name));

    console.log('\n[Case C] constraints/comments do not fabricate bases');
    for (const name of ['LiveConstraintOnly', 'LiveCommentOnly']) {
      const result = await query(client, root, 'forward_edges', 'Class', name, 'negative-' + name);
      check('C', name + ' has no inheritance edge', inheritanceEdges(result).length === 0,
        JSON.stringify(inheritanceEdges(result)));
    }

    console.log('\n[Case D] qualified/generic references stay neutral');
    for (const [owner, written] of [['LiveGeneric', 'GenericBase<int>'],
      ['LiveQualified', 'LiveNs.QualifiedBase']]) {
      const result = await query(client, root, 'forward_edges', 'Class', owner, 'neutral-' + owner);
      check('D', written + ' remains intact',
        hasEdge(result, 'HasBaseType', 'Class', owner, 'TypeRef', written));
      check('D', written + ' is not guessed as typed',
        !(result.edges ?? []).some((edge) => ['Extends', 'Implements'].includes(edge.relation)));
    }

    console.log('\n[Case E] Struct/Record ownership remains distinct');
    for (const [ownerType, owner, written] of [
      ['Struct', 'LiveStruct', 'ILiveOne'],
      ['Record', 'LiveRecord', 'ExternalRecordBase'],
    ]) {
      const result = await query(client, root, 'forward_edges', ownerType, owner, 'owner-' + owner);
      check('E', ownerType + '/' + owner + ' publishes HasBaseType',
        hasEdge(result, 'HasBaseType', ownerType, owner, 'TypeRef', written));
      check('E', owner + ' never fabricates Class ownership',
        !(result.edges ?? []).some((edge) =>
          edge.subject?.entity_type === 'Class' && edge.subject?.name === owner));
    }

    console.log('\n[Case F] typed response evidence establishes capability');
    const reverse = await query(client, root, 'reverse_edges', 'Interface', 'ILiveOne',
      'reverse-interface');
    check('F', 'reverse query derives Implements',
      hasEdge(reverse, 'Implements', 'Class', 'LiveMulti', 'Interface', 'ILiveOne'));
    check('F', 'typed result establishes capability',
      reverse.coverage?.capability_established === true, JSON.stringify(reverse.coverage));
    check('F', 'coverage remains a lower bound',
      reverse.coverage?.result_semantics === 'lower_bound' &&
      reverse.coverage?.source_complete === false);
    check('F', 'neutral alternative guidance remains',
      reverse.coverage?.alternative_query?.entity_type === 'TypeRef' &&
      reverse.coverage?.alternative_query?.relation === 'HasBaseType');

    console.log('\n[Case G] repeated query is stable and duplicate-free');
    const repeated = await query(client, root, 'reverse_edges', 'Interface', 'ILiveOne',
      'reverse-interface-repeat');
    const digest = (result) => (result.edges ?? []).map(JSON.stringify).sort();
    const first = digest(reverse);
    const second = digest(repeated);
    check('G', 'repeated edge set is stable', JSON.stringify(first) === JSON.stringify(second));
    check('G', 'repeated edge set has no duplicates', new Set(second).size === second.length);
  } finally {
    await client.close();
    fs.rmSync(root, {
      recursive: true,
      force: true,
      maxRetries: 10,
      retryDelay: 100,
    });
  }
}

async function main() {
  console.log('binary: ' + binary);
  if (!fs.existsSync(binary)) {
    console.error('\nBinary not found. Build it first with the repository release command.');
    process.exit(2);
  }
  console.log('binary mtime: ' + fs.statSync(binary).mtime.toISOString());
  const stale = binaryIsStale();
  if (stale) console.log('  !! STALE BINARY: rebuild before treating this run as evidence.');
  await runScenario();
  if (stale) failures.unshift('selected binary predates one or more v0.9.2 source markers');

  const evidenceDir = path.join(repoRoot, 'target', 'live-acceptance', 'csharp-base-types');
  fs.mkdirSync(evidenceDir, { recursive: true });
  const evidenceFile = path.join(evidenceDir, 'responses.json');
  fs.writeFileSync(evidenceFile, JSON.stringify({ binary, stale, failures, captures }, null, 2) + '\n');

  console.log('\n-- summary --');
  console.log('evidence: ' + evidenceFile);
  if (failures.length === 0) console.log('ALL LIVE MCP CHECKS PASSED');
  else {
    console.log(failures.length + ' LIVE MCP CHECK(S) FAILED:');
    for (const failure of failures) console.log('  - ' + failure);
    process.exitCode = 1;
  }
}
main().catch((error) => { console.error(error); process.exit(1); });
