// Comprehensive live MCP acceptance for Angular Template Correctness and Linkage.
// Phases 1-6: ownership, rendering, identity, occurrence refinement, lifecycle,
// diagnostics, and coverage. Operator evidence only; not a src/tests regression.
//
// Usage from repository root:
//   node verification/live-acceptance/angular_template_linkage_live_acceptance.mjs [binary]

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
  for (const profile of ['release', 'debug']) {
    for (const name of names) {
      const candidate = path.resolve('target', profile, name);
      if (fs.existsSync(candidate)) candidates.push(candidate);
    }
  }
  if (!candidates.length) return path.resolve('target', 'debug', names[0]);
  return candidates.sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs)[0];
}

const binary = path.resolve(process.argv[2] ?? resolveBinary());
const sourceMarkers = [
  'src/angular_meta/semantic.rs',
  'src/angular_meta/template_ordered.rs',
  'src/angular_meta/template_occurrences.rs',
  'src/angular_meta/template_ownership.rs',
  'src/mcp/tool_handlers/core/provide_angular.rs',
  'src/mcp/tool_handlers/query/angular_templates.rs',
  'src/mcp/tool_handlers/query/edges.rs',
].map((item) => path.resolve(item));

function binaryIsStale() {
  try {
    const binaryTime = fs.statSync(binary).mtimeMs;
    return sourceMarkers.some(
      (marker) => fs.existsSync(marker) && binaryTime < fs.statSync(marker).mtimeMs,
    );
  } catch {
    return false;
  }
}

class McpClient {
  constructor(cwd) {
    this.child = spawn(binary, [], { cwd, stdio: ['pipe', 'pipe', 'pipe'] });
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = '';
    this.child.stderr.on('data', (chunk) => { this.stderr += chunk.toString(); });
    this.child.on('error', (error) => this.rejectAll(error));
    this.child.on('exit', (code) => {
      if (code !== 0 && this.pending.size) {
        this.rejectAll(new Error('clean-ctx exited with code ' + code));
      }
    });
    readline.createInterface({ input: this.child.stdout }).on('line', (line) => {
      let message;
      try { message = JSON.parse(line.trim()); } catch { return; }
      const settle = this.pending.get(message.id);
      if (!settle) return;
      this.pending.delete(message.id);
      settle.resolve(message);
    });
  }

  rejectAll(error) {
    for (const [, settle] of this.pending) settle.reject(error);
    this.pending.clear();
  }

  request(method, params = {}) {
    const id = this.nextId++;
    const answer = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('timeout waiting for ' + method)), 120000);
      this.pending.set(id, {
        resolve: (message) => { clearTimeout(timer); resolve(message); },
        reject: (error) => { clearTimeout(timer); reject(error); },
      });
    });
    this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n');
    return answer;
  }

  notify(method, params = {}) {
    this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method, params }) + '\n');
  }

  async initialize() {
    await this.request('initialize', {
      protocolVersion: '2025-03-26',
      capabilities: {},
      clientInfo: { name: 'angular-template-live-acceptance', version: '1.0.0' },
    });
    this.notify('notifications/initialized');
  }

  async close() {
    this.child.stdin.end();
    if (this.child.exitCode !== null) return;
    await new Promise((resolve) => {
      const force = setTimeout(() => this.child.kill(), 1000);
      const finish = () => { clearTimeout(force); resolve(); };
      this.child.once('exit', finish);
      this.child.once('error', finish);
    });
  }
}

function check(group, claim, condition, detail = '') {
  console.log('    [' + (condition ? 'PASS' : 'FAIL') + '] ' + claim +
    (detail ? ' — ' + detail : ''));
  if (!condition) failures.push(group + ': ' + claim);
}

function resultOf(response, label) {
  if (response?.error) throw new Error(label + ': ' + JSON.stringify(response.error));
  if (response?.result?.isError) {
    throw new Error(label + ': ' + JSON.stringify(response.result.content));
  }
  if (!response?.result) throw new Error(label + ': missing result');
  captures[label] = response.result;
  return response.result;
}

