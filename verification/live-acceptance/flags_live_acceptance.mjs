// Live MCP acceptance harness for the repeated-`Flags` projection fix
// (`CoreOp::Flags` per method id now accumulates instead of overwriting).
//
// THIS IS A HAND-OFF ARTIFACT, NOT A TEST, NOT COVERAGE, NOT RED->GREEN
// EVIDENCE, AND NOT PART OF THE CI GATE. It exists so the operator can drive a
// freshly built binary over MCP stdio and read the REAL rendered output. The
// contract for this fix lives in tracked tests under src/tests/**, which the
// CI gate compiles and runs:
//
//   src/tests/ir/hierarchical_flags.rs             (RED-FLAG1..6, 11, 12 + wire)
//   src/tests/ir/hierarchical_flags_languages.rs   (RED-FLAG7..RED-FLAG10)
//
// Usage (after rebuilding the binary):
//   node verification/live-acceptance/flags_live_acceptance.mjs [path/to/clean-ctx(.exe)]
//
// Per case it prints the source shape, the rendered method line, the expected
// flags and the ACTUAL flags taken from the rendered `fl:` field.

import { spawn } from 'node:child_process';
import readline from 'node:readline';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

/** Freshest binary wins: a stale artifact would invalidate every check. */
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
  if (candidates.length === 0) return path.resolve('target', 'debug', names[0]);
  return candidates.sort(
    (left, right) => fs.statSync(right).mtimeMs - fs.statSync(left).mtimeMs,
  )[0];
}

const binary = process.argv[2]
  ? path.resolve(process.argv[2])
  : resolveDefaultBinary();

// The fix's own boundary: a binary older than this file predates the fix.
const CHANGE_MARKERS = [
  path.resolve('src', 'ir', 'hierarchical', 'encode.rs'),
  path.resolve('src', 'ir', 'pipeline.rs'),
  path.resolve('src', 'ir', 'pipeline', 'core.rs'),
];

function binaryIsStale() {
  try {
    const binaryTime = fs.statSync(binary).mtimeMs;
    return CHANGE_MARKERS.some(
      (marker) => fs.existsSync(marker) && binaryTime < fs.statSync(marker).mtimeMs,
    );
  } catch {
    return false;
  }
}

const failures = [];

/** Line-delimited JSON-RPC client over the binary's stdio. */
class McpClient {
  constructor(cwd) {
    this.child = spawn(binary, [], { cwd, stdio: ['pipe', 'pipe', 'pipe'] });
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = '';
    this.child.on('error', (error) => {
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
    this.child.stdin.write(
      `${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`,
    );
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

function check(caseName, description, condition, detail) {
  const status = condition ? 'PASS' : 'FAIL';
  if (!condition) failures.push(`${caseName}: ${description}`);
  console.log(`    [${status}] ${description}${detail ? ` — ${detail}` : ''}`);
}

/** The rendered `M <name> ...` line(s) of the output. */
function methodLines(text) {
  return text
    .split('\n')
    .filter((line) => /^M /.test(line.trim()))
    .map((line) => line.trim());
}

/** The flag list of a rendered method line, from its `fl:` field. */
function flagsOf(line) {
  const match = /\bfl:([^\s]+)/.exec(line);
  return match ? match[1].split(',') : [];
}

function makeWorkspace(label, files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `clean-ctx-flags-${label}-`));
  for (const [name, contents] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), contents);
  }
  return root;
}

async function provideContext(client, root, file, args) {
  return client.request('tools/call', {
    name: 'provide_code_context',
    arguments: {
      filePath: path.join(root, file),
      workspaceRoot: root,
      ...args,
    },
  });
}

function responseParts(response) {
  const result = response?.result ?? {};
  const kind = result?._meta?.content_kind ?? 'missing';
  const text = result?.content?.[0]?.text ?? '';
  return { kind, text, error: response?.error };
}

// ── Fixtures ───────────────────────────────────────────────────────────

/**
 * Each case: a real source shape, the method whose rendered `fl:` field is
 * measured, and the flags that MUST be present now.
 *
 * `required` lists the values whose loss is the defect (declaration modifiers
 * and control-flow flags together). A leading pattern-additive op (for example
 * `OBSERVABLE`, which the additive recognizer emits BEFORE `DefMethod`) is
 * deliberately NOT required: a `Flags` op that precedes its `DefMethod` has no
 * node to attach to yet, which is a separate, unfixed boundary — this harness
 * measures only the projection of ops that land while the method is open.
 */
