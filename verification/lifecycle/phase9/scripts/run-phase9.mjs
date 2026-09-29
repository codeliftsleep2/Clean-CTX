// Phase 9 live production-path operator harness.
// NOT A TEST, NOT CI EVIDENCE, AND NOT A SUBSTITUTE FOR src/tests/**.

import { spawn, spawnSync } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(packageRoot, '..', '..', '..');
const stateRoot = path.join(repoRoot, 'target', 'phase9-verification', 'state');
const workspace = path.join(stateRoot, 'workspace');
const fallbackDir = path.join(workspace, '.clean-ctx', 'fallback');
const dbPath = path.join(workspace, '.clean-ctx', 'persistence.db');
const primary = path.join(workspace, 'primary.ts');
const peer = path.join(workspace, 'peer.cs');
const seedScript = path.join(scriptDir, 'seed-recovery.py');
const failures = [];

function binaryDefault() {
  const name = process.platform === 'win32' ? 'clean-ctx.exe' : 'clean-ctx';
  const candidates = ['release', 'debug']
    .map((profile) => path.join(repoRoot, 'target', profile, name))
    .filter(fs.existsSync);
  if (!candidates.length) return path.join(repoRoot, 'target', 'debug', name);
  return candidates.sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs)[0];
}

const binary = path.resolve(process.argv[2] ?? binaryDefault());

function check(scenario, claim, condition, detail = '') {
  const status = condition ? 'PASS' : 'FAIL';
  console.log(`[${status}] ${scenario}: ${claim}${detail ? ` — ${detail}` : ''}`);
  if (!condition) failures.push(`${scenario}: ${claim}${detail ? ` (${detail})` : ''}`);
}

function hash(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function snapshot(paths) {
  return Object.fromEntries(paths.map((item) => [item, fs.existsSync(item) ? hash(fs.readFileSync(item)) : null]));
}

function sameSnapshot(before, after) {
  return JSON.stringify(before) === JSON.stringify(after);
}

class Client {
  constructor() {
    this.child = spawn(binary, [], { cwd: workspace, stdio: ['pipe', 'pipe', 'pipe'] });
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = '';
    this.child.stderr.on('data', (chunk) => { this.stderr += chunk.toString(); });
    this.child.on('error', (error) => this.rejectAll(error));
    this.child.on('exit', (code) => this.rejectAll(new Error(`clean-ctx exited with ${code}`)));
    readline.createInterface({ input: this.child.stdout }).on('line', (line) => {
      let message;
      try { message = JSON.parse(line); } catch { return; }
      const settle = this.pending.get(message.id);
      if (settle) { this.pending.delete(message.id); settle.resolve(message); }
    });
  }

  rejectAll(error) {
    for (const [, settle] of this.pending) settle.reject(error);
    this.pending.clear();
  }

  request(method, params = {}) {
    const id = this.nextId++;
    const result = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`timeout: ${method}`)), 120_000);
      this.pending.set(id, {
        resolve: (message) => { clearTimeout(timer); resolve(message); },
        reject: (error) => { clearTimeout(timer); reject(error); },
      });
    });
    this.child.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`);
    return result;
  }

  tool(name, args = {}) {
    return this.request('tools/call', { name, arguments: args });
  }

  async initialize() {
    await this.request('initialize', {
      protocolVersion: '2025-03-26',
      capabilities: {},
      clientInfo: { name: 'phase9-operator-harness', version: '1' },
    });
    this.child.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' })}\n`);
  }

  close() {
    this.child.stdin.end();
    this.child.kill();
  }
}

function resultOf(response, label) {
  if (response?.error) throw new Error(`${label}: ${response.error.message}`);
  if (response?.result?.isError) throw new Error(`${label}: ${response.result.content?.[0]?.text}`);
  return response.result;
}

function seed(mode) {
  const python = process.env.PYTHON ?? (process.platform === 'win32' ? 'py' : 'python3');
  const prefix = python === 'py' ? ['-3'] : [];
  const stage = path.join(stateRoot, `${mode}.stage`);
  const run = spawnSync(python, [...prefix, seedScript, mode, dbPath, primary, stage], {
    cwd: repoRoot,
    encoding: 'utf8',
  });
  if (run.status !== 0) throw new Error(`recovery seed failed: ${run.stderr || run.stdout}`);
}

function prepareWorkspace() {
  fs.rmSync(workspace, { recursive: true, force: true });
  fs.mkdirSync(fallbackDir, { recursive: true });
  fs.copyFileSync(path.join(packageRoot, 'fixtures', 'primary.ts'), primary);
  const peerText = fs.readFileSync(path.join(packageRoot, 'fixtures', 'peer.cs'), 'utf8').replace(/\r?\n/g, '\r\n');
  fs.writeFileSync(peer, Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), Buffer.from(peerText, 'utf8')]));
  fs.writeFileSync(path.join(workspace, '.clean-ctx.json'), JSON.stringify({
    persistence: { enabled: true, auto_save: true, max_history_days: 30, db_path: '.clean-ctx/persistence.db' },
  }, null, 2));
  const artifacts = {
    '01-save.json': { type: 'save_context', file_path: primary, source_hash: 'legacy-hash' },
    '02-delta.json': { type: 'append_delta', context_id: 'legacy-context', delta_payload: 'AA==' },
    '03-clear.json': { type: 'clear_file', file_path: primary },
  };
  for (const [name, value] of Object.entries(artifacts)) {
    fs.writeFileSync(path.join(fallbackDir, name), JSON.stringify(value));
  }
  fs.writeFileSync(path.join(fallbackDir, '04-malformed.json'), '{not-json');
}

