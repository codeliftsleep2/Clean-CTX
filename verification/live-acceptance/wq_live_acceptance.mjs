// Live MCP acceptance harness for the sparse `workspace_query` discovery
// diagnostics (LLM context-cost defect).
//
// It drives the REAL binary over MCP stdio and reports, for each case:
//   * the semantic answer (count + a short digest),
//   * the serialized `discovery` diagnostic (or ABSENT),
//   * diagnostics characters and the repository's own chars/4 token estimate,
//   * PASS/FAIL for the contract checks:
//       - none of the ten pre-fix flat diagnostic keys is present,
//       - no default-valued field appears in a reported diagnostic,
//       - the semantic payload survives.
//
// Usage (after the binary is rebuilt):
//   node verification/live-acceptance/wq_live_acceptance.mjs [path/to/clean-ctx(.exe)]
//
// This file is a handoff artifact; it lives outside the tracked tree.

import { spawn } from 'node:child_process';
import readline from 'node:readline';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const LEGACY_FLAT_KEYS = [
  'hydration_attempted',
  'discovery_provider',
  'discovery_status',
  'discovery_completed',
  'fallback_occurred',
  'fallback_reason',
  'candidates_discovered',
  'candidates_compiled',
  'project_coverage',
  'project_coverage_truncated',
];

const DEFAULT_VALUED = {
  provider: 'cbm',
  status: 'completed',
  attempted: true,
  fallback: false,
  discovered: 0,
  compiled: 0,
  projects_truncated: 0,
};

/// Freshest binary wins: `cargo build --release` leaves any older debug artifact
// in place, and measuring that one silently invalidates every check.
function resolveDefaultBinary() {
  const names =
    process.platform === 'win32' ? ['clean-ctx.exe', 'clean-ctx'] : ['clean-ctx'];
  const candidates = [];
  for (const profile of ['release', 'debug']) {
    for (const name of names) {
      const candidate = path.resolve('target', profile, name);
      if (fs.existsSync(candidate)) candidates.push(candidate);
    }
  }
  if (candidates.length === 0) {
    return path.resolve('target', 'debug', names[0]);
  }
  return candidates.sort(
    (left, right) => fs.statSync(right).mtimeMs - fs.statSync(left).mtimeMs,
  )[0];
}

// The binary is resolved to an ABSOLUTE path: each case spawns it with a
// different working directory, so a relative path cannot be used.
const binary = process.argv[2]
  ? path.resolve(process.argv[2])
  : resolveDefaultBinary();

// The changed production file that MUST be newer than the binary for the sparse
// projection to be present in it. If the binary predates it, the run is
// measuring a stale artifact — reported loudly instead of as a contract FAIL.
const CHANGE_MARKER = path.resolve(
  'src',
  'mcp',
  'tool_handlers',
  'query',
  'diagnostics.rs',
);

function binaryIsStale() {
  try {
    return fs.statSync(binary).mtimeMs < fs.statSync(CHANGE_MARKER).mtimeMs;
  } catch {
    return false;
  }
}

const repoRoot = process.cwd();
const failures = [];
let staleBinary = false;

/** Line-delimited JSON-RPC client over the binary's stdio. */
class McpClient {
  constructor(cwd) {
    this.child = spawn(binary, [], { cwd, stdio: ['pipe', 'pipe', 'pipe'] });
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = '';
    this.spawnError = null;
    this.child.on('error', (error) => {
      this.spawnError = error;
      for (const [id, resolver] of this.pending) {
        this.pending.delete(id);
        resolver({ error: { message: `spawn failed: ${error.message}` } });
      }
    });
    this.child.stderr.on('data', (chunk) => {
      this.stderr += chunk.toString();
    });
    readline
      .createInterface({ input: this.child.stdout })
      .on('line', (line) => {
        const trimmed = line.trim();
        if (!trimmed) return;
        let message;
        try {
          message = JSON.parse(trimmed);
        } catch {
          return;
        }
        if (message.id === undefined) return;
        const resolver = this.pending.get(message.id);
        if (!resolver) return;
        this.pending.delete(message.id);
        resolver(message);
      });
  }

  request(method, params) {
    const id = this.nextId++;
    const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params });
    const answer = new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error(`timeout waiting for ${method}`)),
        120_000,
      );
      this.pending.set(id, (message) => {
        clearTimeout(timer);
        resolve(message);
      });
    });
    this.child.stdin.write(`${payload}\n`);
    return answer;
  }

  notify(method, params) {
    this.child.stdin.write(
      `${JSON.stringify({ jsonrpc: '2.0', method, params })}\n`,
    );
  }

  close() {
    this.child.stdin.end();
    this.child.kill();
  }
}