const CASES = [
  {
    label: 'C# · static + return',
    file: 'FlagProbe.cs',
    method: 'Pick',
    required: ['STATIC', 'RET'],
    source: `namespace Ordering;

public static class FlagProbe
{
    public static int Pick(int[] values)
    {
        if (values.Length == 0)
        {
            return 0;
        }

        return values.Length;
    }
}
`,
  },
  {
    label: 'C# · async + private + if/throw/return',
    file: 'AsyncProbe.cs',
    method: 'CountAsync',
    required: ['ASYNC', 'PRIVATE', 'RET'],
    source: `public class AsyncProbe
{
    private async System.Threading.Tasks.Task<int> CountAsync(int[] values)
    {
        if (values.Length == 0)
        {
            throw new System.ArgumentException(nameof(values));
        }

        return await System.Threading.Tasks.Task.FromResult(values.Length);
    }
}
`,
  },
  {
    label: 'TypeScript · static + return',
    file: 'FlagProbe.ts',
    method: 'pick',
    required: ['STATIC', 'RET'],
    source: `export class FlagProbe {
    static pick(values: string[]): boolean {
        if (values.length === 0) {
            return false;
        }

        return true;
    }
}
`,
  },
  {
    label: 'TypeScript · async + return',
    file: 'AsyncProbe.ts',
    method: 'load',
    required: ['ASYNC', 'RET'],
    source: `export class AsyncProbe {
    async load(id: string): Promise<string> {
        if (id.length === 0) {
            return "";
        }

        return id;
    }
}
`,
  },
  {
    label: 'Java · public static + return',
    file: 'FlagProbe.java',
    method: 'pick',
    required: ['EXPORT', 'STATIC', 'RET'],
    source: `public class FlagProbe {
    public static int pick(int[] values) {
        if (values.length == 0) {
            return 0;
        }

        return values.length;
    }
}
`,
  },
  {
    label: 'Rust · pub + return',
    file: 'flag_probe.rs',
    method: 'pick',
    required: ['EXPORT', 'RET'],
    source: `pub struct FlagProbe;

impl FlagProbe {
    pub fn pick(values: &[i32]) -> usize {
        if values.is_empty() {
            return 0;
        }

        values.len()
    }
}
`,
  },
];

/** The declared shape of the measured method: its first source line. */
function sourceShape(source, method) {
  const line = source
    .split('\n')
    .find((candidate) => candidate.includes(method) && candidate.includes('('));
  return (line ?? '').trim();
}

// ── Runner ─────────────────────────────────────────────────────────────

async function initialize(client) {
  await client.request('initialize', {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'flags-live-acceptance', version: '1.0.0' },
  });
  client.notify('notifications/initialized', {});
}

async function runCase(client, entry) {
  console.log(`\n[live case] ${entry.label}`);
  const root = makeWorkspace(entry.method, { [entry.file]: entry.source });
  console.log(`  file:      ${entry.file}`);
  console.log(`  source:    ${sourceShape(entry.source, entry.method)}`);
  console.log(`  expected:  ${entry.required.join(',')} (all must be present)`);

  for (const fidelity of ['low', 'medium']) {
    const response = await provideContext(client, root, entry.file, { fidelity });
    const { kind, text, error } = responseParts(response);
    if (error) {
      check(entry.label, `${fidelity}: request succeeds`, false, JSON.stringify(error));
      continue;
    }

    const lines = methodLines(text);
    const measured = lines.find((line) => line.includes(entry.method)) ?? '';
    const actual = flagsOf(measured);
    console.log(`  -- ${fidelity} (content_kind: ${kind}) --`);
    console.log(`     rendered: ${measured || '(no M line found)'}`);
    console.log(`     actual:   ${actual.length ? actual.join(',') : '(no fl: field)'}`);

    check(
      entry.label,
      `${fidelity}: compressed path (not raw_passthrough)`,
      kind !== 'raw_passthrough',
      kind,
    );
    check(
      entry.label,
      `${fidelity}: an M line for ${entry.method} was rendered`,
      measured !== '',
      measured,
    );
    for (const flag of entry.required) {
      check(
        entry.label,
        `${fidelity}: fl: carries ${flag}`,
        actual.includes(flag),
        `actual: ${actual.join(',') || '(none)'}`,
      );
    }
    check(
      entry.label,
      `${fidelity}: no repeated flag value`,
      new Set(actual).size === actual.length,
      actual.join(','),
    );
  }
}

async function main() {
  console.log(`binary: ${binary}`);
  if (!fs.existsSync(binary)) {
    console.log('FAIL: binary not found — build it first, then re-run.');
    process.exitCode = 1;
    return;
  }
  if (binaryIsStale()) {
    console.log(
      'WARNING: the binary is OLDER than the changed sources — rebuild before ' +
        'trusting any FAIL below (a stale artifact always looks like a failure).',
    );
  }

  const client = new McpClient(process.cwd());
  try {
    await initialize(client);
    for (const entry of CASES) {
      await runCase(client, entry);
    }
  } finally {
    client.close();
  }

  console.log(
    `\n${failures.length === 0 ? 'ALL CHECKS PASSED' : `FAILURES: ${failures.length}`}`,
  );
  for (const failure of failures) console.log(`  - ${failure}`);
  console.log(
    '\nREMINDER: this harness is operator evidence only — never a test result, ' +
      'never RED->GREEN evidence, never CI coverage.',
  );
  if (failures.length > 0) process.exitCode = 1;
}

await main();
