"""Complete diagnostic input fingerprint. No exclusions and no source writes.

Limits are observation safeguards: hitting one refuses the comparison, never
produces a partial fingerprint. Run as a separately time-bounded child.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat


def digest_file(path, budget):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            budget[0] += len(chunk)
            if budget[0] > 20 * 1024**3:
                raise ValueError("20 GiB fingerprint read ceiling; no complete observation")
            value.update(chunk)
    return value.hexdigest()


def fingerprint(workspace, config, output):
    settings = json.loads(config.read_text(encoding="utf-8"))
    extra = settings.get("additional_roots", [])
    if not isinstance(extra, list) or not all(isinstance(p, str) and Path(p).is_absolute() for p in extra):
        raise ValueError("effective config additional_roots must be absolute directory paths")
    roots = sorted(set([workspace.resolve(strict=True), *[Path(p).resolve(strict=True) for p in extra]]))
    if any(not p.is_dir() for p in roots):
        raise ValueError("every configured source root must be a directory")
    if any(output.resolve().is_relative_to(p) for p in roots):
        raise ValueError("fingerprint output must be outside all source roots")
    rows = []
    budget = [0]
    for root in roots:
        pending = [root]
        while pending:
            path = pending.pop()
            metadata = path.lstat()
            row = {"root":str(root), "path":path.relative_to(root).as_posix()}
            if path.is_symlink() or (hasattr(path, "is_junction") and path.is_junction()):
                target = path.resolve(strict=False)
                if not any(target.is_relative_to(p) for p in roots):
                    raise ValueError(f"link target outside fingerprint roots: {path}")
                row.update(kind="link", target=os.readlink(path), resolved=str(target))
            elif stat.S_ISDIR(metadata.st_mode):
                row["kind"] = "directory"
                with os.scandir(path) as entries:
                    for entry in entries:
                        pending.append(Path(entry.path))
                        if len(rows) + len(pending) > 200000:
                            raise ValueError("200000-entry fingerprint ceiling; no complete observation")
            elif stat.S_ISREG(metadata.st_mode):
                row.update(kind="file", bytes=metadata.st_size, sha256=digest_file(path, budget))
                after = path.lstat()
                if (metadata.st_size, metadata.st_mtime_ns, metadata.st_ino) != (after.st_size, after.st_mtime_ns, after.st_ino):
                    raise ValueError(f"input changed while hashing: {path}")
            else:
                raise ValueError(f"unsupported input kind; no complete observation: {path}")
            rows.append(row)
            if len(rows) + len(pending) > 200000:
                raise ValueError("200000-entry fingerprint ceiling; no complete observation")
    value = {"roots":[str(p) for p in roots], "config_sha256":digest_file(config, budget),
             "entries":sorted(rows, key=lambda r:(r["root"], r["path"]))}
    output.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")), encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    fingerprint(args.workspace, args.config.resolve(strict=True), args.output)
