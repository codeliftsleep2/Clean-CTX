"""One originating-machine baseline/candidate pair; no build, retry or publication."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def validate_identity(identity, exe):
    if sha(exe) != identity["executable_sha256"]:
        raise ValueError("executable does not match its pinned identity")
    for label in ["source_manifest", "build_log"]:
        if sha(ROOT / identity[label]) != identity[label + "_sha256"]:
            raise ValueError(f"{label} does not match pinned build evidence")
    messages = [json.loads(line) for line in (ROOT / identity["build_log"]).read_text(encoding="utf-8").splitlines()]
    if not any(m.get("reason") == "build-finished" and m.get("success") is True for m in messages):
        raise ValueError("unsuccessful build evidence")
    if not any(m.get("reason") == "compiler-artifact" and m.get("target", {}).get("name") == "clean_ctx"
               and m.get("profile", {}).get("test") is True and m.get("executable")
               and sorted(m.get("features", [])) == sorted(identity["features"]) for m in messages):
        raise ValueError("all-feature libtest build identity missing")


def snapshot(workspace, config, out, name):
    path = out / (name + ".json")
    command = [sys.executable, str(HERE / "workspace_inputs_33.py"), "--workspace", str(workspace),
               "--config", str(config), "--output", str(path)]
    spec = importlib.util.spec_from_file_location("pair_memory", ROOT / "verification/ops-r1/p0/run_bounded.py")
    reader = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(reader)
    resources = {"wall_seconds":120, "working_set_mib":512, "peak_sampled_bytes":0}
    with (out / (name + ".stderr.txt")).open("wb") as stderr:
        child = subprocess.Popen(command, cwd=ROOT, stderr=stderr, stdout=subprocess.DEVNULL)
        resources["pid"] = child.pid
        def stop():
            if child.poll() is None:
                resources["stop_reason"] = "FINGERPRINT_DEADLINE"
                child.kill()
        timer = threading.Timer(120, stop)
        timer.daemon = True
        timer.start()
        release = lambda: None
        try:
            memory, release = reader.memory_reader(child.pid)
            while child.poll() is None:
                try:
                    used = memory()
                except OSError:
                    if child.poll() is not None:
                        break
                    raise
                resources["peak_sampled_bytes"] = max(resources["peak_sampled_bytes"], used)
                if used > 512 * 1024**2:
                    resources["stop_reason"] = "FINGERPRINT_MEMORY_CEILING"
                    child.kill()
                    break
                time.sleep(0.1)
            if child.wait(timeout=5) != 0 or resources.get("stop_reason"):
                raise ValueError("fingerprint did not complete under its resource controls")
        finally:
            timer.cancel()
            if child.poll() is None:
                child.kill()
                child.wait(timeout=5)
            release()
            (out / (name + ".resources.json")).write_text(json.dumps(resources, indent=2), encoding="utf-8")
    return sha(path)


def capture_pair_case(command, out, label, launcher_seconds=120):
    with (out / (label + ".launcher.stdout.txt")).open("wb") as stdout, (out / (label + ".launcher.stderr.txt")).open("wb") as stderr:
        child = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr)
        try:
            return child.wait(timeout=launcher_seconds)
        except subprocess.TimeoutExpired:
            # Keep the launcher alive until its owned tree is stopped. Killing
            # only the Python launcher could orphan its Rust child/watchdog.
            taskkill = Path(os.environ["SystemRoot"]) / "System32/taskkill.exe"
            subprocess.run([str(taskkill), "/PID", str(child.pid), "/T", "/F"],
                           stdout=stdout, stderr=stderr, check=True, timeout=5)
            child.wait(timeout=5)
            raise RuntimeError(f"{launcher_seconds}-second launcher deadline; owned process tree stopped; no next query")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--route", required=True, help="one route established to be absent")
    parser.add_argument("--identities", type=Path, default=HERE / "BUILD_IDENTITIES_33.json")
    args = parser.parse_args()
    workspace, config = args.workspace.resolve(strict=True), args.config.resolve(strict=True)
    identities = read(args.identities)
    if sys.platform != "win32":
        parser.error("this prepared pair launcher supports Windows process-tree termination only")
    if identities["platform"] != sys.platform or identities["machine"].lower() != platform.machine().lower():
        parser.error("prepared executables do not match this OS/architecture; do not substitute an unpinned binary")
    out = ROOT / "target/ops-r1/cost-33" / ("field-pair-" + time.strftime("%Y%m%d-%H%M%S"))
    if out.is_relative_to(workspace):
        parser.error("evidence must be outside the affected source root")
    out.mkdir(parents=True, exist_ok=False)
    receipt = {"agent":"Agent MaxHeadRoom", "status":"INCOMPLETE", "identities":identities,
               "category":"real-workspace isolated in-process public dispatch; not stdio/Claude or CI",
               "cold_state":"one fresh child and McpState per case; persistence, CBM and proxy disabled",
               "os_cache":"not cold; input fingerprints read the files before each case",
               "runs":{}, "harness_hashes":{p.name:sha(p) for p in [Path(__file__), HERE / "workspace_inputs_33.py", HERE / "capture_cost_33.py", ROOT / "verification/ops-r1/p0/run_bounded.py"]}}
    def save():
        (out / "pair.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    save()
    try:
        for label in ["baseline", "candidate"]:
            identity = identities[label]
            exe = (ROOT / identity["executable"]).resolve(strict=True)
            validate_identity(identity, exe)
        original = None
        for label in ["baseline", "candidate"]:
            before = snapshot(workspace, config, out, label + "-before")
            if original is None:
                original = before
            if before != original:
                raise ValueError("affected inputs/configuration changed before the next request; no query launched")
            identity = identities[label]
            capture = out / label
            command = [sys.executable, str(HERE / "capture_cost_33.py"), "--exe", str(ROOT / identity["executable"]),
                       "--exe-sha256", identity["executable_sha256"], "--workspace", str(workspace),
                       "--config", str(config), "--route", args.route, "--label", label,
                       "--output-directory", str(capture)]
            result = capture_pair_case(command, out, label)
            observed = read(capture / "receipt.json")
            receipt["runs"][label] = {"capture":str(capture), "launcher_exit_code":result, "receipt":observed}
            save()
            after = snapshot(workspace, config, out, label + "-after")
            if after != original:
                raise ValueError("affected inputs/configuration changed during request; pair invalid")
            if not all(observed.get(k) is True for k in ["source_unchanged", "binary_unchanged", "config_unchanged"]):
                raise ValueError("capture identity check incomplete or changed")
            if observed.get("termination_unconfirmed") or observed.get("exit_code") is None:
                raise ValueError("child termination unconfirmed; no next query")
            if observed["status"] != "COMPLETED":
                if label != "baseline" or observed.get("stop_reason") not in ["WALL_DEADLINE", "MEMORY_CEILING"]:
                    raise ValueError("unexpected error or candidate resource stop; review partial receipts")
                continue
            completed = read(capture / "completed.json")
            stages = completed["stages"]
            required = ["nominated_owners", "compile_attempts", "compile_completed", "distinct_operation_bases",
                        "project_acquisitions", "shared_acquisitions", "compile_peak", "acquire_peak", "sealed_proofs"]
            if not completed.get("external_workspace") or not all(isinstance(stages.get(k), int) for k in required):
                raise ValueError("real-workspace completion/acquisition telemetry missing")
            response = read(capture / "query-response.json")
            if "error" in response or response["result"].get("isError") is True:
                raise ValueError("query refusal is not empty success")
            answer = response["result"]["structuredContent"]
            coverage = answer["coverage"]
            if answer["count"] != 0 or answer["registrations"] != [] or coverage["source_complete"] is not False or coverage["omitted_possible"] is not True or coverage["result_semantics"] != "lower_bound":
                raise ValueError("selector was not absent or unknown-absence semantics changed")
            receipt["runs"][label].update(completed=completed, answer=answer)
        baseline, candidate = receipt["runs"]["baseline"], receipt["runs"]["candidate"]
        if "answer" in baseline and baseline["answer"] != candidate["answer"]:
            raise ValueError("completed structured answers differ; no equivalence claim")
        receipt.update(status="PAIR_CAPTURED_REQUIRES_REVIEW", input_fingerprint_sha256=original,
                       structured_answers_equal=baseline.get("answer") == candidate["answer"] if "answer" in baseline else None,
                       incident_190="OPEN; no automatic field acceptance")
    except Exception as error:
        receipt.update(status="STOPPED_REVIEW_REQUIRED", error=str(error))
    finally:
        save()
    print(out)
    return 0 if receipt["status"] == "PAIR_CAPTURED_REQUIRES_REVIEW" else 1


if __name__ == "__main__":
    raise SystemExit(main())
