// Agent MaxHeadRoom: route-declared membership, not activation or injector scope.
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';

export const ownerFiles = ['route-owner.ts','route-neighbor.ts'];
const group = "import {Token as Left} from './route-left'; import {Token as Right} from './route-right'; export const GROUP=[Left,[{provide:Right,useValue:1},Left]];";
const source = namespace => `import type {Routes as NativeRoutes} from '@angular/router';
import {Token as Left} from './route-left'; import {Token as Right} from './route-right';
${namespace ? "import * as groups from './route-barrel';" : "import {PublicGroup as Imported} from './route-barrel';"}
export const routes:NativeRoutes=[
 {path:'probe',providers:[...[${namespace ? 'groups.PublicGroup' : 'Imported'}]],children:[{path:'child',providers:[Right]}]},
 {path:'probe',providers:[Left,Left]},
 {path:'',providers:[Right]},
 {path:'',providers:[Right]}];`;
const manifest = JSON.stringify({name:'@angular/router',version:'20.0.0',types:'index.d.ts'});
export const fixture = {
  'node_modules/@angular/router/package.json':manifest,
  'node_modules/@angular/router/index.d.ts':'export type Routes=unknown[];',
  'route-left.ts':'export class Token {}', 'route-right.ts':'export class Token {}',
  'route-group.ts':group,
  'route-barrel.ts':"export {GROUP as PublicGroup} from './route-group';",
  'route-fake.ts':'export type Routes=unknown[];',
  'route-owner.ts':source(false),'route-neighbor.ts':source(true),
};
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const normalize = file => {
  const value = file.replace(/^\\\\\?\\/,'').replaceAll('\\','/');
  return process.platform === 'win32' ? value.toLowerCase() : value;
};

