"""Static authoring inventory for the owner-reviewed B11 proposal; never activates it."""
import csv
import hashlib
import io
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT/'verification/architectural-intelligence-b11'
CONTROLLING = {'statement', 'qualification', 'scope', 'constraint',
               'verification_obligation', 'rationale', 'rejected_alternative',
               'rejection_reason', 'consequence'}


def rows(name):
    with (EVIDENCE/name).open(encoding='utf-8', newline='') as stream:
        return list(csv.DictReader(stream))


def write(name, values):
    stream = io.StringIO(newline='')
    writer = csv.DictWriter(stream, fieldnames=list(values[0]))
    writer.writeheader()
    writer.writerows(values)
    (EVIDENCE/name).write_bytes(stream.getvalue().encode('utf-8'))


def inspect(path):
    data = (ROOT/path).read_bytes()
    assert not data.startswith(b'\xef\xbb\xbf'), path
    text = data.decode('utf-8')
    fence = None
    block = []
    declarations = []
    opened = []
    selectors = {}
    offset = 0
    for raw in text.splitlines(keepends=True):
        line = raw.rstrip('\r\n')
        char = line[:1]
        count = len(line) - len(line.lstrip(char)) if char in ('`', '~') else 0
        if fence:
            marker, length, selected = fence
            if char == marker and count >= length and not line[count:].strip():
                if selected:
                    declarations.append(block)
                block = []
                fence = None
            elif selected:
                block.append(line)
        elif count >= 3:
            fence = (char, count, line[count:].strip() == 'architecture-graph')
        elif line.startswith('<!-- pcg:'):
            assert line.endswith(' -->'), (path, line)
            name, operation = line[9:-4].rsplit(':', 1)
            if operation == 'begin':
                assert name not in selectors and all(item[0] != name for item in opened), (path, name)
                opened.append((name, offset + len(raw)))
            else:
                assert operation == 'end' and opened, (path, line)
                expected, start = opened.pop()
                assert expected == name and start < offset, (path, name, expected)
                selectors[name] = (start, offset)
        offset += len(raw)
    assert not opened and fence is None, path
    return data, declarations, selectors


original = rows('B11_PROPOSED_PROTECTED_COMPONENTS.csv')
reviewed_ids = {row['id'] for row in original}
assert len(reviewed_ids) == 137
corpus = rows('B11_PROPOSED_GOVERNED_CORPUS.csv')
expected = {identity: row['source_path'] for row in corpus for identity in row['expected_ids'].split(';')}
assert len(corpus) == 153 and len(expected) == 218
units = {}
sources = {}
for path in sorted(set(expected.values())):
    data, declarations, selectors = inspect(path)
    sources[path] = (data, selectors)
    for block in declarations:
        identity = None
        bindings = []
        selected = False
        for line in block:
            if line.startswith('node: '):
                identity = line[6:]
            if line and not line.startswith(' '):
                selected = line == 'knowledge_content:'
            elif selected and line.startswith('  - '):
                role, locator = line[4:].split(' ', 1)
                role, _, label = role.partition(':')
                source, selector = locator.rsplit('#', 1)
                bindings.append((role, label, path if source == 'self' else source, selector))
        assert identity in expected and expected[identity] == path, (identity, path)
        assert identity not in units, identity
        units[identity] = bindings
assert set(units) == set(expected)

selection = []
pins = []
for identity in sorted(reviewed_ids):
    assert units[identity], identity
    for role, label, source, selector in units[identity]:
        if role not in CONTROLLING:
            continue
        if source not in sources:
            data, _, selectors = inspect(source)
            sources[source] = (data, selectors)
        data, selectors = sources[source]
        text = data.decode('utf-8')
        start, end = (0, len(text)) if selector == 'whole' else selectors[selector]
        selected = text[start:end].encode('utf-8')
        selection.append(dict(id=identity, role=role, label=label, selector=selector))
        pins.append(dict(id=identity, role=role, label=label, selector=selector,
                         source_path=source, source_sha256=hashlib.sha256(data).hexdigest(),
                         content_sha256=hashlib.sha256(selected).hexdigest(),
                         basis='Owner-reviewed identity and explicit source-owned controlling role; dispositions 03/04/05',
                         status='proposed; runtime reconciliation and exact operator bootstrap still required'))
assert {row['id'] for row in selection} == reviewed_ids
keys = {(row['id'], row['role'], row['label'], row['selector']) for row in selection}
assert len(keys) == len(selection)
nonwhole = []
for row in original:
    if row['selector'] == 'whole':
        continue
    key = (row['id'], row['role'], row['label'], row['selector'])
    retained = key in keys
    nonwhole.append(dict(id=row['id'], role=row['role'], label=row['label'],
                         previous_selector=row['selector'], source_path=row['source_path'],
                         disposition='retain' if retained else 'refine mixed boundary',
                         basis='Existing source-owned normative definition, applicable qualification or mandatory verification boundary' if retained else 'Exact controlling clauses retained under explicit refined bindings; original mixed scope remains observation',
                         predecessor_content_sha256=row['content_sha256']))
assert len(nonwhole) == 159
write('B11_FINAL_PROTECTED_SCOPE.csv', selection)
write('B11_FINAL_PROTECTED_COMPONENT_PINS.csv', pins)
write('B11_NONWHOLE_COMPONENT_REVIEW.csv', nonwhole)
print(json.dumps({'governed_paths': len(corpus), 'identities': len(units),
                  'protected_identities': len(reviewed_ids), 'proposed_components': len(selection),
                  'nonwhole_reviewed': len(nonwhole),
                  'nonwhole_retained': sum(row['disposition'] == 'retain' for row in nonwhole),
                  'nonwhole_refined': sum(row['disposition'] != 'retain' for row in nonwhole),
                  'governance_activated': False}))
