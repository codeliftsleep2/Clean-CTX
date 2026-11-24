"""Locally supported non-compiling integration guards, each externally bounded."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("bounded", ROOT / "verification/ops-r1/e1/measure.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
OUT = ROOT / "target/ops-r1/e6c-local-guards-20261124"
commands = {
    "format": ["cargo", "fmt", "--all", "--", "--check"],
    "grammar-versions": ["pwsh", "-NoProfile", "-File", "scripts/check-tree-sitter-versions.ps1"],
    "size-validator-tests": ["pwsh", "-NoProfile", "-File", "scripts/tests/check-file-sizes.tests.ps1"],
    "active-file-sizes": ["pwsh", "-NoProfile", "-File", "scripts/check-file-sizes.ps1"],
    "encoding": ["pwsh", "-NoProfile", "-File", "scripts/check-utf8.ps1"],
    "whitespace": ["git", "diff", "--check"],
}
results = {}
for name, command in commands.items():
    results[name] = m.bounded(command, OUT / name, 60, 512)
m.save(OUT / "summary.json", results)
print(json.dumps({name: value["exit_code"] for name, value in results.items()}))
