CREATE TABLE document_schema(version INTEGER PRIMARY KEY CHECK(version=2),profile TEXT NOT NULL);
CREATE TABLE document_source(
 path TEXT NOT NULL,sha TEXT NOT NULL,body BLOB NOT NULL,
 PRIMARY KEY(path,sha)
);
CREATE TABLE document_revision(
 revision TEXT PRIMARY KEY,id TEXT NOT NULL,path TEXT NOT NULL,source_sha TEXT NOT NULL,
 declaration TEXT NOT NULL,evidence TEXT NOT NULL,previous TEXT,
 FOREIGN KEY(previous) REFERENCES document_revision(revision),
 FOREIGN KEY(path,source_sha) REFERENCES document_source(path,sha)
);
CREATE INDEX document_revision_identity ON document_revision(id);
CREATE TABLE document_generation(g TEXT PRIMARY KEY);
CREATE TABLE document_member(
 g TEXT NOT NULL,id TEXT NOT NULL,revision TEXT NOT NULL,
 PRIMARY KEY(g,id),FOREIGN KEY(g) REFERENCES document_generation(g),
 FOREIGN KEY(revision) REFERENCES document_revision(revision)
);
CREATE TABLE document_head(slot INTEGER PRIMARY KEY CHECK(slot=1),g TEXT NOT NULL,
 FOREIGN KEY(g) REFERENCES document_generation(g));
CREATE TABLE document_publication(sequence INTEGER PRIMARY KEY,g TEXT NOT NULL,previous_g TEXT,
 FOREIGN KEY(g) REFERENCES document_generation(g),FOREIGN KEY(previous_g) REFERENCES document_generation(g));
CREATE TABLE document_edge(
 g TEXT NOT NULL,source TEXT NOT NULL,kind TEXT NOT NULL,target TEXT NOT NULL,evidence TEXT NOT NULL,
 PRIMARY KEY(g,source,kind,target),
 FOREIGN KEY(g,source) REFERENCES document_member(g,id),
 FOREIGN KEY(g,target) REFERENCES document_member(g,id)
);
CREATE INDEX document_edge_reverse ON document_edge(g,target,kind,source);
CREATE VIRTUAL TABLE document_search USING fts5(id UNINDEXED,title,body,track);
CREATE TABLE knowledge_unit(
 revision TEXT PRIMARY KEY,scope TEXT NOT NULL,title TEXT NOT NULL,
 kind TEXT NOT NULL,status TEXT NOT NULL,track TEXT NOT NULL,
 FOREIGN KEY(revision) REFERENCES document_revision(revision)
);
CREATE INDEX knowledge_metadata ON knowledge_unit(kind,status,track,revision);
CREATE TABLE knowledge_component(
 revision TEXT NOT NULL,ordinal INTEGER NOT NULL,role TEXT NOT NULL,label TEXT,
 path TEXT NOT NULL,source_sha TEXT NOT NULL,start INTEGER NOT NULL CHECK(start>=0),
 end INTEGER NOT NULL CHECK(end>start),content_sha TEXT NOT NULL,selector TEXT NOT NULL,
 PRIMARY KEY(revision,ordinal),
 FOREIGN KEY(revision) REFERENCES knowledge_unit(revision),
 FOREIGN KEY(path,source_sha) REFERENCES document_source(path,sha)
);
CREATE INDEX knowledge_role ON knowledge_component(role,label,revision,ordinal);
CREATE VIRTUAL TABLE knowledge_search USING fts5(
 id UNINDEXED,revision UNINDEXED,ordinal UNINDEXED,origin UNINDEXED,label UNINDEXED,text
);