function makeWorkspace(label, files, config) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `clean-ctx-${label}-`));
  for (const [name, contents] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), contents);
  }
  if (config) {
    fs.writeFileSync(
      path.join(root, '.clean-ctx.json'),
      `${JSON.stringify(config, null, 2)}\n`,
    );
  }
  return root;
}

function check(caseName, description, condition, detail) {
  const status = condition ? 'PASS' : 'FAIL';
  if (!condition) failures.push(`${caseName}: ${description}`);
  console.log(`    [${status}] ${description}${detail ? ` — ${detail}` : ''}`);
}

function note(message) {
  console.log(`    [note] ${message}`);
}

function inspect(caseName, response, { requireAnswer = true } = {}) {
  const result = response?.result;
  const structured = result?.structuredContent ?? {};
  const diagnostic = structured.discovery;
  const flat = LEGACY_FLAT_KEYS.filter((key) => key in structured);
  const text = diagnostic === undefined ? '' : JSON.stringify(diagnostic);
  const defaultValued = Object.entries(diagnostic ?? {}).filter(([key, value]) => {
    if (!(key in DEFAULT_VALUED)) return false;
    return JSON.stringify(value) === JSON.stringify(DEFAULT_VALUED[key]);
  });
  const count = structured.count ?? 0;
  const items =
    structured.edges ?? structured.entities ?? structured.dependencies ?? [];

  console.log(`  semantic: count=${count} sample=${JSON.stringify(items[0] ?? null)}`);
  console.log(
    `  diagnostics: ${diagnostic === undefined ? 'ABSENT' : text} (${text.length} chars, ~${Math.ceil(text.length / 4)} tokens)`,
  );
  check(caseName, 'no pre-fix flat diagnostic key is present', flat.length === 0, flat.join(', '));
  check(
    caseName,
    'no default-valued field in the reported diagnostic',
    defaultValued.length === 0,
    defaultValued.map(([key]) => key).join(', '),
  );
  if (requireAnswer) {
    check(caseName, 'the semantic answer is present', items.length > 0, `count=${count}`);
  } else {
    note(`semantic answer presence is informational for ${caseName} (count=${count})`);
  }
  return { diagnostic, structured, count, flat };
}

async function callWorkspaceQuery(client, args) {
  return client.request('tools/call', { name: 'workspace_query', arguments: args });
}

async function initialize(client) {
  await client.request('initialize', {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'wq-live-acceptance', version: '1.0.0' },
  });
  client.notify('notifications/initialized', {});
}

// ── Live case A — steady state ─────────────────────────────────────────
//
// The real repository is the CBM-covered workspace on this machine, so an
// unchanged repeated query there is exactly the boring steady state: the answer
// plus a `discovery` key that is ABSENT (or, when CBM cannot cover the root, the
// two genuinely exceptional facts provider="filesystem" and its reason).
async function caseSteadyState() {
  console.log('\n[Live case A] steady state — repeated find_entities over this repository');
  const client = new McpClient(repoRoot);
  try {
    await initialize(client);
    // A name that exists in this repository, so hydration has something to
    // discover whether CBM or the filesystem fallback supplies candidates.
    const args = {
      type: 'find_entities',
      name: 'WorkspaceIndex',
      workspaceRoot: repoRoot,
    };
    const first = await callWorkspaceQuery(client, args);
    console.log('  -- first call');
    const firstResult = inspect('A1', first, { requireAnswer: false });
    const second = await callWorkspaceQuery(client, args);
    console.log('  -- repeated identical call (discovery cache hit)');
    const { diagnostic } = inspect('A2', second, { requireAnswer: false });
    const fields = Object.keys(diagnostic ?? {}).sort();
    check(
      'A2',
      'the repeat reports no candidates it did not discover',
      fields.every((field) => field !== 'discovered' && field !== 'compiled'),
      fields.join(', ') || 'ABSENT',
    );
    if (firstResult.diagnostic === undefined) {
      note(
        'discovery was complete with nothing noteworthy: the steady state carries no diagnostic at all',
      );
    } else if (firstResult.diagnostic.provider === undefined) {
      note(
        `discovery did real work on this call while provider/status stayed default: ${JSON.stringify(firstResult.diagnostic)}`,
      );
    } else {
      note(
        `CBM could not cover this root, so the fallback facts are genuinely exceptional: ${JSON.stringify(firstResult.diagnostic)}`,
      );
    }
  } finally {
    client.close();
  }
}

