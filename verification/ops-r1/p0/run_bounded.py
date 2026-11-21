"""Operator-only OPS-R1 child capture. Never builds or applies proposed source.

60s / 512 MiB / 5s are external capture controls, not semantic fitness thresholds.
No subprocess is started without an explicit --case and matching build receipt.
"""
import argparse
import ctypes
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time
import threading

ROOT = pathlib.Path(__file__).resolve().parents[3]
PROPOSAL = ROOT / "verification/ops-r1/proposals/p0/proposal-index.json"
CASES = {"owner-1": (1, 0), "owner-4": (4, 0), "owner-16": (16, 0),
         "project-4": (4, 1), "project-16": (16, 1)}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def source_check(baseline=False):
    proposal = json.loads(PROPOSAL.read_text(encoding="utf-8"))
    paths = subprocess.check_output(["git", "ls-files", "-z", "--", "src", "Cargo.toml", "Cargo.lock", "build.rs", ".cargo"], cwd=ROOT).decode().split("\0")
    paths = sorted(set(n for n in paths if n) | (set() if baseline else {p["path"] for p in proposal["files"]}))
    rows=[]
    for name in paths:
        data=(ROOT/name).read_bytes()
        if pathlib.PurePosixPath(name).suffix in ['.rs','.toml'] or name=='Cargo.lock': data=data.replace(b"\r\n",b"\n")
        rows.append({"path":name,"sha256":digest(data)})
    value = digest(json.dumps(rows,separators=(",", ":")).encode())
    if value != proposal["baseline_source_tree_sha256" if baseline else "candidate_source_tree_sha256"]:
        raise ValueError("Source tree does not match the reviewed, applied P0 proposal; no child launched")
    return proposal


