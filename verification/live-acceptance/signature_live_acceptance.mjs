// Live MCP acceptance harness for the structural method-identity fix
// (provide_code_context: generic method names and tuple return types).
//
// THIS IS A HAND-OFF ARTIFACT, NOT A TEST. It exists so the operator can drive
// a freshly built binary and read real output. The contract for this fix lives
// in tracked tests under src/tests/**, which the CI gate compiles and runs:
//
//   src/tests/compaction/signature.rs                  (shared structural boundary)
//   src/tests/ir/method_signature_shape.rs             (RED-SIG1..RED-SIG12)
//   src/tests/ir/signature_cross_language.rs           (TS / Java / Rust probes)
//   src/tests/mcp/provider_code_context_signature.rs   (end-to-end dispatch)
//
// A PASS printed below is NEVER test evidence and never substitutes for those.
//
// Usage (after the binary is rebuilt):
//   node verification/live-acceptance/signature_live_acceptance.mjs [path/to/clean-ctx(.exe)]
//
// Per case it reports the actual rendered method lines, `content_kind`, the
// fidelity, and PASS/FAIL for:
//   * Pair present (with both type parameters at medium/high),
//   * `TSecond>` absent as a method identity,
//   * GetPair present, Tenth present,
//   * `static(+2)` absent (no fabricated overload group),
//   * tuple members not interpreted as the parameter list,
//   * real formal parameters preserved (three, not four),
//   * no fabricated `X <body expression>` / extends line.

import { spawn } from 'node:child_process';
import readline from 'node:readline';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

/// Freshest binary wins: a stale debug artifact would invalidate every check.
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

// The sources this harness measures. If the binary predates any of them, the
// run is measuring a stale artifact — reported loudly instead of as a FAIL.
const CHANGE_MARKERS = [
  path.resolve('src', 'compaction', 'signature.rs'),
  path.resolve('src', 'compaction', 'method.rs'),
  path.resolve('src', 'ir', 'pipeline.rs'),
  path.resolve('src', 'ir', 'pipeline', 'signature.rs'),
  path.resolve('src', 'ir', 'layers', 'csharp.rs'),
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

function summarize(caseName, text, extra = {}) {
  const lines = text
    .split('\n')
    .filter((line) => /^\s*(M|X|cl:|\/\/)/.test(line))
    .slice(0, 24);
  console.log('    rendered method/class lines:');
  for (const line of lines) console.log(`      ${line}`);
  for (const [key, value] of Object.entries(extra)) {
    console.log(`    ${key}: ${value}`);
  }
  return text;
}

async function initialize(client) {
  await client.request('initialize', {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'signature-live-acceptance', version: '1.0.0' },
  });
  client.notify('notifications/initialized', {});
}

const CS_FIXTURE = `using System.Linq;
using System.Linq.Expressions;

namespace Ordering;

public static class QueryablePairExtensions
{
    public static IOrderedQueryable<TFirst> Pair<TFirst, TSecond>(
        this IQueryable<TFirst> source,
        Expression<Func<TFirst, TSecond>> keySelector,
        ListSortDirection direction)
    {
        return source.OrderByDescending(keySelector);
    }

    public static (int alpha, int beta) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }

    public static (string name, int count) Tenth(string[] names)
    {
        return (names[0], names.Length);
    }

    public static int Pick(int[] values, bool ascending)
    {
        return values.Length == 0 ? 0 : values.OrderByDescending(v => v).First();
    }
${Array.from(
  { length: 30 },
  (_, i) => `
    public static int Filler${i}(int value, string label)
    {
        var adjusted = value * ${i} + label.Length;
        var bounded = adjusted > 100 ? 100 : adjusted;
        var described = label + ":" + bounded;
        System.Console.WriteLine(described);
        return bounded;
    }`,
).join('\n')}
}
`;

const TS_FIXTURE = `export function pair<A, B>(a: A, b: B): [A, B] {
  return [a, b];
}
${Array.from(
  { length: 30 },
  (_, i) => `
export function filler${i}(value: number, label: string): number {
  const adjusted = value * ${i} + label.length;
  const bounded = adjusted > 100 ? 100 : adjusted;
  const described = label + ":" + bounded;
  console.log(described);
  return bounded;
}`,
).join('\n')}
`;

