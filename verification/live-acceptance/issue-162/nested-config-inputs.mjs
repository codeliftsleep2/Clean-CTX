// Agent MaxHeadRoom: direct nested config pairs and written cycles, never evaluation.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as base} from './config-composition.mjs';
const forms=[
 ["import {mergeApplicationConfig} from '@angular/core';",'mergeApplicationConfig'],
 ["import {mergeApplicationConfig as combine} from '@angular/core';",'combine'],
 ["import {combine} from './merge-api';",'combine'],
 ["import * as core from '@angular/core';",'core.mergeApplicationConfig'],
];
const leaf=i=>`nested-leaf-${i}.ts`;
const inner=i=>`nested-inner-${i}.ts`;
const outer=i=>`nested-outer-${i}.ts`;
const innerSource=(i,args='Base')=>`${forms[i][0]} import type {ApplicationConfig} from '@angular/core'; import {leafConfig as Base} from './nested-leaf-${i}'; import {nestedConfig as Outer} from './nested-outer-${i}'; export const innerConfig:ApplicationConfig=${forms[i][1]}(${args});`;
const outerSource=(i,args='Inner,Inner,Base')=>`${forms[i][0]} import type {ApplicationConfig} from '@angular/core'; import {innerConfig as Inner} from './nested-input-${i}'; import {leafConfig as Base} from './nested-leaf-${i}'; export const nestedConfig:ApplicationConfig=${forms[i][1]}(${args});`;
export const ownerFiles=[...forms.flatMap((_,i)=>[inner(i),outer(i)]),'nested-self.ts'];
export const fixture={...base,
 ...Object.fromEntries(forms.flatMap((_,i)=>[
  [leaf(i),"import type {ApplicationConfig} from '@angular/core'; export const leafConfig:ApplicationConfig={providers:[]};"],
  [`nested-input-${i}.ts`,`export {innerConfig} from './nested-inner-${i}';`],
  [inner(i),innerSource(i,i===3?'Outer,Base':'Base')],
  [outer(i),outerSource(i)],
 ])),
 'nested-self.ts':"import {mergeApplicationConfig} from '@angular/core'; import type {ApplicationConfig} from '@angular/core'; export const selfConfig:ApplicationConfig=mergeApplicationConfig(selfConfig);",
};
// This suite has no bootstrap caller: acquisition must discover real config owners.
for(const f of Object.keys(fixture))if(f.startsWith('merge-result-')||f.startsWith('merge-caller-'))delete fixture[f];
const norm=f=>{const s=f.replaceAll('\\','/');return process.platform==='win32'?s.toLowerCase():s;};
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 let sequence=0;
 const initialHashes=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const key=f=>norm(path.join(root,f));
 const checkpoint=async call=>{for(const f of ownerFiles){await call('compress_code_context',{filePath:path.join(root,f),fidelity:'high'});await call('save_context',{filePath:path.join(root,f)});}};
 const facts=call=>async({noInner=[],noOuter=[],noLeaf=[],cycle=true,edited=[]}={})=>{
  const wanted=[];
  for(let i=0;i<4;i++) {
   if(!noInner.includes(i)) {
    if(!noLeaf.includes(i))wanted.push([key(inner(i)),key(leaf(i))]);
    if(i===3&&cycle&&!noOuter.includes(i))wanted.push([key(inner(i)),key(outer(i))]);
   }
   if(!noOuter.includes(i)) {
    if(!noInner.includes(i))wanted.push([key(outer(i)),key(inner(i))]);
    if(!noLeaf.includes(i))wanted.push([key(outer(i)),key(leaf(i))]);
   }
  }
  wanted.push([key('nested-self.ts'),key('nested-self.ts')]);wanted.sort();
  for(const reverse of [false,true]) {
   const selected=[];
   for(const name of reverse?['nestedConfig','innerConfig','selfConfig','leafConfig']:['nestedConfig','innerConfig','selfConfig']) {
    const response=await call('workspace_query',{type:reverse?'reverse_edges':'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name});
    const result=response.structuredContent;const edges=result?.edges;
    check(`q${sequence++}/structured`,Array.isArray(edges),response);
    check(`q${sequence++}/lower-bound-no-inheritance`,result.coverage?.result_semantics==='lower_bound'&&!edges.some(e=>e.relation==='RegistersProvider'&&ownerFiles.some(f=>key(f)===norm(e.subject.file))),result);
    selected.push(...edges.filter(e=>e.relation==='IncludesApplicationConfig'&&ownerFiles.some(f=>key(f)===norm(e.subject.file))));
   }
   const actual=selected.map(e=>[norm(e.subject.file),norm(e.object.file)]).sort();
   check(`q${sequence++}/exact-direct-pair-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${sequence++}/public-identity`,selected.every(e=>e.subject.entity_type==='ApplicationConfig'&&e.object.entity_type==='ApplicationConfig'&&!('canonical'in e.subject)&&!('canonical'in e.object)),selected);
  }
  check(`q${sequence++}/unchanged-owners`,ownerFiles.every((f,i)=>edited.includes(f)||hash(fs.readFileSync(path.join(root,f)))===initialHashes[i]));
 };
 await session(binary,'cold-cycles-warm-checkpoint',async({call})=>{const answer=facts(call);await answer();await answer();await checkpoint(call);});
 await session(binary,'restart-support-cycles-retirement',async({call})=>{
  for(const f of ownerFiles)await call('restore_context',{filePath:path.join(root,f)});
  const answer=facts(call);await answer();
  mutate(outer(0),outerSource(0,'Inner,Base'));await answer({edited:[outer(0)]});
  mutate(outer(0),outerSource(0,'Base'));
  // Role remains independently valid; only the written outer-to-inner pair retires.
  const response=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'nestedConfig'});
  const pairs=response.structuredContent.edges.filter(e=>norm(e.subject.file)===key(outer(0))&&e.relation==='IncludesApplicationConfig');
  check('final-support-removal',pairs.length===1&&norm(pairs[0].object.file)===key(leaf(0)),pairs);
  mutate(outer(0),fixture[outer(0)]);await answer();
  mutate(inner(3),innerSource(3));await answer({cycle:false,edited:[inner(3)]});
  mutate(inner(3),fixture[inner(3)]);await answer();
  mutate(inner(1),null);await call('refresh_workspace',{});await answer({noInner:[1],edited:[inner(1)]});
  mutate(inner(1),fixture[inner(1)]);await call('refresh_workspace',{});await answer();
  mutate(outer(1),null);await call('refresh_workspace',{});await answer({noOuter:[1],edited:[outer(1)]});
  mutate(outer(1),fixture[outer(1)]);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate(inner(0),"export const innerConfig={providers:[]};");
 await session(binary,'stale-restore-role-barrel-package-repair',async({call,client})=>{
  const restored=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,outer(0))}},'stale-before-query');
  check('stale-before-query',!!restored.error||restored.result?.isError===true,restored);
  const answer=facts(call);await answer({noInner:[0],edited:[inner(0)]});
  mutate(inner(0),fixture[inner(0)]);await answer();
  mutate('nested-input-2.ts',"export {leafConfig as innerConfig} from './nested-leaf-2';");
  // Retargeting an unchanged owner to an object config deduplicates its Base support.
  const changed=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'nestedConfig'});
  const rows=changed.structuredContent.edges.filter(e=>e.relation==='IncludesApplicationConfig'&&norm(e.subject.file)===key(outer(2)));
  check('barrel-retarget-exact',rows.length===1&&norm(rows[0].object.file)===key(leaf(2)),rows);
  mutate('nested-input-2.ts',fixture['nested-input-2.ts']);await answer();
  mutate('merge-api.ts',"export {mergeApplicationConfig as combine} from './merge-fake';");await answer({noInner:[2],noOuter:[2]});
  mutate('merge-api.ts',fixture['merge-api.ts']);await answer();
  mutate('node_modules/@angular/core/package.json',JSON.stringify({name:'@angular/core',version:'20.0.0',exports:{'.':null}}));
  const denied=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'nestedConfig'});
  check('package-refusal',!denied.structuredContent.edges.some(e=>e.relation==='IncludesApplicationConfig'),denied);
  mutate('node_modules/@angular/core/package.json',fixture['node_modules/@angular/core/package.json']);await answer();
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.paths={'@angular/core':['./merge-fake.ts']};mutate('tsconfig.json',JSON.stringify(config));
  const refused=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'ApplicationConfig',name:'nestedConfig'});
  check('config-refusal',!refused.structuredContent.edges.some(e=>e.relation==='IncludesApplicationConfig'),refused);
  mutate('tsconfig.json',allFixture['tsconfig.json']);await answer();
 });
}
