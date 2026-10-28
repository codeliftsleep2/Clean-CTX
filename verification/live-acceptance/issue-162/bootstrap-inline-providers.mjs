// Agent MaxHeadRoom: source-declared inline membership only; no effective providers.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {fixture as base} from './bootstrap-references.mjs';
export const ownerFiles = [0,1,2,3].map(i=>`inline-main-${i}.ts`);
const imports = [
 ["import {bootstrapApplication as launch} from '@angular/platform-browser';",'launch'],
 ["import {bootstrapApplication as start} from '@angular/platform-browser';",'start'],
 ["import {start} from './boot-api';",'start'],
 ["import * as platform from '@angular/platform-browser';",'platform.bootstrapApplication'],
];
imports[0]=["import {bootstrapApplication} from '@angular/platform-browser';",'bootstrapApplication'];
const repeat = i=>i===0 ? 2:1;
const source = (i,mode='full') => {
 const side=i%2, callee=imports[i][1];
 const inline=mode==='empty' ? `${callee}(Root,{providers:[]});` : `${callee}(Root,{providers:[...GROUP]});`.repeat(mode==='single' ? 1:repeat(i));
 return `${imports[i][0]} import {Root} from './boot-root-${side}'; import {appConfig} from './boot-config-${side}'; import {GROUP} from './inline-group-${side}'; ${inline}${callee}(Root,appConfig);`;
};
const group=i=>`import {Token as Left} from './inline-left-${i}'; import {Token as Right} from './inline-right-${i}'; export const GROUP=[Left,[{provide:Right,useValue:1},Left]];`;
export const fixture={...base,
 ...Object.fromEntries([0,1].flatMap(i=>[[`inline-left-${i}.ts`,'export class Token {}'],[`inline-right-${i}.ts`,'export class Token {}'],[`inline-group-${i}.ts`,group(i)]])),
 ...Object.fromEntries(ownerFiles.map((f,i)=>[f,source(i)])),
};
// Keep this suite's bootstrap rows isolated from the reference preservation fixture.
for (const file of Object.keys(fixture)) if (file.startsWith('boot-main-') || file==='boot-refused.ts') delete fixture[file];
const normalize=file=>{const value=file.replaceAll('\\','/');return process.platform==='win32'?value.toLowerCase():value;};
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
 let sequence=0;
 const originalHashes=ownerFiles.map(file=>hash(Buffer.from(fixture[file])));
 const acquire=(call,file)=>call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
 const checkpoint=async call=>{for(const file of ownerFiles){await acquire(call,file);await call('save_context',{filePath:path.join(root,file)});}};
 const facts=call=>async(options={})=>{
  const {noCalls=[],noProviders=[],noLeft=[],mainMode='full',edited=[]}=options;
  const wanted=[];
  for(let i=0;i<4;i++) {
   if(noCalls.includes(i))continue;
   const count=i===0&&mainMode!=='full'?1:repeat(i);
   const owner=normalize(path.join(root,ownerFiles[i]));
   for(let n=0;n<count+1;n++)wanted.push([owner,'DeclaresBootstrapRoot',normalize(path.join(root,`boot-root-${i%2}.ts`))]);
   wanted.push([owner,'UsesApplicationConfig',normalize(path.join(root,`boot-config-${i%2}.ts`))]);
   if(noProviders.includes(i)||(i===0&&mainMode==='empty'))continue;
   for(let n=0;n<count;n++)for(const side of ['left','right'])if(side!=='left'||!noLeft.includes(i))wanted.push([owner,'RegistersProvider',normalize(path.join(root,`inline-${side}-${i%2}.ts`))]);
  }
  wanted.sort();
  for(const reverse of [false,true]) {
   const selected=[];
   for(const [kind,name]of reverse?[['Component','Root'],['ApplicationConfig','appConfig'],['ProviderToken','Token']]:[['ApplicationBootstrap','bootstrapApplication']]) {
    const response=await call('workspace_query',{type:reverse?'reverse_edges':'forward_edges',domain:'angular',entity_type:kind,name});
    const edges=response.structuredContent?.edges;check(`q${sequence++}/structured`,Array.isArray(edges),response);
    selected.push(...edges.filter(e=>e.subject.entity_type==='ApplicationBootstrap'&&['RegistersProvider','DeclaresBootstrapRoot','UsesApplicationConfig'].includes(e.relation)));
   }
   const actual=selected.map(e=>[normalize(e.subject.file),e.relation,normalize(e.object.file)]).sort();
   check(`q${sequence++}/exact-occurrence-source-target-multiset`,JSON.stringify(actual)===JSON.stringify(wanted),{actual,wanted});
   check(`q${sequence++}/private-canonical`,selected.every(e=>!('canonical'in e.subject)&&!('canonical'in e.object)),selected);
  }
  check(`q${sequence++}/unchanged-callers`,ownerFiles.every((file,i)=>edited.includes(i)||hash(fs.readFileSync(path.join(root,file)))===originalHashes[i]));
 };
 await session(binary,'cold-warm-checkpoint',async({call})=>{const answer=facts(call);await answer();await answer();await checkpoint(call);});
 await session(binary,'restart-mutation-retirement',async({call})=>{
  for(const file of ownerFiles)await call('restore_context',{filePath:path.join(root,file)});
  const answer=facts(call);await answer();
  mutate('inline-left-0.ts','export interface Token {}');await answer({noLeft:[0,2]});
  mutate('inline-left-0.ts',fixture['inline-left-0.ts']);await answer();
  mutate('inline-group-0.ts','export const GROUP=[];');await answer({noProviders:[0,2]});
  mutate('inline-group-0.ts',group(0));await answer();
  mutate(ownerFiles[0],source(0,'single'));await answer({mainMode:'single',edited:[0]});
  mutate(ownerFiles[0],source(0,'empty'));await answer({mainMode:'empty',edited:[0]});
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();
  mutate(ownerFiles[0],null);await call('refresh_workspace',{});await answer({noCalls:[0],edited:[0]});
  mutate(ownerFiles[0],fixture[ownerFiles[0]]);await answer();await checkpoint(call);
 });
 mutate('inline-group-0.ts','export const GROUP=[];');
 await session(binary,'stale-restore-api-package-config',async({call,client})=>{
  for(const i of [0,2]){const response=await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,filePath:path.join(root,ownerFiles[i])}},`stale/${i}`);check(`stale-before-query/${i}`,!!response.error||response.result?.isError===true,response);}
  const answer=facts(call);await answer({noProviders:[0,2]});mutate('inline-group-0.ts',group(0));await answer();
  mutate('boot-api.ts',"export {bootstrapApplication as start} from './boot-fake';");await answer({noCalls:[2]});mutate('boot-api.ts',fixture['boot-api.ts']);await answer();
  mutate('node_modules/@angular/platform-browser/package.json',JSON.stringify({name:'@angular/platform-browser',version:'20.0.0',exports:{'.':null}}));await answer({noCalls:[0,1,2,3]});mutate('node_modules/@angular/platform-browser/package.json',fixture['node_modules/@angular/platform-browser/package.json']);await answer();
  const redirected=JSON.parse(allFixture['tsconfig.json']);redirected.compilerOptions.paths={'@angular/platform-browser':['./boot-fake.ts']};mutate('tsconfig.json',JSON.stringify(redirected));await answer({noCalls:[0,1,2,3]});mutate('tsconfig.json',allFixture['tsconfig.json']);await answer();
 });
}
