"""Stage an exact corrected-fixture RED/GREEN pair, restoring candidate source safely."""
import hashlib
import json
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
mode, run_name = sys.argv[1:3]
RUN = Path(run_name).resolve()
assert RUN.is_relative_to((ROOT / 'target/ops-r1').resolve())
sha = lambda data: hashlib.sha256(data).hexdigest()
template = json.loads((EXEC / 'reviewed-green-currentness-28-green-sources.json').read_bytes())
changed = json.loads((OUT / 'green-preparation.json').read_bytes())['changed_production_files']
test = 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {entry['path'] for entry in template['files']})
with zipfile.ZipFile(OUT / 'historical-red.zip') as source:
    originals = {name: source.read('source/' + name) for name in changed}


def identity(overrides=None):
    overrides = overrides or {}
    rows = []
    for name in names:
        data = overrides[name] if name in overrides else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


def restore():
    record = json.loads((RUN / 'candidate-record.json').read_bytes())
    for name in changed:
        saved = RUN / 'candidate-files' / name
        assert sha(saved.read_bytes()) == record['files'][name]
        assert (ROOT / name).read_bytes() in [originals[name], saved.read_bytes()], 'Concurrent source change: ' + name
        (ROOT / name).write_bytes(saved.read_bytes())
    assert identity() == record['candidate_source_sha256']


if mode == 'restore':
    restore()
    print('Candidate source restored and identity verified')
    sys.exit(0)

if mode == 'red':
    with zipfile.ZipFile(OUT / 'failed-green-original-fixture.zip') as history:
        old_test = history.read('source/' + test)
    assert identity({test: old_test}) == template['candidate_source_tree_sha256']
    record = {'candidate_source_sha256': identity(), 'corrected_test_sha256': sha((ROOT / test).read_bytes()), 'files': {}}
    for name in changed:
        saved = RUN / 'candidate-files' / name
        saved.parent.mkdir(parents=True, exist_ok=True)
        saved.write_bytes((ROOT / name).read_bytes())
        record['files'][name] = sha(saved.read_bytes())
    (RUN / 'candidate-record.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
    for name, data in originals.items():
        (ROOT / name).write_bytes(data)
    version = 'currentness-28-corrected-red'
    cases = "{'retained-currentness':(1,40)}"
elif mode == 'green':
    baseline_record = json.loads((RUN / 'corrected-red-source.json').read_bytes())
    assert identity() == baseline_record['source_sha256']
    failure = Path(sys.argv[3]).resolve()
    assert failure.is_relative_to((ROOT / 'target/ops-r1').resolve())
    receipt = json.loads((failure / 'receipt.json').read_bytes())
    assert receipt['exit_code'] == 101 and receipt['post_source_and_binary_identity_verified']
    assert receipt['source_tree_sha256'] == baseline_record['source_sha256']
    build_path = ROOT / 'target/ops-r1/currentness-28-corrected-red-build-01/build-receipt.json'
    build = json.loads(build_path.read_bytes())
    binary = Path(build['executable'])
    assert sha(binary.read_bytes()) == build['executable_sha256']
    shutil.copy2(binary, RUN / ('corrected-red-' + binary.name))
    archive = OUT / 'corrected-historical-red.zip'
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
        for name in names:
            saved.write(ROOT / name, 'source/' + name)
        for path in sorted(failure.rglob('*')):
            if path.is_file():
                saved.write(path, 'execution/' + path.relative_to(failure).as_posix())
        for path in [build_path, EXEC / 'capture.py', EXEC / 'reviewed-green-currentness-28-corrected-red-sources.json']:
            saved.write(path, 'pins/' + path.name)
    restore()
    version = 'currentness-28-corrected-green'
    cases = "dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39),'retained-currentness':(1,40)})"
else:
    raise ValueError('Unknown stage')

manifest = dict(template)
manifest['standing'] = 'Corrected fixture: owner identity supplied by caller; disk-backed config must be freshly validated'
manifest['candidate_source_tree_sha256'] = identity()
pin = EXEC / ('reviewed-green-' + version + '-sources.json')
assert not pin.exists(), 'Preserve previous source pins'
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + version + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace('if amendment not in (', "if amendment not in ('" + version + "',", 1)
text = text.replace('    proposal=pathlib', "        if amendment=='" + version + "':source_key='green-" + version + "'\n    proposal=pathlib", 1)
text = text.replace('    core.CASES=', "    if source_key=='green-" + version + "':selected_cases=" + cases + '\n    core.CASES=', 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
summary = {'source_sha256': manifest['candidate_source_tree_sha256'], 'test_sha256': sha((ROOT / test).read_bytes()),
           'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()), 'version': version}
(RUN / ('corrected-' + mode + '-source.json')).write_bytes((json.dumps(summary, indent=2) + '\n').encode('utf-8'))
print(json.dumps(summary, indent=2))
