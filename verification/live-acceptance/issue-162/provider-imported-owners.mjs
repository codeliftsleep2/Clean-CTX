// Agent MaxHeadRoom: declared structure only; no injector/runtime closure claim.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';

const kinds = [['Component','Component'], ['Directive','Directive'], ['NgModule','Module']];
export const ownerFiles = kinds.flatMap(([api]) => [0,1,2].map(n => `owner-${api}-${n}.ts`));
const group = `import {Token as Left} from './composition-left';
import {Token as Right} from './composition-right';
export const GROUP=[{provide:Left,useValue:1},[{provide:Right,useValue:2},{provide:Left,useValue:3}]];
export const ALIAS=GROUP;`;
const owner = (api, n = 0) => `import {${api} as Owner} from '@angular/core';
import {GROUP as Direct} from './composition-group';
import {PublicGroup as Barrel} from './composition-barrel';
import * as groups from './composition-group';
import {Local} from './composition-local';
const NESTED=[...[${['Direct','Barrel','groups.ALIAS'][n]}]];
@Owner({providers:[Local,...NESTED],viewProviders:[Local,...NESTED]})
export class CompositionHost {}`;
export const fixture = {
  'composition-left.ts':'export class Token {}',
  'composition-right.ts':'export class Token {}',
  'composition-local.ts':'export class Local {}',
  'composition-group.ts':group,
  'composition-barrel.ts':"export {GROUP as PublicGroup} from './composition-group';",
  'composition-foreign.ts':'export declare function Owner(metadata:unknown):any;',
  ...Object.fromEntries(kinds.flatMap(([api]) => [0,1,2].map(n => [`owner-${api}-${n}.ts`,owner(api,n)]))),
};
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const normalized = file => {
  const value = file.replace(/^\\\\\?\\/, '').replaceAll('\\','/');
  return process.platform === 'win32' ? value.toLowerCase() : value;
};

