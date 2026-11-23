"""Preserve phase-29 matched captures and focused regression provenance."""
import hashlib,json,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
OUT=Path(__file__).resolve().parent
RUN=ROOT/'target/ops-r1/phase-29-measurement-20261123-095829-462'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pair={}
archive=OUT/'validation-and-measurement-evidence.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as z:
    def save_tree(folder,prefix):
        for p in sorted(folder.rglob('*')):
            if p.is_file():z.write(p,prefix+'/'+p.relative_to(folder).as_posix())
    for phase in ['baseline','candidate']:
        folder=Path((RUN/(phase+'-output.txt')).read_text(encoding='utf-8-sig').strip().splitlines()[-1])
        review=json.loads((folder/'review.json').read_bytes())
        assert review['status']=='ALL_SELECTED_CELLS_COMPLETED' and len(review['runs'])==1
        run=review['runs'][0]; receipt=Path(run['receipt']); value=json.loads(receipt.read_bytes())
        assert sha(receipt)==run['receipt_sha256']
        assert sha(receipt.parent/'performance.json')==run['performance_sha256']
        assert value['status']=='TRACKED_TEST_COMPLETED' and value['exit_code']==0
        assert value['post_source_and_binary_identity_verified']
        for key,expected in value.items():
            if key.endswith('_sha256') and (receipt.parent/key[:-7]).is_file():
                assert sha(receipt.parent/key[:-7])==expected
        cold,warm=run['performance']
        assert cold['operations'][0]['acquisitions']==[1,1,64]
        for counter in ['compile_attempts','publication_attempts','project_acquisitions']:
            assert warm['counters_after'][counter]==warm['counters_before'][counter]
        pair[phase]={'source_sha256':review['source_tree_sha256'],'executable_sha256':review['executable_sha256'],
            'cold_query_ms':cold['query_elapsed_ms'],'warm_query_ms':warm['query_elapsed_ms'],
            'warm_witness_checks':warm['counters_after']['witness_checks']-warm['counters_before']['witness_checks'],
            'process_seconds':value['elapsed_seconds'],'sampled_peak_working_set_bytes':value['observed_peak_working_set_bytes']}
        z.write(folder/'review.json',phase+'/review.json');save_tree(receipt.parent,phase+'/capture')
        build=Path(review['build_receipt']);assert sha(build)==review['build_receipt_sha256']
        save_tree(build.parent,phase+'/build')
    focused=ROOT/'target/ops-r1/phase-29-green-20261123-094839-289/receipts.json'
    cases=json.loads(focused.read_bytes());assert len(cases)==18
    z.write(focused,'root-green/receipts.json')
    for case in cases:
        receipt=Path(case['receipt']);assert sha(receipt)==case['sha256']
        assert case['status']=='TRACKED_TEST_COMPLETED'
        save_tree(receipt.parent,'root-green/'+case['case'])
    save_tree(ROOT/'target/ops-r1/phase-29-green-build-01','root-green/build')
    for p in sorted((ROOT/'src').rglob('*.rs')):z.write(p,'root-source/'+p.relative_to(ROOT).as_posix())
    for name in ['Cargo.toml','Cargo.lock','build.rs']:
        if (ROOT/name).is_file():z.write(ROOT/name,'root-source/'+name)
    for name in ['measurement-producer.patch','measurement-preparation.json','measurement-baseline-inputs.zip']:
        z.write(OUT/name,'definitions/'+name)
a,b=pair['baseline'],pair['candidate']
result={'agent':'Agent MaxHeadRoom','date':'2026-11-23','decision':'FIX','pair':pair,
 'root_source_sha256':'c6fa2eeff1e27404eef7bbe284493983d448bb64bbc15705e31515e518714c02',
 'root_focused_tests_passed':18,'historical_work_red':176,'required_and_green_work':111,
 'warm_checks_saved':a['warm_witness_checks']-b['warm_witness_checks'],
 'observed_warm_speedup':a['warm_query_ms']/b['warm_query_ms'],
 'qualification':'One matched 64-owner pair, warm observer bookkeeping disabled in both. Cold worker instrumentation remains. No statistical guarantee, leak claim, field-incident resolution or full-CI claim.',
 'boundary':'Final consumption closure only; typed exact facts; owner hashes and admissions independent; fresh standalone calls; preparation and publication unchanged.',
 'archive':archive.name,'archive_sha256':sha(archive)}
(OUT/'results.json').write_bytes((json.dumps(result,indent=2)+'\n').encode())
print(json.dumps(result,indent=2))
