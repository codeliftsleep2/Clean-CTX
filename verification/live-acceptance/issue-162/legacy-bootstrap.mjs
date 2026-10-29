// Agent MaxHeadRoom: written native Module association through actual binary/stdio.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as base} from './bootstrap-references.mjs';
const factory="import {PlatformRef} from '@angular/core'; export declare function platformBrowserDynamic():PlatformRef;";
const core=base['node_modules/@angular/core/index.d.ts']+' export declare function NgModule(metadata:unknown):any; export declare class PlatformRef {bootstrapModule(module:unknown):unknown;}';
const forms=[
 ["import {platformBrowserDynamic} from '@angular/platform-browser-dynamic';",'platformBrowserDynamic'],
 ["import {platformBrowserDynamic as launch} from '@angular/platform-browser-dynamic';",'launch'],
 ["import {start as launch} from '../../legacy-api';",'launch'],
 ["import * as platform from '@angular/platform-browser-dynamic';",'platform.platformBrowserDynamic'],
];
export const ownerFiles=['apps/left/main.ts','apps/right/main.ts','apps/barrel/main.ts','apps/namespace/main.ts'];
const source=(i,target=i%2,repeats=2)=>`${forms[i][0]} import {AppModule} from '../../legacy-module-${target}'; ${`${forms[i][1]}().bootstrapModule(AppModule);`.repeat(repeats)}`;
export const fixture={...base,'node_modules/@angular/core/index.d.ts':core,
 'node_modules/@angular/platform-browser-dynamic/package.json':JSON.stringify({name:'@angular/platform-browser-dynamic',version:'20.0.0',types:'index.d.ts'}),
 'node_modules/@angular/platform-browser-dynamic/index.d.ts':factory,
 'legacy-api.ts':"export {launch as start} from './legacy-export';",
 'legacy-export.ts':"export {platformBrowserDynamic as launch} from '@angular/platform-browser-dynamic';",
 'legacy-foreign.ts':'export function platformBrowserDynamic(){return {bootstrapModule(value:unknown){}};}',
 'legacy-noise.ts':"import {platformBrowserDynamic} from '@angular/platform-browser-dynamic'; import {AppModule} from './legacy-module-0'; const foreign={bootstrapModule(value:unknown){}}; foreign.bootstrapModule(AppModule); function shadow(platformBrowserDynamic:any){platformBrowserDynamic().bootstrapModule(AppModule);} platformBrowserDynamic(1).bootstrapModule(AppModule); platformBrowserDynamic()['bootstrapModule'](AppModule); platformBrowserDynamic()?.bootstrapModule(AppModule);",
 ...Object.fromEntries([0,1].map(i=>[`legacy-module-${i}.ts`,`import {NgModule} from '@angular/core'; import {Token} from './legacy-token-${i}'; @NgModule({providers:[Token]}) export class AppModule {}`])),
 ...Object.fromEntries([0,1].map(i=>[`legacy-token-${i}.ts`,'export class Token {}'])),
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
const norm=f=>{const s=f.replaceAll('\\','/');return process.platform==='win32'?s.toLowerCase():s;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 let revision=0;const key=f=>norm(path.join(root,f));const hashes=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const facts=call=>async({absent=[],targets=[0,1,0,1],counts=[2,2,2,2],edited=[],noModule=[],none=false}={})=>{
  const wanted=[];
  for(let i=0;i<4;i++)if(!none&&!absent.includes(i)&&!noModule.includes(targets[i]))for(let n=0;n<counts[i];n++)wanted.push([key(ownerFiles[i]),key(`legacy-module-${targets[i]}.ts`)]);
  wanted.sort();
  for(const reverse of [false,true]) {
   const r=await call('workspace_query',{type:reverse?'reverse_edges':'forward_edges',domain:'angular',entity_type:reverse?'Module':'ApplicationBootstrap',name:reverse?'AppModule':'bootstrapModule'});
   const e=r.structuredContent?.edges;check(`q${revision++}/structured-lower-bound`,Array.isArray(e)&&r.structuredContent.coverage?.result_semantics==='lower_bound',r);
   const selected=e.filter(x=>x.relation==='DeclaresBootstrapModule');
   const actual=selected.map(x=>[norm(x.subject.file),norm(x.object.file)]).sort();
   check(`q${revision++}/exact-occurrence-source-target-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${revision++}/native-public-private-proof`,selected.every(x=>x.subject.entity_type==='ApplicationBootstrap'&&x.subject.name==='bootstrapModule'&&x.object.entity_type==='Module'&&x.object.name==='AppModule'&&!('canonical'in x.subject)&&!('canonical'in x.object)),selected);
   check(`q${revision++}/no-provider-inheritance`,!e.some(x=>x.subject.name==='bootstrapModule'&&x.relation==='RegistersProvider'),e);
  }
  const modern=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationBootstrap',name:'bootstrapApplication'});
  check(`q${revision++}/modern-sibling`,modern.structuredContent?.edges.some(e=>e.relation==='DeclaresBootstrapRoot'),modern);
  check(`q${revision++}/unchanged-callers`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===hashes[i]));
 };
 const checkpoint=async call=>{for(const f of ownerFiles){await call('compress_code_context',{filePath:path.join(root,f),fidelity:'high'});await call('save_context',{filePath:path.join(root,f)});}};
 await session(binary,'cold-ecological-warm-checkpoint',async({call})=>{const answer=facts(call);await answer();await answer();
  const scoped=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationBootstrap',name:'bootstrapModule',withinPath:path.join(root,'apps/left')});
  const edges=scoped.structuredContent.edges.filter(e=>e.relation==='DeclaresBootstrapModule');check('withinPath-answers-only',edges.length===2&&edges.every(e=>norm(e.subject.file)===key(ownerFiles[0])),scoped);await checkpoint(call);
 });
 await session(binary,'restart-retirement-target-module-repair',async({call})=>{
  for(const f of ownerFiles)await call('restore_context',{filePath:path.join(root,f)});
  const answer=facts(call);await answer();
  mutate(ownerFiles[0],source(0,0,1));await answer({counts:[1,2,2,2],edited:[0]});
  mutate(ownerFiles[0],source(0,1,1));await answer({targets:[1,1,0,1],counts:[1,2,2,2],edited:[0]});
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[1],null);await call('refresh_workspace',{});await answer({absent:[1],edited:[1]});
  mutate(ownerFiles[1],fixture[ownerFiles[1]]);await call('refresh_workspace',{});await answer();
  mutate('legacy-module-0.ts','export class AppModule {}');await answer({noModule:[0]});
  mutate('legacy-module-0.ts',fixture['legacy-module-0.ts']);await answer();await checkpoint(call);
 });
 mutate('node_modules/@angular/platform-browser-dynamic/index.d.ts','export declare function platformBrowserDynamic():unknown;');
 await session(binary,'stale-restore-return-member-barrel-package-config',async({call,client})=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[0])}},'stale-before-query');check('stale-restore-before-query',!!denied.error||denied.result?.isError===true,denied);
  const answer=facts(call);await answer({none:true});
  mutate('node_modules/@angular/platform-browser-dynamic/index.d.ts',factory);await answer();
  mutate('node_modules/@angular/core/index.d.ts',core.replace('bootstrapModule(module:unknown)','other(module:unknown)'));await answer({none:true});
  mutate('node_modules/@angular/core/index.d.ts',core);await answer();
  mutate('legacy-export.ts',"export {platformBrowserDynamic as launch} from './legacy-foreign';");await answer({absent:[2]});
  mutate('legacy-export.ts',fixture['legacy-export.ts']);await answer();
  mutate('node_modules/@angular/platform-browser-dynamic/package.json',JSON.stringify({name:'@angular/platform-browser-dynamic',version:'20.0.0',exports:{'.':null}}));await answer({none:true});
  mutate('node_modules/@angular/platform-browser-dynamic/package.json',fixture['node_modules/@angular/platform-browser-dynamic/package.json']);await answer();
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.paths={'@angular/platform-browser-dynamic':['./legacy-foreign.ts']};mutate('tsconfig.json',JSON.stringify(config));await answer({none:true});
  mutate('tsconfig.json',allFixture['tsconfig.json']);await answer();
 });
}
