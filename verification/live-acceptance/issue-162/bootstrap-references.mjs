// Agent MaxHeadRoom: written bootstrap associations, never runtime application identity.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
const core = "export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}";
const platform = "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;";
const manifest = JSON.stringify({name:'@angular/platform-browser',version:'20.0.0',types:'index.d.ts'});
const rootSource = "import {Component} from '@angular/core'; @Component({}) export class Root {}";
const configSource = "import type {ApplicationConfig} from '@angular/core'; export const appConfig:ApplicationConfig={providers:[]};";
const forms = [
  ["import {bootstrapApplication as launch} from '@angular/platform-browser';",'launch'],
  ["import {bootstrapApplication as launch} from '@angular/platform-browser';",'launch'],
  ["import {start as launch} from './boot-api';",'launch'],
  ["import * as platform from '@angular/platform-browser';",'platform.bootstrapApplication'],
];
// The direct spelling is distinct from the aliased API route.
forms[0] = ["import {bootstrapApplication} from '@angular/platform-browser';",'bootstrapApplication'];
export const ownerFiles = forms.map((_,i)=>`boot-main-${i}.ts`);
const source = (i,body) => `${forms[i][0]} import {Root} from './boot-root-${i%2}'; import {appConfig} from './boot-config-${i%2}'; ${body ?? `${forms[i][1]}(Root,appConfig);`.repeat(i===0 ? 2 : 1)}`;
export const fixture = {
  'node_modules/@angular/core/package.json':JSON.stringify({name:'@angular/core',version:'20.0.0',types:'index.d.ts'}),
  'node_modules/@angular/core/index.d.ts':core,
  'node_modules/@angular/platform-browser/package.json':manifest,
  'node_modules/@angular/platform-browser/index.d.ts':platform,
  'boot-api.ts':"export {bootstrapApplication as start} from '@angular/platform-browser';",
  'boot-fake.ts':'export function bootstrapApplication(root:unknown,config?:unknown){}',
  'boot-refused.ts':"import {bootstrapApplication} from './boot-fake'; import {Root} from './boot-root-0'; import {appConfig} from './boot-config-0'; bootstrapApplication(Root,appConfig);",
  ...Object.fromEntries([0,1].flatMap(i=>[[`boot-root-${i}.ts`,rootSource],[`boot-config-${i}.ts`,configSource]])),
  ...Object.fromEntries(ownerFiles.map((file,i)=>[file,source(i)])),
};
const normalize = file => {
  const value = file.replaceAll('\\','/');
  return process.platform === 'win32' ? value.toLowerCase() : value;
};
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
  let revision = 0;
  const ownerHashes = ownerFiles.map(file=>hash(Buffer.from(fixture[file])));
  const acquire = async (call,file) => call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
  const checkpoint = async call => {
    for (const file of ownerFiles) {
      await acquire(call,file);
      await call('save_context',{filePath:path.join(root,file)});
    }
  };
  const facts = call => async (options={}) => {
    const {noRoots=[],noConfigs=[],noCalls=[],mainCounts=[2,2],edited=[]} = options;
    const wanted = [];
    for (let i=0;i<4;i++) {
      if (noCalls.includes(i)) continue;
      for (const [relation,target,disabled,count] of [
        ['DeclaresBootstrapRoot',`boot-root-${i%2}.ts`,noRoots,i===0 ? mainCounts[0] : 1],
        ['UsesApplicationConfig',`boot-config-${i%2}.ts`,noConfigs,i===0 ? mainCounts[1] : 1],
      ]) if (!disabled.includes(i)) for (let n=0;n<count;n++) wanted.push([
        normalize(path.join(root,ownerFiles[i])),relation,normalize(path.join(root,target))]);
    }
    wanted.sort();
    for (const reverse of [false,true]) {
      const selectors = reverse ? [['Component','Root'],['ApplicationConfig','appConfig']] : [['ApplicationBootstrap','bootstrapApplication']];
      const selected = [];
      for (const [kind,name] of selectors) {
        const response = await call('workspace_query',{type:reverse ? 'reverse_edges':'forward_edges',domain:'angular',entity_type:kind,name});
        const edges = response.structuredContent?.edges;
        check(`q${revision++}/structured`,Array.isArray(edges),response);
        selected.push(...edges.filter(e=>['DeclaresBootstrapRoot','UsesApplicationConfig'].includes(e.relation)));
      }
      const actual = selected.map(e=>[normalize(e.subject.file),e.relation,normalize(e.object.file)]).sort();
      check(`q${revision++}/exact-public-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
      check(`q${revision++}/public-types-private-canonical`,selected.every(e=>e.subject.entity_type==='ApplicationBootstrap'
        && e.subject.name==='bootstrapApplication' && e.subject.domain==='angular' && e.object.domain==='angular'
        && e.object.entity_type===(e.relation==='DeclaresBootstrapRoot' ? 'Component':'ApplicationConfig')
        && !('canonical' in e.subject) && !('canonical' in e.object)),selected);
    }
    check(`q${revision++}/unchanged-callers`,ownerFiles.every((file,i)=>edited.includes(i)
      || hash(fs.readFileSync(path.join(root,file)))===ownerHashes[i]));
  };
  await session(binary,'cold-warm-checkpoint',async ({call})=>{
    for (const file of [...ownerFiles,'boot-refused.ts']) await acquire(call,file);
    const answer = facts(call); await answer(); await answer(); await checkpoint(call);
  });
  await session(binary,'restart-mutation-retirement',async ({call})=>{
    for (const file of ownerFiles) await call('restore_context',{filePath:path.join(root,file)});
    const answer = facts(call); await answer();
    mutate('boot-root-0.ts','export class Root {}'); await answer({noRoots:[0,2]});
    mutate('boot-root-0.ts',rootSource); await answer();
    mutate('boot-config-1.ts','export const appConfig={providers:[]};'); await answer({noConfigs:[1,3]});
    mutate('boot-config-1.ts',configSource); await answer();
    mutate('boot-api.ts',"export {bootstrapApplication as start} from './boot-fake';"); await answer({noCalls:[2]});
    mutate('boot-api.ts',fixture['boot-api.ts']); await answer();
    for (const [body,counts] of [
      ['bootstrapApplication(Root,appConfig);',[1,1]],
      ['bootstrapApplication(Root); bootstrapApplication(Root,appConfig);',[2,1]],
      ['bootstrapApplication(undefined,appConfig); bootstrapApplication(Root,appConfig);',[1,2]],
      ['export class Replaced {}',[0,0]],
    ]) {
      mutate(ownerFiles[0],source(0,body)); await answer({mainCounts:counts,edited:[0]});
    }
    mutate(ownerFiles[0],fixture[ownerFiles[0]]); await acquire(call,ownerFiles[0]); await answer();
    mutate(ownerFiles[0],null); await call('refresh_workspace',{}); await answer({noCalls:[0],edited:[0]});
    mutate(ownerFiles[0],fixture[ownerFiles[0]]); await acquire(call,ownerFiles[0]); await answer(); await checkpoint(call);
  });
  mutate('boot-root-0.ts','export class Root {}');
  await session(binary,'stale-restore-package-config',async ({call,client})=>{
    for (const i of [0,2]) {
      const response = await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[i])}},`stale/${i}`);
      check(`stale-before-query/${i}`,!!response.error || response.result?.isError===true,response);
    }
    const answer = facts(call); await answer({noRoots:[0,2]});
    mutate('boot-root-0.ts',rootSource); await answer();
    mutate('node_modules/@angular/platform-browser/package.json',JSON.stringify({name:'@angular/platform-browser',version:'20.0.0',exports:{'.':null}}));
    await answer({noCalls:[0,1,2,3]});
    mutate('node_modules/@angular/platform-browser/package.json',manifest); await answer();
    const redirected = JSON.parse(allFixture['tsconfig.json']);
    redirected.compilerOptions.paths = {'@angular/platform-browser':['./boot-fake.ts']};
    mutate('tsconfig.json',JSON.stringify(redirected)); await answer({noCalls:[0,1,2,3]});
    mutate('tsconfig.json',allFixture['tsconfig.json']); await answer();
  });
}
