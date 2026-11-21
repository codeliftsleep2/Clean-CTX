"""Disposition 21: exactly the sixteen unexecuted controls, stop on first failure."""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
ADAPTER = HERE / 'capture.py'
MANIFEST = HERE / 'reviewed-green-amendment-18-sources.json'
BUILD = ROOT / 'target/ops-r1/shared-green-amendment-18-build-reviewed/build-receipt.json'
SELECTION = HERE.parent / 'test-selections.json'
HISTORY = ROOT / 'target/ops-r1/preservation-bundle-20261120-190637/bundle-review.json'
PINS = {
    ADAPTER: 'eab639d02d55659fb41ea307895f19a9bb4fb486a448860bb16ae47329ee14d3',
    MANIFEST: '21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0',
    BUILD: '714f362e9ab3d949fc4cbda84759872f80cc7b85020a6808b600b347415205b7',
    SELECTION: '2b8c04a0e4d10f5413095a04a5b87cb4416e53e7afc8457254f0cd52726505fa',
    ROOT / 'target/debug/deps/clean_ctx-c147437a0212f889.exe':
        'e2d03e433a4137750f9511dcc5e1178541b049c53a66094799d9660cbe92b403',
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify():
    for path, expected in PINS.items():
        if sha(path) != expected:
            raise ValueError(f'Pinned identity mismatch: {path}')
    manifest = json.loads(MANIFEST.read_bytes())
    names = subprocess.check_output(
        ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
        cwd=ROOT).decode().split('\0')
    names = sorted({n for n in names if n} | {p['path'] for p in manifest['files']})
    rows = []
    for name in names:
        data = (ROOT / name).read_bytes()
        if name.endswith(('.rs', '.toml')) or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': hashlib.sha256(data).hexdigest()})
    digest = hashlib.sha256(json.dumps(rows, separators=(',', ':')).encode()).hexdigest()
    if digest != '25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb':
        raise ValueError('Source-tree mismatch')


def main():
    verify()
    selected = [x for x in json.loads(SELECTION.read_bytes()) if x['case'] != 'fitness']
    old = json.loads(HISTORY.read_bytes())
    prior = old['runs']
    if len(selected) != 25 or len(prior) != 9:
        raise ValueError('Historical selection count mismatch')
    if [x['case'] for x in selected[:9]] != [x['case'] for x in prior]:
        raise ValueError('Historical order mismatch')
    corrected = ROOT / 'target/ops-r1/p0-config_roots_and_route_changes_stay_independent-20261120-205902/receipt.json'
    if sha(corrected) != 'b9e462461a6741a609e1e770cf8f61e067ca7eb1ab6809068b3697590e0095dd':
        raise ValueError('Corrected case-9 receipt mismatch')
    remaining = selected[9:]
    if len(remaining) != 16 or len({x['identity'] for x in selected}) != 25:
        raise ValueError('Unique identity reconciliation failed')
    if len({x['case'] for x in selected}) != 25:
        raise ValueError('Duplicate selectors')
    module = ast.parse(ADAPTER.read_text())
    cases = next(ast.literal_eval(n.value) for n in module.body
                 if isinstance(n, ast.Assign) and any(
                     isinstance(t, ast.Name) and t.id == 'CASES' for t in n.targets))
    manifest = json.loads(MANIFEST.read_bytes())
    for item in remaining:
        units, index = cases[item['case']]
        if units != 1 or manifest['test_identities'][index] != item['identity']:
            raise ValueError('Adapter selector identity mismatch')
        if not (ROOT / item['source']).is_file():
            raise ValueError('Tracked test source missing')
    out = ROOT / 'target/ops-r1' / ('remaining-21-' + time.strftime('%Y%m%d-%H%M%S'))
    out.mkdir(exist_ok=False)
    record = {'agent': 'Agent MaxHeadRoom', 'authorization': 'Owner Disposition 21',
              'status': 'IN_PROGRESS', 'selection': remaining, 'runs': [],
              'script_sha256': sha(Path(__file__)), 'source_tree_sha256':
              manifest['candidate_source_tree_sha256']}
    save = lambda: (out / 'bundle-review.json').write_text(
        json.dumps(record, indent=2) + '\n', encoding='utf-8')
    save()
    env = dict(os.environ, OPS_R1_STAGE='green', OPS_R1_GREEN_SOURCE='amendment-18')
    try:
        for number, item in enumerate(remaining, 1):
            verify()
            print(f'[{number}/16] {item["identity"]}', flush=True)
            result = subprocess.run([sys.executable, str(ADAPTER), '--build-receipt', str(BUILD),
                                     '--case', item['case'], '--wall-seconds', '60',
                                     '--memory-mib', '512', '--stop-wait', '5'],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            (out / f'{number:02d}-adapter-stdout.txt').write_text(result.stdout, encoding='utf-8')
            (out / f'{number:02d}-adapter-stderr.txt').write_text(result.stderr, encoding='utf-8')
            run = {'case': item['case'], 'identity': item['identity'], 'adapter_exit': result.returncode}
            record['runs'].append(run)
            save()
            if result.returncode:
                raise ValueError('Adapter failure: ' + result.stdout + result.stderr)
            folder = Path(result.stdout.strip())
            if not folder.resolve().is_relative_to((ROOT / 'target/ops-r1').resolve()):
                raise ValueError('Unexpected evidence directory')
            receipt = json.loads((folder / 'receipt.json').read_bytes())
            stdout = (folder / 'stdout.txt').read_text()
            run.update(receipt=str((folder / 'receipt.json').relative_to(ROOT)),
                       receipt_sha256=sha(folder / 'receipt.json'), observations=receipt)
            verify()
            good = (receipt['exit_code'] == 0 and receipt['status'] == 'TRACKED_TEST_COMPLETED'
                    and receipt['post_source_and_binary_identity_verified']
                    and receipt['command'][2] == item['identity']
                    and re.search(r'^running 1 test\s*$', stdout, re.M)
                    and re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;', stdout))
            run['passed'] = bool(good)
            save()
            if not good:
                raise ValueError('Test outcome/selection/identity stop: ' + str(folder))
            print('PASS: ' + str(folder), flush=True)
        record['status'] = 'ALL_SIXTEEN_PASSED'
    except Exception as error:
        record.update(status='STOPPED_AT_FIRST_FAILURE', error=str(error))
    record.update(executed=len(record['runs']),
                  passed=sum(bool(x.get('passed')) for x in record['runs']),
                  not_executed=16-len(record['runs']))
    save()
    print(str(out), flush=True)
    return 0 if record['status'] == 'ALL_SIXTEEN_PASSED' else 1


if __name__ == '__main__':
    sys.exit(main())
