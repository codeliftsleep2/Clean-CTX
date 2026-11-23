"""One bounded, isolated libtest public-dispatch observation. No build or retry.

Run on the machine holding the workspace. All evidence stays under local target/.
This is not a production stdio/Claude receipt and does not establish leak freedom.
"""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
TEST = "mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_workspace_cost_observation"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources():
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--", "src", "Cargo.toml", "Cargo.lock", "build.rs", ".cargo"], cwd=ROOT
    ).decode("utf-8").split("\0")
    paths += subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard", "-z", "--", "src"], cwd=ROOT
    ).decode("utf-8").split("\0")
    return [{"path":p, "sha256":sha(ROOT / p)} for p in sorted(set(paths)) if p]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--exe-sha256", required=True)
    parser.add_argument("--workspace", type=Path)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--route", default="__ops_r1_absent__")
    parser.add_argument("--label", choices=["baseline", "candidate", "fixture"], required=True)
    parser.add_argument("--owners", type=int, choices=[1,4,16,64], default=1)
    parser.add_argument("--layout", choices=["flat", "directories", "directories-empty-types"], default="flat")
    parser.add_argument("--wall-seconds", type=float, default=60)
    parser.add_argument("--memory-mib", type=int, default=512)
    parser.add_argument("--output-directory", type=Path)
    args = parser.parse_args()
    if not math.isfinite(args.wall_seconds) or not 0 < args.wall_seconds <= 120:
        parser.error("wall limit must be finite and in (0,120]")
    if not 0 < args.memory_mib <= 4096:
        parser.error("memory limit must be in (0,4096] MiB")
    if args.label != "fixture" and (not args.workspace or not args.config):
        parser.error("field observations require explicit workspace and effective JSON config")
    if args.label == "fixture" and (args.workspace or args.config):
        parser.error("fixture cannot be labeled as a field observation")
    exe = args.exe.resolve(strict=True)
    if sha(exe) != args.exe_sha256:
        parser.error("executable hash mismatch")
    workspace = args.workspace.resolve(strict=True) if args.workspace else None
    config = args.config.resolve(strict=True) if args.config else None
    if workspace and not workspace.is_dir():
        parser.error("workspace must be a directory")
    # Reuse the existing platform-specific exact-process working-set reader.
    spec = importlib.util.spec_from_file_location("bounded", ROOT / "verification/ops-r1/p0/run_bounded.py")
    bounded = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(bounded)
    evidence_root = ROOT / "target/ops-r1/cost-33"
    out = (args.output_directory or evidence_root / (args.label + "-" + time.strftime("%Y%m%d-%H%M%S"))).resolve()
    if not out.is_relative_to(evidence_root.resolve()):
        parser.error("output must stay under target/ops-r1/cost-33")
    if workspace and out.is_relative_to(workspace):
        parser.error("evidence must stay outside workspace source roots")
    out.mkdir(parents=True, exist_ok=False)
    before = sources()
    (out / "sources.json").write_text(json.dumps(before, indent=2), encoding="utf-8")
    env = {k:v for k,v in os.environ.items() if not k.startswith("OPS_R1_")}
    env.update(OPS_R1_CAPTURE="1", OPS_R1_EVIDENCE_DIR=str(out), OPS_R1_UNITS=str(args.owners), OPS_R1_LAYOUT=args.layout)
    if workspace:
        env.update(OPS_R1_WORKSPACE=str(workspace), OPS_R1_CONFIG=str(config), OPS_R1_ROUTE=args.route)
    command = [str(exe), "--exact", TEST, "--test-threads=1", "--nocapture"]
    receipt = {"category":"diagnostic public in-process dispatch; not stdio, CI or defect RED/GREEN",
               "label":args.label, "owners":args.owners, "layout":args.layout, "command":command, "executable_sha256":sha(exe),
               "source_manifest_sha256":sha(out / "sources.json"),
               "binary_source_association":"requires separate build receipt; hashes alone do not prove association",
               "config_sha256":sha(config) if config else None,
               "limits":{"wall_seconds":args.wall_seconds,"working_set_mib":args.memory_mib,"stop_wait_seconds":5},
               "status":"INCOMPLETE", "memory_sampling_seconds":0.1}
    def save():
        (out / "receipt.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    save()
    child = None
    release = lambda: None
    timer = None
    deadline = threading.Event()
    started = time.monotonic()
    try:
        with (out / "stdout.txt").open("wb") as stdout, (out / "stderr.txt").open("wb") as stderr, (out / "memory.jsonl").open("w", encoding="utf-8") as memory:
            child = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
            receipt["pid"] = child.pid
            save()
            def stop():
                if child.poll() is None:
                    deadline.set()
                    try:
                        child.kill()
                    except OSError:
                        pass
            timer = threading.Timer(max(0, args.wall_seconds - (time.monotonic() - started)), stop)
            timer.daemon = True
            timer.start()
            read, release = bounded.memory_reader(child.pid)
            samples = []
            while child.poll() is None:
                try:
                    used = read()
                except OSError:
                    if child.poll() is not None:
                        break
                    raise
                row = {"seconds":time.monotonic()-started,"working_set_bytes":used}
                samples.append(row)
                memory.write(json.dumps(row) + "\n")
                memory.flush()
                if used > args.memory_mib * 1024 * 1024:
                    receipt["stop_reason"] = "MEMORY_CEILING"
                    child.kill()
                    break
                time.sleep(0.1)
            child.wait(timeout=5)
            receipt.update(exit_code=child.returncode, samples=len(samples),
                           baseline_sample_bytes=samples[0]["working_set_bytes"] if samples else None,
                           sampled_peak_bytes=max((s["working_set_bytes"] for s in samples), default=None))
            if deadline.is_set():
                receipt["stop_reason"] = "WALL_DEADLINE"
            output = (out / "stdout.txt").read_text(encoding="utf-8", errors="replace")
            receipt["status"] = "COMPLETED" if child.returncode == 0 and "1 passed; 0 failed; 0 ignored;" in output and (out / "completed.json").is_file() else "STOPPED_OR_FAILED"
    except BaseException as error:
        receipt.update(status="CAPTURE_ERROR", error=str(error))
    finally:
        if timer:
            timer.cancel()
        if child and child.poll() is None:
            child.kill()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                receipt["termination_unconfirmed"] = True
        release()
        receipt["elapsed_seconds"] = time.monotonic() - started
        receipt["source_unchanged"] = before == sources()
        receipt["binary_unchanged"] = sha(exe) == args.exe_sha256
        receipt["config_unchanged"] = not config or sha(config) == receipt["config_sha256"]
        if not all(receipt[k] for k in ["source_unchanged", "binary_unchanged", "config_unchanged"]):
            receipt["status"] = "IDENTITY_DRIFT"
        receipt["evidence_hashes"] = {p.name:sha(p) for p in out.iterdir() if p.is_file() and p.name != "receipt.json"}
        save()
    print(out)
    return 0 if receipt["status"] == "COMPLETED" else 1


if __name__ == "__main__":
    raise SystemExit(main())