export async function lifecycle({session,binary,root,fixture:allFixture,mutate,check}) {
  const ownerHashes = Object.fromEntries(ownerFiles.map(file => [file,hash(Buffer.from(fixture[file]))]));
  let revision = 0;
  const facts = (call,phase) => async (options={}) => {
    const {groupEnabled=true,right=true,absent=[],fieldRemoved=[],routeRemoved=[],emptyRemoved=[],edited=[]} = options;
    const rows = [];
    for (const file of ownerFiles) {
      if (absent.includes(file)) continue;
      const put = (route,token) => rows.push([route,normalize(path.join(root,file)),normalize(path.join(root,`route-${token}.ts`))]);
      if (!fieldRemoved.includes(file) && groupEnabled) {
        put('probe','left'); if (right) put('probe','right');
      }
      if (!routeRemoved.includes(file)) put('probe','left');
      if (right) {
        put('probe/child','right');put('','right');
        if (!emptyRemoved.includes(file)) put('','right');
      }
    }
    for (const name of ['probe','probe/child',null]) {
      const reverse = name === null;
      const response = await call('workspace_query',{type:reverse ? 'reverse_edges':'forward_edges',
        domain:'angular',entity_type:reverse ? 'ProviderToken':'Route',name:reverse ? 'Token':name});
      const edges = response.structuredContent?.edges;
      const label = `${phase}/${revision++}/${reverse ? 'reverse-including-empty' : name}`;
      check(`${label}/structured`,Array.isArray(edges),response);
      const selected = edges.filter(e => e.relation === 'RegistersProvider' && e.subject.entity_type === 'Route');
      const actual = selected.map(e => [e.subject.name,normalize(e.subject.file),normalize(e.object.file)]).sort();
      const expected = rows.filter(row => reverse || row[0] === name).sort();
      check(`${label}/exact-occurrence-multiset`,JSON.stringify(actual) === JSON.stringify(expected),{actual,expected});
      check(`${label}/public-provenance`,selected.every(e => e.subject.domain === 'angular'
        && e.object.domain === 'angular' && e.object.entity_type === 'ProviderToken' && e.object.name === 'Token'
        && !('canonical' in e.subject) && !('canonical' in e.object)),selected);
    }
    check(`${phase}/unchanged-owners`,ownerFiles.filter(file => !edited.includes(file)).every(file =>
      hash(fs.readFileSync(path.join(root,file))) === ownerHashes[file]));
  };
  const acquire = async (call,file) => call('compress_code_context',{filePath:path.join(root,file),fidelity:'high'});
  const checkpoint = async call => {
    for (const file of ownerFiles) {
      await acquire(call,file); await call('save_context',{filePath:path.join(root,file)});
    }
  };
  await session(binary,'cold-warm-checkpoint',async ({call}) => {
    for (const file of ownerFiles) await acquire(call,file);
    const answer = facts(call,'cold-warm'); await answer(); await answer(); await checkpoint(call);
  });
  await session(binary,'restart-mutation-deletion',async ({call}) => {
    for (const file of ownerFiles) await call('restore_context',{filePath:path.join(root,file)});
    const answer = facts(call,'restart-mutations'); await answer();
    // Dependency repairs keep both owner sources byte-identical and query only.
    mutate('route-right.ts','export interface Token {}'); await answer({right:false});
    mutate('route-right.ts',fixture['route-right.ts']); await answer();
    mutate('route-barrel.ts',"export type {GROUP as PublicGroup} from './route-group';"); await answer({groupEnabled:false});
    mutate('route-barrel.ts',fixture['route-barrel.ts']); await answer();
    mutate('route-group.ts',group.replace('const GROUP=','let GROUP=')); await answer({groupEnabled:false});
    mutate('route-group.ts',group.replace('const GROUP=[','const GROUP=[...GROUP,')); await answer();
    mutate('route-group.ts',null); await answer({groupEnabled:false});
    mutate('route-group.ts',group); await answer();
    const file = ownerFiles[0], original = fixture[file];
    const field = 'providers:[...[Imported]]';
    if (!original.includes(field)) throw new Error('fixture providers marker missing');
    mutate(file,original.replace(field,'providers:[]')); await answer({fieldRemoved:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer();
    mutate(file,original.replace(" {path:'probe',providers:[Left,Left]},",''));
    await answer({routeRemoved:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer();
    mutate(file,original.replace(" {path:'',providers:[Right]},",''));
    await answer({emptyRemoved:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer();
    mutate(file,original.replace(':NativeRoutes',':FakeRoutes')+' type FakeRoutes=unknown[];');
    await answer({absent:[file],edited:[file]});
    mutate(file,original); await acquire(call,file); await answer();
    mutate(file,"export const routes=[];"); await answer({absent:[file],edited:[file]});
    mutate(file,null); await call('refresh_workspace',{}); await answer({absent:[file],edited:[file]});
    // Explicit reacquisition after owner/root recreation is deliberate: no claim
    // of automatic discovery of a removed route identity is made by this suite.
    mutate(file,original); await acquire(call,file); await answer(); await checkpoint(call);
  });
  mutate('route-group.ts',group.replace('const GROUP=','let GROUP='));
  await session(binary,'stale-restore-refusal',async ({call,client}) => {
    for (const file of ownerFiles) {
      const response = await client.request('tools/call',{name:'restore_context',arguments:{workspaceRoot:root,
        filePath:path.join(root,file)}},`stale-restore/${file}`);
      check(`stale-restore/${file}`,!!response.error || response.result?.isError === true,response);
    }
    const answer = facts(call,'stale-repair'); await answer({groupEnabled:false});
    mutate('route-group.ts',group); await answer();
    const redirected = JSON.parse(allFixture['tsconfig.json']);
    redirected.compilerOptions.paths = {'@angular/router':['./route-fake.ts']};
    mutate('tsconfig.json',JSON.stringify(redirected)); await answer({absent:ownerFiles});
    mutate('tsconfig.json',allFixture['tsconfig.json']); await answer();
    mutate('node_modules/@angular/router/package.json',JSON.stringify({name:'@angular/router',version:'20.0.0',exports:{'.':null}}));
    await answer({absent:ownerFiles});
    mutate('node_modules/@angular/router/package.json',manifest); await answer();
  });
}
