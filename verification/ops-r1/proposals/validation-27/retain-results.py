"""Retain actual per-call RED/GREEN and the single matched enclosing-operation pair."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
RUN = ROOT / 'target/ops-r1/validation-27-green-20261122-085820-316'
PAIR = ROOT / 'target/ops-r1/validation-27-measurement-20261122-090920-675'
BUILD = ROOT / 'target/ops-r1/validation-27-green-build-01'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((OUT / 'execution/reviewed-green-validation-27-green-sources.json').read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})
rows = []
for name in names:
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
assert sha(json.dumps(rows, separators=(',', ':')).encode()) == manifest['candidate_source_tree_sha256']
controls = json.loads((RUN / 'receipts.json').read_bytes())
assert len(controls) == 13
folders = []
for entry in controls:
    path = Path(entry['receipt'])
    assert sha(path.read_bytes()) == entry['sha256']
    value = json.loads(path.read_bytes())
    assert value['status'] == 'TRACKED_TEST_COMPLETED' and value['exit_code'] == 0
    assert value['post_source_and_binary_identity_verified']
    assert value['source_tree_sha256'] == manifest['candidate_source_tree_sha256']
    folders.append(path.parent)
pair = {}
reviews = []
for phase in ['baseline', 'candidate']:
    folder = Path((PAIR / (phase + '-output.txt')).read_text(encoding='utf-8-sig').strip().splitlines()[-1])
    review_path = folder / 'review.json'
    review = json.loads(review_path.read_bytes())
    assert review['status'] == 'ALL_SELECTED_CELLS_COMPLETED' and len(review['runs']) == 1
    run = review['runs'][0]
    receipt_path = Path(run['receipt'])
    assert sha(receipt_path.read_bytes()) == run['receipt_sha256']
    assert sha((receipt_path.parent / 'performance.json').read_bytes()) == run['performance_sha256']
    assert run['observations']['post_source_and_binary_identity_verified']
    cold, warm = run['performance']
    assert cold['operations'][0]['acquisitions'] == [1, 1, 64]
    assert cold['counters_after']['compile_completed'] == 64
    assert cold['counters_after']['publication_completed'] == 64
    assert warm['operations'][0]['acquisitions'] == [0, 0, 0]
    pair[phase] = {'source_sha256': review['source_tree_sha256'],
                   'executable_sha256': review['executable_sha256'],
                   'process_seconds': run['observations']['elapsed_seconds'],
                   'peak_working_set_bytes': run['observations']['observed_peak_working_set_bytes'],
                   'cold_query_ms': cold['query_elapsed_ms'], 'warm_query_ms': warm['query_elapsed_ms'],
                   'cold_witness_checks': cold['counters_after']['witness_checks'] - cold['counters_before']['witness_checks'],
                   'warm_witness_checks': warm['counters_after']['witness_checks'] - warm['counters_before']['witness_checks'],
                   'cold_operations': cold['operations']}
    folders.append(receipt_path.parent)
    reviews.append(review_path)
archive = OUT / 'validation-and-measurement-evidence.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.write(ROOT / name, 'root-source/' + name)
    for prefix, folder in [('root-build', BUILD), ('focused-controls', RUN), ('pair', PAIR)]:
        for path in sorted(folder.rglob('*')):
            if path.is_file():
                saved.write(path, prefix + '/' + path.relative_to(folder).as_posix())
    for folder in folders:
        value = json.loads((folder / 'receipt.json').read_bytes())
        for key, expected in value.items():
            if key.endswith('_sha256') and (folder / key[:-7]).is_file():
                assert sha((folder / key[:-7]).read_bytes()) == expected
        for path in sorted(folder.rglob('*')):
            if path.is_file():
                saved.write(path, 'captures/' + folder.name + '/' + path.relative_to(folder).as_posix())
    for path in reviews:
        saved.write(path, 'reviews/' + path.parent.name + '.json')
    for path in sorted((OUT / 'execution').iterdir()):
        if path.is_file():
            saved.write(path, 'execution/' + path.name)
result = {'agent': 'Agent MaxHeadRoom', 'date': '2026-11-22', 'decision': 'FIX',
          'root_source_sha256': manifest['candidate_source_tree_sha256'], 'focused_controls_passed': len(controls),
          'pair': pair, 'cold_checks_saved': pair['baseline']['cold_witness_checks'] - pair['candidate']['cold_witness_checks'],
          'evidence_archive': archive.name, 'evidence_archive_sha256': sha(archive.read_bytes()),
          'qualification': 'One matched pair, not a timing guarantee; no warm improvement or leak claim',
          'remaining_work': 'Warm currentness cost remains; full gates deferred until engineering is complete'}
(OUT / 'results.json').write_bytes((json.dumps(result, indent=2) + '\n').encode('utf-8'))
print(json.dumps(result, indent=2))
