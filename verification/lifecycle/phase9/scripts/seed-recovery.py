"""Seed deterministic edit-intent states for the Phase 9 operator harness.

Operator-only fixture support. This does not verify the Rust implementation and
must never be reported as a tracked test or CI result.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
from pathlib import Path


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("prior", "target", "failure"))
    parser.add_argument("db", type=Path)
    parser.add_argument("source", type=Path)
    parser.add_argument("stage", type=Path)
    args = parser.parse_args()

    source_path = str(args.source.resolve())
    actual = args.source.read_bytes()
    connection = sqlite3.connect(args.db)
    context = connection.execute(
        "SELECT id, fidelity, ir_binary, source_hash FROM contexts WHERE file_path = ?",
        (source_path,),
    ).fetchone()
    if context is None:
        raise SystemExit(f"no persisted context for {source_path}")
    context_id, fidelity, ir_binary, persisted_hash = context
    snapshot = connection.execute(
        "SELECT semantic_version, edges_json FROM semantic_edge_snapshots "
        "WHERE context_id = ? ORDER BY semantic_version DESC LIMIT 1",
        (context_id,),
    ).fetchone()
    if snapshot is None:
        raise SystemExit(f"no semantic-edge snapshot for {source_path}")
    version, edges_json = snapshot

    if digest(actual) != persisted_hash:
        raise SystemExit("fixture source and persisted baseline are not aligned")

    prior = actual
    target = actual
    prior_hash = persisted_hash
    target_hash = persisted_hash
    if args.mode == "prior":
        target = b"uncommitted target bytes\n"
        target_hash = digest(target)
    elif args.mode == "target":
        prior = b"older exact source bytes\n"
        prior_hash = digest(prior)
    else:
        prior = b"unrelated prior bytes\n"
        target = b"unrelated target bytes\n"
        prior_hash = digest(prior)
        target_hash = digest(target)

    args.stage.parent.mkdir(parents=True, exist_ok=True)
    args.stage.write_bytes(target)
    transition_id = f"phase9-{args.mode}-{digest(actual)[:16]}"
    connection.execute("DELETE FROM edit_intents WHERE file_path = ?", (source_path,))
    connection.execute(
        """
        INSERT INTO edit_intents (
            file_path, transition_id, prior_hash, target_hash,
            prior_version, target_version, prior_source, target_source,
            target_ir, target_edges_json, fidelity, stage_path
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            source_path,
            transition_id,
            prior_hash,
            target_hash,
            version,
            version,
            prior,
            target,
            ir_binary,
            edges_json,
            fidelity,
            str(args.stage.resolve()),
        ),
    )
    connection.commit()
    connection.close()
    print(json.dumps({"mode": args.mode, "file": source_path, "transition": transition_id}))


if __name__ == "__main__":
    main()
