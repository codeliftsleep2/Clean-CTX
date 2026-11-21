"""Verify and retain existing captures and exact candidate source; never run tests."""
import csv
import hashlib
import io
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
OUT = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()
groups = [
    'performance-25-performance-25-baseline-20261120-234850',
    'performance-25-performance-25-baseline-20261120-235505',
    'performance-25-performance-25-baselineb-20261121-001101',
    'performance-25-performance-25-cost-green-20261121-004623',
    'performance-25-performance-25-directory-green-20261121-013611',
]
index = {'standing': 'Actual operator tracked-test captures; no CI or field acceptance', 'groups': []}
folders = set()
manifest_path = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution/reviewed-green-performance-25-directory-green-sources.json'
manifest = json.loads(manifest_path.read_bytes())
tracked = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {p['path'] for p in manifest['files']})
source_rows = []
for name in names:
    data = (ISO / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    source_rows.append({'path': name, 'sha256': sha(data)})
assert sha(json.dumps(source_rows, separators=(',', ':')).encode()) == manifest['candidate_source_tree_sha256']
archive_path = OUT / 'performance-captures-and-final-source.zip'
members = []
with zipfile.ZipFile(archive_path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    def retain(path, name):
        data = path.read_bytes()
        archive.writestr(name, data)
        members.append((name, len(data), sha(data)))

    for group in groups:
        review = ROOT / 'target/ops-r1' / group / 'review.json'
        data = json.loads(review.read_bytes())
        retain(review, 'reviews/' + group + '.json')
        index['groups'].append({'name': group, 'status': data['status'], 'review_sha256': sha(review.read_bytes()), 'selected_count': len(data['selected_cases']), 'captured_count': len(data['runs'])})
        for run in data['runs']:
            receipt = Path(run['receipt'])
            assert sha(receipt.read_bytes()) == run['receipt_sha256']
            if run.get('performance_sha256'):
                assert sha((receipt.parent / 'performance.json').read_bytes()) == run['performance_sha256']
            folders.add(receipt.parent)
    controls = ROOT / 'target/ops-r1/directory-validation-20261121-013451-823/receipts.json'
    entries = json.loads(controls.read_bytes())
    assert len(entries) == 13
    for entry in entries:
        receipt = Path(entry['receipt'])
        assert sha(receipt.read_bytes()) == entry['sha256']
        data = json.loads(receipt.read_bytes())
        assert data['status'] == 'TRACKED_TEST_COMPLETED' and data['post_source_and_binary_identity_verified']
        assert data['source_tree_sha256'] == manifest['candidate_source_tree_sha256']
        folders.add(receipt.parent)
    retain(controls, 'controls/receipts.json')
    for folder in sorted(folders):
        data = json.loads((folder / 'receipt.json').read_bytes())
        for key, expected in data.items():
            if key.endswith('_sha256') and (folder / key[:-7]).is_file():
                assert sha((folder / key[:-7]).read_bytes()) == expected, (folder, key)
        for path in sorted(folder.rglob('*')):
            if path.is_file():
                retain(path, 'captures/' + folder.name + '/' + path.relative_to(folder).as_posix())
    for name in names:
        retain(ISO / name, 'source/' + name)
    for name in ['capture.py', 'invoke-build-capture.ps1', manifest_path.name]:
        retain(manifest_path.parent / name, 'pins/' + name)
    build = ISO / 'target/ops-r1/performance-25-directory-green-build-01'
    for path in sorted(build.iterdir()):
        if path.is_file():
            retain(path, 'build/' + path.name)
    content = io.StringIO(newline='')
    writer = csv.writer(content)
    writer.writerow(['member', 'bytes', 'sha256'])
    writer.writerows(members)
    archive.writestr('members.csv', content.getvalue().encode())
index.update(archive=archive_path.name, archive_sha256=sha(archive_path.read_bytes()),
             member_count=len(members) + 1, retained_capture_count=len(folders),
             final_source_sha256=manifest['candidate_source_tree_sha256'],
             final_controls_passed=len(entries))
(OUT / 'performance-evidence-index.json').write_text(json.dumps(index, indent=2) + '\n', encoding='utf-8')
print(json.dumps({key: index[key] for key in ['archive', 'member_count', 'retained_capture_count', 'final_controls_passed']}))
