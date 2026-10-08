"""Read-only operator capture, using the existing Phase 9 SQLite access pattern.

The production Rust decoder validates Binary0x04 and selects the version-aligned
semantic snapshot. This helper does not implement an alternative binary codec.
"""

import hashlib
import json
import sqlite3
import sys
from pathlib import Path

database, source, output = map(Path, sys.argv[1:])
owner = str(source.resolve())
with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as connection:
    context = connection.execute(
        "SELECT id, file_path, fidelity, ir_binary, source_hash "
        "FROM contexts WHERE file_path = ?", (owner,),
    ).fetchone()
    if context is None:
        raise SystemExit(f"No durable baseline for {owner}")
    context_id, file_path, fidelity, binary, source_hash = context
    if not binary.startswith(bytes([0xCC, 0x02, 0x04])):
        raise SystemExit("Durable baseline is not physical Binary0x04")
    if hashlib.sha256(source.read_bytes()).hexdigest() != source_hash:
        raise SystemExit("Durable baseline and fixture source are not aligned")
    snapshots = connection.execute(
        "SELECT file_path, source_hash, semantic_version, edges_json "
        "FROM semantic_edge_snapshots WHERE context_id = ?", (context_id,),
    ).fetchall()
    if not snapshots:
        raise SystemExit("Durable semantic-edge snapshot missing")
output.mkdir(parents=True, exist_ok=True)
(output / "baseline.bin").write_bytes(binary)
manifest = {
    "file_path": file_path, "source_hash": source_hash,
    "fidelity": fidelity, "physical_version": 4,
    "binary_sha256": hashlib.sha256(binary).hexdigest(),
    "snapshots": [
        {"file_path": path, "source_hash": digest, "semantic_version": version,
         "edges": json.loads(edges)}
        for path, digest, version, edges in snapshots
    ],
}
(output / "baseline.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
print(f"Captured durable Binary0x04 and semantic-edge snapshots for {source.name}")
