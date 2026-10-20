import path from 'node:path';

function canonical(file) {
  const value = path.resolve(file).replace(/^\\\\\?\\/, '').replaceAll('\\', '/');
  return process.platform === 'win32' ? value.toLowerCase() : value;
}

export function structured(result, label) {
  if (!result.structuredContent || typeof result.structuredContent !== 'object') {
    throw new Error(`${label}: missing structuredContent`);
  }
  return result.structuredContent;
}

export async function query(client, root, args, label) {
  return structured(await client.call('workspace_query', {
    workspaceRoot: root, fidelity: 'high', ...args,
  }, label), label);
}

export async function publishAll(client, root, files, reverse, phase) {
  const names = Object.keys(files).sort();
  if (reverse) names.reverse();
  for (const file of names) {
    await client.call('provide_code_context', {
      workspaceRoot: root, filePath: path.join(root, file), fidelity: 'high',
    }, `${phase}/publish/${file}`);
  }
}

export async function refresh(client, root, label) {
  const result = structured(await client.call('refresh_workspace', {
    workspaceRoot: root,
  }, label), label);
  if (result.refreshed !== true) throw new Error(`${label}: refresh not acknowledged`);
}

export async function observe(client, root, item, check, phase, present = true) {
  const label = `${phase}/${item.name}`;
  const identity = { domain: 'dotnet', entity_type: item.role, name: item.name };
  const roles = await query(client, root, { type: 'find_entities', ...identity }, `${label}/role`);
  if (!Array.isArray(roles.entities)) throw new Error(`${label}: missing entities array`);
  const occurrences = roles.entities.filter((entity) => entity.domain === identity.domain
    && entity.entity_type === item.role && entity.name === item.name);
  check(`${label}/role-count`, occurrences.length === (present ? 1 : 0), occurrences);
  const expectedFile = canonical(path.join(root, item.file));
  for (const entity of occurrences) {
    check(`${label}/role-file`, typeof entity.file === 'string'
      && canonical(entity.file) === expectedFile, entity);
  }

  const result = await query(client, root, { type: 'forward_edges', ...identity }, `${label}/edges`);
  if (!Array.isArray(result.edges)) throw new Error(`${label}: missing edges array`);
  const relevant = result.edges.filter((edge) => edge.relation === item.relation
    && edge.subject?.domain === 'dotnet' && edge.subject?.entity_type === item.role
    && edge.subject?.name === item.name);
  const targets = relevant.map((edge) => edge.object?.name).sort();
  const expected = present ? [...item.targets].sort() : [];
  check(`${label}/exact-targets`, JSON.stringify(targets) === JSON.stringify(expected), {
    actual: targets, expected,
  });
  for (const edge of relevant) {
    check(`${label}/typed-object`, edge.object?.domain === 'dotnet'
      && edge.object?.entity_type === item.targetType, edge);
    check(`${label}/subject-file`, typeof edge.subject?.file === 'string'
      && canonical(edge.subject.file) === expectedFile, edge);
    check(`${label}/object-file`, typeof edge.object?.file === 'string'
      && canonical(edge.object.file) === expectedFile, edge);
  }
  // Preserve full relationship facts, including unrelated relations, for history
  // comparison. Only physical root prefixes differ across fresh sessions.
  const portable = (entity) => ({
    domain: entity?.domain, entity_type: entity?.entity_type, name: entity?.name,
    file: typeof entity?.file === 'string'
      ? path.posix.relative(canonical(root), canonical(entity.file)) : entity?.file,
  });
  const facts = result.edges.map((edge) => JSON.stringify({
    relation: edge.relation, subject: portable(edge.subject), object: portable(edge.object),
  })).sort();
  check(`${label}/duplicate-free`, new Set(facts).size === facts.length, facts);
  return { roles: occurrences.map(portable).map(JSON.stringify).sort(), facts };
}

export async function snapshot(client, root, cases, check, phase) {
  const value = {};
  for (const [index, item] of cases.entries()) {
    value[item.name] = await observe(client, root, item, check, phase);
    if ((index + 1) % 10 === 0 || index === cases.length - 1) {
      console.log(`  ${phase}: ${index + 1}/${cases.length} identities`);
    }
  }
  return value;
}

export async function negative(client, root, name, check, phase) {
  const roles = await query(client, root, {
    type: 'find_entities', domain: 'dotnet', entity_type: 'Controller', name,
  }, `${phase}/lookalike-role`);
  if (!Array.isArray(roles.entities)) throw new Error('lookalike: missing entities array');
  check(`${phase}/lookalike-not-controller`, !roles.entities.some((entity) =>
    entity.domain === 'dotnet' && entity.entity_type === 'Controller' && entity.name === name), roles.entities);
  const source = await query(client, root, {
    type: 'find_entities', domain: 'builtin', entity_type: 'Class', name,
  }, `${phase}/lookalike-source`);
  if (!Array.isArray(source.entities)) throw new Error('lookalike: missing source entities');
  check(`${phase}/lookalike-source-preserved`, source.entities.some((entity) =>
    entity.domain === 'builtin' && entity.entity_type === 'Class' && entity.name === name), source);
}