const RS_FIXTURE = `pub struct Pairer;

impl Pairer {
    pub fn pair<A, B>(a: A, b: B) -> (i32, i32) {
        (1, 2)
    }
${Array.from(
  { length: 30 },
  (_, i) => `
    pub fn filler${i}(value: i64, label: &str) -> i64 {
        let adjusted = value * ${i} + label.len() as i64;
        let bounded = if adjusted > 100 { 100 } else { adjusted };
        println!("{}", bounded);
        bounded
    }`,
).join('\n')}
}
`;

const JAVA_FIXTURE = `public class Pairer {
    public <A, B> Result<A> pair(A a, B b) {
        return null;
    }
${Array.from(
  { length: 30 },
  (_, i) => `
    public int filler${i}(int value, String label) {
        int adjusted = value * ${i} + label.length();
        int bounded = adjusted > 100 ? 100 : adjusted;
        System.out.println(bounded);
        return bounded;
    }`,
).join('\n')}
}
`;

function makeWorkspace(label, files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `clean-ctx-signature-${label}-`));
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

/**
 * The PARAMETER section of a rendered method line: everything after `p:` up to
 * the next declaration arrow. `render_llm` renders parameters as
 * `p:<name>:<type>` and the DECLARED return type after the next `→`, so a
 * field-shift defect (tuple members read as parameters) shows up here — while
 * the tuple legitimately appears as the return type at high fidelity.
 */
function paramSection(line) {
  const start = line.indexOf('p:');
  if (start < 0) return '';
  const rest = line.slice(start + 2);
  const end = rest.indexOf('→');
  return (end < 0 ? rest : rest.slice(0, end)).trim();
}

/** How many times `needle` occurs in `haystack`. */
function occurrences(haystack, needle) {
  return haystack.split(needle).length - 1;
}

/** The C# case: the reported defect's fixture, across the fidelity matrix. */
async function caseCsharpMatrix() {
  console.log('\n[Live case A] C# — generic name + tuple return across the fidelity matrix');
  const root = makeWorkspace('cs-matrix', { 'QueryablePairExtensions.cs': CS_FIXTURE });
  const client = new McpClient(root);
  try {
    await initialize(client);
    for (const fidelity of ['low', 'medium', 'high']) {
      const response = await provideContext(client, root, 'QueryablePairExtensions.cs', {
        fidelity,
      });
      const { kind, text, error } = responseParts(response);
      if (error) {
        check('A', `${fidelity}: request succeeds`, false, JSON.stringify(error));
        continue;
      }
      console.log(`\n  -- fidelity: ${fidelity} --`);
      summarize('A', text, { content_kind: kind });

      check('A', `${fidelity}: compressed path (not raw_passthrough)`, kind !== 'raw_passthrough', kind);
      check('A', `${fidelity}: GetPair is a method identity`, text.includes('M GetPair'));
      check('A', `${fidelity}: Tenth is a method identity`, text.includes('M Tenth'));
      check('A', `${fidelity}: TSecond> is never a method identity`, !text.includes('M TSecond>'));
      check('A', `${fidelity}: no fabricated overload group`, !text.includes('static(+2)'));
      check(
        'A',
        `${fidelity}: no fabricated extends line`,
        !text.split('\n').some((line) => line.trimStart().startsWith('X ')) &&
          !text.includes('OrderByDescending'),
      );
      check(
        'A',
        `${fidelity}: generic identity intact`,
        fidelity === 'low'
          ? text.includes('M Pair')
          : text.includes('M Pair<TFirst, TSecond>'),
      );
      const getPairLine =
        text.split('\n').find((line) => line.includes('M GetPair')) ?? '';
      const getPairParams = paramSection(getPairLine);
      check(
        'A',
        `${fidelity}: tuple members are not the parameter list`,
        !getPairParams.includes('alpha') && !getPairParams.includes('beta'),
        getPairParams || '(no parameter section at this fidelity)',
      );
      if (fidelity === 'low') {
        // Low renders no parameter section for a non-overloaded method (the
        // established Low contract), so there is nothing to assert here — and
        // recording a vacuous PASS would be dishonest reporting.
        console.log(
          '    [info] low hides the parameter section for a single-overload method; ' +
            'the declared-parameter checks run at medium/high',
        );
      } else {
        check(
          'A',
          `${fidelity}: the declared parameter survives`,
          getPairParams.includes('int[] values'),
          getPairParams,
        );
      }
      if (fidelity === 'high') {
        // The positive half of the tuple fix: the tuple IS the return type,
        // rendered after the parameter section.
        check(
          'A',
          'high: the tuple is the RETURN type, never the identity',
          getPairLine.includes('→ (int alpha, int beta)') &&
            !getPairLine.includes('M (int') &&
            !getPairLine.includes('M static'),
          getPairLine.trim(),
        );
      }
      if (fidelity !== 'low') {
        const pairLine = text.split('\n').find((line) => line.includes('M Pair')) ?? '';
        const pairParams = paramSection(pairLine);
        check(
          'A',
          `${fidelity}: the three written parameters stay three`,
          ['source', 'keySelector', 'direction'].every((name) =>
            pairParams.includes(name),
          ) &&
            // The old generic-comma inflation added a FOURTH parameter by
            // splitting `Expression<Func<TFirst, TSecond>> keySelector`
            // in two, so `keySelector` appeared twice.
            occurrences(pairParams, 'keySelector') === 1,
          pairParams,
        );
      }
    }
  } finally {
    client.close();
  }
}

