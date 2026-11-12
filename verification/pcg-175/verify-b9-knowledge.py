"""Read-only independent verification of actual-root B9 stdio captures.

Run at the repository root after invoke-workspace-knowledge.ps1. Requires the
preserved B9 baseline snapshot. Generates mission tables, never edits SQLite.
This is operator verification, not a tracked Rust test or independent evaluation.
"""
from pathlib import Path
import argparse,csv,hashlib,json,sqlite3

def load(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def digest(raw):return hashlib.sha256(raw).hexdigest()
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('capture',type=Path)
parser.add_argument('--reports',type=Path,default=Path('docs/architecture/knowledge-normalization'))
args=parser.parse_args();R=Path.cwd();B=args.reports;B.mkdir(parents=True,exist_ok=True)
def table(name,rows):
    assert rows,name
    with (B/name).open('w',encoding='utf-8',newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
profile=load(R/'.clean-ctx/pcg-documents/profile.json')
c=sqlite3.connect(Path(profile['database_path']).as_uri()+'?mode=ro',uri=True)
assert c.execute('select version from document_schema').fetchone()[0]==2
assert not list(c.execute('pragma foreign_key_check'))
g=c.execute('select g from document_head').fetchone()[0]
baseline=load(R/'target/pcg-b9/baseline.json');old={n['id']:n for n in baseline['nodes']}
members={i:(r,p,h,json.loads(d),json.loads(e)) for i,r,p,h,d,e in c.execute('select m.id,m.revision,r.path,r.source_sha,r.declaration,r.evidence from document_member m join document_revision r using(revision) where g=?',(g,))}
edges=set(c.execute('select source,kind,target from document_edge where g=?',(g,)))
assert set(old)==set(members) and len(members)==218
assert set(map(tuple,baseline['edges']))==edges and len(edges)==221
preserved=[]
for ident,(revision,path,source_sha,declaration,evidence) in members.items():
    prior=old[ident];assert declaration==prior['declaration'] and path==prior['path']
    assert digest((R/path).read_bytes())==source_sha
    historical=c.execute('select r.path,r.source_sha,r.declaration,s.body,r.evidence from document_revision r join document_source s on s.path=r.path and s.sha=r.source_sha where revision=?',(prior['revision'],)).fetchone()
    assert historical and historical[0]==prior['path'] and historical[1]==prior['source_sha256']
    assert json.loads(historical[2])==prior['declaration'] and digest(historical[3])==prior['source_sha256']
    assert json.loads(historical[4])==prior['evidence']
    for ev in prior['evidence']:assert any(n['kind']==ev['kind'] and n['path']==ev['path'] for n in evidence)
    seen=set();at=revision
    while at!=prior['revision']:
        assert at and at not in seen;seen.add(at)
        at=c.execute('select previous from document_revision where revision=?',(at,)).fetchone()[0]
    scope=c.execute('select scope from knowledge_unit where revision=?',(revision,)).fetchone()[0]
    preserved.append(dict(id=ident,scope=scope,baseline_revision=prior['revision'],final_revision=revision,baseline_original_body='exact retained',declaration='exact unchanged',relationships='exact unchanged',evidence_associations='all preserved',generation=g))
table('B9_GRAPH_PRESERVATION.csv',preserved)
incoming={'SUPERSEDES','FALSIFIES','VALIDATES','RECLOSES','BANKS'}
def closure(seed):
    found={seed}
    while True:
        selected={e for e in edges if e[0] in found or (e[2] in found and e[1] in incoming)}
        expanded=found|{v for e in selected for v in [e[0],e[2]]}
        if expanded==found:return found,selected
        found=expanded
receipt=load(args.capture/'receipt.json');assert receipt['process_exit_code']==0 and not receipt['error']
assert receipt['binary_sha256']==digest((R/'target/debug/clean-ctx.exe').read_bytes())
out=[];contexts=[];fts=[];history=[];sources=[];allgets=set();boundaries=[];passmatches=None;selected_content={}
for obs in receipt['queries']:
    number=obs['request'];q=load(args.capture/f'{number}.request.json')['params']['arguments'];p=load(args.capture/f'{number}.payload.json')
    assert p['result_state']=='ready' and p['catalog_current'] and p['complete'] and not p['diagnostics'],(number,p.get('diagnostics'))
    assert p['generation']==g and p['catalog_coverage']['represented']==218 and not p['catalog_coverage']['unrepresented']
    assert p['token_budget_enforced'] is False
    ids={u['id'] for u in p['units']};assert len(ids)==len(p['units'])
    for unit in p['units']:
        ident=unit['id'];assert unit['revision']==members[ident][0]
        if 'components' in unit:
            expected=[]
            for row in c.execute('select ordinal,role,label,path,source_sha,start,end,content_sha,selector from knowledge_component where revision=? order by ordinal',(unit['revision'],)):
                ordinal,role,label,path,source_sha,start,end,content_sha,selector=row
                selected=(not q.get('role') or q['role']==role) and (not q.get('label') or q['label']==label)
                expected.append((row,selected))
            has_selected=any(selected for _,selected in expected)
            expected=[row for row,selected in expected if selected or (has_selected and row[1]=='qualification')]
            assert [x['ordinal'] for x in unit['components']]==[x[0] for x in expected]
            for component,row in zip(unit['components'],expected):
                ordinal,role,label,path,source_sha,start,end,content_sha,selector=row
                raw=(R/path).read_bytes();assert digest(raw)==source_sha
                assert raw[start:end].decode('utf-8')==component['text'] and digest(raw[start:end])==content_sha
                assert component['source_sha256']==source_sha and component['content_sha256']==content_sha
                assert component['source_request']['sourceSha256']==source_sha
                if q['mode']=='get' and not q.get('role'):
                    boundaries.append(dict(id=ident,scope=unit['scope'],ordinal=ordinal,role=role,label=label,path=path,selector=selector,start=start,end=end,source_sha256=source_sha,content_sha256=content_sha,exact='source bytes verified',revision=unit['revision'],generation=g))
            for ev in unit['evidence']:assert digest((R/ev['path']).read_bytes())==ev['sha256'] and 'text' not in ev
    if q['mode']=='search':
        query='select distinct k.id from knowledge_search k join document_member m on m.id=k.id and m.revision=k.revision join knowledge_unit u on u.revision=m.revision where m.g=?'
        parameters=[g]
        for field,column in [('track','u.track'),('kind','u.kind'),('status','u.status'),('role','k.origin'),('label','k.label')]:
            if field in q:query+=' and '+column+'=?';parameters.append(q[field])
        if 'text' in q:query+=' and k.text match ?';parameters.append(q['text'])
        assert ids=={x[0] for x in c.execute(query,parameters)}
        assert not p['sources'] and all('components' not in u for u in p['units'])
        if q.get('text')=='PassPipeline':
            passmatches={u['id']:[m['origin'] for m in u['matches']] for u in p['units']}
            assert set(passmatches)=={'PIPELINE-001','DOC-308','DOC-634','PCG-175-DISCOVERY'},passmatches
        fts.append(dict(text=q.get('text',''),role=q.get('role',''),label=q.get('label',''),membership=json.dumps(sorted(ids)),matches=json.dumps({u['id']:u['matches'] for u in p['units']},separators=(',',':')),correct='exact independent indexed selection',source_body_materialization=False,generation=g))
    elif q['mode']=='context':
        expected,selected=closure(q['id']);assert ids==expected
        actual={(e['source'],e['kind'],e['target']) for e in p['relationships']};assert actual==selected
        assert {w['node'] for w in p['paths']}==ids
        for witness in p['paths']:
            at=q['id']
            for step in witness['steps']:
                a,k,b=step['source'],step['kind'],step['target'];assert (a,k,b) in actual
                if step['direction']=='outgoing':assert at==a;at=b
                else:assert at==b and k in incoming;at=a
            assert at==witness['node']
        contexts.append(dict(id=q['id'],nodes=len(ids),edges=len(actual),witnesses=len(p['paths']),members=json.dumps(sorted(ids)),complete='exact declared closure',content='all declared exact components',evidence='exact current source references',generation=g))
    elif q['mode']=='related':
        direction=q.get('direction','both');expected={e for e in edges if (not q.get('edgeKind') or e[1]==q['edgeKind']) and ((e[0]==q['id'] and direction in ['outgoing','both']) or (e[2]==q['id'] and direction in ['incoming','both']))}
        assert {(e['source'],e['kind'],e['target']) for e in p['relationships']}==expected
        assert ids=={q['id']}|{v for e in expected for v in [e[0],e[2]]}
        assert all('components' not in u for u in p['units'])
    elif q['mode']=='source':
        for source in p['sources']:
            raw=source['text'].encode('utf-8') if source['encoding']=='utf8' else bytes(source['bytes'])
            assert raw==(R/source['path']).read_bytes() and digest(raw)==source['source_sha256']
            sources.append(dict(id=q['id'],path=source['path'],sha256=source['source_sha256'],exact_original=True,encoding=source['encoding'],generation=g))
    elif q['mode']=='get':
        assert ids=={q['id']}
        selected_content[(q['id'],q.get('role'),q.get('label'))]='\n'.join(x['text'] for x in p['units'][0]['components'])
        if not q.get('role'):allgets.add(q['id'])
        else:history.append(dict(id=q['id'],role=q['role'],label=q.get('label',''),content=json.dumps(p['units'][0]['components'],ensure_ascii=False,separators=(',',':')),complete='requested predicate plus required qualifications',lifecycle=p['units'][0]['status'],generation=g))
    out.append(dict(request=number,operation=q['mode'],id=q.get('id',''),text=q.get('text',''),role=q.get('role',''),label=q.get('label',''),state=p['result_state'],complete=True,current=True,units=len(ids),generation=g,binary_sha256=receipt['binary_sha256'],profile_sha256=receipt['profile_sha256'],independent_check='PASS'))
assert allgets==set(old) and passmatches is not None
def witness(identity,role,label,*phrases):
    text=selected_content[(identity,role,label)].lower()
    for phrase in phrases:assert phrase.lower() in text,(identity,role,label,phrase)
witness('PIPELINE-001',None,None,'CoreIRPass','LanguageLayerPass','MetaLayerPass','PatternRecognitionPass','AliasResolutionPass','ValidationPass','Rationale for ordering')
assert '### WIRE-001' not in selected_content[('PIPELINE-001',None,None)]
witness('PERF-001-REPLAY','outcome','REJECT','Owner accepted REJECT','earlier-Candidate reuse')
witness('PERF-001-REPLAY','counterexample','registered_delete','registered-deletion case is decisive','542-byte','607-byte','text-cache','Production remains original')
witness('DOC-206','observation','earlier_focused_green','18 passed','0 failed','post-repair live acceptance remains unrun')
witness('DOC-206','observation','live_639_644','639/644','five failed','direct entity deletion','explicit configuration deletion','ambiguous','missing entity','changed assembly marker')
witness('DOC-206','observation','valid_tracked_red','0 passed','1 failed','tracked behavioral RED')
witness('DOC-206','observation','reported_local_repair','get_mut','Focused GREEN and live','NOT RUN after this repair')
witness('AUTH.ANGULAR_RESOLVER','scope','written_membership','RouteOccurrence','DeclarationId','no','executable','Public key-specific answers remain OPEN')
witness('AUTH.ANGULAR_RESOLVER','qualification','historical_input_drift','68 of 1,224','different current binary','Current-producer revalidation')
witness('AUTH.ANGULAR_RESOLVER','verification_obligation','public_keys_and_current_producer','public key','OPEN','No executable eligibility')
witness('DOC-228','observation','historical_binary_acceptance','319/319','three FINISHED sessions','Public resolver-key questions remain OPEN','Full repository Final Verification Gate')
for name,rows in [('B9_SOURCE_BOUNDARY_INVENTORY.csv',boundaries),('B9_FTS_CORRECTNESS.csv',fts),('B9_CONTEXT_COMPLETENESS.csv',contexts),('B9_HISTORICAL_DECISION_REGRESSIONS.csv',history),('B9_SOURCE_EVIDENCE_VERIFICATION.csv',sources),('B9_PRODUCTION_VERIFICATION.csv',out)]:table(name,rows)
summary=dict(generation=g,identities=218,typed_relationships=221,queries=len(out),all_identity_gets=len(allgets),source_owned_components=len(boundaries),contexts=len(contexts),PassPipeline=passmatches,binary_sha256=receipt['binary_sha256'],profile_sha256=receipt['profile_sha256'])
(args.capture/'verification-summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8');print(json.dumps(summary,indent=2))
c.close()
