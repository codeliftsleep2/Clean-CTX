"""Summarize the B8 production capture against its preserved B7 graph snapshot.

Run from the repository root after run-b8-document-regression.ps1. Requires the
original target/pcg-b7/baseline.json, original exact B7 request file, and the B8
before/after captures. Reads the actual document database read-only and writes
the mission evidence tables. This is operator verification, never a Rust test,
CI gate, independent agent evaluation or evidence of authenticated approval.
"""
from pathlib import Path
import argparse, copy, csv, hashlib, importlib.util, json, sqlite3, sys

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--capture-root',type=Path,default=Path('target/pcg-b8/after'))
parser.add_argument('--report-dir',type=Path,default=Path('docs/architecture/knowledge-normalization'))
args=parser.parse_args()
R=Path.cwd(); T=R/'target/pcg-b8'; B=args.report_dir; capture=args.capture_root.resolve()
B.mkdir(parents=True,exist_ok=True)
def load(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def table(name, rows):
    with (B/name).open('w',encoding='utf-8',newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
def encoded(value): return json.dumps(value,ensure_ascii=False,separators=(',',':')).encode()
def sha(data): return hashlib.sha256(data).hexdigest()

sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('consumer',R/'verification/pcg-175/compare-document-source-responses.py')
consumer=importlib.util.module_from_spec(spec);spec.loader.exec_module(consumer)
profile=load(R/'.clean-ctx/pcg-documents/profile.json')
c=sqlite3.connect(Path(profile['database_path']).as_uri()+'?mode=ro',uri=True)
g=c.execute('select g from document_head where slot=1').fetchone()[0]
members=list(c.execute('select m.id,m.revision,r.path,r.source_sha,r.declaration from document_member m join document_revision r on r.revision=m.revision where m.g=? order by m.id',(g,)))
edges=list(c.execute('select source,kind,target from document_edge where g=? order by source,kind,target',(g,)))
baseline=load(R/'target/pcg-b7/baseline.json')
old={r[0]:r for r in baseline['members']};new={r[0]:r for r in members}
assert old.keys()==new.keys() and len(old)==218
assert set(map(tuple,baseline['edges']))==set(edges) and len(edges)==221
changes=[];preservation=[]
for ident,row in new.items():
    prior=old[ident]
    assert row[2]==prior[2] and json.loads(row[4])==json.loads(prior[4])
    assert sha((R/row[2]).read_bytes())==row[3]
    historical=c.execute('select path,source_sha,declaration,body from document_revision where revision=?',(prior[1],)).fetchone()
    assert historical and historical[0]==prior[2] and historical[1]==prior[3]
    assert json.loads(historical[2])==json.loads(prior[4]) and sha(historical[3])==prior[3]
    advanced=row[1]!=prior[1]
    if advanced:
        assert row[2]=='docs/architecture/PCG_DOCUMENT_AUTHORING.md'
        changes.append(ident)
        predecessor=row[1];seen=set()
        while predecessor!=prior[1]:
            assert predecessor and predecessor not in seen
            seen.add(predecessor)
            predecessor=c.execute('select previous from document_revision where revision=?',(predecessor,)).fetchone()[0]
    preservation.append(dict(unit='node',identity=ident,kind='',target='',baseline_revision=prior[1],final_revision=row[1],baseline_source_sha256=prior[3],final_source_sha256=row[3],declaration_unchanged=True,revision_disposition='Authorized normative source policy supersession' if advanced else 'Exact original revision',generation=g))
for source,kind,target in edges:
    preservation.append(dict(unit='edge',identity=source,kind=kind,target=target,baseline_revision='',final_revision='',baseline_source_sha256='',final_source_sha256='',declaration_unchanged=True,revision_disposition='Exact original typed topology',generation=g))
table('B8_GRAPH_PRESERVATION.csv',preservation)

incoming={'SUPERSEDES','FALSIFIES','VALIDATES','RECLOSES','BANKS'}
def expected_context(seed):
    selected={seed}
    while True:
        relevant={e for e in edges if e[0] in selected or (e[2] in selected and e[1] in incoming)}
        expanded=selected|{v for e in relevant for v in [e[0],e[2]]}
        if selected==expanded:return selected,relevant
        selected=expanded
def validate(q,p):
    assert p['generation']==g and p['catalog_current'] and p['result_state']=='ready' and not p['diagnostics']
    assert p['budget']['limit'] is None and p['budget']['enforced'] is False
    inline=consumer.reconstruct(copy.deepcopy(p)) if p.get('response_format')=='source_refs_v1' else p
    docs=inline['documents'];ids={d['declaration']['node'] for d in docs}
    assert len(ids)==len(docs)
    for d in docs:
        ident=d['declaration']['node'];row=new[ident]
        assert d['revision']==row[1] and d['source_sha256']==row[3]
        assert sha(d['source_text'].encode())==row[3]==sha((R/d['path']).read_bytes())
        assert d['declaration']==json.loads(row[4])
        for ev in d['evidence']:
            assert sha((R/ev['path']).read_bytes())==ev['sha256']
            if ev.get('text') is not None:
                assert isinstance(ev['text'],str) and sha(ev['text'].encode())==ev['sha256']
    mode=q['mode'];seed=q.get('id')
    if mode=='search':
        if q.get('text'):
            selected={r[0] for r in c.execute('select id from document_search where document_search match ?',(q['text'],))}
        else:selected=set(new)
        if q.get('track'):selected={i for i in selected if json.loads(new[i][4])['track']==q['track']}
        assert ids==selected
    elif mode=='get':assert ids=={seed}
    elif mode=='related':assert ids=={b if a==seed else a for a,k,b in edges if a==seed or b==seed}
    elif mode=='context':
        expected,expected_edges=expected_context(seed)
        ctx=p['context'];assert ctx['complete'] and ids==expected
        actual_edges={(e['source'],e['kind'],e['target']) for e in ctx['relationships']}
        assert actual_edges==expected_edges and len(actual_edges)==len(ctx['relationships'])
        assert {w['node'] for w in ctx['paths']}==ids and len(ctx['paths'])==len(ids)
        for witness in ctx['paths']:
            at=seed
            for step in witness['steps']:
                a,k,b=step['source'],step['kind'],step['target'];assert (a,k,b) in actual_edges
                if step['direction']=='outgoing':assert at==a;at=b
                else:assert at==b and k in incoming;at=a
            assert at==witness['node']
    return inline

before=load(T/'before/receipt.json'); oldq=load(T/'regression-queries.json')
assert oldq==load(R/'target/pcg-b7/assessment-queries.json')==load(R/'verification/pcg-175/b8-regression-queries.json')
allrows=[];discovery=[];contexts=[];composition=[]
for phase in ['regression','context-pairs','architecture']:
    folder=capture/phase;receipt=load(folder/'receipt.json')
    assert receipt['process_exit_code']==0 and not receipt['error']
    for obs in receipt['queries']:
        number=obs['request'];q=load(folder/f'{number}.request.json')['params']['arguments'];p=load(folder/f'{number}.payload.json')
        inline=validate(q,p);docs=inline['documents'];ctx=p['context']
        summary=dict(phase=phase,operation=q['mode'],query=q.get('text',q.get('id')),format=q.get('responseFormat','inline_v1'),state=p['result_state'],current=p['catalog_current'],complete=ctx['complete'] if ctx else 'No closure claim',nodes=len(docs),edges=len(ctx['relationships']) if ctx else '',witnesses=len(ctx['paths']) if ctx else '',source_content='Exact hash and current filesystem bytes',membership='Exact independent selected set',generation=g,binary_sha256=receipt['binary_sha256'],profile_sha256=receipt['profile_sha256'],serialized_tokens=obs['serialized_tokens'],token_enforcement=False,receipt=str(folder.relative_to(R)/f'{number}.response.json'))
        allrows.append(dict(summary,
            returned_membership=json.dumps([dict(id=d['declaration']['node'],revision=d['revision'],path=d['path'],source_sha256=d['source_sha256']) for d in docs],separators=(',',':')),
            relationships=json.dumps(ctx['relationships'],separators=(',',':')) if ctx else '',
            witness_paths=json.dumps(ctx['paths'],separators=(',',':')) if ctx else ''))
        if phase=='regression':
            prior=before['queries'][number-2]
            assert q==oldq[number-2]
            row=dict(summary,before_state=prior['result_state'],before_tokens=prior['serialized_tokens'],before_diagnostics=json.dumps(prior['diagnostics'],separators=(',',':')),exact_B7_request=True)
            if q['mode']=='search':discovery.append(row)
            if q['mode']=='context':contexts.append(row)
        sources=[d['source_text'] for d in p['documents'] if 'source_text' in d]+[s['text'] for s in p.get('sources',[])]
        total=sum(len(s.encode()) for s in sources);unique=sum(len(s.encode()) for s in set(sources))
        evidence=[ev for d in docs for ev in d['evidence']]
        composition.append(dict(phase=phase,operation=q['mode'],query=q.get('text',q.get('id')),format=q.get('responseFormat','inline_v1'),generation=g,binary_sha256=receipt['binary_sha256'],nodes=len(docs),identity_metadata_json_bytes=sum(len(encoded({k:v for k,v in d.items() if k not in ['source_text','evidence']})) for d in p['documents']),proposition_content='Contained in whole source;not independently bounded',rationale='Prose;not separate structured field',complete_source_utf8_bytes=total,repeated_returned_source_utf8_bytes=total-unique,evidence_associations=len(evidence),historical_evidence='Exact source/artifact associations; no new execution claim',relationship_json_bytes=len(encoded(ctx['relationships'])) if ctx else 0,witness_json_bytes=len(encoded(ctx['paths'])) if ctx else 0,source_pool_entries=len(p.get('sources',[])),exact_serialized_result_bytes=(folder/f'{number}.serialized-result.json').stat().st_size,transport='MCP text inner JSON; metadata byte groups are descriptive and not an additive partition',receipt=str(folder.relative_to(R)/f'{number}.response.json')))
assert len(discovery)==10 and len(contexts)==6 and len(allrows)==33
table('B8_DISCOVERY_VERIFICATION.csv',discovery)
table('B8_CONTEXT_VERIFICATION.csv',contexts)
table('B8_POST_REMOVAL_QUERY_RECEIPTS.csv',allrows)
table('B8_POST_REMOVAL_PAYLOAD_COMPOSITION.csv',composition)
with (capture/'reconstruction.csv').open(encoding='utf-8',newline='') as f:transport=list(csv.DictReader(f))
for row in transport:
    row['standing']='Independent exact reconstruction of complete current production formats;not independent agent certification'
    row['binary_sha256']=allrows[0]['binary_sha256']
table('B8_TRANSPORT_VERIFICATION.csv',transport)
summary=dict(generation=g,nodes=len(new),edges=len(edges),advanced_revisions=changes,exact_original_revisions=218-len(changes),production_queries=len(allrows),B7_searches_ready=len(discovery),required_complete_contexts=len(contexts),exact_reconstructed_pairs=len(transport),binary_sha256=allrows[0]['binary_sha256'],profile_sha256=allrows[0]['profile_sha256'])
(capture/'summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
print(json.dumps(summary,indent=2))
c.close()
