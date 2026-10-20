// Agent MaxHeadRoom. Synthetic #155 scenarios; no explicit dependency publication.
import fs from 'node:fs';
import path from 'node:path';

export const roles = {
  Controller: ['Microsoft.AspNetCore.Mvc.ControllerBase',
    '[global::Microsoft.AspNetCore.Mvc.HttpGetAttribute] public int Get() { return 1; }', 'ControllerAction', 'Get'],
  Hub: ['Microsoft.AspNetCore.SignalR.Hub', 'public void Broadcast() {}', 'HubMethodTargets', 'Broadcast'],
  MapperProfile: ['AutoMapper.Profile', 'public Target() { CreateMap<Source, Destination>(); }', 'MapsFrom', 'Source'],
  DbContext: ['Microsoft.EntityFrameworkCore.DbContext',
    'public global::Microsoft.EntityFrameworkCore.DbSet<Row> Rows { get; set; }', 'HasEntity', 'Row'],
};

export function fixture(role = 'Hub') {
  const [base, body] = roles[role];
  return {
    'Target.cs': `public class Target : Bridge { ${body} }`,
    'Bridge.cs': 'public class Bridge : Foundation {}',
    'Foundation.cs': `public class Foundation : global::${base} {}`,
    'Other.cs': 'public class Other : PlainBase {}',
    'Plain.cs': 'public class PlainBase {}',
    'Unrelated.ts': 'export class Unrelated {}',
  };
}

export function write(root, files) {
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source, 'utf8');
  }
}

export function target(role, surface) {
  return surface === 'entities_in_file'
    ? { type: surface, file_path: 'Target.cs', fidelity: 'high' }
    : { type: surface, domain: 'dotnet', entity_type: role, name: 'Target' };
}

