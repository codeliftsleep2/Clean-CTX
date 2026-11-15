"""Read-only catalog preservation and exact source-acquisition receipt checks."""
import hashlib
import csv
import json
import sqlite3
import sys
from pathlib import Path


def snapshot(database):
    with sqlite3.connect(Path(database).resolve().as_uri() + '?mode=ro', uri=True) as connection:
        connection.execute('BEGIN')
        head = connection.execute('SELECT g FROM document_head WHERE slot=1').fetchone()[0]
        tables = {row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        source_rows = connection.execute('SELECT path,sha,body FROM document_source ORDER BY path,sha').fetchall()
        historical_sources = []
        for path, expected_hash, body in source_rows:
            actual_hash = hashlib.sha256(body).hexdigest()
            assert actual_hash == expected_hash, ('Stored source corruption', path, expected_hash)
            historical_sources.append([path, expected_hash, actual_hash])
        historical_rows = {}
        for row in connection.execute('SELECT * FROM document_revision ORDER BY revision'):
            encoded = json.dumps(row, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
            historical_rows[row[0]] = hashlib.sha256(encoded).hexdigest()
        return {
            'generation': head,
            'schema': connection.execute('SELECT version FROM document_schema').fetchone()[0],
            'members': connection.execute(
                'SELECT m.id,r.path,m.revision FROM document_member m JOIN document_revision r '
                'ON r.revision=m.revision WHERE m.g=? ORDER BY m.id', (head,)).fetchall(),
            'relationships': connection.execute(
                'SELECT source,kind,target FROM document_edge WHERE g=? ORDER BY source,kind,target',
                (head,)).fetchall(),
            'historical_revisions': [row[0] for row in connection.execute(
                'SELECT revision FROM document_revision ORDER BY revision')],
            'historical_revision_rows': historical_rows,
            'historical_sources': historical_sources,
            'publication_history': connection.execute('SELECT * FROM document_publication ORDER BY sequence').fetchall(),
            'components': connection.execute(
                'SELECT count(*) FROM knowledge_component k JOIN document_member m '
                'ON m.revision=k.revision WHERE m.g=?', (head,)).fetchone()[0],
            'governance_publications': connection.execute(
                'SELECT gp.g,gp.state_digest,gp.authorization_digest,gp.predecessor_g '
                'FROM document_publication p JOIN document_governance_publication gp ON gp.g=p.g '
                'ORDER BY p.sequence').fetchall() if 'document_governance_publication' in tables else [],
            'authorization_grants': connection.execute(
                'SELECT digest,workspace,predecessor_g FROM document_governance_authorization '
                'ORDER BY digest').fetchall() if 'document_governance_authorization' in tables else [],
        }


mode, first, second = sys.argv[1:]
if mode == 'snapshot':
    result = snapshot(first)
    output = Path(second)
else:
    assert mode == 'verify'
    root, directory = Path(first).resolve(), Path(second).resolve()
    before = json.loads((directory/'catalog-before.json').read_text(encoding='utf-8'))
    after = json.loads((directory/'catalog-after.json').read_text(encoding='utf-8'))
    assert [row[:2] for row in before['members']] == [row[:2] for row in after['members']], 'Membership/ownership drift'
    assert before['relationships'] == after['relationships'], 'Typed relationship drift'
    assert set(before['historical_revisions']) <= set(after['historical_revisions']), 'Historical revision loss'
    for revision, digest in before['historical_revision_rows'].items():
        assert after['historical_revision_rows'][revision] == digest, ('Historical revision mutation', revision)
    assert {tuple(row) for row in before['historical_sources']} <= {tuple(row) for row in after['historical_sources']}, 'Historical source mutation/loss'
    assert before['publication_history'] == after['publication_history'][:len(before['publication_history'])], 'Publication history mutation'
    acquired = json.loads((directory/'acquisition/2.payload.json').read_text(encoding='utf-8'))
    assert acquired['complete'] and acquired['catalog_current'] and acquired['result_state'] == 'ready'
    checked = set()
    components = {}
    for item in acquired['acquisitions']:
        assert item['complete'] and item['result_state'] == 'ready'
        for component in item['components']:
            path = (root/component['path']).resolve(strict=True)
            assert path.is_relative_to(root), 'Source escaped workspace'
            source = path.read_bytes()
            assert hashlib.sha256(source).hexdigest() == component['source_sha256']
            selected = source[component['start']:component['end']]
            assert hashlib.sha256(selected).hexdigest() == component['content_sha256']
            assert selected.decode('utf-8') == component['text'], 'Acquired text differs from exact source span'
            checked.add((component['path'], component['start'], component['end']))
            key = (item['reference']['id'], component['role'], component['label'] or '', component['selector'])
            components[key] = component
    selection = directory/'selection-pins.csv'
    if selection.exists():
        with selection.open(encoding='utf-8', newline='') as stream:
            pins = list(csv.DictReader(stream))
        for pin in pins:
            key = (pin['id'], pin['role'], pin['label'], pin['selector'])
            actual = components[key]
            assert actual['source_sha256'] == pin['source_sha256'], ('Selection source drift', key)
            assert actual['content_sha256'] == pin['content_sha256'], ('Selection content drift', key)
            assert actual['path'] == pin['source_path'], ('Selection ownership drift', key)
        assert len({pin['id'] for pin in pins}) == 137
        assert len(pins) == 342
        assert not after['authorization_grants'] and not after['governance_publications'], 'Premature real-workspace governance activation'
        historical = components[('AUTH.PROXY_TRANSFORMS', 'historical_applicability', 'superseded_stripping', 'PROXY-superseded-stripping')]
        assert 'Any existing `cache_control` headers sent by the client are stripped first' in historical['text']
    old = {row[0]: row[2] for row in before['members']}
    result = {
        'membership_and_typed_relationships_preserved': True,
        'historical_revisions_preserved': True,
        'generation': after['generation'],
        'identities': len(after['members']), 'relationships': len(after['relationships']),
        'components': after['components'], 'exact_source_spans_verified': len(checked),
        'proposed_protected_components_verified': len(pins) if selection.exists() else None,
        'changed_revisions': [{'id': row[0], 'previous': old[row[0]], 'current': row[2]}
                              for row in after['members'] if old[row[0]] != row[2]],
        'governance_activation': 'not_requested',
        'qualification': 'Production acquisition and preservation evidence; not CI or independent certification',
    }
    output = directory/'verification.json'
with output.open('x', encoding='utf-8', newline='\n') as stream:
    json.dump(result, stream, indent=2, ensure_ascii=False)
    stream.write('\n')
print(f'{mode} evidence: {output}')
