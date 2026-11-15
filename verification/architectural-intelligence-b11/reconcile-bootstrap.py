"""Bounded read-only SQLite/capture reconciliation; never publishes governance."""
import csv
import hashlib
import json
import sqlite3
from pathlib import Path


def read_csv(path):
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream))


root = Path(__file__).resolve().parents[2]
evidence = root / "verification/architectural-intelligence-b11"
profile = json.loads((root / ".clean-ctx/pcg-documents/profile.json").read_text(encoding="utf-8"))
database = Path(profile["database_path"]).resolve()
connection = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)
connection.execute("BEGIN")
head = connection.execute("SELECT g FROM document_head WHERE slot=1").fetchone()[0]
accepted = connection.execute(
    "SELECT m.id,r.path,r.revision,r.source_sha,r.declaration FROM document_member m "
    "JOIN document_revision r ON r.revision=m.revision WHERE m.g=? ORDER BY m.id", (head,)
).fetchall()
proposal = read_csv(evidence / "B11_PROPOSED_GOVERNED_CORPUS.csv")
captured = read_csv(evidence / "B11_BOOTSTRAP_ACCEPTED_OWNERSHIP.csv")
expected = {}
paths = set()
for row in proposal:
    path = row["source_path"]
    assert "\\" not in path and not Path(path).is_absolute(), path
    assert all(part not in (".", "..", "") for part in path.split("/")), path
    assert path not in paths, f"duplicate path: {path}"
    paths.add(path)
    resolved = (root / path).resolve(strict=True)
    assert resolved.is_relative_to(root), f"foreign root: {path}"
    assert any(resolved.is_relative_to((root / item).resolve()) for item in profile["document_roots"]), path
    for identity in row["expected_ids"].split(";"):
        assert identity not in expected, f"duplicate identity: {identity}"
        expected[identity] = path
assert expected == {identity: path for identity, path, *_ in accepted}
assert {row["id"]: (row["path"], row["revision"]) for row in captured} == {
    identity: (path, revision) for identity, path, revision, *_ in accepted
}
drift = []
for identity, path, revision, source_hash, declaration in accepted:
    assert json.loads(declaration)["node"] == identity
    if hashlib.sha256((root / path).read_bytes()).hexdigest() != source_hash:
        drift.append({"id": identity, "path": path, "accepted_revision": revision})
protected = read_csv(evidence / "B11_PROPOSED_PROTECTED_COMPONENTS.csv")
for row in protected:
    stored = connection.execute(
        "SELECT role,label,path,selector,content_sha,source_sha FROM knowledge_component "
        "WHERE revision=? AND ordinal=?", (row["predecessor_revision"], int(row["ordinal"]))
    ).fetchone()
    assert stored is not None
    assert stored == (row["role"], row["label"] or None, row["source_path"], row["selector"],
                      row["content_sha256"], row["source_sha256"]), row["id"]
connection.rollback()
connection.close()
report = {
    "category": "read-only bootstrap reconciliation; not admission or owner authorization",
    "accepted_generation": head, "governed_paths": len(paths), "identities": len(expected),
    "protected_identities": len({row["id"] for row in protected}), "protected_components": len(protected),
    "whole_source_components_requiring_review": sum(row["selector"] == "whole" for row in protected),
    "working_source_drift": drift, "membership": "exact accepted identity/path mapping",
    "captured_predecessors": "exact SQLite/capture match", "activation": "not performed",
}
output = root / "target/pcg-b11/bootstrap-reconciliation.json"
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report, indent=2))