// Validate the coverage schema keywords used by the registered contract. Unknown
// assertion keywords fail rather than silently granting schema conformance.
function valid(value, schema) {
  const supported = new Set(['type', 'description', 'properties', 'required', 'additionalProperties',
    'items', 'enum', 'const', 'oneOf', 'minimum', 'maxItems', 'uniqueItems']);
  for (const key of Object.keys(schema)) if (!supported.has(key)) throw new Error(`unsupported schema keyword: ${key}`);
  if (schema.oneOf && schema.oneOf.filter((part) => valid(value, part)).length !== 1) return false;
  if (schema.type) {
    const type = Array.isArray(value) ? 'array' : value === null ? 'null' : typeof value;
    if (schema.type === 'integer' ? !Number.isInteger(value) : schema.type !== type) return false;
  }
  if (schema.enum && !schema.enum.includes(value)) return false;
  if ('const' in schema && value !== schema.const) return false;
  if (schema.minimum !== undefined && value < schema.minimum) return false;
  if (Array.isArray(value)) {
    if (schema.maxItems !== undefined && value.length > schema.maxItems) return false;
    if (schema.uniqueItems && new Set(value.map(JSON.stringify)).size !== value.length) return false;
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
  constructor(client, root, coverageSchema, check) {
    Object.assign(this, { client, root, coverageSchema, check });
  }
  async query(args, label) {
    const response = await this.client.call('workspace_query', {
      ...args, workspaceRoot: this.root,
    }, label);
    const result = response.structuredContent;
    if (!result || typeof result !== 'object') throw new Error(`${label}: missing structuredContent`);
    if (!args.queries) this.coverage(result.coverage, label);
    return result;
  }
  coverage(coverage, label, complete = true) {
    this.check(`${label}/registered-coverage`, !!coverage && valid(coverage, this.coverageSchema), coverage);
    this.check(`${label}/dependency-proof`, coverage?.dependency_proof?.status === (complete ? 'complete' : 'unproven'), coverage);
    if (complete) this.check(`${label}/proof-generation`, typeof coverage?.dependency_proof?.generation === 'string'
      && coverage.dependency_proof.generation.length > 0, coverage);
  }
  async provide(label, batch = false, reverse = false) {
    let items = [
      { id: 'target', filePath: path.join(this.root, 'Target.cs'), fidelity: 'high' },
      { id: 'other', filePath: path.join(this.root, 'Other.cs'), fidelity: 'high' },
    ];
    if (reverse) items.reverse();
    const result = await this.client.call('provide_code_context', {
      workspaceRoot: this.root,
      ...(batch ? { files: items, responseMode: 'structured' }
        : { filePath: items[0].filePath, fidelity: 'high' }),
    }, label);
    if (!batch) this.coverage(result._meta?.semantic_coverage, label);
    else {
      const results = result.structuredContent?.results;
      if (!Array.isArray(results)) throw new Error(`${label}: missing batch results`);
      this.check(`${label}/order`, JSON.stringify(results.map((item) => item.id)) === JSON.stringify(items.map((item) => item.id)));
      this.check(`${label}/count`, results.length === items.length);
      for (const item of results) {
        this.check(`${label}/${item.id}/success`, item.status === 'ok', item);
        this.coverage(item.meta?.semantic_coverage, `${label}/${item.id}`);
      }
      // Proof generations identify owner-specific requirements, not a shared
      // publication counter. Compare each owner with its own final observation.
      for (const item of results) {
        const current = await this.client.call('provide_code_context', {
          workspaceRoot: this.root, filePath: items.find((input) => input.id === item.id).filePath,
          fidelity: 'high',
        }, `${label}/${item.id}/final-observation`);
        this.coverage(current._meta?.semantic_coverage, `${label}/${item.id}/final-observation`);
        this.check(`${label}/${item.id}/proof-still-current`,
          item.meta.semantic_coverage.dependency_proof.generation
            === current._meta.semantic_coverage.dependency_proof.generation);
      }
    }
    return result;
  }
  facts(result, surface, role, label, present = true) {
    const key = surface === 'entities_in_file' ? 'entities' : 'edges';
    if (!Array.isArray(result[key])) throw new Error(`${label}: missing ${key}`);
    const items = result[key];
    const matching = surface === 'entities_in_file'
      ? items.filter((item) => item.domain === 'dotnet' && item.entity_type === role && item.name === 'Target')
      : items.filter((item) => item.relation === roles[role][2]
        && item.subject?.name === 'Target' && item.subject?.entity_type === role);
    this.check(`${label}/role-or-relation`, present ? matching.length > 0 : matching.length === 0, matching);
    const canonical = (file) => {
      if (typeof file !== 'string') return null;
      const resolved = path.resolve(file).replace(/^\\\\\?\\/, '').replaceAll('\\', '/');
      return process.platform === 'win32' ? resolved.toLowerCase() : resolved;
    };
    const owner = canonical(path.join(this.root, 'Target.cs'));
    for (const item of matching) this.check(`${label}/owner-provenance`,
      canonical(surface === 'entities_in_file' ? item.file : item.subject?.file) === owner, item);
    if (surface === 'entities_in_file') this.check(`${label}/source-survives`,
      items.some((item) => item.domain === 'builtin' && item.entity_type === 'Class'
        && item.name === 'Target' && canonical(item.file) === owner), items);
    if (surface !== 'entities_in_file' && present) this.check(`${label}/payload`,
      matching.some((item) => item.object?.name === roles[role][3]), matching);
    if (surface === 'entities_in_file') this.check(`${label}/exact-file`,
      result.coverage.source_complete === true && result.coverage.result_semantics === 'exact', result.coverage);
    const normalized = items.map((item) => JSON.stringify(item)).sort();
    this.check(`${label}/duplicate-free`, new Set(normalized).size === normalized.length);
    return normalized;
  }
  async convergence(role, surface) {
    const cold = await this.query(target(role, surface), 'cold');
    const baseline = this.facts(cold, surface, role, 'cold');
    const warm = await this.query(target(role, surface), 'warm');
    this.check('cold-warm/facts', JSON.stringify(baseline) === JSON.stringify(this.facts(warm, surface, role, 'warm')));
    this.check('cold-warm/proof', cold.coverage.dependency_proof.generation === warm.coverage.dependency_proof.generation);
  }
  async mixed(reverse) {
    const items = [
      { id: 'edges', ...target('Hub', 'forward_edges') },
      { id: 'file', ...target('Hub', 'entities_in_file') },
      { id: 'other', type: 'entities_in_file', file_path: 'Other.cs', fidelity: 'high' },
      { id: 'unrelated', type: 'find_entities', name: 'Unrelated' },
    ];
    if (reverse) items.reverse();
    const result = await this.query({ queries: items }, 'mixed');
    if (!Array.isArray(result.results)) throw new Error('mixed: missing results');
    this.check('mixed/order', JSON.stringify(result.results.map((item) => item.id)) === JSON.stringify(items.map((item) => item.id)));
    const proofs = [];
    for (const item of result.results) {
      this.check(`mixed/${item.id}/success`, item.status === 'ok', item);
      if (item.id === 'unrelated') continue;
      this.coverage(item.result?.coverage, `mixed/${item.id}`);
      proofs.push({ id: item.id, generation: item.result?.coverage?.dependency_proof?.generation });
      if (item.id === 'file' || item.id === 'edges') this.facts(item.result,
        item.id === 'file' ? 'entities_in_file' : 'forward_edges', 'Hub', `mixed/${item.id}`);
    }
    for (const proof of proofs) {
      const input = items.find((item) => item.id === proof.id);
      const current = await this.query(input, `mixed/${proof.id}/final-observation`);
      this.check(`mixed/${proof.id}/proof-still-current`,
        proof.generation === current.coverage.dependency_proof.generation);
    }
  }
}

export function attributeFixture(kind) {
  return {
    'Target.cs': kind === 'class-shadow'
      ? 'using Microsoft.AspNetCore.Mvc; namespace Demo; [ApiController] public class Target {}'
      : `using Microsoft.AspNetCore.Mvc; ${kind === 'ambiguity' ? '' : 'namespace Demo;'} public class Target : ControllerBase { [HttpGet] public int Get() { return 1; } }`,
    'Shadow.cs': kind === 'class-shadow' ? 'namespace Demo; public class ApiControllerAttribute {}'
      : `namespace ${kind === 'ambiguity' ? 'Microsoft.AspNetCore.Mvc' : 'Demo'}; public class HttpGetAttribute {}`,
  };
}

export async function attributes(o, kind) {
  const args = target('Controller', 'entities_in_file');
  // Ambiguity deliberately requires an unproven certificate, so bypass the
  // default complete-proof assertion while retaining all transport checks.
  const get = async (label) => {
    const raw = await o.client.call('workspace_query', { ...args, workspaceRoot: o.root }, label);
    const result = raw.structuredContent;
    if (!Array.isArray(result?.entities)) throw new Error(`${label}: missing entities`);
    o.coverage(result.coverage, label, kind !== 'ambiguity');
    return result;
  };
  const cold = await get('attribute/cold');
  const warm = await get('attribute/warm');
  o.check('attribute/cold-warm', JSON.stringify(cold.entities) === JSON.stringify(warm.entities));
  if (kind === 'ambiguity') {
    o.check('attribute/truthful-partial', cold.coverage.status === 'semantic_dependencies_unresolved'
      && cold.coverage.source_complete === false && cold.coverage.omitted_possible === true
      && Array.isArray(cold.coverage.reasons) && cold.coverage.reasons.length > 0, cold.coverage);
  } else if (kind === 'class-shadow') {
    o.check('attribute/no-controller', !cold.entities.some((item) => item.domain === 'dotnet' && item.entity_type === 'Controller'));
  } else {
    const edges = await o.query(target('Controller', 'forward_edges'), 'attribute/edges');
    if (!Array.isArray(edges.edges)) throw new Error('attribute: missing edges');
    o.check('attribute/no-http-authority', !edges.edges.some((edge) => edge.relation === 'ControllerAction'));
  }
}
