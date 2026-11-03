// Agent MaxHeadRoom: generation-78 unqualified written resolver use, not execution.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
export const ownerFiles=['direct','alias','barrel','namespace','local-a','local-b'].map(n=>`apps/resolvers/${n}.ts`);
const forms=[
 ["import {Gate} from '../../resolver-a';import {Gate as Right} from '../../resolver-b';",'Gate','Right'],
 ["import {Gate as A} from '../../resolver-a';import {Gate as B} from '../../resolver-b';",'A','B'],
 ["import {A,B} from '../../resolver-api';",'A','B'],
 ["import * as left from '../../resolver-a';import * as right from '../../resolver-b';",'left.Gate','right.Gate'],
 ['function Gate(){return true;}function RightGate(){return false;}', 'Gate','RightGate'],
 ['function Gate(){return true;}function RightGate(){return false;}', 'Gate','RightGate'],
];
const guards=['canActivate','canMatch','canDeactivate','canLoad','canActivateChild'];
const guardRelations=['GuardedBy','GuardedByCanMatch','GuardedByCanDeactivate','GuardedByCanLoad','GuardedByCanActivateChild'];
export function source(i,{field=true,object=true,root=true}={}){
 const [imports,a,b]=forms[i];
 const prefix=`import type {Routes} from '@angular/router';import {Screen} from '../../resolver-screen';import {RouteToken} from '../../resolver-token';${imports}`;
 if(!root)return `${prefix} export const routes:Routes=[];`;
 return `${prefix} export const routes:Routes=[{path:'same',${field?`resolve:{first:${a},second:${b},again:${a}},`:''}providers:[RouteToken]},{path:'same',resolve:{first:${b}}},${object?`{path:'same',resolve:{first:${a}}},`:''}{path:'',resolve:{first:${a}}},{path:'',resolve:{first:${b}}},{path:'parent',children:[{path:'child',resolve:{first:${a}}},{path:'child',resolve:{first:${b}}}]},{path:'controls',component:Screen,loadComponent:()=>import('../../resolver-lazy').then(m=>m.Screen),loadChildren:()=>import('../../resolver-child').then(m=>m.routes),${guards.map(f=>`${f}:[${b}]`).join(',')}}];`;
}
const core='export declare function Component(metadata:unknown):any;export declare function NgModule(metadata:unknown):any;export declare function Directive(metadata:unknown):any;export declare function Injectable(metadata?:unknown):any;export interface ApplicationConfig {providers:unknown[];}export declare function mergeApplicationConfig(...configs:ApplicationConfig[]):ApplicationConfig;';
export const fixture={
 'node_modules/@angular/core/index.d.ts':core,
 'node_modules/@angular/router/package.json':JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'}),
 'node_modules/@angular/router/index.d.ts':'export interface Route {path?:string;component?:unknown;resolve?:unknown;providers?:unknown[];children?:Route[];loadComponent?:unknown;loadChildren?:unknown;canActivate?:unknown[];canMatch?:unknown[];canDeactivate?:unknown[];canLoad?:unknown[];canActivateChild?:unknown[];}export type Routes=Route[];',
 'node_modules/@angular/platform-browser/package.json':JSON.stringify({name:'@angular/platform-browser',version:'20.0.0',types:'index.d.ts'}),
 'node_modules/@angular/platform-browser/index.d.ts':'export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;',
 'resolver-a.ts':'export const Gate=()=>true;',
 'resolver-b.ts':'export const Gate=()=>false;',
 'resolver-api.ts':"export {Gate as A} from './resolver-a';export {Gate as B} from './resolver-b';",
 'resolver-ambiguous.ts':"export * from './resolver-a';export * from './resolver-b';",
 'resolver-token.ts':'export class RouteToken {}',
 'resolver-screen.ts':"import {Component} from '@angular/core';@Component({}) export class Screen {}",
 'resolver-lazy.ts':"import {Component} from '@angular/core';@Component({}) export class Screen {}",
 'resolver-child.ts':"import type {Routes} from '@angular/router';export const routes:Routes=[];",
 'resolver-policy.ts':"import type {Routes} from '@angular/router';import {Gate} from './resolver-a';import {Gate as Right} from './resolver-b';import type {Gate as T} from './resolver-b';import {Gate as Ambiguous} from './resolver-ambiguous';export const routes:Routes=[{path:'partial',resolve:{good:Gate,missing:Missing,typeonly:T,ambiguous:Ambiguous,callback:()=>Right}},{path:'duplicate-key',resolve:{dup:Gate,'d\\u0075p':Right,good:Right}},{path:'duplicate-outer',resolve:{first:Gate},resolve:{second:Right}},{path:'refused',resolve:{missing:Missing}}];",
 'resolver-legacy.ts':"import type {Routes} from '@angular/router';import {Gate} from './resolver-b';export let routes:Routes=[{path:'legacy',resolve:{first:Gate}}];",
 'delegate-tokens.ts':'export class OwnerToken {} export class OwnerTarget {} export class ViewToken {} export class ViewTarget {} export class ConfigToken {} export class ConfigTarget {} export class BootToken {} export class BootTarget {}',
 'delegate-groups.ts':"import {OwnerToken,OwnerTarget,ViewToken,ViewTarget,ConfigToken,ConfigTarget,BootToken,BootTarget} from './delegate-tokens';export const ORDINARY=[{provide:OwnerToken,useClass:OwnerTarget}];export const VIEW=[{provide:ViewToken,useClass:ViewTarget}];export const CONFIG=[{provide:ConfigToken,useClass:ConfigTarget}];export const BOOT=[{provide:BootToken,useClass:BootTarget}];",
 'delegate-owner.ts':"import {Component} from '@angular/core';import {ORDINARY,VIEW} from './delegate-groups';@Component({providers:[...ORDINARY],viewProviders:[...VIEW]}) export class DelegateHost {}",
 'delegate-config.ts':"import type {ApplicationConfig} from '@angular/core';import {CONFIG} from './delegate-groups';export const delegateConfig:ApplicationConfig={providers:[...CONFIG]};",
 'delegate-merge.ts':"import {mergeApplicationConfig} from '@angular/core';import type {ApplicationConfig} from '@angular/core';import {delegateConfig} from './delegate-config';export const delegateMerged:ApplicationConfig=mergeApplicationConfig(delegateConfig,delegateConfig);",
 'delegate-boot.ts':"import {bootstrapApplication} from '@angular/platform-browser';import {DelegateHost} from './delegate-owner';import {delegateConfig} from './delegate-config';import {BOOT} from './delegate-groups';bootstrapApplication(DelegateHost,{providers:[...BOOT]});bootstrapApplication(DelegateHost,delegateConfig);",
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
export const requiredScenarios=[
 'cold/exact','cold/repeat','cold/siblings','cold/delegated','cold/save',
 'restore/before-query','restore/exact','restore/policies','restore/repeat',
 'current/stale-target','current/target-written-value','current/target-repair',
 'current/export-missing','current/export-repair','current/barrel-remap','current/barrel-repair',
 'current/package-stale','current/package-repair','current/config-stale','current/config-repair',
 'current/field-retirement','current/object-retirement','current/root-retirement','current/repair',
 'current/siblings','current/delegated',
];
const norm=f=>{const n=f.replaceAll('\\','/');return process.platform==='win32'?n.toLowerCase():n;};
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}){
 const key=f=>norm(path.join(root,f));let revision=0;const done=new Set();
 const scenario=async(name,action)=>{if(!requiredScenarios.includes(name)||done.has(name))throw new Error(`invalid/duplicate scenario ${name}`);await action();done.add(name);check(`scenario/${name}`,true);};
 const owners=[...ownerFiles,'resolver-policy.ts','resolver-legacy.ts','delegate-owner.ts','delegate-config.ts','delegate-merge.ts','delegate-boot.ts'];
 const query=(call,type,entity_type,name)=>call('workspace_query',{type,domain:'angular',entity_type,name});
 const rows=(answer)=>{check(`r${revision++}/structured`,Array.isArray(answer.structuredContent?.edges),answer);return answer.structuredContent.edges;};
 const exact=(name,actual,wanted)=>check(name,JSON.stringify(actual.sort())===JSON.stringify(wanted.sort()),{actual,wanted});
 const tuples=edges=>edges.map(e=>[e.subject.name,norm(e.subject.file),norm(e.object.file),e.object.name]);
 const expected=({field=true,object=true,root:hasRoot=true,noA=false,barrelMissing=false,barrelB=false}={})=>{
  const out=[];
  for(let i=0;i<ownerFiles.length;i++){
   if(i===0&&!hasRoot)continue;
   const local=i>=4;
   const a=[local?ownerFiles[i]:i===2&&barrelB?'resolver-b.ts':'resolver-a.ts','Gate'];
   const b=[local?ownerFiles[i]:'resolver-b.ts',local?'RightGate':'Gate'];
   const validA=!(noA&&!local)&&!(barrelMissing&&i===2);
   const add=(label,refs)=>{
    const seen=new Set();for(const ref of refs){if(ref===a&&!validA)continue;const identity=JSON.stringify(ref);if(seen.has(identity))continue;seen.add(identity);out.push([label,key(ownerFiles[i]),key(ref[0]),ref[1]]);}
   };
   if(i!==0||field)add('same',[a,b,a]);add('same',[b]);if(i!==0||object)add('same',[a]);
   add('',[a]);add('',[b]);add('parent/child',[a]);add('parent/child',[b]);
  }
  return out;
 };
 const facts=async(call,options={})=>{
  const wanted=expected(options);
  for(const name of ['same','parent/child']){
   const answer=await query(call,'forward_edges','Route',name);const edges=rows(answer).filter(e=>e.relation==='ResolvedBy');
   exact(`r${revision++}/forward-${name}`,tuples(edges),wanted.filter(e=>e[0]===name));
   check(`r${revision++}/written-use-private-proof`,edges.every(e=>e.object.entity_type==='Resolver'&&!('canonical' in e.subject)&&!('canonical' in e.object)),edges);
  }
  const reverse=[];for(const name of ['Gate','RightGate'])reverse.push(...rows(await query(call,'reverse_edges','Resolver',name)).filter(e=>e.relation==='ResolvedBy'&&ownerFiles.some(f=>norm(e.subject.file)===key(f))));
  exact(`r${revision++}/reverse-including-empty`,tuples(reverse),wanted);
  check(`r${revision++}/unchanged-independent-callers`,ownerFiles.every((file,i)=>i===0&&options.edited?true:hash(fs.readFileSync(path.join(root,file)))===hash(Buffer.from(fixture[file]))));
 };
 const siblings=async(call,active=6)=>{
  const controls=rows(await query(call,'forward_edges','Route','controls'));
  for(const [relation,count] of [['RouteMapsTo',2*active],['LoadsChildren',active],...guardRelations.map(r=>[r,active])]){
   check(`sibling/${relation}/${revision++}`,controls.filter(e=>e.relation===relation).length===count,controls);
  }
  const same=rows(await query(call,'forward_edges','Route','same'));
  check(`sibling/providers/${revision++}`,same.filter(e=>e.relation==='RegistersProvider').length===active,same);
 };
 const checkpoint=async call=>{for(const file of owners){await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});await call('save_context',{filePath:path.join(root,file)});}};
 const delegated=async call=>{
  for(const file of ['delegate-owner.ts','delegate-config.ts','delegate-merge.ts','delegate-boot.ts'])await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
  for(const [kind,name,relation,owner,target,count] of [
   ['Component','DelegateHost','RegistersProvider','delegate-owner.ts','OwnerToken',1],
   ['Component','DelegateHost','RegistersViewProvider','delegate-owner.ts','ViewToken',1],
   ['ApplicationConfig','delegateConfig','RegistersProvider','delegate-config.ts','ConfigToken',1],
   ['ApplicationBootstrap','bootstrapApplication','RegistersProvider','delegate-boot.ts','BootToken',1],
   ['ApplicationBootstrap','bootstrapApplication','DeclaresBootstrapRoot','delegate-boot.ts','DelegateHost',2],
   ['ApplicationBootstrap','bootstrapApplication','UsesApplicationConfig','delegate-boot.ts','delegateConfig',1],
   ['ApplicationConfig','delegateMerged','IncludesApplicationConfig','delegate-merge.ts','delegateConfig',1],
  ]){
   const edges=rows(await query(call,'forward_edges',kind,name)).filter(e=>e.relation===relation&&norm(e.subject.file)===key(owner));
   const targetFile=target==='DelegateHost'?'delegate-owner.ts':target==='delegateConfig'?'delegate-config.ts':'delegate-tokens.ts';
   check(`delegated/${kind}/${relation}/${revision++}`,edges.length===count&&edges.every(e=>e.object.name===target&&norm(e.object.file)===key(targetFile)),edges);
  }
  for(const [token,target] of [['OwnerToken','OwnerTarget'],['ViewToken','ViewTarget'],['ConfigToken','ConfigTarget'],['BootToken','BootTarget']]){
   const edges=rows(await query(call,'forward_edges','ProviderToken',token)).filter(e=>e.relation==='ProviderUsesClass'&&norm(e.subject.file)===key('delegate-tokens.ts')&&norm(e.object.file)===key('delegate-tokens.ts'));
   check(`delegated/exact-use-${token}`,edges.length===1&&edges[0].object.name===target,edges);
  }
 };
 const policies=async call=>{
  for(const [name,target,file] of [['partial','Gate','resolver-a.ts'],['duplicate-key','Gate','resolver-b.ts'],['duplicate-outer',null,null],['refused',null,null]]){
   const edges=rows(await query(call,'forward_edges','Route',name)).filter(e=>e.relation==='ResolvedBy');
   exact(`policy/${name}/${target?'positive':'expected-refusal'}`,tuples(edges),target?[[name,key('resolver-policy.ts'),key(file),target]]:[]);
  }
  const legacy=rows(await query(call,'forward_edges','Route','legacy')).filter(e=>e.relation==='ResolvedBy');
  check('policy/outside-envelope-legacy',legacy.length===1&&legacy[0].object.name==='Gate'&&norm(legacy[0].subject.file)===key('resolver-legacy.ts'),legacy);
 };
 const stale=async(client,file,label)=>{
  const denied=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,file)}},label);
  check(`expected-refusal/${label}`,denied.error?.code===-32603&&denied.error?.data?.component==='semantic_dependencies'&&denied.error?.data?.reason==='semantic_dependency_stale',denied);
 };
 await session(binary,'resolver78-cold-exposure',async({call})=>{
  await scenario('cold/exact',()=>facts(call));await scenario('cold/repeat',()=>facts(call));
  await scenario('cold/siblings',()=>siblings(call));await scenario('cold/delegated',()=>delegated(call));await scenario('cold/save',()=>checkpoint(call));
 });
 await session(binary,'resolver78-fresh-durable-restore',async({call})=>{
  await scenario('restore/before-query',async()=>{for(const file of owners)await call('restore_context',{filePath:path.join(root,file)});});
  await scenario('restore/exact',()=>facts(call));await scenario('restore/policies',()=>policies(call));await scenario('restore/repeat',()=>facts(call));
  await checkpoint(call);
 });
 mutate('resolver-a.ts','export const Gone=1;');
 await session(binary,'resolver78-currentness-retirement',async({call,client})=>{
  await scenario('current/stale-target',async()=>{await stale(client,ownerFiles[0],'stale-target-before-query');await facts(call,{noA:true});});
  mutate('resolver-a.ts','export const Gate=42;');await scenario('current/target-written-value',()=>facts(call));
  mutate('resolver-a.ts',fixture['resolver-a.ts']);await scenario('current/target-repair',async()=>{await facts(call);await checkpoint(call);});
  mutate('resolver-api.ts',"export {Gate as B} from './resolver-b';");
  await scenario('current/export-missing',async()=>{await stale(client,ownerFiles[2],'stale-export');await facts(call,{barrelMissing:true});});
  mutate('resolver-api.ts',fixture['resolver-api.ts']);await scenario('current/export-repair',async()=>{await facts(call);await checkpoint(call);});
  mutate('resolver-api.ts',"export {Gate as A} from './resolver-b';export {Gate as B} from './resolver-b';");
  await scenario('current/barrel-remap',async()=>{await stale(client,ownerFiles[2],'stale-barrel-remap');await facts(call,{barrelB:true});});
  mutate('resolver-api.ts',fixture['resolver-api.ts']);await scenario('current/barrel-repair',async()=>{await facts(call);await checkpoint(call);});
  const manifest=JSON.parse(fixture['node_modules/@angular/router/package.json']);manifest.version='20.0.1';mutate('node_modules/@angular/router/package.json',JSON.stringify(manifest));
  await scenario('current/package-stale',async()=>{await stale(client,ownerFiles[0],'stale-package');await facts(call);});
  mutate('node_modules/@angular/router/package.json',fixture['node_modules/@angular/router/package.json']);await scenario('current/package-repair',async()=>{await facts(call);await checkpoint(call);});
  const config=JSON.parse(allFixture['tsconfig.json']);config.compilerOptions.baseUrl='.';mutate('tsconfig.json',JSON.stringify(config));
  await scenario('current/config-stale',async()=>{await stale(client,ownerFiles[0],'stale-config');await facts(call);});
  mutate('tsconfig.json',allFixture['tsconfig.json']);await scenario('current/config-repair',()=>facts(call));
  for(const [name,option] of [['field-retirement','field'],['object-retirement','object'],['root-retirement','root']]){
   mutate(ownerFiles[0],source(0,{[option]:false}));await scenario(`current/${name}`,()=>facts(call,{[option]:false,edited:true}));
  }
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await scenario('current/repair',()=>facts(call));
  await scenario('current/siblings',()=>siblings(call));await scenario('current/delegated',()=>delegated(call));
 });
 check('required-named-scenarios-complete',done.size===requiredScenarios.length&&requiredScenarios.every(name=>done.has(name)),{required:requiredScenarios,completed:[...done]});
}
