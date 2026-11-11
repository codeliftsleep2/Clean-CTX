CREATE TABLE pcg_schema(version INTEGER PRIMARY KEY CHECK(version=1));
INSERT INTO pcg_schema VALUES(1);
CREATE TABLE corpus(
 p TEXT NOT NULL,c TEXT NOT NULL,namespace TEXT NOT NULL,
 PRIMARY KEY(p,c)
);
CREATE TABLE generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL CHECK(length(g)=64),
 manifest_sha TEXT NOT NULL CHECK(length(manifest_sha)=64),policy TEXT NOT NULL,
 parser TEXT NOT NULL,source_commit TEXT NOT NULL,fingerprint TEXT NOT NULL,
 enrollment_complete INTEGER NOT NULL CHECK(enrollment_complete IN(0,1)),
 PRIMARY KEY(p,c,g),UNIQUE(p,c,manifest_sha,policy,parser),
 FOREIGN KEY(p,c) REFERENCES corpus(p,c) ON DELETE CASCADE
);
CREATE TABLE active_generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,
 PRIMARY KEY(p,c),FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g)
);
CREATE TABLE source_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,sid TEXT NOT NULL,srev TEXT NOT NULL,
 path TEXT NOT NULL,git_commit TEXT NOT NULL,blob TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 check_mode TEXT NOT NULL CHECK(check_mode IN('current','historical')),
 PRIMARY KEY(p,c,g,sid,srev),UNIQUE(p,c,g,sid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE evidence_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 sid TEXT NOT NULL,srev TEXT NOT NULL,bstart INTEGER NOT NULL CHECK(bstart>=0),
 bend INTEGER NOT NULL CHECK(bend>bstart),quote_sha TEXT NOT NULL CHECK(length(quote_sha)=64),
 quote_utf8 BLOB NOT NULL CHECK(length(quote_utf8)=bend-bstart),
 PRIMARY KEY(p,c,g,eid,erev),UNIQUE(p,c,g,eid),
 FOREIGN KEY(p,c,g,sid,srev) REFERENCES source_revision(p,c,g,sid,srev) ON DELETE CASCADE
);
CREATE TABLE record_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 role TEXT NOT NULL CHECK(role IN('claim','decision','constraint','evidence')),
 required INTEGER NOT NULL CHECK(required IN(0,1)),eid TEXT NOT NULL,erev TEXT NOT NULL,
 authority TEXT NOT NULL CHECK(authority IN('hypothesis','unverified_extraction','attributable_observation','verified_claim','explicitly_approved_decision')),
 attribution TEXT NOT NULL,verification TEXT NOT NULL CHECK(verification IN('unknown','supported','contradicted','inconclusive')),
 verification_subject TEXT NOT NULL,execution TEXT NOT NULL CHECK(execution IN('not_run','report_of_execution','executed_artifact')),
 lifecycle TEXT NOT NULL CHECK(lifecycle IN('active','superseded','retracted','falsified')),
 subject TEXT NOT NULL,disposition TEXT NOT NULL,
 PRIMARY KEY(p,c,g,rid,rrev),UNIQUE(p,c,g,rid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE evidence_ref(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,required INTEGER NOT NULL CHECK(required IN(0,1)),
 PRIMARY KEY(p,c,g,rid,rrev,eid,erev),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE selector(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,
 purpose TEXT NOT NULL CHECK(purpose IN('coverage','applicability')),
 workspace TEXT NOT NULL,file_path TEXT,entity_json TEXT,operation TEXT,task TEXT,
 effective_from TEXT,effective_until TEXT,normalized_json TEXT NOT NULL,
 PRIMARY KEY(p,c,g,xid),UNIQUE(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE binding(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,bid TEXT NOT NULL,brev TEXT NOT NULL,
 rid TEXT NOT NULL,rrev TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 PRIMARY KEY(p,c,g,bid),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,xid,xrev) REFERENCES selector(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE relation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,aid TEXT NOT NULL,arev TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('DEPENDS_ON','BLOCKED_BY','USES_EVIDENCE','VALIDATES','FALSIFIES','SUPERSEDES','PRESERVES','IMPLEMENTS')),
 src TEXT NOT NULL,src_rev TEXT NOT NULL,dst TEXT NOT NULL,dst_rev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,gate_json TEXT,
 PRIMARY KEY(p,c,g,aid),
 CHECK((kind='BLOCKED_BY' AND gate_json IS NOT NULL) OR (kind<>'BLOCKED_BY' AND gate_json IS NULL)),
 FOREIGN KEY(p,c,g,src,src_rev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,dst,dst_rev) REFERENCES record_revision(p,c,g,rid,rrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE INDEX record_subject ON record_revision(p,c,g,subject,lifecycle);
CREATE INDEX selector_scope ON selector(p,c,g,purpose,workspace,file_path,operation,task);
CREATE INDEX binding_selector ON binding(p,c,g,xid,rid);
CREATE INDEX relation_forward ON relation(p,c,g,src,kind,dst);
CREATE INDEX relation_reverse ON relation(p,c,g,dst,kind,src);
CREATE INDEX evidence_source ON evidence_revision(p,c,g,sid,srev);
CREATE INDEX evidence_required ON evidence_ref(p,c,g,rid,rrev,required,eid,erev);
CREATE INDEX source_check ON source_revision(p,c,g,check_mode,sid);
