// Agent MaxHeadRoom: member projection acceptance, separate from Rust coverage.
import path from 'node:path';
export const memberApi = "export {input as read, output as emit, model as bind} from '@angular/core';";
export const foreignMemberApi = "export {input as read, output as emit, model as bind} from './member-foreign';";
export const memberSource = `
import {Component as View} from '@angular/core';
import {read as local, bind as localModel} from './member-api';
import * as api from './member-api';
const misleading = "@View({}) class MemberPanel {fake=api.read('');}";
function noise<T>(input:T) { return input; }
@View({}) export class MemberPanel {
 title=local(''); changed=api.emit<string>();
 selected=localModel.required<string>(); other=api.bind.required<number>();
 method(input:()=>unknown) { return input(); }
}
`;
export const memberFixture = {
  'members.ts': memberSource, 'member-neighbor.ts': memberSource,
  'member-api.ts': memberApi,
  'member-foreign.ts': 'export declare function input<T>(value:T):unknown; export declare function output<T>():unknown; export declare const model: {required:<T>()=>unknown;};',
};
export function memberAssertions({call, check, root, normalize, name}) {
  let sequence = 0;
  return async (owners = ['members.ts', 'member-neighbor.ts']) => {
    const label = `${name}/members/${++sequence}`;
    for (const file of ['members.ts', 'member-neighbor.ts']) {
      await call('workspace_query', {type:'entities_in_file',file_path:path.join(root,file),fidelity:'high'});
    }
    const response = await call('workspace_query', {type:'forward_edges',domain:'angular',entity_type:'Component',name:'MemberPanel'});
    const edges = response.structuredContent?.edges;
    check(`${label}/structured`, Array.isArray(edges), response);
    const selected = edges.filter(edge => ['HasInput','HasOutput'].includes(edge.relation));
    const actual = selected.map(edge => [normalize(edge.subject.file),edge.relation,edge.object.name]).sort();
    const expected = owners.flatMap(file => [
      [normalize(path.join(root,file)),'HasInput','title'],
      [normalize(path.join(root,file)),'HasOutput','changed'],
      ...['selected','other'].flatMap(member => ['HasInput','HasOutput'].map(relation => [normalize(path.join(root,file)),relation,member])),
    ]).sort();
    check(`${label}/exact-members-and-independent-owners`, JSON.stringify(actual) === JSON.stringify(expected), {actual,expected});
    check(`${label}/canonical-private`, selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
  };
}
