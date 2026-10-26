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
export const positionOrdinary = 'providers:[{provide:Left,useValue:101},{provide:Right,useValue:102},{provide:Left,useValue:103}]';
export const positionView = 'viewProviders:[{provide:Left,useValue:101},{provide:Right,useValue:102},{provide:Left,useValue:103}]';
export const functionalSource = `
import {Component as View, Directive as Behavior} from '@angular/core';
import * as api from './functional-api';
import type {Match, Resolution, Intercept} from './functional-api';
import {AuditApi as Left} from './functional-left';
import {AuditApi as Right} from './functional-right';
import * as right from './functional-right';
import * as strategies from './provider-strategies';
import * as factoryLeft from './factory-left';
import * as factoryRight from './factory-right';
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
@View({providers:[{provide:Left,useClass:strategies.Implementation},
 {provide:Right,useFactory:strategies.makeAudit}]})
export class StrategyProviderHost {}
@View({providers:[{provide:Left,useClass:Left},{provide:Right,useClass:right.AuditApi},
 {provide:Left,useClass:Left},{provide:'audit.class',useClass:strategies.Implementation}]})
export class ClassPairProviderHost {}
@View({providers:[{provide:Left,useFactory:factoryLeft.makeAudit},
 {provide:Right,useFactory:factoryRight.makeAudit},{provide:Left,useFactory:factoryLeft.makeAudit},
 {provide:'audit.factory',useFactory:factoryLeft.makeAudit}]})
export class FactoryPairProviderHost {}
@View({providers:[{provide:Left,useExisting:Right},{provide:Left,useExisting:Right},
 {provide:Right,useExisting:factoryRight.makeAudit},{provide:Left,useExisting:'audit.target'},
 {provide:'audit.alias',useExisting:Left},{provide:'audit.source',useExisting:'audit.target'}]})
export class ExistingPairProviderHost {}
@View({providers:[{provide:Left,useValue:factoryLeft.makeAudit},
 {provide:Right,useValue:factoryRight.makeAudit},{provide:Left,useValue:factoryLeft.makeAudit}]})
export class ValueReferenceProviderHost {}
@Behavior({selector:'[providerProbe]',providers:[{provide:Left,useValue:1},
 {provide:Right,useValue:2},{provide:Left,useValue:3}]})
export class DirectiveProviderHost {}
@View({${positionOrdinary},${positionView}})
export class PositionProviderHost {}
`;
export const functionalFixture = {
  'functional.ts':functionalSource, 'functional-neighbor.ts':functionalSource,
  'functional-left.ts':'export class AuditApi {}', 'functional-right.ts':'export class AuditApi {}',
  'functional-api.ts':functionalApi(),
  'provider-type-only.ts':'export class RefusedApi {}',
  'provider-implementation.ts':'export class Implementation {} export function makeAudit(){return new Implementation();}',
  'provider-strategies.ts':"export {Implementation,makeAudit} from './provider-implementation';",
  'factory-left.ts':'export function makeAudit(){return {};}',
  'factory-right.ts':'export function makeAudit(){return {};}',
  'provider-foreign.ts':'export function Component(metadata:unknown){return metadata;}',
  'functional-foreign.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
  'node_modules/@angular/router/package.json':JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'}),
  'node_modules/@angular/router/index.d.ts':'export type CanMatchFn=(...args:unknown[])=>unknown; export type ResolveFn<T>=(...args:unknown[])=>T;',
  'node_modules/@angular/common/package.json':commonManifest(),
  'node_modules/@angular/common/index.d.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
  'node_modules/@angular/common/http/index.d.ts':'export type HttpInterceptorFn=(...args:unknown[])=>unknown;',
};
export function functionalAssertions({call,check,root,normalize,name,group}) {
  let sequence = 0;
  return async ({tokens = ['functional-left.ts','functional-right.ts'], http = true,
    owners = ['functional.ts','functional-neighbor.ts'], providerOwners = owners,
    strategyOwners = providerOwners, implementation = true, factoryRight = true,
    ordinaryPositionOwners = providerOwners, viewPositionOwners = providerOwners} = {}) => {
    const label = `${name}/functional/${++sequence}`;
    for (const file of ['functional.ts','functional-neighbor.ts']) {
      await call('workspace_query',{type:'entities_in_file',file_path:path.join(root,file),fidelity:'high'});
    }
    if (group === 'roles') for (const [kind,owner] of [['Guard','functionalGuard'],['Resolver','functionalResolver'],['Interceptor','functionalInterceptor']]) {
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
    for (const providerName of ['FunctionalProviderHost','ShorthandProviderHost','StrategyProviderHost','ValueReferenceProviderHost','DirectiveProviderHost']) {
      if (group !== providerName) continue;
      const providerKind = providerName === 'DirectiveProviderHost' ? 'Directive' : 'Component';
      const providers = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:providerKind,name:providerName});
      const registrations = providers.structuredContent?.edges;
      check(`${label}/${providerName}/structured`,Array.isArray(registrations),providers);
      const selectedProviders = registrations.filter(edge => edge.relation === 'RegistersProvider');
      check(`${label}/${providerName}/exact-owner-role`,selectedProviders.every(edge => edge.subject.entity_type === providerKind));
      const actualProviders = selectedProviders.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const registrationTokens = providerName === 'StrategyProviderHost' && !implementation
        ? tokens.filter(token => token !== 'functional-left.ts')
        : providerName === 'ValueReferenceProviderHost' && !factoryRight
          ? tokens.filter(token => token !== 'functional-right.ts') : tokens;
      const expectedProviders = (providerName === 'StrategyProviderHost' ? strategyOwners : providerOwners).flatMap(file => registrationTokens.map(token =>
        [normalize(path.join(root,file)),'AuditApi',normalize(path.join(root,token))])).sort();
      check(`${label}/${providerName}/exact-declaration-identities`,JSON.stringify(actualProviders) === JSON.stringify(expectedProviders),{actualProviders,expectedProviders});
      check(`${label}/${providerName}/canonical-private`,selectedProviders.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    if (group === 'positions') {
    const positions = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Component',name:'PositionProviderHost'});
    const positionEdges = positions.structuredContent?.edges;
    check(`${label}/positions/structured`,Array.isArray(positionEdges),positions);
    for (const [relation,positionOwners] of [['RegistersProvider',ordinaryPositionOwners],['RegistersViewProvider',viewPositionOwners]]) {
      const selected = positionEdges.filter(edge => edge.relation === relation);
      const actual = selected.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expected = positionOwners.flatMap(owner => tokens.map(token => [normalize(path.join(root,owner)),'AuditApi',normalize(path.join(root,token))])).sort();
      check(`${label}/positions/${relation}/exact-facts`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/positions/${relation}/owner-and-private`,selected.every(edge => edge.subject.entity_type === 'Component' && !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    }
    if (group === 'class') {
    for (const tokenName of ['AuditApi','"audit.class"']) {
      const response = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ProviderToken',name:tokenName});
      const classEdges = response.structuredContent?.edges;
      check(`${label}/class-pairs/${tokenName}/structured`,Array.isArray(classEdges),response);
      const selected = classEdges.filter(edge => edge.relation === 'ProviderUsesClass');
      const actual = selected.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expected = [];
      if (tokenName === 'AuditApi') {
        // Equal explicit/shorthand triples deduplicate within one asserting file.
        for (const owner of providerOwners) for (const token of tokens) {
          expected.push([normalize(path.join(root,token)),'AuditApi',normalize(path.join(root,token))]);
        }
        if (implementation && tokens.includes('functional-left.ts')) for (const owner of strategyOwners) {
          expected.push([normalize(path.join(root,'functional-left.ts')),'Implementation',normalize(path.join(root,'provider-implementation.ts'))]);
        }
      } else if (implementation) for (const owner of strategyOwners) {
        expected.push([normalize(path.join(root,owner)),'Implementation',normalize(path.join(root,'provider-implementation.ts'))]);
      }
      expected.sort();
      check(`${label}/class-pairs/${tokenName}/exact-identities`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/class-pairs/${tokenName}/private`,selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    }
    if (group === 'factory') {
    for (const tokenName of ['AuditApi','"audit.factory"']) {
      const response = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ProviderToken',name:tokenName});
      const edges = response.structuredContent?.edges;
      check(`${label}/factory-pairs/${tokenName}/structured`,Array.isArray(edges),response);
      const selected = edges.filter(edge => edge.relation === 'ProviderUsesFactory');
      const actual = selected.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expected = [];
      if (tokenName === 'AuditApi') {
        for (const owner of providerOwners) for (const token of tokens) {
          if (token === 'functional-right.ts' && !factoryRight) continue;
          const target = token === 'functional-left.ts' ? 'factory-left.ts' : 'factory-right.ts';
          expected.push([normalize(path.join(root,token)),'makeAudit',normalize(path.join(root,target))]);
        }
        if (tokens.includes('functional-right.ts')) for (const owner of strategyOwners) {
          expected.push([normalize(path.join(root,'functional-right.ts')),'makeAudit',normalize(path.join(root,'provider-implementation.ts'))]);
        }
      } else for (const owner of providerOwners) {
        expected.push([normalize(path.join(root,owner)),'makeAudit',normalize(path.join(root,'factory-left.ts'))]);
      }
      expected.sort();
      check(`${label}/factory-pairs/${tokenName}/exact-identities`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/factory-pairs/${tokenName}/private`,selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    }
    if (group === 'existing') {
    for (const tokenName of ['AuditApi','"audit.alias"','"audit.source"']) {
      const response = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ProviderToken',name:tokenName});
      const edges = response.structuredContent?.edges;
      check(`${label}/existing-pairs/${tokenName}/structured`,Array.isArray(edges),response);
      const selected = edges.filter(edge => edge.relation === 'ProviderUsesExisting');
      const actual = selected.map(edge => [normalize(edge.subject.file),edge.object.name,normalize(edge.object.file)]).sort();
      const expected = [];
      for (const owner of providerOwners) {
        if (tokenName === 'AuditApi') {
          if (tokens.includes('functional-left.ts') && tokens.includes('functional-right.ts')) {
            expected.push([normalize(path.join(root,'functional-left.ts')),'AuditApi',normalize(path.join(root,'functional-right.ts'))]);
          }
          if (tokens.includes('functional-right.ts') && factoryRight) {
            expected.push([normalize(path.join(root,'functional-right.ts')),'makeAudit',normalize(path.join(root,'factory-right.ts'))]);
          }
          if (tokens.includes('functional-left.ts')) {
            expected.push([normalize(path.join(root,'functional-left.ts')),'"audit.target"',normalize(path.join(root,owner))]);
          }
        } else if (tokenName === '"audit.alias"') {
          if (tokens.includes('functional-left.ts')) expected.push([normalize(path.join(root,owner)),'AuditApi',normalize(path.join(root,'functional-left.ts'))]);
        } else expected.push([normalize(path.join(root,owner)),'"audit.target"',normalize(path.join(root,owner))]);
      }
      expected.sort();
      check(`${label}/existing-pairs/${tokenName}/exact-identities`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/existing-pairs/${tokenName}/private`,selected.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)));
    }
    }
    if (group === 'value') {
    const values = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ProviderToken',name:'AuditApi'});
    const valueEdges = values.structuredContent?.edges;
    check(`${label}/value-reference/structured`,Array.isArray(valueEdges),values);
    const valueProjection = valueEdges.filter(edge => edge.relation === 'ProviderUsesValue' && edge.object.name === 'makeAudit');
    // This legacy projection intentionally cannot distinguish equal value labels.
    const valueOwners = tokens.includes('functional-left.ts') || (factoryRight && tokens.includes('functional-right.ts')) ? providerOwners : [];
    check(`${label}/value-reference/legacy-count`,valueProjection.length === valueOwners.length,valueProjection);
    check(`${label}/value-reference/private`,valueProjection.every(edge => !('canonical' in edge.subject) && !('canonical' in edge.object)),valueProjection);
    const valueProvenance = valueProjection.map(edge => normalize(edge.object.file)).sort();
    const expectedValueProvenance = valueOwners.map(owner => normalize(path.join(root,owner))).sort();
    check(`${label}/value-reference/asserting-source`,JSON.stringify(valueProvenance) === JSON.stringify(expectedValueProvenance),{valueProvenance,expectedValueProvenance});
    }
    if (group === 'roles') {
    const decoy = await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Guard',name:'loose'});
    const edges = decoy.structuredContent?.edges;
    check(`${label}/decoy-structured`,Array.isArray(edges),decoy);
    check(`${label}/inner-role-does-not-escape`,edges.every(edge => edge.relation !== 'Injects'),edges);
    }
  };
}
