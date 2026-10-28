// Agent MaxHeadRoom: pair-level written config inputs, no evaluated provider union.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as base} from './bootstrap-references.mjs';
const core='export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];} export declare function mergeApplicationConfig(...configs:ApplicationConfig[]):ApplicationConfig;';
const forms=[
 ["import {mergeApplicationConfig} from '@angular/core';",'mergeApplicationConfig'],
 ["import {mergeApplicationConfig as combine} from '@angular/core';",'combine'],
 ["import {combine} from './merge-api';",'combine'],
 ["import * as core from '@angular/core';",'core.mergeApplicationConfig'],
];
export const ownerFiles=[0,1,2,3].map(i=>`merge-result-${i}.ts`);
const source=(i,args=i===0?'Base,Base':'Base,Feature')=>`${forms[i][0]} import type {ApplicationConfig} from '@angular/core'; import {leafConfig as Base} from './merge-base-${i%2}'; import {leafConfig as Feature} from './merge-feature-${i%2}'; export const appConfig:ApplicationConfig=${forms[i][1]}(${args});`;
const caller=i=>`import {bootstrapApplication} from '@angular/platform-browser'; import {Root} from './boot-root-${i%2}'; import {appConfig} from './merge-result-${i}'; bootstrapApplication(Root,appConfig);`;
export const fixture={...base,'node_modules/@angular/core/index.d.ts':core,
 'merge-api.ts':"export {mergeApplicationConfig as combine} from '@angular/core';",
 'merge-fake.ts':'export function mergeApplicationConfig(...configs:unknown[]){return {};}',
 ...Object.fromEntries([0,1].flatMap(i=>['base','feature'].map(part=>[`merge-${part}-${i}.ts`,`import type {ApplicationConfig} from '@angular/core'; export const leafConfig:ApplicationConfig={providers:[]};`]))),
 ...Object.fromEntries(ownerFiles.map((file,i)=>[file,source(i)])),
 ...Object.fromEntries([1,2,3].map(i=>[`merge-caller-${i}.ts`,caller(i)])),
};
for(const file of Object.keys(fixture))if(file.startsWith('boot-main-')||file==='boot-refused.ts')delete fixture[file];
const normalize=file=>{const value=file.replaceAll('\\','/');return process.platform==='win32'?value.toLowerCase():value;};
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 let revision=0;
 const resultHashes=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const acquire=(call,file)=>call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
 const checkpoint=async call=>{for(const f of ownerFiles){await acquire(call,f);await call('save_context',{filePath:path.join(root,f)});}};
 const facts=call=>async(options={})=>{
  const {noRole=[],noBase=[],noFeature=[],noCaller=[0],mainEmpty=false,edited=[]}=options;
  const key=f=>normalize(path.join(root,f));const wanted=[];
  for(let i=0;i<4;i++) {
   if(!noRole.includes(i)) {
    if(!noBase.includes(i)&&!(i===0&&mainEmpty))wanted.push([key(ownerFiles[i]),'IncludesApplicationConfig',key(`merge-base-${i%2}.ts`)]);
    if(i!==0&&!noFeature.includes(i))wanted.push([key(ownerFiles[i]),'IncludesApplicationConfig',key(`merge-feature-${i%2}.ts`)]);
   }
   if(!noCaller.includes(i)) {
    wanted.push([key(`merge-caller-${i}.ts`),'DeclaresBootstrapRoot',key(`boot-root-${i%2}.ts`)]);
    if(!noRole.includes(i))wanted.push([key(`merge-caller-${i}.ts`),'UsesApplicationConfig',key(ownerFiles[i])]);
   }
  }
  wanted.sort();
  for(const reverse of [false,true]) {
   const selected=[];
   const selectors=reverse?[['ApplicationConfig','leafConfig'],['ApplicationConfig','appConfig'],['Component','Root']]:[['ApplicationConfig','appConfig'],['ApplicationBootstrap','bootstrapApplication']];
   for(const[kind,name]of selectors) {
    const response=await call('workspace_query',{type:reverse?'reverse_edges':'forward_edges',domain:'angular',entity_type:kind,name});
    const edges=response.structuredContent?.edges;check(`q${revision++}/structured`,Array.isArray(edges),response);
    check(`q${revision++}/no-flattening`,!edges.some(e=>e.relation==='RegistersProvider'&&(e.subject.entity_type==='ApplicationBootstrap'||e.subject.name==='appConfig')),edges);
    selected.push(...edges.filter(e=>['IncludesApplicationConfig','DeclaresBootstrapRoot','UsesApplicationConfig'].includes(e.relation)));
   }
   const actual=selected.map(e=>[normalize(e.subject.file),e.relation,normalize(e.object.file)]).sort();
   check(`q${revision++}/exact-public-pair-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${revision++}/private-proof`,selected.every(e=>!('canonical'in e.subject)&&!('canonical'in e.object)),selected);
  }
  check(`q${revision++}/unchanged-results`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===resultHashes[i]));
 };
 await session(binary,'cold-standalone-warm-checkpoint',async({call})=>{const answer=facts(call);await answer();await answer();await checkpoint(call);});
 await session(binary,'restart-support-mutation-retirement',async({call})=>{
  for(const f of ownerFiles)await call('restore_context',{filePath:path.join(root,f)});
  const answer=facts(call);await answer();
  mutate('merge-result-0.ts',source(0,'Base'));await answer({edited:[0]});
  mutate('merge-result-0.ts',source(0,''));await answer({mainEmpty:true,edited:[0]});
  mutate('merge-result-0.ts',fixture['merge-result-0.ts']);await answer();
  mutate('merge-base-0.ts','export const leafConfig={providers:[]};');await answer({noBase:[0,2]});
  mutate('merge-base-0.ts',fixture['merge-base-0.ts']);await answer();
  mutate('merge-caller-1.ts',null);await call('refresh_workspace',{});await answer({noCaller:[0,1]});
  mutate('merge-caller-1.ts',fixture['merge-caller-1.ts']);await answer();
  mutate('merge-result-1.ts',null);await call('refresh_workspace',{});await answer({noRole:[1],edited:[1]});
  mutate('merge-result-1.ts',fixture['merge-result-1.ts']);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate('merge-feature-0.ts','export const leafConfig={providers:[]};');
 await session(binary,'stale-restore-api-package-config',async({call,client})=>{
  const response=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,'merge-result-2.ts')}},'stale-before-query');
  check('stale-before-query',!!response.error||response.result?.isError===true,response);
  const answer=facts(call);await answer({noFeature:[0,2]});
  mutate('merge-feature-0.ts',fixture['merge-feature-0.ts']);await answer();
  mutate('merge-api.ts',"export {mergeApplicationConfig as combine} from './merge-fake';");await answer({noRole:[2]});
  mutate('merge-api.ts',fixture['merge-api.ts']);await answer();
  mutate('node_modules/@angular/core/package.json',JSON.stringify({name:'@angular/core',version:'20.0.0',exports:{'.':null}}));
  // The native Component role also disappears; check this phase without root rows.
  const denied=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'appConfig'});
  check('package-refuses-composition',!denied.structuredContent.edges.some(e=>e.relation==='IncludesApplicationConfig'),denied);
  mutate('node_modules/@angular/core/package.json',fixture['node_modules/@angular/core/package.json']);await answer();
  const redirected=JSON.parse(allFixture['tsconfig.json']);redirected.compilerOptions.paths={'@angular/core':['./merge-fake.ts']};mutate('tsconfig.json',JSON.stringify(redirected));
  const refused=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'appConfig'});check('config-refuses-composition',!refused.structuredContent.edges.some(e=>e.relation==='IncludesApplicationConfig'),refused);
  mutate('tsconfig.json',allFixture['tsconfig.json']);await answer();
 });
}