function structured(response, label) {
  const result = resultOf(response, label);
  if (!result.structuredContent) throw new Error(label + ': missing structuredContent');
  return result.structuredContent;
}

async function tool(client, name, args, label) {
  return resultOf(await client.request('tools/call', { name, arguments: args }), label);
}

async function publish(client, root, file, fidelity = 'edit') {
  return tool(client, 'provide_code_context', {
    workspaceRoot: root,
    filePath: file,
    fidelity,
  }, 'publish-' + path.basename(file) + '-' + fidelity);
}

async function query(client, root, type, entityType, name, label) {
  return structured(await client.request('tools/call', {
    name: 'workspace_query',
    arguments: {
      type,
      workspaceRoot: root,
      domain: 'angular',
      entity_type: entityType,
      name,
    },
  }), label);
}

async function refresh(client, root, label) {
  return structured(await client.request('tools/call', {
    name: 'refresh_workspace',
    arguments: { workspaceRoot: root },
  }), label);
}

function canonical(file) {
  return path.resolve(file);
}

function inlineIdentity(file, component) {
  return 'inline:' + canonical(file) + '#component=' + component;
}

function edge(result, relation, subjectType, subjectName, objectType, objectName) {
  return (result.edges ?? []).find((item) =>
    item.relation === relation &&
    item.subject?.domain === 'angular' &&
    item.subject?.entity_type === subjectType &&
    item.subject?.name === subjectName &&
    item.object?.domain === 'angular' &&
    item.object?.entity_type === objectType &&
    item.object?.name === objectName);
}

function relationEdges(result, relation) {
  return (result.edges ?? []).filter((item) => item.relation === relation);
}

function diagnosticCodes(result) {
  return (result.template_diagnostics ?? []).map((item) => item.code);
}

function writeComponent(file, className, selector, metadata) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, [
    "import { Component } from '@angular/core';",
    "@Component({ selector: '" + selector + "', " + metadata + ' })',
    'export class ' + className + ' {}',
    '',
  ].join('\n'));
}

function openingSpans(source, prefix) {
  const spans = [];
  let cursor = 0;
  while ((cursor = source.indexOf(prefix, cursor)) >= 0) {
    const end = source.indexOf('>', cursor);
    spans.push([cursor, end + 1]);
    cursor = end + 1;
  }
  return spans;
}

function occurrenceMatches(item, span, syntax, kind, header) {
  const evidence = item.evidence?.template_occurrence;
  const ancestry = evidence?.control_ancestry ?? [];
  return evidence?.span?.start_byte === span[0] &&
    evidence?.span?.end_byte === span[1] &&
    ancestry.some((frame) =>
      frame.syntax === syntax && frame.kind === kind &&
      frame.written_header.includes(header));
}

function padded(source) {
  return '<!-- compression padding -->\n'.repeat(300) + source;
}