// ── Live case B — fresh compilation ────────────────────────────────────
async function caseFreshCompilation() {
  console.log('\n[Live case B] fresh compilation — a workspace the session never saw');
  const root = makeWorkspace('fresh', {
    'FreshTarget.ts': 'export class FreshTarget {}\n',
  });
  const client = new McpClient(root);
  try {
    await initialize(client);
    const response = await callWorkspaceQuery(client, {
      type: 'find_entities',
      name: 'FreshTarget',
      workspaceRoot: root,
    });
    const { diagnostic } = inspect('B', response);
    check(
      'B',
      'this query reports the files it discovered/compiled',
      (diagnostic?.discovered ?? 0) > 0 || (diagnostic?.compiled ?? 0) > 0,
      JSON.stringify(diagnostic ?? null),
    );
  } finally {
    client.close();
  }
}

// ── Live case C — filesystem fallback ─────────────────────────────────
async function caseFallback() {
  console.log('\n[Live case C] filesystem fallback — a root CBM does not cover');
  const root = makeWorkspace('fallback', {
    'FallbackTarget.ts': 'export class FallbackTarget {}\n',
  });
  const client = new McpClient(root);
  try {
    await initialize(client);
    const response = await callWorkspaceQuery(client, {
      type: 'find_entities',
      name: 'FallbackTarget',
      workspaceRoot: root,
    });
    const { diagnostic } = inspect('C', response);
    if (diagnostic === undefined) {
      note(
        'CBM covered this root in this environment, so the fallback path was not exercised — re-run where CBM is unavailable to observe provider/fallback_reason',
      );
    } else {
      check(
        'C',
        'the engaged provider and the fallback reason are reported',
        diagnostic.provider !== undefined &&
          typeof diagnostic.fallback_reason === 'string',
        JSON.stringify(diagnostic),
      );
      check(
        'C',
        'the reported fields are exactly the exceptional ones',
        Object.keys(diagnostic).every((key) =>
          [
            'provider',
            'status',
            'fallback_reason',
            'discovered',
            'compiled',
            'projects',
            'projects_truncated',
          ].includes(key),
        ),
        Object.keys(diagnostic).join(', '),
      );
    }
  } finally {
    client.close();
  }
}

// ── Live case D — project coverage exception ──────────────────────────
async function caseProjectException() {
  console.log('\n[Live case D] project coverage exception — an unregistered additional root');
  const root = makeWorkspace(
    'exception',
    { 'PrimaryTarget.ts': 'export class PrimaryTarget {}\n' },
    { additional_roots: [path.join(os.tmpdir(), 'clean-ctx-missing-root')] },
  );
  const client = new McpClient(root);
  try {
    await initialize(client);
    const response = await callWorkspaceQuery(client, {
      type: 'find_entities',
      name: 'PrimaryTarget',
      workspaceRoot: root,
    });
    const { diagnostic } = inspect('D', response);
    const projects = diagnostic?.projects ?? [];
    check(
      'D',
      'the unregistered additional root appears as a structured exceptional entry',
      projects.length > 0 && typeof projects[0].status === 'string',
      JSON.stringify(projects),
    );
    check(
      'D',
      'the entry stays minimal and keeps machine-readable fields',
      projects.length > 0 &&
        Object.keys(projects[0]).every((key) =>
          ['project', 'status', 'readiness', 'reason'].includes(key),
        ),
      JSON.stringify(projects[0] ?? null),
    );
  } finally {
    client.close();
  }
}

async function main() {
  console.log(`binary: ${binary}`);
  if (!fs.existsSync(binary)) {
    console.error('\nBinary not found. Build it first:\n  cargo build --all-features\n');
    process.exit(2);
  }
  console.log(`binary mtime: ${fs.statSync(binary).mtime.toISOString()}`);
  staleBinary = binaryIsStale();
  if (staleBinary) {
    console.log(
      '  !! STALE BINARY: it predates src/mcp/tool_handlers/query/diagnostics.rs, so every\n' +
        '     check below would measure the OLD response shape. Rebuild first:\n' +
        '       cargo build --all-features',
    );
  }
  await caseSteadyState();
  await caseFreshCompilation();
  await caseFallback();
  await caseProjectException();

  if (staleBinary) {
    failures.unshift('run measured a STALE binary (rebuild with cargo build --all-features)');
  }

  console.log('\n-- summary --');
  if (failures.length === 0) {
    console.log('ALL LIVE CHECKS PASSED');
  } else {
    console.log(`${failures.length} LIVE CHECK(S) FAILED:`);
    for (const failure of failures) console.log(`  - ${failure}`);
    process.exitCode = 1;
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
