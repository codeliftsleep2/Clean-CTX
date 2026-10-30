// Agent MaxHeadRoom: written lazy selection through the rebuilt binary/stdio path.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as eager,ownerFiles as eagerOwners} from './route-components.mjs';
export const ownerFiles=['apps/lazy/direct.ts','apps/lazy/reexport.ts','apps/lazy/alias.ts'];
const callback=(i,b=false)=>`()=>import('../../${i===0?'screen':i===1?'lazy-export':'lazy-alias'}-${b?'b':'a'}').then(m=>m.${i===2?'PublicScreen':'Screen'})`;
const source=(i,{repeat=true,field=true,children=true,providers=true}={})=>`import type {Routes} from '@angular/router'; import {Screen as A} from '../../screen-a'; import {Token} from '../../screen-token'; export const routes:Routes=[{path:'same',${field?`loadComponent:${callback(i)},`:''}${providers?'providers:[Token]':''}},{path:'same',loadComponent:${callback(i,true)}},${repeat?`{path:'same',loadComponent:${callback(i)}},`:''}{path:'',loadComponent:${callback(i)}},{path:'',loadComponent:${callback(i,true)}},${children?`{path:'parent',children:[{path:'child',loadComponent:${callback(i)}},{path:'child',loadComponent:${callback(i,true)}}]},`:''}{path:'eager',component:A}];`;
const base=Object.fromEntries(Object.entries(eager).filter(([f])=>!eagerOwners.includes(f)&&f!=='screen-noise.ts'));
export const fixture={...base,
 'node_modules/@angular/router/index.d.ts':eager['node_modules/@angular/router/index.d.ts']+' export interface Route {loadComponent?:unknown;loadChildren?:unknown;}',
 'lazy-export-a.ts':"export {Screen} from './screen-a';",
 'lazy-export-b.ts':"export {Screen} from './screen-b';",
 'lazy-alias-a.ts':"export {Screen as PublicScreen} from './screen-a';",
 'lazy-alias-b.ts':"export {Screen as PublicScreen} from './screen-b';",
 'lazy-child.ts':"import {Routes} from '@angular/router'; export const childRoutes:Routes=[];",
 'lazy-noise.ts':"import type {Routes} from '@angular/router'; import {Screen} from './screen-a'; export const routes:Routes=[{path:'refused',loadComponent:async()=>import('./screen-a').then(m=>m.Screen)},{path:'refused',loadComponent:()=>import('./screen-a').then(m=>m.Screen),loadComponent:()=>import('./screen-b').then(m=>m.Screen)},{path:'children',loadChildren:()=>import('./lazy-child').then(m=>m.childRoutes)}];",
 'lazy-legacy.ts':"import type {Routes} from '@angular/router'; export let routes:Routes=[{path:'legacy',loadComponent:()=>import('./screen-a').then(m=>m.Screen)}];",
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
const norm=f=>{const s=f.replaceAll('\\','/');return process.platform==='win32'?s.toLowerCase():s;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 const key=f=>norm(path.join(root,f));let serial=0;
 const originals=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const facts=call=>async({absent=[],repeat=true,field=true,children=true,noA=false,aliasMissing=false,aliasB=false,edited=[]}={})=>{
  const expected=[];
  for(let i=0;i<3;i++)if(!absent.includes(i)){
   const a=aliasB&&i===2?'screen-b.ts':'screen-a.ts';
   const validA=!(noA&&!(aliasB&&i===2))&&!(aliasMissing&&i===2);
   const add=(label,target)=>expected.push([label,key(ownerFiles[i]),key(target)]);
   if(validA&&(i!==0||field))add('same',a);
   add('same','screen-b.ts');
   if(validA&&(i!==0||repeat))add('same',a);
   if(validA)add('',a);add('','screen-b.ts');
   if(i!==0||children){if(validA)add('parent/child',a);add('parent/child','screen-b.ts');}
  }
  for(const bucket of ['same','parent/child',null]){
   const r=await call('workspace_query',{type:bucket===null?'reverse_edges':'forward_edges',domain:'angular',entity_type:bucket===null?'Component':'Route',name:bucket??'Screen'});
   const body=r.structuredContent;
   const edges=body?.edges?.filter(e=>e.relation==='RouteMapsTo'&&['same','','parent/child'].includes(e.subject.name));
   check(`q${serial++}/lower-bound`,Array.isArray(edges)&&body.coverage?.result_semantics==='lower_bound'&&body.coverage?.source_complete===false,r);
   const actual=edges.map(e=>[e.subject.name,norm(e.subject.file),norm(e.object.file)]).sort();
   const wanted=expected.filter(e=>bucket===null||e[0]===bucket).sort();
   check(`q${serial++}/exact-lazy-occurrence-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${serial++}/native-target-private`,edges.every(e=>e.object.name==='Screen'&&e.object.entity_type==='Component'&&!('canonical'in e.subject)&&!('canonical'in e.object)),edges);
  }
  for(const [name,relation,wanted] of [['refused','RouteMapsTo',0],['children','LoadsChildren',1],['legacy','RouteMapsTo',noA?0:1],['eager','RouteMapsTo',noA?0:3-absent.length]]){
   const r=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name});
   check(`q${serial++}/${name}-preservation`,r.structuredContent?.edges?.filter(e=>e.relation===relation).length===wanted,r);
  }
  check(`q${serial++}/unchanged-callers`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===originals[i]));
 };
 const checkpoint=async call=>{for(const file of ownerFiles){await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});await call('save_context',{filePath:path.join(root,file)});}};
 await session(binary,'cold-warm-scope-checkpoint',async({call})=>{
  const answer=facts(call);await answer();await answer();
  const r=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'same',withinPath:path.join(root,ownerFiles[0])});
  check('answers-only-withinPath',r.structuredContent.edges.filter(e=>e.relation==='RouteMapsTo').length===3,r);
  await answer();await checkpoint(call);
 });
 await session(binary,'restart-object-field-child-provider-root-lifecycle',async({call})=>{
  for(const file of ownerFiles)await call('restore_context',{filePath:path.join(root,file)});
  const answer=facts(call);await answer();
  for(const [options,expected] of [[{repeat:false},{repeat:false}],[{field:false},{field:false}],[{children:false},{children:false}],[{providers:false},{}]]){
   mutate(ownerFiles[0],source(0,options));await answer({...expected,edited:[0]});
  }
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[1],null);await call('refresh_workspace',{});await answer({absent:[1],edited:[1]});
  mutate(ownerFiles[1],fixture[ownerFiles[1]]);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate('screen-a.ts','export class Screen {}');
 await session(binary,'stale-role-export-package-config-repair',async({call,client})=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[0])}},'stale-lazy-restore');
  check('stale-restore-before-query',!!denied.error||denied.result?.isError===true,denied);
  const answer=facts(call);await answer({noA:true});
  mutate('screen-a.ts',fixture['screen-a.ts']);await answer();
  mutate('lazy-alias-a.ts','export {};');await answer({aliasMissing:true});
  mutate('lazy-alias-a.ts',"export {Screen as PublicScreen} from './screen-b';");await answer({aliasB:true});
  mutate('lazy-alias-a.ts',fixture['lazy-alias-a.ts']);await answer();
  const manifest=JSON.parse(fixture['node_modules/@angular/core/package.json']);manifest.version='20.0.1';
  mutate('node_modules/@angular/core/package.json',JSON.stringify(manifest));await answer();
  mutate('node_modules/@angular/core/package.json',fixture['node_modules/@angular/core/package.json']);await answer();
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.baseUrl='.';
  mutate('tsconfig.json',JSON.stringify(config));await answer();
  mutate('tsconfig.json',allFixture['tsconfig.json']);await call('refresh_workspace',{});await answer();
 });
}
