// Agent MaxHeadRoom: synthetic string-token binary assertions, separate from Rust tests.
import path from 'node:path';
export const stringValues = (value = 'audit.api') => `
import { InjectionToken } from '@angular/core';
export const /* trivia */ ROOT = '${value}';
export const ALIAS = ROOT;
export const OBJECT_ONE = new InjectionToken<unknown>('audit.api');
export const OBJECT_TWO = new InjectionToken<unknown>('audit.api');
export let mutable = 'not-proved';
const misleading = "ALIAS = 'phantom'";
function noise<T>(ROOT:T) { return ROOT; }
`;
export const stringSource = `
import {Injectable as Service, Component as View, inject as obtain} from '@angular/core';
import * as core from '@angular/core';
import {Select} from './string-api';
import {Shared, OBJECT_ONE, OBJECT_TWO} from './string-barrel';
const local = Shared;
const prose = "@Select('phantom')";
@Service() export class StringOwner {
 constructor(@Select(local) derived:unknown, @core.Inject('audit\u002eapi') direct:unknown,
 @Select(OBJECT_ONE) first:unknown, @Select(OBJECT_TWO) second:unknown) {}
 unsupportedStringField = obtain('field-only');
 method(Shared:unknown) { return Shared; }
}
@View({selector:'string-host',providers:[
 {provide:local,useValue:1}, {provide:'audit.api',useValue:2}
]}) export class StringHost {}
`;
export const neighborSource = `
import {Injectable as Service} from '@angular/core';
import * as api from './string-api';
import * as tokens from './string-barrel';
@Service() export class StringOwner { constructor(@api.Select(tokens.Shared) value:unknown) {} }
`;
export const stringFixture = {
  'string-values.ts': stringValues(), 'strings.ts': stringSource,
  'string-neighbor.ts': neighborSource,
  'string-barrel.ts': "export {ALIAS as Shared, OBJECT_ONE, OBJECT_TWO} from './string-values';",
  'string-api.ts': "export {Inject as Select} from '@angular/core';",
};
export function stringAssertions({call, check, root, normalize, hash, fs, name}) {
  let sequence = 0;
  return async (value = 'audit.api') => {
    const label = `${name}/strings/${++sequence}`;
    for (const file of ['strings.ts', 'string-neighbor.ts']) {
      await call('workspace_query', {type:'entities_in_file', file_path:path.join(root,file), fidelity:'high'});
    }
    const response = await call('workspace_query', {type:'forward_edges', domain:'angular', entity_type:'Service', name:'StringOwner'});
    const edges = response.structuredContent?.edges;
    check(`${label}/structured`, Array.isArray(edges), response);
    const selected = edges.filter(edge => edge.relation === 'Injects');
    const actual = selected.map(edge => [normalize(edge.subject.file), edge.object.name, normalize(edge.object.file)]).sort();
    const file = name => normalize(path.join(root,name));
    const wanted = [
      [file('strings.ts'), '"audit.api"', file('strings.ts')],
      [file('string-neighbor.ts'), JSON.stringify(value), file('string-neighbor.ts')],
      ...['OBJECT_ONE','OBJECT_TWO'].map(token => [file('strings.ts'),token,file('string-values.ts')]),
    ];
    if (value !== 'audit.api') wanted.push([file('strings.ts'),JSON.stringify(value),file('strings.ts')]);
    wanted.sort();
    check(`${label}/exact-values-objects-and-owner-multiplicity`, JSON.stringify(actual) === JSON.stringify(wanted), {actual,wanted});
    check(`${label}/canonical-private`, selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    const providers = await call('workspace_query', {type:'forward_edges',domain:'angular',entity_type:'Component',name:'StringHost'});
    const registrations = providers.structuredContent?.edges?.filter(edge => edge.relation === 'RegistersProvider');
    check(`${label}/provider-structured`, Array.isArray(registrations), providers);
    const names = registrations.map(edge => edge.object.name).sort();
    const expected = [...new Set(['"audit.api"', JSON.stringify(value)])].sort();
    check(`${label}/provider-token-values`, JSON.stringify(names) === JSON.stringify(expected), {names,expected});
    check(`${label}/provider-owner`, registrations.every(edge => normalize(edge.subject.file) === file('strings.ts')
      && normalize(edge.object.file) === file('strings.ts') && !('canonical' in edge.object)));
    for (const [sourceFile, source] of [['strings.ts',stringSource],['string-neighbor.ts',neighborSource]]) {
      check(`${label}/source-unchanged/${sourceFile}`, hash(fs.readFileSync(path.join(root,sourceFile))) === hash(Buffer.from(source)));
    }
  };
}