export async function lifecycle({session,binary,root,mutate,check}) {
  const originalHashes = Object.fromEntries(ownerFiles.map(file => [file,digest(Buffer.from(fixture[file]))]));
  // Each state uses three queries, one per actual public owner type. All three
  // same-named source owners must survive independently in each exact multiset.
  const assertions = (call, phase) => async (options = {}) => {
    const {imported=true,right=true,importedAbsent=[],absent=[],ordinaryAbsent=[],viewAbsent=[],edited=[]} = options;
    for (const [api,type] of kinds) {
      const response = await call('workspace_query', {type:'forward_edges',domain:'angular',
        entity_type:type,name:'CompositionHost'});
      const edges = response.structuredContent?.edges;
      check(`${phase}/${api}/structured`,Array.isArray(edges),response);
      const registrations = edges.filter(edge => ['RegistersProvider','RegistersViewProvider'].includes(edge.relation));
      const actual = registrations.map(edge => [edge.relation,edge.subject.entity_type,
        normalized(edge.subject.file),edge.object.name,normalized(edge.object.file)]).sort();
      const wanted = [];
      for (const n of [0,1,2]) {
        const file = `owner-${api}-${n}.ts`;
        if (absent.includes(file)) continue;
        for (const relation of ['RegistersProvider',...(api === 'Component' ? ['RegistersViewProvider'] : [])]) {
          if ((relation === 'RegistersProvider' ? ordinaryAbsent : viewAbsent).includes(file)) continue;
          const tokens = [['Local','composition-local.ts'],...(imported && !importedAbsent.includes(file) ? [['Token','composition-left.ts'],
            ...(right ? [['Token','composition-right.ts']] : [])] : [])];
          for (const [token,source] of tokens) wanted.push([relation,type,normalized(path.join(root,file)),
            token,normalized(path.join(root,source))]);
        }
      }
      wanted.sort();
      check(`${phase}/${api}/exact-position-owner-token-multiset`,JSON.stringify(actual) === JSON.stringify(wanted),{actual,wanted});
      check(`${phase}/${api}/public-provenance`,registrations.every(edge =>
        edge.subject.domain === 'angular' && edge.object.domain === 'angular'
        && edge.subject.name === 'CompositionHost' && !('canonical' in edge.subject) && !('canonical' in edge.object)),registrations);
    }
    check(`${phase}/unchanged-owners`,ownerFiles.filter(file => !edited.includes(file)).every(file =>
      digest(fs.readFileSync(path.join(root,file))) === originalHashes[file]));
  };
  const checkpoint = async call => {
    for (const file of ownerFiles) {
      await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
      await call('save_context',{filePath:path.join(root,file)});
    }
  };
  await session(binary,'cold-warm-checkpoint',async ({call}) => {
    const facts = assertions(call,'cold-warm');
    for (const file of ownerFiles) await call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
    await facts(); await facts(); await checkpoint(call);
  });
  await session(binary,'restart-mutation-deletion',async ({call}) => {
    for (const file of ownerFiles) await call('restore_context',{filePath:path.join(root,file)});
    const facts = assertions(call,'restart-mutations');
    await facts();
    // Shared contributor mutations must refresh all nine unchanged owners.
    mutate('composition-barrel.ts',"export type {GROUP as PublicGroup} from './composition-group';");
    await facts({importedAbsent:kinds.map(([api]) => `owner-${api}-1.ts`)});
    mutate('composition-barrel.ts',fixture['composition-barrel.ts']); await facts();
    mutate('composition-right.ts','export interface Token {}'); await facts({right:false});
    mutate('composition-right.ts',fixture['composition-right.ts']); await facts();
    mutate('composition-group.ts',group.replace('const GROUP=','let GROUP=')); await facts({imported:false});
    mutate('composition-group.ts',group); await facts();
    mutate('composition-group.ts',group.replace('const GROUP=[','const GROUP=[...GROUP,')); await facts();
    mutate('composition-group.ts',null); await facts({imported:false});
    mutate('composition-group.ts',group); await facts();
    for (const [api] of kinds) {
      const file = `owner-${api}-0.ts`;
      mutate(file,owner(api).replace('providers:[Local,...NESTED]','providers:[]'));
      await facts({ordinaryAbsent:[file],edited:[file]});
      mutate(file,owner(api));
    }
    const component = 'owner-Component-0.ts';
    mutate(component,owner('Component').replace('viewProviders:[Local,...NESTED]','viewProviders:[]'));
    await facts({viewAbsent:[component],edited:[component]});
    mutate(component,owner('Component')); await facts();
    for (const [api] of kinds) {
      const file = `owner-${api}-0.ts`;
      mutate(file,owner(api).replace("from '@angular/core'","from './composition-foreign'"));
      await facts({absent:[file],edited:[file]});
      mutate(file,owner(api));
      mutate(file,null); await facts({absent:[file],edited:[file]});
      mutate(file,owner(api));
    }
    await facts(); await checkpoint(call);
  });
  // Contributor changes before startup: restore refusal precedes query repair.
  mutate('composition-group.ts',group.replace('const GROUP=','let GROUP='));
  await session(binary,'stale-restore-refusal',async ({call,client}) => {
    for (const file of ownerFiles) {
      const response = await client.request('tools/call',{name:'restore_context',arguments:{
        workspaceRoot:root,filePath:path.join(root,file)}},`stale-restore/${file}`);
      check(`stale-restore/${file}`,!!response.error || response.result?.isError === true,response);
    }
    const facts = assertions(call,'stale-repair');
    await facts({imported:false});
    mutate('composition-group.ts',group); await facts();
    const redirected = {compilerOptions:{module:'esnext',moduleResolution:'bundler',moduleDetection:'legacy',noLib:true,types:[],
      paths:{'@angular/core':['./composition-foreign.ts']}}};
    mutate('tsconfig.json',JSON.stringify(redirected)); await facts({absent:ownerFiles});
    mutate('tsconfig.json',JSON.stringify({compilerOptions:{module:'esnext',moduleResolution:'bundler',moduleDetection:'legacy',noLib:true,types:[]}}));
    await facts();
    mutate('node_modules/@angular/core/package.json',JSON.stringify({name:'foreign-core',version:'20.0.0',types:'index.d.ts'}));
    await facts({absent:ownerFiles});
    mutate('node_modules/@angular/core/package.json',JSON.stringify({name:'@angular/core',version:'20.0.0',types:'index.d.ts'}));
    await facts();
  });
}
