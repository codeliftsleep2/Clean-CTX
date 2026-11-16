// Agent MaxHeadRoom: operator binary/stdio evidence, never Rust/CI or field evidence.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {McpClient} from '../../csharp-compilation-identity/client.mjs';
import {fixture as established} from '../route-resolver78.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const build = path.resolve(process.argv[2] ?? '');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const norm = file => path.resolve(file).replaceAll('\\', '/').toLowerCase();
const output = path.join(repo, 'target/ang-r2', `registration-stdio-${new Date().toISOString().replaceAll(/[:.]/g, '-')}`);
fs.mkdirSync(output, {recursive:true});
const report = {agent:'Agent MaxHeadRoom', category:'operator binary/stdio; not Rust/CI or Claude field acceptance', status:'INCOMPLETE', checks:[], sessions:[], mutations:[]};
const save = () => fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify(report,null,2)+'\n', 'utf8');
const check = (name, pass, detail) => {
  if (report.checks.length >= 500) throw new Error('Bounded suite exceeds 500 checks');
  report.checks.push({name, pass:!!pass, ...(pass ? {} : {detail})}); save();
  if (!pass) throw new Error(`Assertion failed: ${name}`);
};
let active;
for (const signal of ['SIGINT','SIGTERM']) process.once(signal, () => { active?.child.kill(); });
const write = (root, file, text) => {
  const target = path.resolve(root,file);
  if (!target.startsWith(root+path.sep)) throw new Error('Fixture escapes root');
  fs.mkdirSync(path.dirname(target),{recursive:true}); fs.writeFileSync(target,text,'utf8');
};
const owner = (key='secondary', target='Gate') => `import type {Routes} from '@angular/router'; import {Gate} from './resolver-a'; import {Gate as Right} from './resolver-b'; import {Screen} from './resolver-screen'; export const routes:Routes=[{path:'dashboard',component:Screen,resolve:{primary:${target},${key}:${target},missing:Missing}},{path:'dashboard',resolve:{primary:Right}}];`;
function fixture(label) {
  const root = path.join(output,label,'workspace'); fs.mkdirSync(root,{recursive:true});
  const config = {cbm:{enabled:false},proxy:{auto_start:false},default_fidelity:'high',persistence:{enabled:true,auto_save:false,db_path:path.join(root,'authority.db')}};
  const files = {
    ...Object.fromEntries(Object.entries(established).filter(([name]) => name.startsWith('node_modules/') || ['resolver-a.ts','resolver-b.ts','resolver-screen.ts'].includes(name))),
    'node_modules/@angular/core/package.json':JSON.stringify({name:'@angular/core',version:'20.0.0',types:'index.d.ts'}),
    '.clean-ctx.json':JSON.stringify(config),
    'tsconfig.json':JSON.stringify({compilerOptions:{module:'esnext',moduleResolution:'bundler',moduleDetection:'legacy',noLib:true,types:[]}}),
    'one.ts':owner(),
  };
  for (const [name,text] of Object.entries(files)) write(root,name,text);
  report[label] = {root,config,files:Object.entries(files).map(([file,text])=>({file,sha256:hash(Buffer.from(text))}))}; save();
  return root;
}
async function session(binary, root, name, action) {
  const dir=path.join(output,name);fs.mkdirSync(dir,{recursive:true});
  const record={name,status:'INCOMPLETE'};report.sessions.push(record);save();
  const client=new McpClient(binary,root,dir);active=client;
  try {
    await client.initialize();
    let sequence=0;
    const call=(tool,args)=>client.call(tool,{workspaceRoot:root,...args},`${name}/${++sequence}/${tool}`);
    await action(call); record.status='PASS';
  } finally { try {await client.close();} finally {active=undefined;save();} }
}
async function facts(call,root,label,{direction='forward',key='secondary',target='resolver-a.ts',file='one.ts',empty=false}={}) {
  const answer=await call('workspace_query',{type:'resolver_registrations',direction,...(direction==='forward'?{route_path:'dashboard'}:{resolver_name:'Gate'})});
  const result=answer.structuredContent;
  check(`${label}/public-contract`,result?.format_version===1 && result.direction===direction && result.coverage?.source_complete===false && result.coverage?.result_semantics==='lower_bound',result);
  const rows=result.registrations;
  check(`${label}/count`,Array.isArray(rows)&&result.count===(empty?0:3)&&rows.length===(empty?0:3),result);
  if (empty) return;
  const tuples=rows.map(r=>[r.key,norm(r.resolver.source)]).sort();
  const wanted=[['primary',norm(path.join(root,target))],[key,norm(path.join(root,target))],['primary',norm(path.join(root,'resolver-b.ts'))]].sort();
  check(`${label}/keys-exact-targets`,JSON.stringify(tuples)===JSON.stringify(wanted),{tuples,wanted});
  const sites=new Set(rows.map(r=>JSON.stringify(r.route)));
  check(`${label}/distinct-route-occurrences`,sites.size===2,rows);
  for(const [i,r] of rows.entries()) {
    const bytes=fs.readFileSync(path.join(root,file));
    const span=r.evidence.key;
    check(`${label}/row-${i}-source`,norm(r.evidence.source)===norm(path.join(root,file)) && norm(r.route.source)===norm(path.join(root,file)) && r.evidence.source_hash===hash(bytes) && r.route.source_hash===hash(bytes),r);
    check(`${label}/row-${i}-key-span`,bytes.subarray(...span).toString('utf8')===r.key,r);
    check(`${label}/row-${i}-target-hash`,r.resolver.source_hash===hash(fs.readFileSync(r.resolver.source)),r);
    check(`${label}/row-${i}-coordinates`,r.route.start<r.route.end && r.route.end<=bytes.length && ['field_pair','field_key','object','pair','key','value'].every(name=>{const [start,end]=r.evidence[name];return start>=r.route.start&&start<end&&end<=r.route.end;}),r);
    check(`${label}/row-${i}-written-authority`,r.route_path==='dashboard'&&r.resolver.name==='Gate'&&!('executable' in r),r);
  }
  check(`${label}/refused-sibling-recorded`,(direction==='reverse'||result.semantic_omissions.length>0) && !rows.some(r=>r.key==='missing'),result);
  const old=(await call('workspace_query',{type:'forward_edges',domain:'angular',entity_type:'Route',name:'dashboard'})).structuredContent.edges;
  check(`${label}/unqualified-membership`,old.filter(e=>e.relation==='ResolvedBy').length===2,old);
  check(`${label}/component-sibling`,old.filter(e=>e.relation==='RouteMapsTo').length===1,old);
}
try {
  if(process.argv.length!==3 || !build.startsWith(path.join(repo,'target/ang-r2/registration-build-'))) throw new Error('Supply the exact successful ANG-R2 build receipt directory');
  const pins=JSON.parse(fs.readFileSync(path.join(build,'source-manifest.json'),'utf8'));
  const binary=JSON.parse(fs.readFileSync(path.join(build,'binary.json'),'utf8'));
  const verifyPins=()=>{
    check('binary-pin',hash(fs.readFileSync(binary.path))===binary.sha256);
    check('source-pins',pins.every(p=>hash(fs.readFileSync(path.join(repo,p.path)))===p.sha256));
  };
  report.binary=binary;report.build=build;
  report.inputs=[fileURLToPath(import.meta.url),fileURLToPath(new URL('../../csharp-compilation-identity/client.mjs',import.meta.url)),fileURLToPath(new URL('../route-resolver78.mjs',import.meta.url))].map(file=>({file,sha256:hash(fs.readFileSync(file))}));
  verifyPins();
  const forward=fixture('forward');const reverse=fixture('reverse');
  check('independent-cold-databases',[forward,reverse].every(root=>!fs.existsSync(path.join(root,'authority.db'))));
  await session(binary.path,forward,'cold-forward',async call=>{
    await facts(call,forward,'cold-forward');
    await facts(call,forward,'warm-forward');
    await call('compress_code_context',{filePath:path.join(forward,'one.ts'),fidelity:'high'});
    await call('save_context',{filePath:path.join(forward,'one.ts')});
  });
  await session(binary.path,reverse,'cold-reverse',call=>facts(call,reverse,'cold-reverse',{direction:'reverse'}));
  await session(binary.path,forward,'restore-and-mutate',async call=>{
    await call('restore_context',{filePath:path.join(forward,'one.ts')});
    await facts(call,forward,'restored');
    const mutate=(file,text)=>{report.mutations.push({file,sha256:text===null?null:hash(Buffer.from(text))});save();if(text===null)fs.unlinkSync(path.join(forward,file));else write(forward,file,text);};
    mutate('one.ts',owner('renamed'));await facts(call,forward,'key-only',{key:'renamed'});
    mutate('one.ts',owner('renamed','Right'));await facts(call,forward,'target-remap',{key:'renamed',target:'resolver-b.ts'});
    mutate('one.ts',"import type {Routes} from '@angular/router'; export const routes:Routes=[];");
    await facts(call,forward,'root-retired',{empty:true});
    mutate('one.ts',null);mutate('moved.ts',owner());
    await facts(call,forward,'owner-relocated',{file:'moved.ts'});
    mutate('moved.ts',null);await facts(call,forward,'source-retired',{empty:true});
    mutate('one.ts',owner());await facts(call,forward,'owner-recreated');
  });
  verifyPins();report.status='PASS';
} catch(error) {report.status='FAIL';report.error=error.stack;process.exitCode=1;}
finally {report.finished=new Date().toISOString();save();console.log(`${report.status}: ${report.checks.length} checks; ${output}`);}