async function run() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'clean-ctx-angular-template-'));
  // This native template acceptance must not admit an unrelated installed CBM.
  fs.writeFileSync(path.join(root, '.clean-ctx.json'), JSON.stringify({ cbm: { enabled: false } }));
  const files = {
    parent: path.join(root, 'parent.ts'),
    child: path.join(root, 'child.ts'),
    duplicateA: path.join(root, 'duplicate-a.ts'),
    duplicateB: path.join(root, 'duplicate-b.ts'),
    external: path.join(root, 'views', 'arbitrary-detail.html'),
    legacy: path.join(root, 'legacy.component.html'),
    modern: path.join(root, 'modern.component.html'),
    mixed: path.join(root, 'mixed.component.html'),
    ordinary: path.join(root, 'ordinary.html'),
    controls: path.join(root, 'controls.component.html'),
    inline: path.join(root, 'inline.ts'),
    missingOwner: path.join(root, 'missing-owner.ts'),
    missingTemplate: path.join(root, 'missing-template.html'),
  };
  fs.mkdirSync(path.dirname(files.external), { recursive: true });

  const externalSource = [
    '@if (showFirst) {',
    '  <child-widget [value]="first">{{ title | uppercase }}</child-widget>',
    '}',
    '@if (showSecond) {',
    '  <child-widget [value]="second"></child-widget>',
    '}',
    '<missing-widget></missing-widget>',
    '<duplicate-widget></duplicate-widget>',
  ].join('\n');
  fs.writeFileSync(files.external, externalSource);
  fs.writeFileSync(files.legacy, padded(
    '<section *ngIf="ready"><child-widget *ngFor="let item of items; trackBy: trackItem" [value]="item"></child-widget></section>',
  ));
  fs.writeFileSync(files.modern, padded(
    '@if (ready) { @for (item of items; track item.id) { <child-widget [value]="item"></child-widget> } @empty { No items. } }',
  ));
  fs.writeFileSync(files.mixed, padded(
    '<section *ngIf="ready">@if (selected) { <child-widget [value]="selected"></child-widget> }</section>',
  ));
  fs.writeFileSync(files.ordinary, '<main>ordinary HTML</main>');
  writeComponent(files.parent, 'ParentComponent', 'parent-host',
    "templateUrl: './views/arbitrary-detail.html'");
  writeComponent(files.child, 'ChildComponent', 'child-widget', "template: ''");
  writeComponent(files.duplicateA, 'DuplicateA', 'duplicate-widget', "template: ''");
  writeComponent(files.duplicateB, 'DuplicateB', 'duplicate-widget', "template: ''");
  writeComponent(files.inline, 'InlineParent', 'inline-parent',
    "template: '<section *ngIf=\"ready\">@if (show) { <child-widget></child-widget> }</section>'");
  writeComponent(files.missingOwner, 'MissingOwner', 'missing-owner',
    "templateUrl: './missing-template.html'");

  const client = new McpClient(root);
  try {
    await client.initialize();

    console.log('\n[Case A] Angular ownership routing and generic HTML preservation');
    const arbitrary = await publish(client, root, files.external, 'high');
    check('A', 'arbitrary templateUrl filename uses Angular processing',
      arbitrary._meta?.is_angular === true);
    check('A', 'arbitrary linked template preserves child binding',
      arbitrary.content?.[0]?.text?.includes('<child-widget') &&
      arbitrary.content?.[0]?.text?.includes('[value]="first"'));
    const ordinaryResponse = await client.request('tools/call', {
      name: 'provide_code_context',
      arguments: { workspaceRoot: root, filePath: files.ordinary, fidelity: 'high' },
    });
    check('A', 'unowned HTML is not silently classified as Angular',
      Boolean(ordinaryResponse.error) &&
      /verbatim/i.test(ordinaryResponse.error.message ?? '') &&
      /raw|uncompressed/i.test(ordinaryResponse.error.message ?? ''));

    console.log('\n[Case B] Angular 15+, modern, and mixed structural equivalence');
    for (const [label, file, expected] of [
      ['legacy', files.legacy, ['@if(ready)', '@for(item of items)', '<child-widget']],
      ['modern', files.modern, ['@if(ready)', '@for(item of items)', '@empty', '<child-widget']],
      ['mixed', files.mixed, ['@if(ready)', '@if(selected)', '<child-widget']],
    ]) {
      const output = await publish(client, root, file, 'medium');
      const text = output.content?.[0]?.text ?? '';
      check('B', label + ' template retains equivalent behavior',
        expected.every((token) => text.includes(token)), text.slice(0, 180));
    }

    console.log('\n[Case B2] Issue 141 complete modern control header delivery');
    fs.writeFileSync(files.controls, padded([
      '@let total = items.length;',
      '@if (ready) { <child-ready></child-ready> }',
      '@else if (loading) { <child-loading></child-loading> }',
      '@else { <child-empty></child-empty> }',
      '@for (item of items; track item.id) { <child-row [item]="item"></child-row> }',
      '@empty { <p>No rows.</p> }',
      '@defer (on viewport; prefetch on idle) { <heavy-panel></heavy-panel> }',
      '@placeholder (minimum 100ms) { <panel-placeholder></panel-placeholder> }',
      '@loading (after 50ms; minimum 200ms) { <panel-loading></panel-loading> }',
      '@error { <panel-error></panel-error> }',
      "@switch (mode) { @case ('a') { <case-a></case-a> } @default { <case-default></case-default> }",
    ].join('\n')));
    const controlTokens = [
      '@let total = items.length;', '@if(ready)', '<child-ready',
      '@else if(loading)', '<child-loading', '<child-empty',
      '@for(item of items)', 'track item.id', '<child-row', '@empty', 'No rows.',
      '@defer(on viewport; prefetch on idle)', '<heavy-panel',
      '@placeholder (minimum 100ms)', '<panel-placeholder',
      '@loading (after 50ms; minimum 200ms)', '<panel-loading', '@error', '<panel-error',
      '@switch(mode)', "@case('a')", '<case-a', '@default', '<case-default',
    ];
    for (const fidelity of ['medium', 'high']) {
      const output = await publish(client, root, files.controls, fidelity);
      const text = output.content?.[0]?.text ?? '';
      // High may legitimately select byte-exact passthrough when preserving
      // comments gives no compression savings. Verify the actual source form,
      // rather than require Medium's normalized directive spelling.
      const byteExact = text === fs.readFileSync(files.controls, 'utf8');
      const deliveredTokens = byteExact ? controlTokens.map((token) => {
        if (token === '@for(item of items)') return '@for (item of items; track item.id)';
        return token.replace(/^@(if|else if|defer|switch|case)\(/, '@$1 (');
      }) : controlTokens;
      let cursor = -1;
      const ordered = deliveredTokens.every((token) => {
        const next = text.indexOf(token, cursor + 1);
        if (next < 0) return false;
        cursor = next;
        return true;
      });
      check('B2', fidelity + ' retains modern headers attached to their ordered children', ordered);
      check('B2', fidelity + ' each condition/binding appears once',
        (byteExact ? ['@if (ready)', '@else if (loading)', 'track item.id', '@let total = items.length;']
          : ['@if(ready)', '@else if(loading)', 'track item.id', '@let total = items.length;'])
          .every((token) => text.split(token).length - 1 === 1));
    }

    console.log('\n[Case C] truthful fidelity degradation');
    const low = await publish(client, root, files.modern, 'low');
    const medium = await publish(client, root, files.modern, 'medium');
    const verbatim = await publish(client, root, files.modern, 'verbatim');
    check('C', 'Low declares aggregate unordered output',
      low._meta?.degradation?.ordering === 'aggregate_unordered' &&
      low._meta?.degradation?.omits?.length > 0);
    check('C', 'Medium declares source order and comment omission',
      medium._meta?.degradation?.ordering === 'source_ordered' &&
      medium._meta?.degradation?.omits?.includes('comments'));
    check('C', 'verbatim is byte-exact and uncompressed',
      verbatim._meta?.degradation === null &&
      verbatim._meta?.byte_exact?.includes('document') &&
      verbatim.content?.[0]?.text === fs.readFileSync(files.modern, 'utf8'));

    console.log('\n[Case D] Component to Template identity and reverse linkage');
    for (const file of [files.child, files.duplicateA, files.duplicateB,
      files.parent, files.inline, files.missingOwner]) {
      await publish(client, root, file);
    }
    const templateId = canonical(files.external);
    const parentForward = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'parent-forward',
    );
    check('D', 'external HasTemplate uses canonical physical identity',
      Boolean(edge(parentForward, 'HasTemplate', 'Component', 'ParentComponent',
        'Template', templateId)));
    const templateReverse = await query(
      client, root, 'reverse_edges', 'Template', templateId, 'template-reverse',
    );
    check('D', 'reverse Template query returns owning Component',
      Boolean(edge(templateReverse, 'HasTemplate', 'Component', 'ParentComponent',
        'Template', templateId)));
    const inlineForward = await query(
      client, root, 'forward_edges', 'Component', 'InlineParent', 'inline-forward',
    );
    check('D', 'inline Template identity is deterministic and Component-scoped',
      Boolean(edge(inlineForward, 'HasTemplate', 'Component', 'InlineParent',
        'Template', inlineIdentity(files.inline, 'InlineParent'))));

    console.log('\n[Case E] occurrence evidence and additive typed refinement');
    const templateForward = await query(
      client, root, 'forward_edges', 'Template', templateId, 'template-forward',
    );
    const neutral = relationEdges(templateForward, 'UsesSelector')
      .filter((item) => item.object?.name === 'child-widget');
    const spans = openingSpans(externalSource, '<child-widget');
    check('E', 'repeated selector uses remain distinct neutral occurrences',
      neutral.length === 2 && neutral[0].occurrence !== neutral[1].occurrence);
    check('E', 'first occurrence retains span and controlling condition',
      occurrenceMatches(neutral[0], spans[0], 'Modern', 'If', 'showFirst'));
    check('E', 'second occurrence retains span and controlling condition',
      occurrenceMatches(neutral[1], spans[1], 'Modern', 'If', 'showSecond'));
    const parentTyped = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'parent-typed',
    );
    const typed = relationEdges(parentTyped, 'UsesComponent');
    check('E', 'unique child produces one typed result per occurrence',
      typed.filter((item) => item.object?.name === 'ChildComponent').length === 2);
    check('E', 'typed refinement does not replace neutral evidence',
      neutral.length === 2);

    console.log('\n[Case F] reverse occurrence queries');
    const reverseNeutral = await query(
      client, root, 'reverse_edges', 'SelectorRef', 'child-widget', 'reverse-neutral',
    );
    const reverseTyped = await query(
      client, root, 'reverse_edges', 'Component', 'ChildComponent', 'reverse-typed',
    );
    check('F', 'reverse SelectorRef retains both source occurrences',
      relationEdges(reverseNeutral, 'UsesSelector').length >= 2);
    check('F', 'reverse Component returns both typed occurrences',
      relationEdges(reverseTyped, 'UsesComponent')
        .filter((item) => item.subject?.name === 'ParentComponent').length === 2);
    check('F', 'reverse Component remains a lower-bound answer',
      reverseTyped.coverage?.source_complete === false &&
      reverseTyped.coverage?.omitted_possible === true);

    console.log('\n[Case G] unresolved, ambiguous, and missing-template diagnostics');
    check('G', 'unresolved selector remains neutral and explicit',
      diagnosticCodes(templateForward).includes('selector_unresolved') &&
      relationEdges(templateForward, 'UsesSelector')
        .some((item) => item.object?.name === 'missing-widget'));
    check('G', 'ambiguous selector remains neutral and explicit',
      diagnosticCodes(templateForward).includes('selector_ambiguous') &&
      !typed.some((item) => ['DuplicateA', 'DuplicateB'].includes(item.object?.name)));
    const missing = await query(
      client, root, 'forward_edges', 'Component', 'MissingOwner', 'missing-template',
    );
    check('G', 'missing linked source retains HasTemplate evidence',
      Boolean(edge(missing, 'HasTemplate', 'Component', 'MissingOwner',
        'Template', canonical(files.missingTemplate))));
    check('G', 'missing linked source reports partiality',
      diagnosticCodes(missing).includes('linked_template_missing') &&
      missing.coverage?.source_complete === false &&
      missing.coverage?.omitted_possible === true);

    console.log('\n[Case H] exact capability and completeness');
    check('H', 'readable Template establishes exact capability',
      templateForward.coverage?.capability_established === true);
    check('H', 'complete Template discovery is reported separately',
      templateForward.coverage?.source_complete === true &&
      templateForward.coverage?.omitted_possible === false &&
      templateForward.coverage?.result_semantics === 'complete_for_authorized_scope');
    check('H', 'successful Angular query has no unsound alternative hint',
      templateForward.coverage?.alternative_query === undefined);

    console.log('\n[Case I] refresh, replacement, selector change, delete, and restore');
    fs.writeFileSync(files.external, '<new-widget></new-widget>');
    await refresh(client, root, 'refresh-replace');
    const replaced = await query(
      client, root, 'forward_edges', 'Template', templateId, 'replaced-template',
    );
    check('I', 'refresh retracts old selector occurrences',
      relationEdges(replaced, 'UsesSelector').length === 1 &&
      relationEdges(replaced, 'UsesSelector')[0]?.object?.name === 'new-widget');
    check('I', 'new unresolved selector is not typed',
      diagnosticCodes(replaced).includes('selector_unresolved'));
    writeComponent(files.child, 'ChildComponent', 'new-widget', "template: ''");
    await publish(client, root, files.child);
    const retyped = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'retyped-parent',
    );
    check('I', 'current child selector restores typed refinement',
      relationEdges(retyped, 'UsesComponent')
        .some((item) => item.object?.name === 'ChildComponent'));
    fs.rmSync(files.external);
    await refresh(client, root, 'refresh-delete');
    const deleted = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'deleted-template',
    );
    check('I', 'deleted external source removes occurrence-derived edges',
      relationEdges(deleted, 'UsesComponent').length === 0);
    check('I', 'deleted source retains written link and reports missing',
      Boolean(edge(deleted, 'HasTemplate', 'Component', 'ParentComponent',
        'Template', templateId)) &&
      diagnosticCodes(deleted).includes('linked_template_missing'));
    fs.mkdirSync(path.dirname(files.external), { recursive: true });
    fs.writeFileSync(files.external, '<new-widget></new-widget>');
    await refresh(client, root, 'refresh-restore');
    const restored = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'restored-template',
    );
    check('I', 'restored source regenerates current typed evidence',
      relationEdges(restored, 'UsesComponent')
        .some((item) => item.object?.name === 'ChildComponent'));

    console.log('\n[Case J] repeated publication and query stability');
    await publish(client, root, files.parent);
    const repeated = await query(
      client, root, 'forward_edges', 'Component', 'ParentComponent', 'repeat-parent',
    );
    const digest = relationEdges(repeated, 'UsesComponent').map(JSON.stringify);
    check('J', 'republication does not duplicate HasTemplate',
      relationEdges(repeated, 'HasTemplate').length === 1);
    check('J', 'query-time typed occurrences are duplicate-free',
      new Set(digest).size === digest.length);
  } finally {
    await client.close();
    captures._server_stderr = client.stderr;
    fs.rmSync(root, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
  }
}

