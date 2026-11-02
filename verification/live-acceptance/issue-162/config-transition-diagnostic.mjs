// Agent MaxHeadRoom: operator-only minimal config-transition comparison.
// Diagnostic evidence, not a Rust regression test, binary acceptance or repair.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {McpClient} from '../csharp-compilation-identity/client.mjs';
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const [binaryArg,largeArg,boundedArg]=process.argv.slice(2);
if(process.argv.length!==5)throw new Error('Expected explicit binary, preserved large report, bounded Module report');
const binary=path.resolve(binaryArg);
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const binaryHash=hash(fs.readFileSync(binary));
const sources=[['bounded',path.resolve(boundedArg)],['large',path.resolve(largeArg)]];
const output=path.join(repo,'target/issue-162',`config-transition-diagnostic-${new Date().toISOString().replaceAll(/[:.]/g,'-')}-${process.pid}`);
fs.mkdirSync(output,{recursive:true});
const report={agent:'Agent MaxHeadRoom',category:'operator diagnostic only; not Rust/CI, binary acceptance or performance repair',status:'INCOMPLETE',binary:{path:binary,sha256:binaryHash},cases:[]};
const save=()=>fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n','utf8');
const norm=f=>{const s=f.replaceAll('\\','/');return process.platform==='win32'?s.toLowerCase():s;};
let active;
for(const signal of ['SIGINT','SIGTERM'])process.once(signal,()=>{
 report.interrupted=signal;save();active?.fail(new Error(signal));active?.child.kill();
});
try {
 for(const [name,sourcePath] of sources){
  if(report.interrupted)throw new Error('diagnostic interrupted');
  const sourceBytes=fs.readFileSync(sourcePath);const source=JSON.parse(sourceBytes);
  if(source.binary?.sha256!==binaryHash)throw new Error('Preserved report binary mismatch');
  const rustInputs=source.inputs.filter(i=>i.file.endsWith('.rs')||['Cargo.toml','Cargo.lock','build.rs'].includes(i.file));
  for(const input of rustInputs)if(hash(fs.readFileSync(path.join(repo,input.file)))!==input.sha256)throw new Error(`Rust/build input changed: ${input.file}; diagnostic requires a matching rebuilt binary`);
  const directory=path.join(output,name);const root=path.join(directory,'workspace');fs.mkdirSync(root,{recursive:true});
  const fixture={...source.fixture};const config=JSON.parse(fixture['.clean-ctx.json']);config.persistence.db_path=path.join(root,'authority.db');fixture['.clean-ctx.json']=JSON.stringify(config);
  for(const [file,text] of Object.entries(fixture)){
   const target=path.resolve(root,file);if(!target.startsWith(root+path.sep))throw new Error('fixture path outside diagnostic root');
   fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,text,'utf8');
  }
  const record={name,sourceReport:sourcePath,sourceReportHash:hash(sourceBytes),typescriptFiles:Object.keys(fixture).filter(f=>f.endsWith('.ts')||f.endsWith('.tsx')).length,fixtureHash:hash(JSON.stringify(Object.entries(fixture).sort())),status:'INCOMPLETE',stages:[]};
  report.cases.push(record);save();
  const client=new McpClient(binary,root,directory);active=client;
  try {
   await client.initialize();
   const listing=await client.request('tools/list',{},`${name}/schema`);
   if(!listing.result?.tools?.some(t=>t.name==='workspace_query'&&t.outputSchema))throw new Error('workspace_query schema unavailable');
   const expected=[];for(const owner of ['direct','reexport','alias'])for(const target of ['feature-a.ts','feature-b.ts','feature-a.ts'])expected.push([norm(path.join(root,`apps/children/${owner}.ts`)),norm(path.join(root,target))]);expected.sort();
   const query=async stage=>{
    const entry={stage,status:'INCOMPLETE',started:new Date().toISOString()};record.stages.push(entry);save();const started=performance.now();
    try {
     const result=await client.call('workspace_query',{workspaceRoot:root,type:'forward_edges',domain:'angular',entity_type:'Route',name:'same'},`${name}/${stage}`);
     const body=result.structuredContent;const edges=body?.edges?.filter(e=>e.relation==='LoadsChildren');
     if(!Array.isArray(edges))throw new Error('structured edge response missing');
     const actual=edges.map(e=>[norm(e.subject.file),norm(e.object.file)]).sort();
     if(JSON.stringify(actual)!==JSON.stringify(expected)||edges.some(e=>e.object.entity_type!=='Module'||e.object.name!=='Screen'))throw new Error('exact Module occurrence/target oracle failed');
     if(body.coverage?.result_semantics!=='lower_bound'||body.coverage?.source_complete!==false)throw new Error('coverage contract changed');
     entry.status='ANSWERED';entry.edges=edges.length;
    }catch(error){entry.status=String(error).includes('120s request timeout')?'TIMEOUT':'FAILED';entry.error=String(error);throw error;}
    finally{entry.elapsedMs=performance.now()-started;save();}
   };
   await query('cold');await query('warm');
   const before=fs.readFileSync(path.join(root,'tsconfig.json'),'utf8');const changed=JSON.parse(before);changed.compilerOptions.baseUrl='.';
   const after=JSON.stringify(changed);if(after===before)throw new Error('config mutation must change bytes');
   record.configMutation={beforeHash:hash(before),afterHash:hash(after),kind:'add baseUrl dot'};
   fs.writeFileSync(path.join(root,'tsconfig.json'),after,'utf8');save();
   await query('after-config-edit');record.status='COMPLETED';
  }catch(error){record.status=record.stages.some(s=>s.status==='TIMEOUT')?'TIMEOUT':'FAILED';record.error=String(error);}
  finally{try{await client.close();}catch(error){record.shutdownError=String(error);record.status='FAILED';}active=undefined;save();}
  console.log(`${name}: ${record.status}; ${record.typescriptFiles} TypeScript files; stages=${record.stages.map(s=>`${s.stage}:${s.status}:${Math.round(s.elapsedMs)}ms`).join(', ')}`);
  if(record.status!=='COMPLETED'&&name==='bounded')break;
 }
 if(hash(fs.readFileSync(binary))!==binaryHash)throw new Error('binary changed during diagnostic');
 report.status='RECORDED';
 const bounded=report.cases.find(c=>c.name==='bounded');const large=report.cases.find(c=>c.name==='large');
 report.interpretation=bounded?.status==='COMPLETED'&&large?.stages.some(s=>s.stage==='after-config-edit'&&s.status==='TIMEOUT')?'MINIMAL_LARGE_CONFIG_TRANSITION_TIMEOUT_REPRODUCED':bounded?.status==='COMPLETED'&&large?.status==='COMPLETED'?'NOT_REPRODUCED_WITHOUT_PRIOR_MUTATION_HISTORY':'INSPECT_STAGE_FAILURES';
}catch(error){report.status='FAILED';report.error=String(error);}
finally{report.finished=new Date().toISOString();save();console.log(`Diagnostic ${report.status}; evidence: ${output}`);process.exitCode=report.status==='RECORDED'&&report.cases.length===2&&report.cases.every(c=>c.status==='COMPLETED')?0:1;}
