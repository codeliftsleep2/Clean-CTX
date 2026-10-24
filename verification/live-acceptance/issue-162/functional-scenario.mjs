// Agent MaxHeadRoom: operator binary evidence; tracked Rust controls remain separate.
import path from 'node:path';
export const commonManifest = (enabled = true, version = '20.0.0') => JSON.stringify({
  name:'@angular/common', version, types:'index.d.ts',
  exports:{'.':{types:'./index.d.ts'},'./http':enabled ? {types:'./http/index.d.ts'} : null},
});
export const functionalApi = (entry = '@angular/common/http') => `
export {inject as obtain} from '@angular/core';
export type {CanMatchFn as Match, ResolveFn as Resolution} from '@angular/router';
export type {HttpInterceptorFn as Intercept} from '${entry}';
`;
export const functionalSource = `
import {Component as View} from '@angular/core';
import * as api from './functional-api';
import type {Match, Resolution, Intercept} from './functional-api';
import {AuditApi as Left} from './functional-left';
import {AuditApi as Right} from './functional-right';
import * as right from './functional-right';
import type {RefusedApi as OnlyType} from './provider-type-only';
const prose="api.obtain(Phantom); Intercept";
function noise<T>(Left:T){return Left;}
export const functionalGuard:Match=()=>{api.obtain(Left);api.obtain(Right);api.obtain(Left);return true;};
export const functionalResolver:Resolution<unknown>=()=>{api.obtain(Left);return api.obtain(Right);};
export const functionalInterceptor:Intercept=()=>{api.obtain(Left);return api.obtain(Right);};
export const loose=()=>{const decoy:Match=()=>true;api.obtain(Left);return decoy;};
@View({providers:[{provide:Left,useValue:1},{provide:Right,useValue:2},
 {provide:Left,useValue:3},{provide:OnlyType,useValue:4}]})
export class FunctionalProviderHost {}
@View({providers:[Left,right.AuditApi,Left,OnlyType]})
export class ShorthandProviderHost {}
`;
export const functionalFixture = {
  'functional.ts':functionalSource, 'functional-neighbor.ts':functionalSource,
  'functional-left.ts':'export class AuditApi {}', 'functional-right.ts':'export class AuditApi {}',
  'functional-api.ts':functionalApi(),
  'provider-type-only.ts':'export class RefusedApi {}',
  'provider-foreign.ts':'export function Component(metadata:unknown){return metadata;}',
  'functional-foreign.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
  'node_modules/@angular/router/package.json':JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'}),
  'node_modules/@angular/router/index.d.ts':'export type CanMatchFn=(...args:unknown[])=>unknown; export type ResolveFn<T>=(...args:unknown[])=>T;',
  'node_modules/@angular/common/package.json':commonManifest(),
  'node_modules/@angular/common/index.d.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
  'node_modules/@angular/common/http/index.d.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
};
export function functionalAssertions({call,check,root,normalize,name}) {
  let sequence = 0;
  return async ({tokens = ['functional-left.ts','functional-right.ts'], http = true,
    owners = ['functional.ts','functional-neighbor.ts'], providerOwners = owners} = {}) => {
    const label = `${name}/functional/${++sequence}`;
    for (const file of ['functional.ts','functional-neighbor.ts']) {
      await call('workspace_query',{type:'entities_in_file',file_path:path.join(root,file),fidelity:'high'});
    }
    for (const [kind,owner] of [['Guard','functionalGuard'],['Resolver','functionalResolver'],['Interceptor','functionalInterceptor']]) {
      const response = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:kind,name:owner});
      const edges = response.structuredContent?.edges;
      check(`${label}/${kind}/structured`,Array.isArray(edges),response);
      const selected = edges.filter(edge => edge.relation === 'Injects');
      const actual = selected.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expected = (kind === 'Interceptor' && !http ? [] : owners.flatMap(file => tokens.map(token =>
        [normalize(path.join(root,file)),'AuditApi',normalize(path.join(root,token))]))).sort();
      check(`${label}/${kind}/exact-owner-token-identities`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/${kind}/canonical-private`,selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    for (const providerName of ['FunctionalProviderHost','ShorthandProviderHost']) {
      const providers = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Component',name:providerName});
      const registrations = providers.structuredContent?.edges;
      check(`${label}/${providerName}/structured`,Array.isArray(registrations),providers);
      const selectedProviders = registrations.filter(edge => edge.relation === 'RegistersProvider');
      const actualProviders = selectedProviders.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expectedProviders = providerOwners.flatMap(file => tokens.map(token =>
        [normalize(path.join(root,file)),'AuditApi',normalize(path.join(root,token))])).sort();
      check(`${label}/${providerName}/exact-declaration-identities`,JSON.stringify(actualProviders) === JSON.stringify(expectedProviders),{actualProviders,expectedProviders});
      check(`${label}/${providerName}/canonical-private`,selectedProviders.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    const decoy = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Guard',name:'loose'});
    const edges = decoy.structuredContent?.edges;
    check(`${label}/decoy-structured`,Array.isArray(edges),decoy);
    check(`${label}/inner-role-does-not-escape`,edges.every(edge => edge.relation !== 'Injects'),edges);
  };
}
