"""Retain the observer-isolated warm comparison without relabeling earlier captures."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
EXEC = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution'
RUN = ROOT / 'target/ops-r1/currentness-28-unobserved-20261122-101635-846'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((EXEC / 'reviewed-green-currentness-28-unobserved-candidate-sources.json').read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})
rows = []
for name in names:
    data = (ISO / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
assert sha(json.dumps(rows, separators=(',', ':')).encode()) == manifest['candidate_source_tree_sha256']
pair = {}
archive = OUT / 'observer-isolated-evidence.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.write(ISO / name, 'candidate-source/' + name)
    for phase in ['baseline', 'candidate']:
        folder = Path((RUN / (phase + '-output.txt')).read_text(encoding='utf-8-sig').strip().splitlines()[-1])
        review = json.loads((folder / 'review.json').read_bytes())
        assert review['status'] == 'ALL_SELECTED_CELLS_COMPLETED' and len(review['runs']) == 1
        run = review['runs'][0]
        path = Path(run['receipt'])
        assert sha(path.read_bytes()) == run['receipt_sha256']
        assert sha((path.parent / 'performance.json').read_bytes()) == run['performance_sha256']
        value = json.loads(path.read_bytes())
        assert value['status'] == 'TRACKED_TEST_COMPLETED' and value['exit_code'] == 0
        assert value['post_source_and_binary_identity_verified']
        cold, warm = run['performance']
        assert warm['counters_after']['compile_attempts'] == warm['counters_before']['compile_attempts']
        assert warm['counters_after']['publication_attempts'] == warm['counters_before']['publication_attempts']
        assert cold['operations'][0]['acquisitions'] == [1, 1, 64]
        pair[phase] = {'source_sha256': review['source_tree_sha256'], 'executable_sha256': review['executable_sha256'],
                       'cold_query_ms': cold['query_elapsed_ms'], 'warm_query_ms': warm['query_elapsed_ms'],
                       'warm_witness_checks': warm['counters_after']['witness_checks'] - warm['counters_before']['witness_checks'],
                       'process_seconds': value['elapsed_seconds'], 'sampled_peak_working_set_bytes': value['observed_peak_working_set_bytes']}
        for key, expected in value.items():
            if key.endswith('_sha256') and (path.parent / key[:-7]).is_file():
                assert sha((path.parent / key[:-7]).read_bytes()) == expected
        saved.write(folder / 'review.json', phase + '/review.json')
        for evidence in sorted(path.parent.rglob('*')):
            if evidence.is_file():
                saved.write(evidence, phase + '/capture/' + evidence.relative_to(path.parent).as_posix())
        build = ISO / ('target/ops-r1/currentness-28-unobserved-' + phase + '-build-01')
        for evidence in sorted(build.iterdir()):
            if evidence.is_file():
                saved.write(evidence, phase + '/build/' + evidence.name)
        pin = EXEC / ('reviewed-green-currentness-28-unobserved-' + phase + '-sources.json')
        saved.write(pin, phase + '/pins/' + pin.name)
    saved.write(OUT / 'measurement-baseline-inputs.zip', 'baseline-original-consumer-inputs.zip')
    for path in sorted(OUT.glob('unobserved-*-*.rs')):
        saved.write(path, 'diagnostic-definitions/' + path.name)
    saved.write(EXEC / 'capture.py', 'candidate-adapter.py')
baseline = pair['baseline']
candidate = pair['candidate']
record = {'agent': 'Agent MaxHeadRoom', 'date': '2026-11-23', 'decision': 'FIX',
          'validated_root_source_sha256': '628e47c3696e9520b48870b4ee72c4ebc2b083cadd8ee7c546d079bddc040206',
          'pair': pair, 'warm_checks_saved': baseline['warm_witness_checks'] - candidate['warm_witness_checks'],
          'observed_warm_ms_saved': baseline['warm_query_ms'] - candidate['warm_query_ms'],
          'qualification': 'Single bounded diagnostic pair; warm caller-thread observer disabled, cold worker bookkeeping remains. No statistical timing, process-total attribution, memory reduction or leak-resolution claim.',
          'complexity': 'One small exact-constraint helper and four caller delegations; no persisted cache or cross-owner freshness reuse',
          'evidence_archive': archive.name, 'evidence_archive_sha256': sha(archive.read_bytes()),
          'remaining_risk': 'Owner-by-project evidence work remains; any cross-owner reuse needs an explicit freshness contract'}
(OUT / 'observer-isolated-results.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
