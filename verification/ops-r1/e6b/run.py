"""Owner-authorized E6-B focused gates; reused watchdog, no E1-E5 experiment rerun."""
import argparse,hashlib,importlib.util,json,os,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'target/ops-r1'/os.environ.get('OPS_E6_RUN','e6b-verification-r9-20261124')
BUILD=ROOT/'target/ops-r1'/os.environ.get('OPS_E6_BUILD','e6b-build-r9-20261124')
spec=importlib.util.spec_from_file_location('bounded',ROOT/'verification/ops-r1/e1/measure.py')
bounded=importlib.util.module_from_spec(spec);spec.loader.exec_module(bounded)
def run(kind):
    pin=json.loads((BUILD/'build-identity.json').read_text(encoding='utf-8'))
    assert hashlib.sha256(Path(pin['executable']).read_bytes()).hexdigest()==pin['sha256']
    assert bounded.sources()==json.loads((BUILD/'build-sources.json').read_text(encoding='utf-8'))
    if kind=='lint':
        r=bounded.bounded(['cargo','clippy','--lib','--all-features','--','-D','warnings'],OUT/'lint',600,8192)
        assert r['exit_code']==0
        return
    patterns=['e6b_'] if kind=='tests' else ['e6b_bounded_validation_growth_red_green'] if kind=='red' else ['e6b_integrated_measurement'] if kind=='measure' else ['source_observations::tests::','request_budget::tests::','direct_identity::tests::','claim_composition::tests::','issue_162_resolver_registration_required_publication_failure_is_not_empty_success','issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed']
    for i,pattern in enumerate(patterns):
        directory=OUT/f'{kind}-{i}'
        if directory.exists():
            prior=json.loads((directory/'receipt.json').read_text(encoding='utf-8'))
            assert prior.get('executable_sha256')==pin['sha256'] and prior.get('source_manifest_sha256')==pin['source_manifest_sha256']
            assert prior.get('exit_code')==(101 if kind=='red' else 0)
            output=(directory/'stdout.txt').read_text(encoding='utf-8')
            result=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed',output)
            assert result and sum(map(int,result.groups()))>0
            continue
        environment=dict(os.environ)
        if kind=='red':environment['OPS_E6_RED']='1'
        if kind=='measure':environment['OPS_E6_OUTPUT']=str(directory)
        result=bounded.bounded([pin['executable'],pattern,'--test-threads=1','--nocapture'],directory,60,512,environment)
        result.update(executable_sha256=pin['sha256'],source_manifest_sha256=pin['source_manifest_sha256'],category='E6-B '+kind)
        bounded.save(directory/'receipt.json',result)
        expected=101 if kind=='red' else 0
        assert result.get('exit_code')==expected,(kind,result)
        output=(directory/'stdout.txt').read_text(encoding='utf-8')
        summary=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed',output)
        assert summary and sum(map(int,summary.groups()))>0,'empty gate'
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('kind',choices=['tests','controls','red','measure','lint']);run(parser.parse_args().kind)
