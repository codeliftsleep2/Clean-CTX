// Agent MaxHeadRoom: source-declared config membership, never application selection.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';

export const ownerFiles = ['config-owner.ts','config-neighbor.ts'];
const group = "import {Token as Left} from './config-left'; import {Token as Right} from './config-right'; export const GROUP=[Left,[{provide:Right,useValue:1},Left]];";
const source = namespace => `${namespace ? "import type * as core from '@angular/core';" : "import type {ApplicationConfig as NativeConfig} from '@angular/core';"}
import {Token as Left} from './config-left'; import {Token as Right} from './config-right';
${namespace ? "import * as groups from './config-barrel';" : "import {PublicGroup as Imported} from './config-barrel';"}
const LOCAL=[Left]; const ALIAS=LOCAL;
export const appConfig:${namespace ? 'core.ApplicationConfig' : 'NativeConfig'}={providers:[...ALIAS,...[${namespace ? 'groups.PublicGroup' : 'Imported'}],Left],viewProviders:[Right]};`;
const manifest = JSON.stringify({name:'@angular/core',version:'20.0.0',types:'index.d.ts'});
const core = 'export declare function NgModule(metadata:unknown):any; export declare function Directive(metadata:unknown):any; export declare function Component(metadata:unknown):any; export declare function Injectable(metadata?:unknown):any; export declare function inject(token:unknown):unknown; export declare function Inject(token:unknown):any; export interface ApplicationConfig {providers:unknown[];}';
export const fixture = {
  'node_modules/@angular/core/package.json':manifest,
  'node_modules/@angular/core/index.d.ts':core,
  'config-left.ts':'export class Token {}','config-right.ts':'export class Token {}',
  'config-group.ts':group,'config-barrel.ts':"export {GROUP as PublicGroup} from './config-group';",
  'config-fake.ts':'export interface ApplicationConfig {providers:unknown[];}',
  'config-owner.ts':source(false),'config-neighbor.ts':source(true),
};
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const normalize = file => {
  const value = file.replace(/^\\\\\?\\/,'').replaceAll('\\','/');
  return process.platform === 'win32' ? value.toLowerCase() : value;
};