async function main() {
  console.log('binary: ' + binary);
  if (!fs.existsSync(binary)) {
    console.error('Binary not found. Build it first with: cargo build --all-features --bin clean-ctx');
    process.exit(2);
  }
  const stale = binaryIsStale();
  console.log('binary mtime: ' + fs.statSync(binary).mtime.toISOString());
  if (stale) console.log('  !! STALE BINARY: rebuild before treating this run as evidence.');
  await run();
  if (stale) failures.unshift('selected binary predates Angular Phase 1-6 source markers');

  const evidenceDir = path.join(
    repoRoot, 'target', 'live-acceptance', 'angular-template-linkage',
  );
  fs.mkdirSync(evidenceDir, { recursive: true });
  const evidenceFile = path.join(evidenceDir, 'responses.json');
  fs.writeFileSync(
    evidenceFile,
    JSON.stringify({ binary, stale, failures, captures }, null, 2) + '\n',
  );

  console.log('\n-- summary --');
  console.log('evidence: ' + evidenceFile);
  if (!failures.length) {
    console.log('ALL ANGULAR TEMPLATE LIVE MCP CHECKS PASSED');
  } else {
    console.log(failures.length + ' LIVE MCP CHECK(S) FAILED:');
    for (const failure of failures) console.log('  - ' + failure);
    process.exitCode = 1;
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