async function withClient(action) {
  const client = new Client();
  try { await client.initialize(); return await action(client); } finally { client.close(); }
}

async function main() {
  if (!fs.existsSync(binary)) throw new Error(`built binary not found: ${binary}`);
  prepareWorkspace();
  const initialPrimary = fs.readFileSync(primary);
  const initialPeer = fs.readFileSync(peer);
  const artifactPaths = fs.readdirSync(fallbackDir).map((name) => path.join(fallbackDir, name));
  const artifactsBefore = snapshot(artifactPaths);

  await withClient(async (client) => {
    const tools = resultOf(await client.request('tools/list'), 'tools/list').tools ?? [];
    check('J', 'registered inspection tool is discoverable', tools.some((tool) => tool.name === 'inspect_legacy_fallbacks'));

    const baseline = resultOf(await client.tool('compress_code_context', {
      filePath: primary, workspaceRoot: workspace, fidelity: 'edit',
    }), 'baseline compression');
    check('A', 'baseline returns structured hierarchy', Boolean(baseline.ir));
    check('C', 'edit fidelity returns the complete method body', baseline.content?.[0]?.text?.includes('return `${label}:ready`'));
    check('A', 'physical persistence database exists', fs.existsSync(dbPath));

    const saved = resultOf(await client.tool('save_context', { filePath: primary }), 'save_context');
    check('A', 'file-scoped checkpoint reports truthfully', /saved|durable/i.test(saved.content?.[0]?.text ?? ''));

    const readOnlyBefore = snapshot([primary, peer, dbPath, ...artifactPaths]);
    resultOf(await client.tool('context_history', { filePath: primary }), 'context_history');
    resultOf(await client.tool('context_stats', { format: 'json' }), 'context_stats');
    resultOf(await client.tool('list_sessions'), 'list_sessions');
    const readOnlyAfter = snapshot([primary, peer, dbPath, ...artifactPaths]);
    check('G', 'history/stats/list leave observable storage and sources unchanged', sameSnapshot(readOnlyBefore, readOnlyAfter));

    const inspected = resultOf(await client.tool('inspect_legacy_fallbacks'), 'inspect_legacy_fallbacks');
    const rows = inspected.structuredContent?.artifacts ?? [];
    const kinds = new Set(rows.map((row) => row.operation));
    check('J', 'save, delta, and clear artifacts are reported', ['save_context', 'append_delta', 'clear_file'].every((kind) => kinds.has(kind)));
    check('J', 'malformed artifact is reported rather than skipped', rows.some((row) => row.availableMetadata === null));
    check('J', 'every artifact is explicitly unrecoverable', rows.length === 4 && rows.every((row) => row.recoverable === false && /lacks complete/i.test(row.reason)));
    check('J', 'inspection leaves quarantined artifacts byte-identical', sameSnapshot(artifactsBefore, snapshot(artifactPaths)));

    const oldBody = '{\n    const label = "café";\n    try {\n      return `${label}:ready`;\n    } catch (error) {\n      return String(error);\n    }\n  }';
    const newBody = '{\n    const label = "café";\n    try {\n      return `${label}:READY`;\n    } catch (error) {\n      return String(error);\n    }\n  }';
    const beforeEdit = fs.readFileSync(primary);
    const edit = resultOf(await client.tool('apply_edit', {
      filePath: primary,
      workspaceRoot: workspace,
      verify: true,
      operations: [{ type: 'replace_body', target: 'Unsafe.run', expectedOldText: oldBody, newText: newBody }],
    }), 'apply_edit');
    const afterEdit = fs.readFileSync(primary);
    const oldBytes = Buffer.from(oldBody);
    const newBytes = Buffer.from(newBody);
    const start = beforeEdit.indexOf(oldBytes);
    check('C', 'bytes before the edit are identical', start >= 0 && beforeEdit.subarray(0, start).equals(afterEdit.subarray(0, start)));
    check('C', 'replacement bytes are exact', afterEdit.subarray(start, start + newBytes.length).equals(newBytes));
    check('C', 'bytes after the edit are identical', beforeEdit.subarray(start + oldBytes.length).equals(afterEdit.subarray(start + newBytes.length)));
    check('C', 'operation returns absolute byte-span information', Number.isInteger(edit.structuredContent?.operations?.[0]?.startByte));

    const deltaSource = fs.readFileSync(primary, 'utf8');
    const deltaTarget = deltaSource.replace(':READY', ':DELTA-TARGET');
    if (deltaTarget === deltaSource) throw new Error('delta fixture body marker was not found');
    fs.writeFileSync(primary, deltaTarget);
    const generated = resultOf(await client.tool('delta_code_context', {
      filePath: primary, workspaceRoot: workspace, fidelity: 'edit',
    }), 'delta generation');
    check('B', 'production emits dv:2 sequence delta', generated.delta?.dv === 2);
    if (generated.delta?.dv !== 2) {
      throw new Error(`delta generation returned no semantic dv:2 payload: ${JSON.stringify(generated)}`);
    }
    const applied = resultOf(await client.tool('apply_delta', {
      delta: generated.delta, currentVersion: generated.from_version,
    }), 'delta application');
    check('B', 'accepted delta advances to target version', applied._meta?.version === generated.to_version || applied.version === generated.to_version);
    const replayed = resultOf(await client.tool('replay_history', { filePath: primary }), 'history replay');
    check('B', 'persisted history replays through registered MCP', Boolean(replayed.ir));

    // Recovery intents carry a complete physical target IR, not a baseline plus
    // an outstanding delta. Checkpoint the replayed target so the crash fixture
    // seeds one coherent IR/version/hash/edge snapshot.
    resultOf(await client.tool('save_context', { filePath: primary }), 'post-replay checkpoint');

    const purgeBefore = snapshot([primary, peer, ...artifactPaths]);
    resultOf(await client.tool('purge_old_deltas', { days: 36500, filePath: primary }), 'purge_old_deltas');
    check('H', 'purge leaves source, peer, and quarantine artifacts unchanged', sameSnapshot(purgeBefore, snapshot([primary, peer, ...artifactPaths])));
    check('I', 'peer CRLF/BOM bytes remain untouched', fs.readFileSync(peer).equals(initialPeer));
  });

  await withClient(async (client) => {
    const restored = resultOf(await client.tool('restore_context', { filePath: primary, workspaceRoot: workspace }), 'restart restore');
    check('A', 'restart restores persisted canonical semantics', Boolean(restored.ir));
  });

  seed('prior');
  await withClient(async (client) => {
    const response = resultOf(await client.tool('restore_context', { filePath: primary, workspaceRoot: workspace }), 'prior-byte recovery');
    check('D', 'prior-byte intent resolves before durable restore', Boolean(response.ir));
  });

  seed('target');
  await withClient(async (client) => {
    const response = resultOf(await client.tool('replay_history', { filePath: primary }), 'target-byte recovery');
    check('D', 'target-byte intent commits before history replay', Boolean(response.ir));
  });

  seed('failure');
  const failureSource = fs.readFileSync(primary);
  await withClient(async (client) => {
    const response = await client.tool('restore_context', { filePath: primary, workspaceRoot: workspace });
    check('E', 'irreconcilable recovery fails structurally', Boolean(response.error || response.result?.isError));
    check('E', 'failed recovery leaves exact source bytes unchanged', fs.readFileSync(primary).equals(failureSource));
  });

  // Remove the deliberately irreconcilable operator seed so deletion can own
  // the complete context lifecycle without first recovering it.
  const python = process.env.PYTHON ?? (process.platform === 'win32' ? 'py' : 'python3');
  const sql = 'import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute("DELETE FROM edit_intents WHERE file_path=?",(sys.argv[2],)); c.commit()';
  const cleared = spawnSync(python, [...(python === 'py' ? ['-3'] : []), '-c', sql, dbPath, primary], { encoding: 'utf8' });
  if (cleared.status !== 0) throw new Error(cleared.stderr);

  await withClient(async (client) => {
    resultOf(await client.tool('restore_context', { filePath: primary, workspaceRoot: workspace }), 'pre-delete restore');
    const beforeDelete = fs.readFileSync(primary);
    const deleted = resultOf(await client.tool('delete_context', { filePath: primary }), 'delete_context');
    check('F', 'registered deletion reports actual deletion', deleted._meta?.deleted === 1);
    check('F', 'deletion never changes source bytes', fs.readFileSync(primary).equals(beforeDelete));
    const missing = await client.tool('restore_context', { filePath: primary, workspaceRoot: workspace });
    check('F', 'deleted context cannot subsequently restore', Boolean(missing.error || missing.result?.isError));
    check('I', 'another file remains byte-identical throughout', fs.readFileSync(peer).equals(initialPeer));
  });

  check('J', 'fallback artifacts remain untouched at end of lifecycle', sameSnapshot(artifactsBefore, snapshot(artifactPaths)));
  check('C', 'fixture included multibyte UTF-8 before editing', initialPrimary.includes(Buffer.from('café')));
  console.log(`\nOperator scenarios: ${failures.length ? 'FAIL' : 'PASS'} (${failures.length} failure(s))`);
  if (failures.length) {
    for (const failure of failures) console.error(`  - ${failure}`);
    process.exitCode = 1;
  }
}

main().catch((error) => {
  console.error(`HARNESS ERROR: ${error.stack ?? error}`);
  process.exitCode = 1;
});
