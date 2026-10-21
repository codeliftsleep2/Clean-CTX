// Agent MaxHeadRoom: bounded Angular module lifecycle oracles.
import fs from 'node:fs';
import path from 'node:path';
export const leaf = (kind = 'Directive') => `import { ${kind} } from '@angular/core'; @${kind}({selector:'[shared]',name:'shared'}) export class Shared {}`;
export const array = "import { Shared } from './leaf'; export const SHARED = [Shared];";
export const target = () => ({ type: 'forward_edges', domain: 'angular', entity_type: 'Module', name: 'AppModule' });
export function fixture(spread = false, field = 'declarations') {
  return { 'leaf.ts': leaf(), 'shared.ts': array, 'unrelated.ts': 'export class Unrelated {}',
    'app.module.ts': `import { NgModule } from '@angular/core'; ${spread ? "import { SHARED } from './shared';" : "import { Shared } from './leaf';"} @NgModule({${field}:[${spread ? '...SHARED' : 'Shared'}]}) export class AppModule {}` };
}
export function write(root, files) {
  for (const [name, text] of Object.entries(files)) {
    const file = path.resolve(root, name);
    if (!file.startsWith(path.resolve(root) + path.sep)) throw new Error(`outside fixture: ${name}`);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text, 'utf8');
  }
}
export function facts(result, expected, relation, check, root, label) {
  check(`${label}/edges`, Array.isArray(result.edges), result);
  const edges = result.edges.filter((edge) => edge.relation === relation);
  const actual = edges.map((edge) => [edge.object?.name, edge.object?.entity_type]).sort();
  check(`${label}/membership`, JSON.stringify(actual) === JSON.stringify([...expected].sort()), { actual, expected });
  for (const edge of edges) {
    check(`${label}/identity`, edge.subject?.domain === 'angular' && edge.subject.entity_type === 'Module'
      && edge.subject.name === 'AppModule' && edge.object?.domain === 'angular', edge);
    // This producer's established provenance is the asserting module, not a
    // canonical declaration-file identity. Do not invent a stronger oracle.
    const normalize = (file) => path.resolve(file).replaceAll('\\', '/').toLowerCase();
    check(`${label}/asserting-owner`, typeof edge.subject.file === 'string'
      && normalize(edge.subject.file) === normalize(path.join(root, 'app.module.ts')), edge);
  }
  check(`${label}/lower-bound`, result.coverage?.source_complete === false
    && result.coverage?.omitted_possible === true && result.coverage?.result_semantics === 'lower_bound', result.coverage);
  return JSON.stringify(actual);
}
const positive = [['Shared', 'Directive']];
const changed = [['Shared', 'Pipe']];
export function scenarios() {
  const cases = [];
  for (const spread of [false, true]) {
    const family = spread ? 'array' : 'direct';
    for (const order of ['control', 'cold', 'contributor-query', 'unrelated', 'repeat']) {
      cases.push({ label: `${family}-${order}`, files: fixture(spread), run: async (o) => {
        if (order === 'control') for (const file of ['leaf.ts', 'shared.ts']) await o.file(file);
        if (order === 'contributor-query') await o.query({ type: 'forward_edges', domain: 'angular', entity_type: 'Directive', name: 'Shared' });
        if (order === 'unrelated') await o.file('unrelated.ts');
        const actual = await o.edges(positive);
        o.converge(family, order, actual);
        await o.edges(positive);
      } });
    }
    for (const mode of ['mutation', 'deletion']) cases.push({ label: `${family}-${mode}`, files: fixture(spread), run: async (o) => {
      await o.edges(positive);
      o.mutate('leaf.ts', mode === 'deletion' ? null : leaf('Pipe'));
      await o.edges(mode === 'deletion' ? [] : changed);
      o.mutate('leaf.ts', leaf('Pipe'));
      await o.edges(changed);
      o.mutate('leaf.ts', leaf());
      await o.edges(positive);
    } });
    for (const reverse of [false, true]) cases.push({ label: `${family}-batch-${reverse}`, files: fixture(spread), run: async (o) => {
      await o.edges(positive);
      o.mutate('leaf.ts', leaf('Pipe'));
      const module = { id: 'module', ...target() };
      const contributor = { id: 'contributor', type: 'entities_in_file', file_path: 'leaf.ts', fidelity: 'high' };
      const items = reverse ? [contributor, module] : [module, contributor];
      const result = await o.query({ queries: items });
      o.check('batch-order', JSON.stringify(result.results?.map((item) => item.id)) === JSON.stringify(items.map((item) => item.id)), result);
      o.check('batch-status', result.results?.every((item) => item.status === 'ok'), result);
      o.facts(result.results.find((item) => item.id === 'module').result, changed);
      await o.edges(changed);
    } });
    cases.push({ label: `${family}-concurrent`, files: fixture(spread), run: async (o) => {
      const results = await Promise.all([o.query(target()), o.query(target())]);
      for (const result of results) o.facts(result, positive);
      await o.edges(positive);
    } });
  }
  for (const [field, relation] of [['imports', 'ImportsModule'], ['exports', 'ExportsFromModule']]) {
    cases.push({ label: field, files: fixture(false, field), relation, run: async (o) => {
      await o.edges(positive); o.mutate('leaf.ts', leaf('Pipe')); await o.edges(changed);
    } });
  }
  for (const deletion of [false, true]) cases.push({ label: `array-membership-${deletion}`, files: fixture(true), run: async (o) => {
    await o.edges(positive); o.mutate('shared.ts', deletion ? null : 'export const SHARED = [];');
    await o.edges([]); o.mutate('shared.ts', array); await o.edges(positive);
  } });
  const aliased = fixture();
  aliased['app.module.ts'] = aliased['app.module.ts'].replace('{ Shared }', '{ Shared as Selected }').replace('[Shared]', '[Selected]');
  cases.push({ label: 'alias', files: aliased, run: async (o) => { await o.edges([['Selected', 'Directive']]); } });
  const missing = fixture(); delete missing['leaf.ts'];
  cases.push({ label: 'missing-appearance', files: missing, run: async (o) => {
    await o.edges([]); o.mutate('leaf.ts', leaf()); await o.edges(positive);
  } });
  const typeOnly = fixture();
  typeOnly['app.module.ts'] = typeOnly['app.module.ts'].replace('import { Shared }', 'import type { Shared }');
  cases.push({ label: 'type-only', files: typeOnly, run: async (o) => { await o.edges([]); } });
  return cases;
}