def memory_reader(pid):
    if os.name == "nt":
        from ctypes import wintypes
        class Counters(ctypes.Structure):
            _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD),
                        *[(n,ctypes.c_size_t) for n in ["PeakWorkingSetSize","WorkingSetSize","QuotaPeakPagedPoolUsage",
                          "QuotaPagedPoolUsage","QuotaPeakNonPagedPoolUsage","QuotaNonPagedPoolUsage","PagefileUsage","PeakPagefileUsage"]]]
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        psapi = ctypes.WinDLL("psapi", use_last_error=True)
        kernel.OpenProcess.argtypes = [wintypes.DWORD,wintypes.BOOL,wintypes.DWORD]
        kernel.OpenProcess.restype = wintypes.HANDLE
        kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE,ctypes.POINTER(Counters),wintypes.DWORD]
        handle = kernel.OpenProcess(0x0400 | 0x0010, False, pid)
        if not handle:
            raise OSError(ctypes.get_last_error(), "Cannot open exact child for memory observation")
        def read():
            value = Counters(); value.cb = ctypes.sizeof(value)
            if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(value), value.cb):
                raise OSError(ctypes.get_last_error(), "Child memory observation failed")
            return value.WorkingSetSize
        return read, lambda: kernel.CloseHandle(handle)
    if sys.platform.startswith("linux"):
        return lambda: int(pathlib.Path(f"/proc/{pid}/statm").read_text().split()[1])*os.sysconf("SC_PAGE_SIZE"), lambda: None
    raise ValueError("Only Windows working set and Linux resident pages are supported; no child may run unmonitored")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-sources-only", action="store_true")
    parser.add_argument("--verify-baseline-only", action="store_true")
    parser.add_argument("--build-receipt", type=pathlib.Path)
    parser.add_argument("--case", choices=CASES)
    parser.add_argument("--wall-seconds", type=float, default=60)
    parser.add_argument("--memory-mib", type=int, default=512)
    parser.add_argument("--stop-wait", type=float, default=5)
    args = parser.parse_args()
    proposal = source_check(baseline=args.verify_baseline_only)
    if args.verify_baseline_only:
        print(proposal["baseline_source_tree_sha256"])
        return 0
    if args.verify_sources_only:
        print(proposal["candidate_source_tree_sha256"])
        return 0
    if not args.case or not args.build_receipt:
        raise ValueError("Explicit reviewed case and completed build receipt required")
    if (args.wall_seconds,args.memory_mib,args.stop_wait)!=(60,512,5):
        raise ValueError("Capture limits differ from reviewed packet; return for operator approval")
    build = json.loads(args.build_receipt.read_text(encoding="utf-8-sig"))
    if build.get("launcher_sha256") != digest(pathlib.Path(__file__).read_bytes()):
        raise ValueError("Launcher changed since reviewed build capture")
    if build.get("command") != "cargo test --lib --all-features --no-run --message-format=json":
        raise ValueError("Unexpected build command/features")
    if build.get("capture_script_sha256") != digest(pathlib.Path(__file__).with_name("invoke-build-capture.ps1").read_bytes()):
        raise ValueError("Build capture definition changed")
    if build.get("build_log_sha256") != digest(args.build_receipt.with_name("build.jsonl").read_bytes()):
        raise ValueError("Build artifact log drift")
    binary = pathlib.Path(build["executable"])
    if build["exit_code"] or build["source_tree_sha256"] != proposal["candidate_source_tree_sha256"] or digest(binary.read_bytes()) != build["executable_sha256"]:
        raise ValueError("Build/source/binary receipt mismatch")
    units, component = CASES[args.case]
    name = proposal["test_identities"][component]
    out = ROOT / "target/ops-r1" / ("p0-"+args.case+"-"+time.strftime("%Y%m%d-%H%M%S"))
    out.mkdir(parents=True,exist_ok=False)
    env = dict(os.environ, OPS_R1_CAPTURE="1", OPS_R1_UNITS=str(units), OPS_R1_EVIDENCE_DIR=str(out))
    command = [str(binary),"--exact",name,"--test-threads=1","--nocapture","--format=pretty"]
    receipt = {"category":"Externally bounded operator tracked-test observation; not leak proof or architectural GREEN",
               "case":args.case,"command":command,"units":units,"component":bool(component),"source_tree_sha256":proposal["candidate_source_tree_sha256"],
               "executable_sha256":build["executable_sha256"],"limits":{"wall_seconds":60,"working_set_mib":512,"post_stop_wait_seconds":5},"status":"INCOMPLETE",
               "monitored_process":"Spawned Rust libtest child; not Claude or production MCP server",
               "memory_source":"Windows WorkingSetSize" if os.name=='nt' else "Linux statm resident pages"}
    save = lambda: (out/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    save()
    child = None; release = lambda:None
    wall_timer = None
    deadline_hit = threading.Event()
    try:
        with (out/"stdout.txt").open("wb") as stdout, (out/"stderr.txt").open("wb") as stderr, (out/"memory.jsonl").open("w",encoding="utf-8") as mem:
            started=time.monotonic()
            child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
            def stop_at_deadline():
                if child.poll() is None:
                    deadline_hit.set()
                    marker={"pid":child.pid,"stop_reason":"WALL_CAPTURE_STOP","seconds":time.monotonic()-started}
                    try:
                        child.kill()
                        marker["kill_requested"]=True
                    except OSError as error:
                        marker["kill_error"]=str(error)
                    (out/"watchdog-stop.json").write_text(json.dumps(marker,indent=2)+"\n",encoding="utf-8")
            wall_timer=threading.Timer(max(0,args.wall_seconds-(time.monotonic()-started)),stop_at_deadline)
            wall_timer.daemon=True
            wall_timer.start()
            receipt["pid"]=child.pid; save()
            read,release=memory_reader(child.pid)
            peak=0; samples=0
            while child.poll() is None:
                elapsed=time.monotonic()-started
                try: used=read()
                except OSError:
                    if child.poll() is not None: break
                    raise
                peak=max(peak,used);samples+=1
                mem.write(json.dumps({"pid":child.pid,"seconds":elapsed,"working_set_bytes":used})+"\n");mem.flush()
                if samples==1: receipt["launch_working_set_bytes"]=used
                if used>args.memory_mib*1024*1024:
                    receipt["stop_reason"]="BASELINE_OVER_LIMIT" if samples==1 else "WORKING_SET_CAPTURE_STOP"
                    break
                if elapsed>=args.wall_seconds:
                    receipt["stop_reason"]="WALL_CAPTURE_STOP";break
                time.sleep(0.1)
            if deadline_hit.is_set(): receipt["stop_reason"]="WALL_CAPTURE_STOP"
            if receipt.get("stop_reason"):
                if child.poll() is None: child.kill()
                child.wait(timeout=args.stop_wait)
                receipt["status"]="STOPPED_NOT_SEMANTIC_RED"
            else:
                code=child.wait(timeout=args.stop_wait)
                receipt["status"]="TRACKED_TEST_COMPLETED" if code==0 else "TRACKED_TEST_FAILED_REVIEW_REASON"
            receipt.update(exit_code=child.returncode,elapsed_seconds=time.monotonic()-started,observed_peak_working_set_bytes=peak,memory_samples=samples)
    except BaseException as error:
        receipt.update(status="CAPTURE_ERROR",error=str(error))
        if child and child.poll() is None:
            child.kill()
            try: child.wait(timeout=args.stop_wait)
            except subprocess.TimeoutExpired: receipt["post_stop_wait_failed"]=True
    finally:
        if wall_timer is not None:
            wall_timer.cancel()
            wall_timer.join(timeout=args.stop_wait)
            if wall_timer.is_alive():receipt["watchdog_thread_incomplete"]=True
        release()
        try:
            source_check()
            if digest(binary.read_bytes()) != build["executable_sha256"]:
                raise ValueError("Executable drift during capture")
            receipt["post_source_and_binary_identity_verified"]=True
        except (OSError,ValueError) as error:
            receipt.update(status="SOURCE_OR_BINARY_DRIFT",post_identity_error=str(error))
        if receipt.get("memory_samples")==0:
            receipt["memory_coverage"]="No sample before child completion; no peak-memory inference"
        for name in ["stdout.txt","stderr.txt","memory.jsonl","fixture.json","query-response.json","stages.jsonl","watchdog-stop.json"]:
            if (out/name).exists():receipt[name+"_sha256"]=digest((out/name).read_bytes())
        if (out/"fixture.json").exists():
            try:
                fixture=json.loads((out/"fixture.json").read_text(encoding="utf-8"))
                verified=True
                for row in fixture["files"]:
                    relative=pathlib.PurePosixPath(row["file"])
                    if relative.is_absolute() or ".." in relative.parts:
                        raise ValueError("Invalid captured fixture-relative path")
                    path=out/"fixture-inputs"/relative
                    verified &= path.is_file() and digest(path.read_bytes())==row["sha256"]
                receipt["fixture_input_hashes_verified"]=verified
                if not verified and receipt["status"]=="TRACKED_TEST_COMPLETED":
                    receipt["status"]="FIXTURE_EVIDENCE_INCOMPLETE"
            except (OSError,ValueError,KeyError,TypeError) as error:
                receipt["fixture_capture_error"]=str(error)
                if receipt["status"]=="TRACKED_TEST_COMPLETED":
                    receipt["status"]="FIXTURE_EVIDENCE_INCOMPLETE"
        save()
    if receipt["status"]=="TRACKED_TEST_COMPLETED":
        text=(out/"stdout.txt").read_text(encoding="utf-8",errors="replace")
        if "1 passed; 0 failed; 0 ignored;" not in text:
            receipt["status"]="NONEMPTY_SELECTION_NOT_ESTABLISHED";save()
    print(out)
    return 0 if receipt["status"]=="TRACKED_TEST_COMPLETED" else 1


if __name__=="__main__":
    sys.exit(main())
