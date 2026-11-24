"""Authorized E1 only. Windows Job Object ceilings and independent wall watchdog.
Build and test subcommands are explicit; no production runtime or proprietary repo.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import shutil
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "target/ops-r1" / os.environ.get("OPS_E1_RUN", "e1-20261123")
if not OUT.resolve().is_relative_to((ROOT/"target/ops-r1").resolve()):
    raise ValueError("Evidence must stay under target/ops-r1")
TEST = "ir::typescript_resolution::e1_measurement::ops_r1_e1_staged_observation"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2), encoding="utf-8")


def sources():
    names = subprocess.check_output(["git", "ls-files", "-z", "--", "src", "Cargo.toml", "Cargo.lock", "build.rs"], cwd=ROOT).decode().split("\0")
    names += subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "-z", "--", "src"], cwd=ROOT).decode().split("\0")
    return [{"path":n, "sha256":sha(ROOT/n)} for n in sorted(set(names)) if n]


class Basic(c.Structure):
    _fields_ = [("per_process", c.c_int64), ("per_job", c.c_int64), ("flags", w.DWORD),
                ("min_ws", c.c_size_t), ("max_ws", c.c_size_t), ("active", w.DWORD),
                ("affinity", c.c_size_t), ("priority", w.DWORD), ("scheduling", w.DWORD)]


class IO(c.Structure):
    _fields_ = [(n, c.c_uint64) for n in ["read_ops", "write_ops", "other_ops", "read_bytes", "write_bytes", "other_bytes"]]


class Extended(c.Structure):
    _fields_ = [("basic", Basic), ("io", IO), ("process_memory", c.c_size_t),
                ("job_memory", c.c_size_t), ("peak_process", c.c_size_t), ("peak_job", c.c_size_t)]


def job(limit):
    k = c.WinDLL("kernel32", use_last_error=True)
    k.CreateJobObjectW.argtypes = [c.c_void_p, w.LPCWSTR]
    k.CreateJobObjectW.restype = w.HANDLE
    k.SetInformationJobObject.argtypes = [w.HANDLE, c.c_int, c.c_void_p, w.DWORD]
    k.AssignProcessToJobObject.argtypes = [w.HANDLE, w.HANDLE]
    k.QueryInformationJobObject.argtypes = [w.HANDLE, c.c_int, c.c_void_p, w.DWORD, c.c_void_p]
    k.TerminateJobObject.argtypes = [w.HANDLE, w.UINT]
    k.CloseHandle.argtypes = [w.HANDLE]
    handle = k.CreateJobObjectW(None, None)
    if not handle:
        raise c.WinError(c.get_last_error())
    info = Extended()
    info.basic.flags = 0x2000 | 0x200  # kill-on-close and aggregate committed-memory ceiling
    info.job_memory = limit
    if not k.SetInformationJobObject(handle, 9, c.byref(info), c.sizeof(info)):
        k.CloseHandle(handle)
        raise c.WinError(c.get_last_error())
    return k, handle


def bounded(command, directory, seconds, mib, env=None):
    directory.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location("e1_memory", ROOT/"verification/ops-r1/p0/run_bounded.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    k, handle = job(mib*1024*1024)
    started = time.monotonic()
    child = None
    timer = None
    release = lambda: None
    receipt = {"command":command, "wall_seconds":seconds, "job_committed_limit_mib":mib,
               "launcher_sha256":sha(Path(__file__)),
               "sampling_seconds":0.01,"category":"E1 measurement, not product acceptance",
               "job_assignment_limitation":"Assignment immediately after spawn; preassignment allocation window is not hard-bounded"}
    save(directory/"receipt.json", receipt)
    try:
        with (directory/"stdout.txt").open("wb") as stdout, (directory/"stderr.txt").open("wb") as stderr, (directory/"memory.jsonl").open("w") as memory:
            child = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
            receipt["pid"] = child.pid
            if not k.AssignProcessToJobObject(handle, w.HANDLE(int(child._handle))):
                raise c.WinError(c.get_last_error())
            def stop():
                receipt["watchdog_fired"] = True
                k.TerminateJobObject(handle, 124)
            timer = threading.Timer(seconds, stop)
            timer.daemon = True
            timer.start()
            read, release = module.memory_reader(child.pid)
            peak = 0
            while child.poll() is None:
                try:
                    used = read()
                except OSError:
                    if child.poll() is not None:
                        break
                    raise
                peak = max(peak, used)
                memory.write(json.dumps({"seconds":time.monotonic()-started,"working_set_bytes":used})+"\n")
                memory.flush()
                time.sleep(0.01)
            child.wait(timeout=5)
            info = Extended()
            if not k.QueryInformationJobObject(handle, 9, c.byref(info), c.sizeof(info), None):
                raise c.WinError(c.get_last_error())
            receipt.update(exit_code=child.returncode, sampled_peak_working_set_bytes=peak,
                           peak_job_committed_bytes=info.peak_job, elapsed_seconds=time.monotonic()-started)
    finally:
        if timer:
            timer.cancel()
            timer.join(timeout=1)
        if child and child.poll() is None:
            k.TerminateJobObject(handle, 125)
            child.kill()
            child.wait(timeout=5)
        release()
        k.CloseHandle(handle)
        save(directory/"receipt.json", receipt)
    return receipt


def write(root, name, text):
    path = root/name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def fixture(environment, size):
    root = OUT/"fixtures"/f"{environment}-{size}"
    if root.exists():
        raise ValueError("Fixture exists; do not overwrite frozen evidence")
    source = "import type { Routes } from '@angular/router';\nexport class UserResolver {}\nexport const routes: Routes = [{path:'users', resolve:{user:UserResolver}}];\n"
    config = '{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","types":["node"],"noLib":true},"files":["app.ts","competitor.d.ts"]}'
    write(root, "app.ts", source)
    write(root, "tsconfig.json", config)
    write(root, "competitor.d.ts", 'declare module "@angular/router" { export interface Routes { conflict: string; } }' if environment == "conflict" else 'export {};')
    write(root, "node_modules/typescript/package.json", '{"name":"typescript","version":"5.9.3"}')
    write(root, "node_modules/@angular/router/package.json", '{"name":"@angular/router","version":"0.0.0-e1-synthetic","types":"index.d.ts","exports":{".":{"types":"./index.d.ts"}}}')
    write(root, "node_modules/@angular/router/index.d.ts", 'export interface Routes {' if environment == "relevant-malformed" else 'export interface Routes extends Array<unknown> {}')
    write(root, "node_modules/@types/node/package.json", '{"name":"@types/node","version":"0.0.0-e1-synthetic","types":"index.d.ts"}')
    count = 256 if size == "growth" else 1
    references = ''.join(f'/// <reference path="units/unit{i}.d.ts" />\n' for i in range(count))
    write(root, "node_modules/@types/node/index.d.ts", references+'declare class NodeProbe {' if environment == "irrelevant-malformed" else references+'declare class NodeProbe {}')
    for i in range(count):
        write(root, f"node_modules/@types/node/units/unit{i}.d.ts", '/*'+('x'*8192)+'*/\n'+f'declare class NodeUnit{i} {{}}\n')
    save(root.parent/(root.name+"-manifest.json"), [{"path":str(p.relative_to(root)),"sha256":sha(p),"bytes":p.stat().st_size} for p in sorted(root.rglob("*")) if p.is_file()])
    return root


def build():
    before = sources()
    save(OUT/"build-sources.json", before)
    command = ["cargo", "test", "--lib", "--all-features", "--no-run", "--message-format=json"]
    receipt = bounded(command, OUT/"build", 600, 8192)
    if receipt["exit_code"] != 0:
        raise ValueError("Bounded build failed; inspect build/stderr.txt")
    artifacts = []
    for line in (OUT/"build/stdout.txt").read_text(encoding="utf-8").splitlines():
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if row.get("reason") == "compiler-artifact" and row.get("profile",{}).get("test") and row.get("executable") and row.get("target",{}).get("name") == "clean_ctx":
            artifacts.append(row["executable"])
    if len(set(artifacts)) != 1 or before != sources():
        raise ValueError("Source drift or ambiguous executable identity")
    exe = Path(artifacts[0])
    archived = OUT/"measurement.exe"
    shutil.copy2(exe, archived)
    save(OUT/"build-identity.json", {"executable":str(archived),"sha256":sha(archived),"source_manifest_sha256":sha(OUT/"build-sources.json"),
         "baseline":"6961a43141d9e3ee2a65cdec47015cc033ed42f8", "instrumented":True})


def matrix():
    identity = json.loads((OUT/"build-identity.json").read_text())
    exe = Path(identity["executable"])
    if sha(exe) != identity["sha256"] or sources() != json.loads((OUT/"build-sources.json").read_text()):
        raise ValueError("Executable or source changed; no experiment started")
    rows = []
    for size in ["small", "growth"]:
        for environment in ["valid", "irrelevant-malformed", "relevant-malformed", "conflict"]:
            root = fixture(environment, size)
            for seconds in [1,2,5]:
                for tasks in [1,2]:
                    out = OUT/"runs"/f"{environment}-{size}-{seconds}s-{tasks}tasks"
                    env = dict(os.environ, OPS_E1_WORKSPACE=str(root), OPS_E1_OUTPUT=str(out),
                               OPS_E1_SECONDS=str(seconds), OPS_E1_TASKS=str(tasks))
                    receipt = bounded([str(exe),"--exact",TEST,"--test-threads=1","--nocapture"],out,seconds+2,512,env)
                    row = {"environment":environment,"size":size,"seconds":seconds,"tasks":tasks,"receipt":receipt,
                           "app_sha256":sha(root/"app.ts"),"config_sha256":sha(root/"tsconfig.json"),
                           "fixture_manifest_sha256":sha(root.parent/(root.name+"-manifest.json")),"executable_sha256":identity["sha256"]}
                    if (out/"completed.json").exists():
                        row["result"] = json.loads((out/"completed.json").read_text())
                    rows.append(row)
                    save(OUT/"matrix-results.json", rows)
                    print(json.dumps({"case":out.name,"exit":receipt.get("exit_code"),"elapsed":receipt.get("elapsed_seconds")}), flush=True)
    if len({r["app_sha256"] for r in rows}) != 1 or len({r["config_sha256"] for r in rows}) != 1:
        raise ValueError("Application/configuration comparison is not controlled")
    if sha(exe) != identity["sha256"] or sources() != json.loads((OUT/"build-sources.json").read_text()):
        raise ValueError("Source/binary drift during experiment")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("operation", choices=["build","matrix","controls","regressions","cold-controls"])
    args = p.parse_args()
    if os.name != "nt":
        raise ValueError("This E1 watchdog uses Windows Job Objects")
    if args.operation == "build":
        build()
    elif args.operation == "matrix":
        matrix()
    else:
        identity = json.loads((OUT/"build-identity.json").read_text())
        if sha(Path(identity["executable"])) != identity["sha256"]:
            raise ValueError("Executable drift")
        if args.operation == "controls":
            bounded([identity["executable"],"--exact",TEST.replace("staged_observation","observation_scope_controls"),"--test-threads=1"],OUT/"controls",10,512)
        elif args.operation == "cold-controls":
            for form in ["alias","barrel","local","namespace"]:
                bounded([identity["executable"],"issue_162_resolver_registration_cold_"+form,"--test-threads=1"],OUT/f"cold-{form}",20,512)
        else:
            patterns = ["issue_162_resolver_registration_cold_", "issue_162_resolver_registration_required_publication_failure_is_not_empty_success",
                        "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed",
                        "ops_r1_shared_seal_checks_each_exact_constraint_once_per_call",
                        "ops_r1_shared_qualification_checks_each_exact_constraint_once_per_call",
                        "ops_r1_warm_resolver_consumption_checks_shared_constraints_once"]
            for index, pattern in enumerate(patterns):
                bounded([identity["executable"],pattern,"--test-threads=1"],OUT/f"regression-{index}",15,512)
