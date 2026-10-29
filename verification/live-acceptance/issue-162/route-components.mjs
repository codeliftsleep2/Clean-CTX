// Agent MaxHeadRoom: ecological written eager membership, never route activation.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as base} from './legacy-bootstrap.mjs';
export const ownerFiles=['apps/routes/direct.ts','apps/routes/alias.ts','apps/routes/barrel.ts','apps/routes/namespace.ts','apps/routes/local.ts'];
const forms=[
 ["import {Screen} from '@route-screen';",'Screen'],
 ["import {Screen as A} from '../../screen-a';",'A'],
 ["import {A} from '../../screen-api';",'A'],
 ["import * as targets from '../../screen-a';",'targets.Screen'],
 ["import {Component} from '@angular/core'; @Component({}) class Screen {}",'Screen'],
];
const source=(i,{repeat=true,component=true,providers=true}={})=>`${forms[i][0]} import type {Routes} from '@angular/router'; import {Screen as B} from '../../screen-b'; import {Token} from '../../screen-token'; export const routes:Routes=[{path:'same',${component?`component:${forms[i][1]},`:''}${providers?'providers:[Token]':''}},{path:'same',component:B},${repeat?`{path:'same',component:${forms[i][1]}},`:''}{path:'',component:${forms[i][1]}},{path:'',component:B},{path:'parent',children:[{path:'child',component:${forms[i][1]}},{path:'child',component:B}]}];`;
export const fixture={...base,
 'node_modules/@angular/router/package.json':JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'}),
 'node_modules/@angular/router/index.d.ts':'export interface Route {path?:string;component?:unknown;providers?:unknown[];children?:Route[];} export type Routes=Route[];',
 'screen-a.ts':"import {Component} from '@angular/core'; @Component({}) export class Screen {}",
 'screen-b.ts':"import {Component} from '@angular/core'; @Component({}) export class Screen {}",
 'screen-token.ts':'export class Token {}',
 'screen-api.ts':"export {Screen as A} from './screen-a';",
 'screen-lazy.ts':"import {Component} from '@angular/core'; @Component({}) export class LazyScreen {}",
 'screen-noise.ts':"import type {Routes} from '@angular/router'; import {Screen as A} from './screen-a'; import {Screen as B} from './screen-b'; import {Token} from './screen-token'; const extra={}; export const routes:Routes=[{path:'refused',component:A,component:B,providers:[Token]},{path:'refused',component:Token},{path:'refused',component:Missing},{path:'lazy',loadComponent:()=>import('./screen-lazy').then(m=>m.LazyScreen)}];",
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
const norm=f=>{const value=f.replaceAll('\\','/');return process.platform==='win32'?value.toLowerCase():value;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.paths={...(config.compilerOptions.paths??{}),'@route-screen':['./screen-a.ts']};
 const configBytes=JSON.stringify(config);mutate('tsconfig.json',configBytes);
 const key=f=>norm(path.join(root,f));let serial=0;
 const hashes=ownerFiles.map(f=>hash(Buffer.from(fixture[f])));
 const facts=call=>async({absent=[],repeat=true,component=true,noA=false,barrel=false,directB=false,edited=[]}={})=>{
  const expected=[];
  for(let i=0;i<5;i++)if(!absent.includes(i)){
   const a=i===4?ownerFiles[i]:(i===0&&directB?'screen-b.ts':'screen-a.ts');
   const validA=!(noA&&i!==4)&&!(barrel&&i===2);
   if(validA&&!(i===0&&!component))expected.push(['same',key(ownerFiles[i]),key(a)]);
   expected.push(['same',key(ownerFiles[i]),key('screen-b.ts')]);
   if(validA&&(i!==0||repeat))expected.push(['same',key(ownerFiles[i]),key(a)]);
   if(validA)expected.push(['',key(ownerFiles[i]),key(a)],['parent/child',key(ownerFiles[i]),key(a)]);
   expected.push(['',key(ownerFiles[i]),key('screen-b.ts')],['parent/child',key(ownerFiles[i]),key('screen-b.ts')]);
  }
  for(const bucket of ['same','parent/child',null]){
   const result=await call('workspace_query',{type:bucket===null?'reverse_edges':'forward_edges',domain:'angular',entity_type:bucket===null?'Component':'Route',name:bucket??'Screen'});
   const body=result.structuredContent;const edges=body?.edges?.filter(e=>e.relation==='RouteMapsTo');
   check(`q${serial++}/structured-lower-bound`,Array.isArray(edges)&&body.coverage?.result_semantics==='lower_bound'&&body.coverage?.source_complete===false&&body.coverage?.omitted_possible===true,result);
   const actual=edges.map(e=>[e.subject.name,norm(e.subject.file),norm(e.object.file)]).sort();
   const wanted=expected.filter(e=>bucket===null||e[0]===bucket).sort();
   check(`q${serial++}/exact-forward-reverse-occurrence-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${serial++}/native-target-private-proof`,edges.every(e=>e.object.name==='Screen'&&e.object.entity_type==='Component'&&!('canonical'in e.subject)&&!('canonical'in e.object)),edges);
  }
  const refused=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'refused'});
  check(`q${serial++}/owned-refusal-no-weak-row`,!refused.structuredContent.edges.some(e=>e.relation==='RouteMapsTo'),refused);
  const lazy=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'lazy'});
  check(`q${serial++}/registered-lazy-preserved`,lazy.structuredContent.edges.filter(e=>e.relation==='RouteMapsTo'&&e.object.name==='LazyScreen').length===1,lazy);
  check(`q${serial++}/unchanged-owners`,ownerFiles.every((f,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,f)))===hashes[i]));
 };
 const checkpoint=async call=>{for(const file of ownerFiles){await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});await call('save_context',{filePath:path.join(root,file)});}};
 await session(binary,'cold-warm-paths-checkpoint',async({call})=>{
  const answer=facts(call);await answer();await answer();
  const scoped=await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'same',withinPath:path.join(root,ownerFiles[0])});
  check('answers-only-withinPath',scoped.structuredContent.edges.filter(e=>e.relation==='RouteMapsTo').length===3,scoped);
  await answer();await checkpoint(call);
 });
 await session(binary,'restart-field-provider-object-root-lifecycle',async({call})=>{
  for(const file of ownerFiles)await call('restore_context',{filePath:path.join(root,file)});
  const answer=facts(call);await answer();
  mutate(ownerFiles[0],source(0,{repeat:false}));await answer({repeat:false,edited:[0]});
  mutate(ownerFiles[0],source(0,{component:false}));await answer({component:false,edited:[0]});
  mutate(ownerFiles[0],source(0,{providers:false}));await answer({edited:[0]});
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[1],null);await call('refresh_workspace',{});await answer({absent:[1],edited:[1]});
  mutate(ownerFiles[1],fixture[ownerFiles[1]]);await call('refresh_workspace',{});await answer();await checkpoint(call);
 });
 mutate('screen-a.ts','export class Screen {}');
 await session(binary,'stale-target-barrel-package-config-repair',async({call,client})=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[0])}},'stale-route-restore');
  check('stale-restore-before-query',!!denied.error||denied.result?.isError===true,denied);
  const answer=facts(call);await answer({noA:true});
  mutate('screen-a.ts',fixture['screen-a.ts']);await answer();
  mutate('screen-api.ts','export {};');await answer({barrel:true});
  mutate('screen-api.ts',fixture['screen-api.ts']);await answer();
  const manifest=JSON.parse(allFixture['node_modules/@angular/core/package.json']);manifest.version='20.0.1';
  mutate('node_modules/@angular/core/package.json',JSON.stringify(manifest));await answer();
  mutate('node_modules/@angular/core/package.json',allFixture['node_modules/@angular/core/package.json']);await answer();
  config.compilerOptions.paths['@route-screen']=['./screen-b.ts'];mutate('tsconfig.json',JSON.stringify(config));await answer({directB:true});
  mutate('tsconfig.json',configBytes);await answer();
 });
}
