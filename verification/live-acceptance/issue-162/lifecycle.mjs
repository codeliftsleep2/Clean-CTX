// Agent MaxHeadRoom: shared lifecycle scenarios, selected by each bounded suite.
import path from 'node:path';
import {functionalApi, commonManifest} from './functional-scenario.mjs';
import {memberSource, memberApi, foreignMemberApi} from './member-scenario.mjs';
import {stringValues} from './string-scenario.mjs';
export async function runLifecycle({session,binary,checkpointFiles,root,fixture,mutate,leaf,check,functionalSource,positionOrdinary,positionView}, suite) {
  await session(binary, 'cold-warm-checkpoint', async ({ call, facts, injectFacts, constructorFacts, stringFacts, memberFacts, functionalFacts }) => {
    for (const file of checkpointFiles) await call('compress_code_context', { filePath: path.join(root, file), fidelity: 'high' });
    await facts(['left.ts', 'right.ts']); await facts(['left.ts', 'right.ts']);
    await injectFacts(); await injectFacts();
    await constructorFacts(); await constructorFacts();
    await stringFacts(); await stringFacts();
    await memberFacts(); await memberFacts();
    await functionalFacts(); await functionalFacts();
    for (const file of checkpointFiles) await call('save_context', { filePath: path.join(root, file) });
  });
  await session(binary, 'restart-mutation-deletion', async ({ call, facts, injectFacts, constructorFacts, stringFacts, memberFacts, functionalFacts, client }) => {
    for (const file of checkpointFiles) await call('restore_context', { filePath: path.join(root, file) });
    await functionalFacts();
    mutate('functional-right.ts', 'export interface AuditApi {}'); await functionalFacts({tokens:['functional-left.ts']});
    mutate('functional-right.ts', 'export class AuditApi {}'); await functionalFacts();
    mutate('provider-strategies.ts', "export type {Implementation,makeAudit} from './provider-implementation';");
    await functionalFacts({strategyOwners:[]});
    mutate('provider-strategies.ts', fixture['provider-strategies.ts']); await functionalFacts();
    mutate('provider-implementation.ts', 'export interface Implementation {} export function makeAudit(){return {};}');
    await functionalFacts({implementation:false});
    mutate('provider-implementation.ts', fixture['provider-implementation.ts']); await functionalFacts();
    mutate('factory-right.ts', 'export interface makeAudit {}'); await functionalFacts({factoryRight:false});
    mutate('factory-right.ts', fixture['factory-right.ts']); await functionalFacts();
    mutate('node_modules/@angular/common/package.json', commonManifest(false)); await functionalFacts({http:false});
    mutate('node_modules/@angular/common/package.json', commonManifest()); await functionalFacts();
    mutate('functional-api.ts', functionalApi('@angular/common')); await functionalFacts({http:false});
    mutate('functional-api.ts', functionalApi()); await functionalFacts();
    const redirected = JSON.parse(fixture['tsconfig.json']);
    redirected.compilerOptions.paths = {'@angular/common/http':['./functional-foreign.ts']};
    mutate('tsconfig.json', JSON.stringify(redirected)); await functionalFacts({http:false});
    mutate('tsconfig.json', fixture['tsconfig.json']); await functionalFacts();
    mutate('functional.ts', 'export const ordinary=()=>true;'); await functionalFacts({owners:['functional-neighbor.ts']});
    mutate('functional.ts', functionalSource); await functionalFacts();
    mutate('functional.ts', functionalSource.replace(positionOrdinary, 'providers:[]'));
    await functionalFacts({ordinaryPositionOwners:['functional-neighbor.ts']});
    mutate('functional.ts', functionalSource); await functionalFacts();
    mutate('functional.ts', functionalSource.replace(positionView, 'viewProviders:[]'));
    await functionalFacts({viewPositionOwners:['functional-neighbor.ts']});
    mutate('functional.ts', functionalSource); await functionalFacts();
    if (suite.group === 'composition') {
      mutate('functional.ts', functionalSource.replace('const GROUP=', 'let GROUP='));
      await functionalFacts({ordinaryPositionOwners:['functional-neighbor.ts'],viewPositionOwners:['functional-neighbor.ts']});
      mutate('functional.ts', functionalSource); await functionalFacts();
      mutate('functional.ts', functionalSource.replace('{provide:Right,useValue:102},', ''));
      await functionalFacts({positionTokensByOwner:{'functional.ts':['functional-left.ts']}});
      mutate('functional.ts', functionalSource); await functionalFacts();
      mutate('functional.ts', functionalSource.replace('const ALIAS=GROUP', 'const ALIAS=ALIAS'));
      await functionalFacts({viewPositionOwners:['functional-neighbor.ts']});
      mutate('functional.ts', functionalSource); await functionalFacts();
    }
    mutate('functional.ts', functionalSource.replace("from '@angular/core'", "from './provider-foreign'"));
    await functionalFacts({providerOwners:['functional-neighbor.ts']});
    mutate('functional.ts', functionalSource); await functionalFacts();
    await memberFacts();
    mutate('member-api.ts', foreignMemberApi); await memberFacts([]);
    mutate('member-api.ts', memberApi); await memberFacts();
    mutate('members.ts', 'export class MemberPanel {}'); await memberFacts(['member-neighbor.ts']);
    mutate('members.ts', memberSource); await memberFacts();
    await stringFacts();
    mutate('string-values.ts', stringValues('other.api')); await stringFacts('other.api');
    mutate('string-values.ts', stringValues()); await stringFacts();
    await facts(['left.ts', 'right.ts']);
    await injectFacts(); await constructorFacts();
    mutate('token-right.ts', 'export interface Dependency {}');
    await injectFacts(['token-left.ts']); await constructorFacts([], ['token-left.ts']);
    mutate('globals.d.ts', 'declare class MissingService {}');
    await injectFacts(['token-left.ts'], 'canonical');
    mutate('globals.d.ts', 'declare interface MissingService {}');
    await injectFacts(['token-left.ts'], 'none');
    mutate('globals.d.ts', 'declare interface UnrelatedNoise {}');
    await injectFacts(['token-left.ts']);
    mutate('token-right.ts', 'export class Dependency {}');
    await injectFacts(); await constructorFacts();
    mutate('constructor-api.ts', "export { Inject as SelectToken } from './constructor-foreign';");
    await constructorFacts(['token-left.ts', 'token-right.ts']); await injectFacts();
    mutate('constructor-api.ts', "export { Inject as SelectToken } from '@angular/core';");
    await constructorFacts();
    mutate('constructor-token.ts', "export type { Dependency as CtorToken } from './token-right';");
    await constructorFacts([]); await injectFacts();
    mutate('constructor-token.ts', "export { Dependency as CtorToken } from './token-right';");
    await constructorFacts();
    const mergedToken = 'export class Dependency {} export interface Dependency { marker: string; }';
    mutate('token-left.ts', mergedToken);
    await injectFacts(); await constructorFacts();
    mutate('token-left.ts', 'export class Dependency {} export class Dependency {} export interface Dependency { marker: string; }');
    await injectFacts(['token-right.ts']); await constructorFacts(['token-right.ts'], ['token-right.ts']);
    mutate('token-left.ts', mergedToken);
    await injectFacts(); await constructorFacts();
    mutate('right.ts', leaf.replace('import {', 'import type {')); await facts(['left.ts']);
    mutate('right.ts', null); await facts(['left.ts']);
    mutate('right.ts', leaf); await facts(['left.ts', 'right.ts']);
    mutate('right.ts', leaf.replace('@Dir({', '@Dir({standalone:false,')); await facts(['left.ts', 'right.ts'], ['left.ts']);
    mutate('right.ts', leaf); await facts(['left.ts', 'right.ts']);
    // File queries refresh the index; they do not replace restored session
    // snapshots. The ambient mutation must make the old checkpoint unsavable.
    if (suite.family === 'modules') {
    const staleSave = await client.request('tools/call', { name: 'save_context', arguments: {
      workspaceRoot: root, filePath: path.join(root, 'app.ts'),
    } }, 'checkpoint/refuse-stale-session');
    check('checkpoint/stale-session-refused', staleSave.error?.code === -32603
      && staleSave.error.message.includes('Missing authoritative semantic-edge state'), staleSave);
    }
    for (const file of checkpointFiles) await call('compress_code_context', { filePath: path.join(root, file), fidelity: 'high' });
    await facts(['left.ts', 'right.ts']); await injectFacts(); await constructorFacts();
    await stringFacts();
    for (const file of checkpointFiles) await call('save_context', { filePath: path.join(root, file) });
  });
  mutate('right.ts', "export class Leaf {}");
  mutate('token-right.ts', 'export interface Dependency {}');
  mutate('string-values.ts', stringValues('other.api'));
  mutate('member-api.ts', foreignMemberApi);
  mutate('node_modules/@angular/common/package.json', commonManifest(true, '20.0.1'));
  await session(binary, 'stale-restore-refusal', async ({ client, facts, injectFacts, constructorFacts, stringFacts, memberFacts, functionalFacts }) => {
    for (const file of checkpointFiles) {
      const response = await client.request('tools/call', { name: 'restore_context', arguments: { workspaceRoot: root, filePath: path.join(root, file) } }, `stale-restore/${file}`);
      check(`stale-restore/rejected/${file}`, !!response.error || response.result?.isError === true, response);
    }
    await functionalFacts();
    mutate('node_modules/@angular/common/package.json', commonManifest()); await functionalFacts();
    await facts(['left.ts']);
    await memberFacts([]);
    mutate('member-api.ts', memberApi); await memberFacts();
    await injectFacts(['token-left.ts']);
    await constructorFacts([], ['token-left.ts']);
    await stringFacts('other.api');
    mutate('string-values.ts', stringValues()); await stringFacts();
    mutate('token-right.ts', 'export class Dependency {}'); await injectFacts(); await constructorFacts();
    mutate('right.ts', leaf); await facts(['left.ts', 'right.ts']);
    mutate('node_modules/@angular/core/package.json', JSON.stringify({ name: '@angular/core', version: '18.2.0', types: 'index.d.ts' }));
    await facts(['left.ts', 'right.ts'], [], false); await constructorFacts();
    mutate('node_modules/@angular/core/package.json', JSON.stringify({ name: '@angular/core', version: '20.0.0', types: 'index.d.ts' }));
    await facts(['left.ts', 'right.ts']); await constructorFacts();
  });
}