/** The Edit + focusMethods case: symbol targeting resolves the real names. */
async function caseEditFocus() {
  console.log('\n[Live case B] C# — edit fidelity + focusMethods on the corrected identities');
  const root = makeWorkspace('cs-focus', { 'QueryablePairExtensions.cs': CS_FIXTURE });
  const client = new McpClient(root);
  try {
    await initialize(client);
    const response = await provideContext(client, root, 'QueryablePairExtensions.cs', {
      fidelity: 'edit',
      focusMethods: ['GetPair', 'Pair<TFirst, TSecond>'],
    });
    const { kind, text, error } = responseParts(response);
    if (error) {
      check('B', 'request succeeds', false, JSON.stringify(error));
      return;
    }
    summarize('B', text, { content_kind: kind });
    check('B', 'compressed path (not raw_passthrough)', kind !== 'raw_passthrough', kind);
    check(
      'B',
      'focusing GetPair yields its verbatim body',
      text.includes('return (values[0], values[1]);'),
    );
    check(
      'B',
      'focusing the generic method yields its verbatim body',
      text.includes('return source.OrderByDescending(keySelector);'),
    );
    check(
      'B',
      'an unfocused method stays signature-only',
      !text.includes('return (names[0], names.Length);'),
    );
  } finally {
    client.close();
  }
}

/** Cross-language probes for the same shared boundary. */
async function caseCrossLanguage() {
  console.log('\n[Live case C] TypeScript / Rust / Java — the analogous probes');
  const cases = [
    { label: 'typescript', file: 'signature.ts', source: TS_FIXTURE },
    { label: 'rust', file: 'signature.rs', source: RS_FIXTURE },
    { label: 'java', file: 'Pairer.java', source: JAVA_FIXTURE },
  ];
  for (const entry of cases) {
    const root = makeWorkspace(entry.label, { [entry.file]: entry.source });
    const client = new McpClient(root);
    try {
      await initialize(client);
      for (const fidelity of ['medium', 'high']) {
        const response = await provideContext(client, root, entry.file, { fidelity });
        const { kind, text, error } = responseParts(response);
        if (error) {
          check('C', `${entry.label} ${fidelity}: request succeeds`, false, JSON.stringify(error));
          continue;
        }
        console.log(`\n  -- ${entry.label} / ${fidelity} (content_kind: ${kind}) --`);
        summarize('C', text);
        check('C', `${entry.label} ${fidelity}: compressed path`, kind !== 'raw_passthrough', kind);
        check('C', `${entry.label} ${fidelity}: pair is a method identity`, text.includes('M pair'));
        check(
          'C',
          `${entry.label} ${fidelity}: a type parameter is never the identity`,
          !text.includes('M B>') && !text.includes('M B\n'),
        );
      }
    } finally {
      client.close();
    }
  }
}

async function main() {
  console.log(`binary: ${binary}`);
  if (!fs.existsSync(binary)) {
    console.error('\nBinary not found. Build it first:\n  cargo build --all-features\n');
    process.exit(2);
  }
  console.log(`binary mtime: ${fs.statSync(binary).mtime.toISOString()}`);
  const stale = binaryIsStale();
  if (stale) {
    console.log(
      '  !! STALE BINARY: it predates the signature fix sources, so every check below\n' +
        '     would measure the OLD behaviour. Rebuild first:\n' +
        '       cargo build --all-features',
    );
  }

  await caseCsharpMatrix();
  await caseEditFocus();
  await caseCrossLanguage();

  if (stale) {
    failures.unshift('run measured a STALE binary (rebuild with cargo build --all-features)');
  }

  console.log('\n-- summary --');
  console.log(
    'Hand-off artifact: these live checks are NOT test evidence. The contract is the\n' +
      'tracked tests under src/tests/** that the CI gate compiles and runs.',
  );
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
