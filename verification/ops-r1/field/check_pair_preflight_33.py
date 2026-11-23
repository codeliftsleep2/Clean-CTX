"""Bounded adapter checks only. No Rust tests, runtime queries or field claims."""
import hashlib
import ctypes
import importlib.util
import json
from pathlib import Path
import sys
import uuid

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def module(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + ".py"))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def main():
    inputs, compare = module("workspace_inputs_33"), module("compare_cost_33")
    identities = compare.read(HERE / "BUILD_IDENTITIES_33.json")
    cases = []
    for label in ["baseline", "candidate"]:
        compare.validate_identity(identities[label], ROOT / identities[label]["executable"])
        cases.append(label + " executable/build/source pins")
    base = ROOT / "target/ops-r1/cost-33"
    directory = base / ("preflight-check-" + uuid.uuid4().hex)
    directory.mkdir()
    root = directory / "workspace"
    root.mkdir()
    (root / "owner.ts").write_text("export class Owner {}", encoding="utf-8")
    dependency = root / "node_modules/local/index.d.ts"
    dependency.parent.mkdir(parents=True)
    dependency.write_text("export interface Local {}", encoding="utf-8")
    config = directory / "effective.json"
    config.write_text('{"additional_roots":[]}', encoding="utf-8")
    output = directory / "fingerprint.json"
    def snapshot():
        inputs.fingerprint(root, config, output)
        return hashlib.sha256(output.read_bytes()).hexdigest()
    original = snapshot()
    if snapshot() != original:
        raise ValueError("unchanged inputs did not produce an identical fingerprint")
    cases.append("identical input fingerprints")
    dependency.write_text("export interface Changed {}", encoding="utf-8")
    if snapshot() == original:
        raise ValueError("dependency mutation was missed")
    cases.append("node_modules content mutation detected")
    dependency.write_text("export interface Local {}", encoding="utf-8")
    (root / "new-negative-path.ts").write_text("", encoding="utf-8")
    if snapshot() == original:
        raise ValueError("new input path was missed")
    cases.append("new source path detected")
    (root / "new-negative-path.ts").unlink()
    config.write_text('{"additional_roots":[],"exclude_patterns":["other"]}', encoding="utf-8")
    if snapshot() == original:
        raise ValueError("configuration mutation was missed")
    cases.append("configuration mutation detected")
    additional = directory / "additional"
    additional.mkdir()
    (additional / "shared.ts").write_text("export class Shared {}", encoding="utf-8")
    config.write_text(json.dumps({"additional_roots":[str(additional.resolve())]}), encoding="utf-8")
    with_additional = snapshot()
    (additional / "shared.ts").write_text("export class ChangedShared {}", encoding="utf-8")
    if snapshot() == with_additional:
        raise ValueError("additional-root input mutation was missed")
    cases.append("additional-root mutation detected")
    if compare.snapshot(root, config, directory, "bounded-input-child") != snapshot():
        raise ValueError("bounded fingerprint child disagrees with direct observation")
    cases.append("bounded fingerprint child completion")
    try:
        inputs.fingerprint(root, config, root / "invalid-evidence.json")
    except ValueError:
        cases.append("in-root evidence refused")
    else:
        raise ValueError("in-root evidence was accepted")
    tampered = dict(identities["candidate"], executable_sha256="0" * 64)
    try:
        compare.validate_identity(tampered, ROOT / identities["candidate"]["executable"])
    except ValueError:
        cases.append("wrong executable pin refused before launch")
    else:
        raise ValueError("wrong executable pin accepted")
    if sys.platform == "win32":
        command = [sys.executable, "-c", "import subprocess,sys,time; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(15)']); print(p.pid,flush=True); time.sleep(15)"]
        try:
            compare.capture_pair_case(command, directory, "owned-tree", launcher_seconds=1)
        except RuntimeError:
            pid = int((directory / "owned-tree.launcher.stdout.txt").read_text().splitlines()[0])
            from ctypes import wintypes
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            kernel.OpenProcess.restype = wintypes.HANDLE
            kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
            kernel.CloseHandle.argtypes = [wintypes.HANDLE]
            handle = kernel.OpenProcess(0x00100000, False, pid)
            if handle:
                try:
                    if kernel.WaitForSingleObject(handle, 0) != 0:
                        raise ValueError("owned descendant still running after tree deadline")
                finally:
                    kernel.CloseHandle(handle)
            elif ctypes.get_last_error() != 87:
                raise OSError(ctypes.get_last_error(), "descendant termination not established")
            cases.append("outer watchdog stops owned launcher and descendant")
        else:
            raise ValueError("outer launcher deadline was not enforced")
    receipt = {"category":"local adapter preflight checks; no Rust test or field query",
               "checks":cases, "status":"PASS"}
    (base / "pair-preflight-checks.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    print(json.dumps(receipt))


if __name__ == "__main__":
    main()
