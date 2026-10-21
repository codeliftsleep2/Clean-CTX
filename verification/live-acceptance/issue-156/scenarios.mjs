// Agent MaxHeadRoom: EF source authority oracles; no manual index publication.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

export const models = {
  direct: 'builder.Entity<App.First>();',
  explicit: 'builder.ApplyConfiguration(new App.Mapping());',
  assembly: 'builder.ApplyConfigurationsFromAssembly(typeof(Target).Assembly);',
};
export const rows = 'namespace App; public class First {} public class Second {}';
export const mapping = (entity = 'First') =>
  `using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<${entity}> {}`;
export const target = () => ({ type: 'forward_edges', domain: 'dotnet', entity_type: 'DbContext', name: 'Target' });
export function fixture(model = models.explicit) {
  return {
    'App.csproj': '<Project Sdk="Microsoft.NET.Sdk" />',
    'Rows.cs': rows,
    'Mapping.cs': mapping(),
    'Bridge.cs': 'namespace App; public class Bridge : Microsoft.EntityFrameworkCore.DbContext {}',
    'Target.cs': `using Microsoft.EntityFrameworkCore; namespace App; public class Target : Bridge {
 protected override void OnModelCreating(ModelBuilder builder) { ${model} }
}`,
    'Lookalike.cs': 'namespace Other; public class First {} public class Second {} public class Mapping {}',
    'Unrelated.cs': 'public class Unrelated {}',
  };
}
export function write(root, files) {
  for (const [name, text] of Object.entries(files)) {
    const file = path.resolve(root, name);
    if (!file.startsWith(path.resolve(root) + path.sep)) throw new Error(`fixture outside root: ${name}`);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text, 'utf8');
  }
}
const canonical = (file) => {
  const resolved = fs.realpathSync.native(path.resolve(file)).replace(/^\\\\\?\\/, '').replaceAll('\\', '/');
  return process.platform === 'win32' ? resolved.toLowerCase() : resolved;
};
const digest = (file) => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

// Deliberately bounded to the coverage schema used by the existing #155 driver.
// Unknown keywords fail the harness instead of silently weakening validation.
function valid(value, schema) {
  const known = new Set(['type', 'description', 'properties', 'required', 'additionalProperties',
    'items', 'enum', 'const', 'oneOf', 'minimum', 'maxItems', 'uniqueItems']);
  for (const key of Object.keys(schema)) if (!known.has(key)) throw new Error(`unsupported schema keyword: ${key}`);
  if (schema.oneOf && schema.oneOf.filter((item) => valid(value, item)).length !== 1) return false;
  if (schema.type) {
    const type = Array.isArray(value) ? 'array' : value === null ? 'null' : typeof value;
    if (schema.type === 'integer' ? !Number.isInteger(value) : schema.type !== type) return false;
  }
  if (schema.enum && !schema.enum.includes(value)) return false;
  if ('const' in schema && value !== schema.const) return false;
  if (schema.minimum !== undefined && value < schema.minimum) return false;
  if (Array.isArray(value)) {
    if (schema.maxItems !== undefined && value.length > schema.maxItems) return false;
    if (schema.uniqueItems && new Set(value.map((item) => JSON.stringify(item))).size !== value.length) return false;
    if (schema.items && !value.every((item) => valid(item, schema.items))) return false;
  }
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    if (schema.required?.some((key) => !(key in value))) return false;
    for (const [key, item] of Object.entries(value)) {
      if (schema.properties?.[key] && !valid(item, schema.properties[key])) return false;
      if (schema.additionalProperties === false && !schema.properties?.[key]) return false;
    }
  }
  return true;
}

