// Agent MaxHeadRoom: exact Module/RouteConfig selection, separate bounded receipts.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as eager} from './route-components.mjs';
export const ownerFiles=['apps/children/direct.ts','apps/children/reexport.ts','apps/children/alias.ts'];
const callback=(i,b=false)=>`()=>import('../../${i===0?'feature':i===1?'children-export':'children-alias'}-${b?'b':'a'}').then(m=>m.${i===2?'PublicScreen':'Screen'})`;
const source=(i,{repeat=true,field=true,children=true,providers=true}={})=>`import type {Routes} from '@angular/router'; import {Screen as A} from '../../screen-a'; import {Token} from '../../screen-token'; import {Guard,Resolver} from '../../route-hooks'; export const routes:Routes=[{path:'same',canActivate:[Guard],resolve:{data:Resolver},${field?`loadChildren:${callback(i)},`:''}${providers?'providers:[Token]':''}},{path:'same',loadChildren:${callback(i,true)}},${repeat?`{path:'same',loadChildren:${callback(i)}},`:''}{path:'',loadChildren:${callback(i)}},{path:'',loadChildren:${callback(i,true)}},${children?`{path:'parent',children:[{path:'child',loadChildren:${callback(i)}},{path:'child',loadChildren:${callback(i,true)}}]},`:''}{path:'eager',component:A},{path:'component-lazy',loadComponent:()=>import('../../screen-b').then(m=>m.Screen)}];`;
// Retain the actual dependency closure used by these role-specific assertions.
// Bootstrap/DI/HTTP/provider preservation has separate suites and Rust evidence.
const requiredBase=new Set(['node_modules/@angular/core/package.json','node_modules/@angular/core/index.d.ts','node_modules/@angular/router/package.json','node_modules/@angular/router/index.d.ts','screen-a.ts','screen-b.ts','screen-token.ts']);
const base=Object.fromEntries(Object.entries(eager).filter(([f])=>requiredBase.has(f)));
export function scenario(role) {
 if(!['module','routes'].includes(role))throw new Error('explicit target role required');
 const targetType=role==='module'?'Module':'RouteConfig';
 const target=role==='module'?"import {NgModule} from '@angular/core'; @NgModule({}) export class Screen {}":"import {Routes} from '@angular/router'; export const Screen:Routes=[];";
 const fixture={...base,
 'route-hooks.ts':"export function Guard(){return true;} export function Resolver(){return null;}",
 'feature-a.ts':target,
 'feature-b.ts':target,
 'node_modules/@angular/router/index.d.ts':eager['node_modules/@angular/router/index.d.ts']+' export interface Route {loadComponent?:unknown;loadChildren?:unknown;}',
 'children-export-a.ts':"export {Screen} from './feature-a';",
 'children-export-b.ts':"export {Screen} from './feature-b';",
 'children-alias-a.ts':"export {Screen as PublicScreen} from './feature-a';",
 'children-alias-b.ts':"export {Screen as PublicScreen} from './feature-b';",
 'lazy-child.ts':"import {Routes} from '@angular/router'; export const childRoutes:Routes=[];",
 'children-noise.ts':"import type {Routes} from '@angular/router'; import {Screen} from './screen-a'; export const routes:Routes=[{path:'refused',loadChildren:async()=>import('./feature-a').then(m=>m.Screen)},{path:'refused',loadChildren:()=>import('./feature-a').then(m=>m.Screen),loadChildren:()=>import('./feature-b').then(m=>m.Screen)},{path:'children',loadChildren:()=>import('./lazy-child').then(m=>m.childRoutes)}];",
 'children-legacy.ts':"import type {Routes} from '@angular/router'; export let routes:Routes=[{path:'legacy',loadChildren:()=>import('./feature-a').then(m=>m.Screen)}];",
 'foreign-router.ts':"export type Routes=unknown[];",
 'children-foreign.ts':"import {Component} from '@angular/core'; @Component({}) class Anchor {} import type {Route as NativeRoute} from '@angular/router'; import type {Routes} from './foreign-router'; import {Screen as Eager} from './screen-a'; export const routes:Routes=[{path:'foreign-legacy',loadChildren:()=>import('./feature-a').then(m=>m.Screen)},{path:'foreign-eager',component:Eager}];",
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
const norm=f=>{const s=f.replaceAll('\\','/');return process.platform==='win32'?s.toLowerCase():s;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 const key=f=>norm(path.join(root,f));let serial=0;
 const originals=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const facts=call=>async({absent=[],repeat=true,field=true,children=true,providers=true,noA=false,aliasMissing=false,aliasB=false,edited=[]}={})=>{
  const expected=[];
  for(let i=0;i<3;i++)if(!absent.includes(i)){
   const a=aliasB&&i===2?'feature-b.ts':'feature-a.ts';
   const validA=!(noA&&!(aliasB&&i===2))&&!(aliasMissing&&i===2);
   const add=(label,target)=>expected.push([label,key(ownerFiles[i]),key(target)]);
   if(validA&&(i!==0||field))add('same',a);
   add('same','feature-b.ts');
   if(validA&&(i!==0||repeat))add('same',a);
   if(validA)add('',a);add('','feature-b.ts');
   if(i!==0||children){if(validA)add('parent/child',a);add('parent/child','feature-b.ts');}
  }
  let sameResponse;
  for(const bucket of ['same','parent/child',null]){
   const r=await call('workspace_query',{type:bucket===null?'reverse_edges':'forward_edges',domain:'angular',entity_type:bucket===null?targetType:'Route',name:bucket??'Screen'});
   if(bucket==='same')sameResponse=r;
   const body=r.structuredContent;
   const edges=body?.edges?.filter(e=>e.relation==='LoadsChildren'&&['same','','parent/child'].includes(e.subject.name));
   check(`q${serial++}/lower-bound`,Array.isArray(edges)&&body.coverage?.result_semantics==='lower_bound'&&body.coverage?.source_complete===false,r);
   const actual=edges.map(e=>[e.subject.name,norm(e.subject.file),norm(e.object.file)]).sort();
   const wanted=expected.filter(e=>bucket===null||e[0]===bucket).sort();
   check(`q${serial++}/exact-children-occurrence-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${serial++}/native-target-private`,edges.every(e=>e.object.name==='Screen'&&e.object.entity_type===targetType&&!('canonical'in e.subject)&&!('canonical'in e.object)),edges);
  }
  for(const [name,relation,wanted] of [['refused','LoadsChildren',0],['children','LoadsChildren',1],['legacy','LoadsChildren',noA?0:1],['foreign-legacy','LoadsChildren',noA?0:1],['eager','RouteMapsTo',3-absent.length],['component-lazy','RouteMapsTo',3-absent.length],['same','RegistersProvider',3-absent.length-(providers?0:1)],['same','GuardedBy',3-absent.length],['same','ResolvedBy',3-absent.length]]){
   // These checks consume the identical fresh same-bucket response above.
   // No cross-batch/session cache, retry or timeout change is introduced.
   const r=name==='same'?sameResponse:await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name});
   check(`q${serial++}/${name}-preservation`,r.structuredContent?.edges?.filter(e=>e.relation===relation).length===wanted,r);
  }
  check(`q${serial++}/unchanged-callers`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===originals[i]));
 };
 const checkpoint=async call=>{for(const file of ownerFiles){await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});await call('save_context',{filePath:path.join(root,file)});}};
 await session(binary,'cold-warm-scope-checkpoint',async({call})=>{
  const answer=facts(call);await answer();await answer();
  const r=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'same',withinPath:path.join(root,ownerFiles[0])});
  check('answers-only-withinPath',r.structuredContent.edges.filter(e=>e.relation==='LoadsChildren').length===3,r);
  await answer();await checkpoint(call);
 });
 await session(binary,'restart-object-field-child-provider-root-lifecycle',async({call})=>{
  for(const file of ownerFiles)await call('restore_context',{filePath:path.join(root,file)});
  const answer=facts(call);await answer();
  for(const [options,expected] of [[{repeat:false},{repeat:false}],[{field:false},{field:false}],[{children:false},{children:false}],[{providers:false},{providers:false}]]){
   mutate(ownerFiles[0],source(0,options));await answer({...expected,edited:[0]});
  }
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[1],null);await call('refresh_workspace',{});await answer({absent:[1],edited:[1]});
  mutate(ownerFiles[1],fixture[ownerFiles[1]]);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate('feature-a.ts','export class Screen {}');
 await session(binary,'stale-role-export-package-config-repair',async({call,client})=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[0])}},'stale-children-restore');
  check('stale-restore-before-query',!!denied.error||denied.result?.isError===true,denied);
  const answer=facts(call);await answer({noA:true});
  mutate('feature-a.ts',fixture['feature-a.ts']);await answer();
  mutate('children-alias-a.ts','export {};');await answer({aliasMissing:true});
  mutate('children-alias-a.ts',"export {Screen as PublicScreen} from './feature-b';");await answer({aliasB:true});
  mutate('children-alias-a.ts',fixture['children-alias-a.ts']);await answer();
  const manifest=JSON.parse(fixture['node_modules/@angular/core/package.json']);manifest.version='20.0.1';
  mutate('node_modules/@angular/core/package.json',JSON.stringify(manifest));await answer();
  mutate('node_modules/@angular/core/package.json',fixture['node_modules/@angular/core/package.json']);await answer();
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.baseUrl='.';
  mutate('tsconfig.json',JSON.stringify(config));await answer();
  mutate('tsconfig.json',allFixture['tsconfig.json']);await call('refresh_workspace',{});await answer();
 });
}

 return {fixture,lifecycle,ownerFiles};
}
