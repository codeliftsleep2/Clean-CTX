// Agent MaxHeadRoom: one bounded generation-77 binary checkpoint, not Rust/CI.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
export const ownerFiles=['direct','alias','barrel','namespace','local-a','local-b'].map(f=>`apps/guards/${f}.ts`);
const forms=[
 ["import {Gate} from '../../guard-a'; import {Gate as Right} from '../../guard-b';",'Gate','Right'],
 ["import {Gate as A} from '../../guard-a'; import {Gate as B} from '../../guard-b';",'A','B'],
 ["import {A,B} from '../../guard-api';",'A','B'],
 ["import * as left from '../../guard-a'; import * as right from '../../guard-b';",'left.Gate','right.Gate'],
 ["function Gate(){return true;} function RightGate(){return false;}",'Gate','RightGate'],
 ["function Gate(){return true;} function RightGate(){return false;}",'Gate','RightGate'],
];
const kinds=[['canActivate','GuardedBy'],['canMatch','GuardedByCanMatch'],['canDeactivate','GuardedByCanDeactivate'],['canLoad','GuardedByCanLoad'],['canActivateChild','GuardedByCanActivateChild']];
const fields=refs=>kinds.map(([field])=>`${field}:[${refs}],`).join('');
const source=(i,{slot=true,object=true,field=true,children=true,providers=true}={})=>{
 const [imports,a,b]=forms[i];
 return `import type {Routes} from '@angular/router'; import {Screen} from '../../screen'; import {Token} from '../../token'; ${imports} export const routes:Routes=[{path:'same',${kinds.filter(([name])=>field||name!=='canMatch').map(([name])=>`${name}:[${a},${b}${slot?`,${a}`:''}],`).join('')}${providers?'providers:[Token]':''}},{path:'same',${fields(b)}},${object?`{path:'same',${fields(a)}},`:''}{path:'',${fields(a)}},{path:'',${fields(b)}},${children?`{path:'parent',children:[{path:'child',${fields(a)}},{path:'child',${fields(b)}}]},`:''}{path:'controls',component:Screen,loadComponent:()=>import('../../lazy-screen').then(m=>m.Screen),loadChildren:()=>import('../../child').then(m=>m.routes)}];`;
};
export const fixture={
 'node_modules/@angular/router/package.json':JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'}),
 'node_modules/@angular/router/index.d.ts':'export interface Route {path?:string;component?:unknown;providers?:unknown[];children?:Route[];canActivate?:unknown[];canMatch?:unknown[];canDeactivate?:unknown[];canLoad?:unknown[];canActivateChild?:unknown[];loadComponent?:unknown;loadChildren?:unknown;} export type Routes=Route[];',
 'guard-a.ts':'export const Gate=()=>true;',
 'guard-b.ts':'export const Gate=()=>false;',
 'guard-api.ts':"export {Gate as A} from './guard-a'; export {Gate as B} from './guard-b';",
 'screen.ts':"import {Component} from '@angular/core'; @Component({}) export class Screen {}",
 'lazy-screen.ts':"import {Component} from '@angular/core'; @Component({}) export class Screen {}",
 'token.ts':'export class Token {}',
 'child.ts':"import type {Routes} from '@angular/router'; export const routes:Routes=[];",
 'guard-values.ts':'export const Numeric=123; export class Plain {} export function Ordinary(){return true;}',
 'guard-noise.ts':`import type {Routes} from '@angular/router'; import {Gate as A} from './guard-a'; import {Gate as B} from './guard-b'; import {Numeric,Plain,Ordinary} from './guard-values'; import type {Gate as T} from './guard-b'; export const routes:Routes=[{path:'partial',${fields('Numeric,Plain,Ordinary,Missing,T')}},{path:'duplicate',canMatch:[A],canMatch:[B],${kinds.filter(([f])=>f!=='canMatch').map(([f])=>`${f}:[B],`).join('')}},{path:'mixed',${fields('B')}}];`,
 'guard-legacy.ts':`import type {Routes} from '@angular/router'; import {Gate} from './guard-b'; export let routes:Routes=[{path:'legacy',${fields('Gate')}}];`,
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
const norm=f=>{const v=f.replaceAll('\\','/');return process.platform==='win32'?v.toLowerCase():v;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 const key=f=>norm(path.join(root,f));let serial=0;
 const originals=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const facts=call=>async({absent=[],object=true,field=true,children=true,providers=true,noA=false,barrelMissing=false,barrelB=false,edited=[]}={})=>{
  const expected=[];
  for(let i=0;i<ownerFiles.length;i++)if(!absent.includes(i)){
   const local=i>=4;
   const a=[local?ownerFiles[i]:barrelB&&i===2?'guard-b.ts':'guard-a.ts','Gate'];
   const b=[local?ownerFiles[i]:'guard-b.ts',local?'RightGate':'Gate'];
   const validA=!(noA&&!local)&&!(barrelMissing&&i===2);
   const add=(label,targets,first=false)=>{
    for(const [fieldName,relation] of kinds){
     if(first&&i===0&&!field&&fieldName==='canMatch')continue;
    const distinct=new Set();
    for(const t of targets){if(t===a&&!validA)continue;const tuple=[relation,label,key(ownerFiles[i]),key(t[0]),t[1]];const id=JSON.stringify(tuple);if(!distinct.has(id)){distinct.add(id);expected.push(tuple);}}
    }
   };
   add('same',[a,b,a],true);add('same',[b]);if(i!==0||object)add('same',[a]);
   add('',[a]);add('',[b]);if(i!==0||children){add('parent/child',[a]);add('parent/child',[b]);}
  }
  let same;
  for(const bucket of ['same','parent/child',null]){
   const answers=[];
   for(const name of bucket===null?['Gate','RightGate']:[bucket])answers.push(await call('workspace_query',{type:bucket===null?'reverse_edges':'forward_edges',domain:'angular',entity_type:bucket===null?'Guard':'Route',name}));
   if(bucket==='same')same=answers[0];
   const edges=answers.flatMap(r=>r.structuredContent?.edges??[]).filter(e=>kinds.some(([,r])=>r===e.relation)&&['same','','parent/child'].includes(e.subject.name));
   check(`g${serial++}/lower-bound`,answers.every(r=>Array.isArray(r.structuredContent?.edges)&&r.structuredContent.coverage?.result_semantics==='lower_bound'&&r.structuredContent.coverage?.source_complete===false),answers);
   const actual=edges.map(e=>[e.relation,e.subject.name,norm(e.subject.file),norm(e.object.file),e.object.name]).sort();
   const wanted=expected.filter(e=>bucket===null||e[1]===bucket).sort();
   check(`g${serial++}/exact-support-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`g${serial++}/written-Guard-private-proof`,edges.every(e=>e.object.entity_type==='Guard'&&!('canonical'in e.subject)&&!('canonical'in e.object)),edges);
  }
  for(const [name,counts] of [['partial',kinds.map(([,r])=>[r,3])],['duplicate',kinds.map(([f,r])=>[r,f==='canMatch'?0:1])],['mixed',kinds.map(([,r])=>[r,1])],['legacy',kinds.map(([,r])=>[r,r==='GuardedBy'?1:0])],['controls',[['RouteMapsTo',2*(6-absent.length)],['LoadsChildren',6-absent.length]]],['same',[['RegistersProvider',6-absent.length-(providers?0:1)]]]]){
   const r=name==='same'?same:await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name});
   check(`g${serial++}/${name}-independent-relations`,Array.isArray(r.structuredContent?.edges)&&counts.every(([relation,wanted])=>r.structuredContent.edges.filter(e=>e.relation===relation).length===wanted),{answer:r,counts});
  }
  check(`g${serial++}/unchanged-callers`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===originals[i]));
 };
 const checkpoint=async call=>{for(const file of ownerFiles){await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});await call('save_context',{filePath:path.join(root,file)});}};
 await session(binary,'guard-cold-warm-scope-save',async({call})=>{
  const answer=facts(call);await answer();await answer();
  const scoped=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'same',withinPath:path.join(root,ownerFiles[0])});
  check('guard-answers-only-withinPath',kinds.every(([,relation])=>scoped.structuredContent?.edges?.filter(e=>e.relation===relation).length===4),scoped);
  await answer();await checkpoint(call);
 });
 await session(binary,'guard-restore-slot-object-field-child-provider',async({call})=>{
  for(const file of ownerFiles)await call('restore_context',{filePath:path.join(root,file)});
  const answer=facts(call);await answer();
  for(const options of [{slot:false},{object:false},{field:false},{children:false},{providers:false}]){
   mutate(ownerFiles[0],source(0,options));await answer({...options,edited:[0]});
  }
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[1],null);await call('refresh_workspace',{});await answer({absent:[1],edited:[1]});
  mutate(ownerFiles[1],fixture[ownerFiles[1]]);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate('guard-a.ts','export const Other=1;');
 await session(binary,'guard-stale-target-barrel-package-config-repair',async({call,client})=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[0])}},'guard-stale-restore');
  check('guard-stale-restore-before-query',!!denied.error||denied.result?.isError===true,denied);
  const answer=facts(call);await answer({noA:true});
  mutate('guard-a.ts','export const Gate=42;');await answer();
  mutate('guard-a.ts',fixture['guard-a.ts']);
  mutate('guard-api.ts',"export {Gate as B} from './guard-b';");await answer({barrelMissing:true});
  mutate('guard-api.ts',"export {Gate as A} from './guard-b'; export {Gate as B} from './guard-b';");await answer({barrelB:true});
  mutate('guard-api.ts',fixture['guard-api.ts']);await answer();
  const manifest=JSON.parse(fixture['node_modules/@angular/router/package.json']);manifest.version='20.0.1';mutate('node_modules/@angular/router/package.json',JSON.stringify(manifest));await answer();
  mutate('node_modules/@angular/router/package.json',fixture['node_modules/@angular/router/package.json']);
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.baseUrl='.';mutate('tsconfig.json',JSON.stringify(config));await answer();
  mutate('tsconfig.json',allFixture['tsconfig.json']);await call('refresh_workspace',{});await answer();
 });
}
