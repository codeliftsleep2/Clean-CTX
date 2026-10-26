// Agent MaxHeadRoom: explicit suite selection; no monolithic check receipt.
import fs from 'node:fs';
import path from 'node:path';
import {spawn} from 'node:child_process';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {suites} from './issue-162/suite-manifest.mjs';

const directory = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(directory, '../..');
const [binaryArg, selection] = process.argv.slice(2);
if (process.argv.length !== 4 || !binaryArg || (selection !== 'all' && !suites.includes(selection))) {
  console.error(`Usage: node verification/live-acceptance/issue_162_live_acceptance.mjs <rebuilt-binary> <suite|all>\nSuites: ${suites.join(', ')}`);
  process.exitCode = 1;
} else {
  const binary = path.resolve(binaryArg);
  const selected = selection === 'all' ? suites : [selection];
  const output = path.join(repo, 'target/live-acceptance/issue-162', `${new Date().toISOString().replaceAll(/[:.]/g,'-')}-${selection}-summary-${process.pid}`);
  fs.mkdirSync(output, {recursive:true});
  const report = {agent:'Agent MaxHeadRoom', category:'operator suite summary; not Rust/CI or field acceptance', selection, status:'INCOMPLETE', suites:[]};
  const save = () => fs.writeFileSync(path.join(output,'report.json'), JSON.stringify(report,null,2)+'\n','utf8');
  let active;
  let interrupted = false;
  for (const signal of ['SIGINT','SIGTERM']) process.once(signal, () => {
    interrupted = true;
    active?.kill(signal);
  });
  try {
    for (const name of selected) {
      if (interrupted) throw new Error('suite run interrupted');
      const evidence = path.join(output,name);
      const entry = path.join(directory,'issue-162/suites',`${name}.mjs`);
      const record = {name,status:'INCOMPLETE',report:path.join(evidence,'report.json')};
      report.suites.push(record); save();
      const code = await new Promise((resolve,reject) => {
        active = spawn(process.execPath,[entry,binary],{stdio:'inherit',env:{...process.env,ISSUE162_SUITE_OUTPUT:evidence}});
        active.once('error',reject);
        active.once('exit',(code,signal) => { active = undefined; resolve(signal ? 1 : code); });
      });
      const receipt = JSON.parse(fs.readFileSync(record.report,'utf8'));
      const inputsHash = crypto.createHash('sha256').update(JSON.stringify([...receipt.inputs].sort((a,b)=>a.file.localeCompare(b.file)))).digest('hex');
      if (!report.binary) { report.binary = receipt.binary; report.inputsHash = inputsHash; }
      if (report.binary.sha256 !== receipt.binary.sha256 || report.inputsHash !== inputsHash) throw new Error('binary/inputs changed between suites');
      record.checks = receipt.checks.length;
      record.status = code === 0 && receipt.status === 'PASS' && receipt.suite === name && record.checks <= 500 ? 'PASS' : 'FAIL';
      save();
      if (record.status !== 'PASS') throw new Error(`suite failed: ${name}`);
    }
    report.status = interrupted ? 'FAIL' : 'PASS';
  } catch (error) {
    report.status = 'FAIL'; report.error = error.stack;
  } finally {
    report.finished = new Date().toISOString(); save();
    console.log(`${report.status}: ${report.suites.filter(item=>item.status==='PASS').length}/${selected.length} suites; summary: ${output}`);
    process.exitCode = report.status === 'PASS' ? 0 : 1;
  }
}
