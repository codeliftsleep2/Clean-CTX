#!/usr/bin/env python3
import json
import os
import platform
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
NODE = ROOT / "tree_node.py"
EVIDENCE = (ROOT / "evidence").resolve()
if ROOT not in EVIDENCE.parents:
    raise RuntimeError(f"unsafe evidence path: {EVIDENCE}")
if EVIDENCE.exists():
    shutil.rmtree(EVIDENCE)
EVIDENCE.mkdir(parents=True)


def wait_files(case_dir: Path, names, timeout=10.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        missing = [name for name in names if not (case_dir / name).exists()]
        if not missing:
            return
        time.sleep(0.05)
    raise RuntimeError(f"timeout waiting for {missing}")


def start_tree(case_dir: Path, mode: str):
    return subprocess.Popen(
        [
            sys.executable,
            str(NODE),
            "--role",
            "parent",
            "--marker-dir",
            str(case_dir),
            "--depth",
            "2",
            "--mode",
            mode,
        ],
        start_new_session=True,
    )


def record(case_dir: Path, role: str, suffix="started"):
    return json.loads((case_dir / f"{role}.{suffix}.json").read_text(encoding="utf-8"))


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False


def group_alive(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
        return True
    except ProcessLookupError:
        return False


def wait_gone(pids, pgid, timeout=10.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        alive = [pid for pid in pids if pid_alive(pid)]
        if not alive and not group_alive(pgid):
            return {"alive_pids": [], "group_alive": False}
        time.sleep(0.05)
    return {
        "alive_pids": [pid for pid in pids if pid_alive(pid)],
        "group_alive": group_alive(pgid),
    }


def case_dir(name):
    path = EVIDENCE / name
    path.mkdir()
    return path


results = []

# L1: normal completion and inherited PGID/SID.
case = case_dir("L1-normal")
root = start_tree(case, "normal")
wait_files(case, [f"{r}.started.json" for r in ("parent", "child", "grandchild")])
records = [record(case, role) for role in ("parent", "child", "grandchild")]
root.wait(timeout=10)
wait_files(case, [f"{r}.exited.json" for r in ("parent", "child", "grandchild")])
same_group = len({r["pgid"] for r in records}) == 1
same_session = len({r["sid"] for r in records}) == 1
gone = wait_gone([r["pid"] for r in records], records[0]["pgid"])
results.append(
    {
        "case": "L1",
        "passed": root.returncode == 0 and same_group and same_session and not gone["group_alive"],
        "detail": {"records": records, "completion": gone},
    }
)

# L2: graceful whole-group SIGTERM.
case = case_dir("L2-graceful")
root = start_tree(case, "linger")
wait_files(case, [f"{r}.started.json" for r in ("parent", "child", "grandchild")])
records = [record(case, role) for role in ("parent", "child", "grandchild")]
pgid = records[0]["pgid"]
os.killpg(pgid, signal.SIGTERM)
root.wait(timeout=10)
wait_files(case, [f"{r}.graceful.json" for r in ("parent", "child", "grandchild")])
gone = wait_gone([r["pid"] for r in records], pgid)
results.append(
    {"case": "L2", "passed": not gone["alive_pids"] and not gone["group_alive"], "detail": {"completion": gone, "records": records}}
)

# L3/L5: forced whole-group SIGKILL, including grandchild.
case = case_dir("L3-forced-L5-nested")
root = start_tree(case, "linger")
wait_files(case, [f"{r}.started.json" for r in ("parent", "child", "grandchild")])
records = [record(case, role) for role in ("parent", "child", "grandchild")]
pgid = records[0]["pgid"]
os.killpg(pgid, signal.SIGKILL)
root.wait(timeout=10)
gone = wait_gone([r["pid"] for r in records], pgid)
results.append(
    {"case": "L3/L5", "passed": not gone["alive_pids"] and not gone["group_alive"], "detail": {"completion": gone, "records": records}}
)

# L4: root exits; descendants remain in and are terminated through original group.
case = case_dir("L4-parent-exits")
root = start_tree(case, "parent-exits")
wait_files(case, [f"{r}.started.json" for r in ("parent", "child", "grandchild")])
records = [record(case, role) for role in ("parent", "child", "grandchild")]
root.wait(timeout=10)
descendants_alive_before = [r["pid"] for r in records[1:] if pid_alive(r["pid"])]
same_group = len({r["pgid"] for r in records}) == 1
pgid = records[0]["pgid"]
group_alive_after_root_exit = group_alive(pgid)
os.killpg(pgid, signal.SIGKILL)
gone = wait_gone([r["pid"] for r in records[1:]], pgid)
results.append(
    {
        "case": "L4",
        "passed": len(descendants_alive_before) == 2 and same_group and group_alive_after_root_exit and not gone["alive_pids"] and not gone["group_alive"],
        "detail": {
            "alive_before": descendants_alive_before,
            "group_alive_after_root_exit": group_alive_after_root_exit,
            "completion": gone,
            "records": records,
        },
    }
)

# L6: explicit setsid detaches child/grandchild from the original group.
case = case_dir("L6-explicit-detach")
root = start_tree(case, "detach-child")
wait_files(case, [f"{r}.started.json" for r in ("parent", "child", "grandchild")])
records = [record(case, role) for role in ("parent", "child", "grandchild")]
before_detach = record(case, "child", "before-detach")
after_detach = record(case, "child", "after-detach")
original_pgid = records[0]["pgid"]
detached_pgid = records[1]["pgid"]
os.killpg(original_pgid, signal.SIGKILL)
root.wait(timeout=10)
original_gone = wait_gone([records[0]["pid"]], original_pgid)
detached_alive = [r["pid"] for r in records[1:] if pid_alive(r["pid"])]
if detached_alive:
    os.killpg(detached_pgid, signal.SIGKILL)
detached_gone = wait_gone([r["pid"] for r in records[1:]], detached_pgid)
results.append(
    {
        "case": "L6",
        "passed": len(detached_alive) == 2 and not detached_gone["alive_pids"] and not detached_gone["group_alive"] and detached_pgid != original_pgid,
        "detail": {
            "before_detach": before_detach,
            "after_detach": after_detach,
            "detached_alive_after_original_kill": detached_alive,
            "original_completion": original_gone,
            "detached_completion_after_fixture_cleanup": detached_gone,
            "records": records,
        },
    }
)

summary = {
    "platform": platform.platform(),
    "kernel": platform.release(),
    "python": platform.python_version(),
    "shell": os.environ.get("SHELL"),
    "runner_image": os.environ.get("ImageOS"),
    "runner_image_version": os.environ.get("ImageVersion"),
    "github_sha": os.environ.get("GITHUB_SHA"),
    "github_ref": os.environ.get("GITHUB_REF"),
    "github_run_id": os.environ.get("GITHUB_RUN_ID"),
    "results": results,
    "all_passed": all(item["passed"] for item in results),
}
(EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
print(json.dumps(summary, indent=2))
raise SystemExit(0 if summary["all_passed"] else 1)
