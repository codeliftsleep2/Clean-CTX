// Agent MaxHeadRoom: synthetic inject evidence through the registered MCP path.
import path from 'node:path';
export const injectSource = `
import { Injectable as Service, inject as obtain } from '@angular/core';
import * as core from '@angular/core';
import { publicInject, PublicLeft as Left } from './inject-barrel';
import { Dependency as Right } from './token-right';
import type { Dependency as Erased } from './token-right';
import { inject as impostor } from './inject-foreign';
import { Unavailable } from './missing-unrelated';
import { SelectToken } from './constructor-api';
import { CtorToken } from './constructor-token';
import * as tokenNamespace from './token-right';
const prose = 'inject(FakeToken)';
const settings = { inject: 'ordinary property' };
function wrap(value: unknown) { return value; }
@Service() export class InjectOwner {
  first = publicInject(Left);
  repeated = obtain(Left);
  qualified = core.inject(Right);
  symbolic = obtain(MissingService);
  erased = obtain(Erased);
  foreign = impostor(Right);
  member = settings.inject(Right);
  optional = obtain?.(Right);
  nested = wrap(obtain(Right));
  method(obtain:(token:unknown)=>unknown) { return obtain(Right); }
}
@Service() export class CtorOwner {
  constructor(@SelectToken(CtorToken) private readonly dep: Left) {}
  sameToken = obtain(CtorToken);
  method(CtorToken:unknown) { return CtorToken; }
}
@Service() export class TypedCtorOwner {
  constructor(readonly first: Left, right: tokenNamespace.Dependency, repeated: Left) {}
  sameToken = obtain(Left);
  note = 'constructor(fake: Phantom)';
}
`;
export const injectFixture = {
  'di.ts': injectSource,
  'token-left.ts': 'export class Dependency {}',
  'token-right.ts': 'export class Dependency {}',
  'inject-barrel.ts': "export { inject as publicInject } from '@angular/core'; export { Dependency as PublicLeft } from './token-left';",
  'inject-foreign.ts': 'export function inject(value:unknown) {return value;}',
  'constructor-api.ts': "export { Inject as SelectToken } from '@angular/core';",
  'constructor-token.ts': "export { Dependency as CtorToken } from './token-right';",
  'constructor-foreign.ts': 'export function Inject(token:unknown) { return token; }',
  'globals.d.ts': 'declare interface UnrelatedNoise { text: string }',
};
export function injectAssertions({ call, check, root, normalize, hash, fs, name }) {
  let sequence = 0;
  return async (tokens = ['token-left.ts', 'token-right.ts'], missing = 'symbolic') => {
    const label = `${name}/inject/${++sequence}`;
    await call('workspace_query', { type: 'entities_in_file', file_path: path.join(root, 'di.ts'), fidelity: 'high' });
    const response = await call('workspace_query', { type: 'forward_edges', domain: 'angular', entity_type: 'Service', name: 'InjectOwner' });
    const edges = response.structuredContent?.edges;
    check(`${label}/structured`, Array.isArray(edges), response);
    const selected = edges.filter((edge) => edge.relation === 'Injects');
    const actual = selected.map((edge) => [edge.object.name, edge.object.entity_type, normalize(edge.object.file)]).sort();
    const wanted = tokens.map((file) => ['Dependency', 'Service', normalize(path.join(root, file))]);
    if (missing !== 'none') wanted.push(['MissingService', 'Service', normalize(path.join(root, missing === 'symbolic' ? 'di.ts' : 'globals.d.ts'))]);
    wanted.sort();
    check(`${label}/exact-mixed-dependencies`, JSON.stringify(actual) === JSON.stringify(wanted), { actual, wanted });
    check(`${label}/owner`, selected.every((edge) => edge.subject.name === 'InjectOwner'
      && edge.subject.domain === 'angular' && edge.subject.entity_type === 'Service'
      && edge.object.domain === 'angular' && normalize(edge.subject.file) === normalize(path.join(root, 'di.ts'))));
    check(`${label}/canonical-private`, selected.every((edge) => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    check(`${label}/source-unchanged`, hash(fs.readFileSync(path.join(root, 'di.ts'))) === hash(Buffer.from(injectSource)));
  };
}

export function constructorAssertions({ call, check, root, normalize, hash, fs, name }) {
  let sequence = 0;
  return async (overrideTokens = ['token-right.ts'], typedTokens = ['token-left.ts', 'token-right.ts']) => {
    await call('workspace_query', { type: 'entities_in_file', file_path: path.join(root, 'di.ts'), fidelity: 'high' });
    for (const [owner, tokens] of [['CtorOwner', overrideTokens], ['TypedCtorOwner', typedTokens]]) {
      const label = `${name}/constructor/${++sequence}/${owner}`;
      const response = await call('workspace_query', { type: 'forward_edges', domain: 'angular', entity_type: 'Service', name: owner });
      const edges = response.structuredContent?.edges;
      check(`${label}/structured`, Array.isArray(edges), response);
      const selected = edges.filter((edge) => edge.relation === 'Injects');
      const actual = selected.map((edge) => [edge.object.name, edge.object.entity_type, normalize(edge.object.file)]).sort();
      const wanted = tokens.map((file) => ['Dependency', 'Service', normalize(path.join(root, file))]).sort();
      check(`${label}/exact-token-facts`, JSON.stringify(actual) === JSON.stringify(wanted), { actual, wanted });
      check(`${label}/owner`, selected.every((edge) => edge.subject.domain === 'angular'
        && edge.subject.entity_type === 'Service' && edge.subject.name === owner
        && edge.object.domain === 'angular' && normalize(edge.subject.file) === normalize(path.join(root, 'di.ts'))));
      check(`${label}/canonical-private`, selected.every((edge) => !('canonical' in edge.subject) && !('canonical' in edge.object)));
      check(`${label}/source-unchanged`, hash(fs.readFileSync(path.join(root, 'di.ts'))) === hash(Buffer.from(injectSource)));
    }
  };
}