export class Oracles {
  constructor(client, root, schema, check, mutate, observe) {
    Object.assign(this, { client, root, schema, check, mutate, observe });
    this.contextHash = digest(path.join(root, 'Target.cs'));
  }
  async query(args, label) {
    const started = performance.now();
    const response = await this.client.call('workspace_query', { workspaceRoot: this.root, ...args }, label);
    this.check(`${label}/structured`, !!response.structuredContent && typeof response.structuredContent === 'object');
    const result = response.structuredContent;
    this.observe({ label, elapsedMs: performance.now() - started, type: args.type ?? 'batch',
      count: result.count, coverage: result.coverage, discovery: result.discovery });
    return result;
  }
  async edges(label, expected = ['First'], complete = true) {
    const result = await this.query(target(), label);
    this.facts(result, expected, label, complete);
    return this.signature(result);
  }
  facts(result, expected, label, complete = true) {
    this.check(`${label}/edges-array`, Array.isArray(result.edges), result);
    const edges = result.edges.filter((edge) => edge.relation === 'HasEntity');
    this.check(`${label}/expected-positive-facts`, JSON.stringify(edges.map((edge) => edge.object?.name).sort())
      === JSON.stringify([...expected].sort()), result);
    for (const edge of edges) {
      this.check(`${label}/${edge.object.name}/identity`, edge.subject?.name === 'Target'
        && edge.subject?.domain === 'dotnet' && edge.subject?.entity_type === 'DbContext'
        && edge.object.domain === 'dotnet' && edge.object.entity_type === 'Entity', edge);
      this.check(`${label}/${edge.object.name}/entity-file`, typeof edge.object.file === 'string'
        && canonical(edge.object.file) === canonical(path.join(this.root, 'Rows.cs')), edge);
      this.check(`${label}/${edge.object.name}/context-file`, typeof edge.subject.file === 'string'
        && canonical(edge.subject.file) === canonical(path.join(this.root, 'Target.cs')), edge);
    }
    const coverage = result.coverage;
    this.check(`${label}/coverage-schema`, !!coverage && valid(coverage, this.schema), coverage);
    this.check(`${label}/lower-bound`, coverage.source_complete === false
      && coverage.result_semantics === 'lower_bound' && coverage.omitted_possible === true, coverage);
    this.check(`${label}/dependency-proof`, coverage.dependency_proof?.status === (complete ? 'complete' : 'unproven'), coverage);
    if (complete) this.check(`${label}/generation`, typeof coverage.dependency_proof?.generation === 'string'
      && coverage.dependency_proof.generation.length > 0, coverage);
    this.check(`${label}/context-unchanged`, this.contextHash === digest(path.join(this.root, 'Target.cs')));
  }
  signature(result) {
    return JSON.stringify(result.edges.filter((edge) => edge.relation === 'HasEntity').map((edge) => ({
      relation: edge.relation, subject: edge.subject.name, object: edge.object.name,
      subjectFile: path.relative(canonical(this.root), canonical(edge.subject.file)).replaceAll('\\', '/'),
      objectFile: path.relative(canonical(this.root), canonical(edge.object.file)).replaceAll('\\', '/'),
    })).sort((left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right))));
  }
  async warm(order) {
    if (order === 'dependency-files') for (const file of ['Lookalike.cs', 'Rows.cs', 'Mapping.cs']) {
      await this.query({ type: 'entities_in_file', file_path: file, fidelity: 'high' }, `warm/${file}`);
    }
    if (order === 'dependency-names') for (const name of ['First', 'Mapping']) {
      await this.query({ type: 'find_entities', name }, `warm/${name}`);
    }
    if (order === 'unrelated') await this.query({ type: 'find_entities', name: 'Unrelated' }, 'warm/unrelated');
    if (order === 'repeat') await this.edges('repeat/first');
  }
  async lifecycle(kind) {
    await this.edges('initial');
    if (kind === 'direct') {
      this.mutate('Rows.cs', null);
      await this.edges('deleted', [], false);
      this.mutate('Rows.cs', rows);
      await this.edges('restored');
      this.mutate('Rows.cs', 'namespace Changed; public class First {} public class Second {}');
      await this.edges('identity-changed', [], false);
      this.mutate('Rows.cs', rows);
      await this.edges('identity-restored');
    } else {
      this.mutate('Mapping.cs', mapping('Second'));
      await this.edges('retargeted', ['Second']);
      this.mutate('Mapping.cs', null);
      await this.edges('configuration-deleted', [], kind === 'assembly');
      this.mutate('Mapping.cs', mapping());
      await this.edges('configuration-restored');
      if (kind === 'assembly') {
        this.mutate('Nested/SecondMapping.cs', 'using Microsoft.EntityFrameworkCore; namespace App; public class SecondMapping : IEntityTypeConfiguration<Second> {}');
        await this.edges('universe-expanded', ['First', 'Second']);
        this.mutate('Nested/Nested.csproj', '<Project />');
        await this.edges('left-project');
        this.mutate('Nested/Nested.csproj', null);
        await this.edges('rejoined-project', ['First', 'Second']);
        this.mutate('Nested/SecondMapping.cs', null);
        await this.edges('universe-contracted');
      }
    }
  }
  async batch(reverse) {
    const items = [{ id: 'target', ...target() },
      { id: 'entity', type: 'entities_in_file', file_path: 'Rows.cs' },
      { id: 'unrelated', type: 'find_entities', name: 'Unrelated' }];
    if (reverse) items.reverse();
    const result = await this.query({ queries: items }, 'batch');
    this.check('batch/count', Array.isArray(result.results) && result.results.length === items.length, result);
    this.check('batch/order', JSON.stringify(result.results.map((item) => item.id)) === JSON.stringify(items.map((item) => item.id)));
    for (const item of result.results) this.check(`batch/${item.id}/ok`, item.status === 'ok', item);
    const answer = result.results.find((item) => item.id === 'target').result;
    this.facts(answer, ['First'], 'batch/target');
    this.check('batch/final-truth', this.signature(answer) === await this.edges('after-batch'));
  }
  async concurrent() {
    const results = await Promise.all([this.query(target(), 'concurrent/first'), this.query(target(), 'concurrent/second')]);
    results.forEach((result, index) => this.facts(result, ['First'], `concurrent/${index}`));
    this.check('concurrent/same-truth', this.signature(results[0]) === this.signature(results[1]));
    this.check('concurrent/warm-truth', this.signature(results[0]) === await this.edges('concurrent/warm'));
  }
}