export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
  const ownerHashes = Object.fromEntries(ownerFiles.map(file => [file,hash(Buffer.from(fixture[file]))]));
  let revision = 0;
  const facts = (call,phase) => async (options={}) => {
    const {groupEnabled=true,right=true,absent=[],fieldRemoved=[],edited=[]} = options;
    const rows = [];
    for (const file of ownerFiles) {
      if (absent.includes(file) || fieldRemoved.includes(file)) continue;
      rows.push([normalize(path.join(root,file)),normalize(path.join(root,'config-left.ts'))]);
      if (groupEnabled && right) rows.push([normalize(path.join(root,file)),normalize(path.join(root,'config-right.ts'))]);
    }
    for (const reverse of [false,true]) {
      const response = await call('workspace_query',{type:reverse ? 'reverse_edges':'forward_edges',
        domain:'angular',entity_type:reverse ? 'ProviderToken':'ApplicationConfig',name:reverse ? 'Token':'appConfig'});
      const edges = response.structuredContent?.edges;
      const label = `${phase}/${revision++}/${reverse ? 'reverse' : 'forward'}`;
      check(`${label}/structured`,Array.isArray(edges),response);
      const selected = edges.filter(e => e.relation === 'RegistersProvider' && e.subject.entity_type === 'ApplicationConfig');
      const actual = selected.map(e => [normalize(e.subject.file),normalize(e.object.file)]).sort();
      check(`${label}/exact-owner-token-multiset`,JSON.stringify(actual) === JSON.stringify([...rows].sort()),{actual,expected:rows});
      check(`${label}/public-provenance`,selected.every(e => e.subject.domain === 'angular'
        && e.subject.name === 'appConfig' && e.object.domain === 'angular'
        && e.object.entity_type === 'ProviderToken' && e.object.name === 'Token'
        && !('canonical' in e.subject) && !('canonical' in e.object)),selected);
      check(`${label}/no-view-membership`,!edges.some(e => e.relation === 'RegistersViewProvider'
        && e.subject.entity_type === 'ApplicationConfig'),edges);
    }
    check(`${phase}/unchanged-owners`,ownerFiles.filter(file => !edited.includes(file)).every(file =>
      hash(fs.readFileSync(path.join(root,file))) === ownerHashes[file]));
  };
  const acquire = async (call,file) => call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
  const checkpoint = async call => {
    for (const file of ownerFiles) {
      await acquire(call,file); await call('save_context',{filePath:path.join(root,file)});
    }
  };
  await session(binary,'cold-warm-checkpoint',async ({call}) => {
    for (const file of ownerFiles) await acquire(call,file);
    const answer = facts(call,'cold-warm'); await answer(); await answer(); await checkpoint(call);
  });
  await session(binary,'restart-mutation-retirement',async ({call}) => {
    for (const file of ownerFiles) await call('restore_context',{filePath:path.join(root,file)});
    const answer = facts(call,'restart-mutations'); await answer();
    mutate('config-right.ts','export interface Token {}'); await answer({right:false});
    mutate('config-right.ts',fixture['config-right.ts']); await answer();
    mutate('config-barrel.ts',"export type {GROUP as PublicGroup} from './config-group';"); await answer({groupEnabled:false});
    mutate('config-barrel.ts',fixture['config-barrel.ts']); await answer();
    mutate('config-group.ts',group.replace('const GROUP=','let GROUP=')); await answer({groupEnabled:false});
    mutate('config-group.ts',group.replace('const GROUP=[','const GROUP=[...GROUP,')); await answer();
    mutate('config-group.ts',null); await answer({groupEnabled:false});
    mutate('config-group.ts',group); await answer();
    const file = ownerFiles[0], original = fixture[file];
    const field = 'providers:[...ALIAS,...[Imported],Left]';
    if (!original.includes(field)) throw new Error('fixture providers marker missing');
    mutate(file,original.replace(field,'providers:[]')); await answer({fieldRemoved:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer();
    for (const replacement of [
      original.replace(':NativeConfig',''),
      original.replace('const appConfig','let appConfig'),
      original.replace(':NativeConfig',':FakeConfig')+" import type {ApplicationConfig as FakeConfig} from './config-fake';",
      original.replace('{providers:','{providers:[],ignored:'),
      'export class appConfig {}',
    ]) {
      mutate(file,replacement); await answer({absent:[file],edited:[file]});
      // Removed identities require explicit known-root reacquisition.
      mutate(file,original); await acquire(call,file); await answer();
    }
    mutate(file,null); await call('refresh_workspace',{}); await answer({absent:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer(); await checkpoint(call);
  });
  mutate('config-group.ts',group.replace('const GROUP=','let GROUP='));
  await session(binary,'stale-restore-config-package',async ({call,client}) => {
    for (const file of ownerFiles) {
      const response = await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,
        filePath:path.join(root,file)}},`stale-restore/${file}`);
      check(`stale-restore/${file}`,!!response.error || response.result?.isError === true,response);
    }
    const answer = facts(call,'stale-repair'); await answer({groupEnabled:false});
    mutate('config-group.ts',group); await answer();
    const redirected = JSON.parse(allFixture['tsconfig.json']);
    redirected.compilerOptions.paths = {'@angular/core':['./config-fake.ts']};
    mutate('tsconfig.json',JSON.stringify(redirected)); await answer({absent:ownerFiles});
    mutate('tsconfig.json',allFixture['tsconfig.json']); await answer();
    mutate('node_modules/@angular/core/package.json',JSON.stringify({name:'@angular/core',version:'20.0.0',exports:{'.':null}}));
    await answer({absent:ownerFiles});
    mutate('node_modules/@angular/core/package.json',manifest); await answer();
  });
}
