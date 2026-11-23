"""Apply the exact currentness-only producer diff to the existing measurement fixture."""
import difflib
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
EXEC = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution'
VERSION = 'currentness-28-performance'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((EXEC / 'reviewed-green-validation-27-performance-sources.json').read_bytes())


def identity(workspace, pin):
    tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=workspace).decode().split('\0')
    rows = []
    for name in sorted(set(filter(None, tracked)) | {entry['path'] for entry in pin['files']}):
        data = (workspace / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


baseline = identity(ISO, manifest)
assert baseline == manifest['candidate_source_tree_sha256']
root_pin = json.loads((OUT / 'execution/reviewed-green-currentness-28-corrected-green-sources.json').read_bytes())
root_source = identity(ROOT, root_pin)
assert root_source == root_pin['candidate_source_tree_sha256']
root_build = json.loads((ROOT / 'target/ops-r1/currentness-28-corrected-green-build-01/build-receipt.json').read_bytes())
assert root_source == root_build['source_tree_sha256']
changed = ['src/ir/typescript_resolution/host.rs', 'src/workspace/index/coverage.rs',
           'src/workspace/index/resolver_registrations.rs', 'src/mcp/semantic_compilation/retained.rs']
expected = []
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/validation-27/validation-and-measurement-evidence.zip') as prior:
    for name in changed:
        a = prior.read('root-source/' + name).decode('utf-8').replace('\r\n', '\n')
        b = (ROOT / name).read_text(encoding='utf-8')
        expected.extend(difflib.unified_diff(a.splitlines(keepends=True), b.splitlines(keepends=True), fromfile='a/' + name, tofile='b/' + name, n=3))
patch = OUT / 'measurement-producer.patch'
assert patch.read_bytes() == ''.join(expected).encode('utf-8')
subprocess.run(['git', 'apply', '--check', str(patch)], cwd=ISO, check=True)
prior_path = ISO / 'target/ops-r1/validation-27-measurement-build-01/build-receipt.json'
prior = json.loads(prior_path.read_bytes())
binary = Path(prior['executable'])
assert sha(binary.read_bytes()) == prior['executable_sha256']
preserved = ROOT / 'target/ops-r1/currentness-28-preserved-measurement-baseline'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(prior_path, preserved / 'build-receipt.json')
with zipfile.ZipFile(OUT / 'measurement-baseline-inputs.zip', 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in changed:
        saved.write(ISO / name, name)
    for name in ['capture.py', 'reviewed-green-validation-27-performance-sources.json']:
        saved.write(EXEC / name, 'execution/' + name)
subprocess.run(['git', 'apply', str(patch)], cwd=ISO, check=True)
subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *changed], cwd=ISO, check=True)
manifest['standing'] = 'Same fixture and observations; exact per-call currentness correction only'
manifest['baseline_source_tree_sha256'] = baseline
manifest['candidate_source_tree_sha256'] = identity(ISO, manifest)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace('if amendment not in (', "if amendment not in ('" + VERSION + "',", 1)
text = text.replace('    proposal=pathlib', "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace('    core.CASES=', "    if source_key=='green-" + VERSION + "':selected_cases=CASES_PERFORMANCE_25\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
for path in [pin, EXEC / 'capture.py']:
    shutil.copy2(path, OUT / ('measurement-candidate-' + path.name))
record = {'baseline_measurement_source_sha256': baseline,
          'candidate_measurement_source_sha256': manifest['candidate_source_tree_sha256'],
          'validated_root_source_sha256': root_source, 'changed_production_files': changed,
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'patch_sha256': sha(patch.read_bytes()), 'fixture_and_instrumentation_unchanged': True}
(OUT / 'measurement-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
