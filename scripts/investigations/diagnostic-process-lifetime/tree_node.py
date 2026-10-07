#!/usr/bin/env python3
import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--role", required=True)
parser.add_argument("--marker-dir", required=True)
parser.add_argument("--depth", required=True, type=int)
parser.add_argument(
    "--mode",
    required=True,
    choices=["normal", "linger", "parent-exits", "detach-child"],
)
parser.add_argument("--normal-delay", type=float, default=0.2)
args = parser.parse_args()

marker_dir = Path(args.marker_dir)
marker_dir.mkdir(parents=True, exist_ok=True)


def marker(suffix: str, event: str) -> None:
    path = marker_dir / f"{args.role}.{suffix}.json"
    path.write_text(
        json.dumps(
            {
                "role": args.role,
                "pid": os.getpid(),
                "ppid": os.getppid(),
                "pgid": os.getpgid(0),
                "sid": os.getsid(0),
                "event": event,
                "utc_ns": time.time_ns(),
            }
        ),
        encoding="utf-8",
    )


def terminate(signum, _frame) -> None:
    marker("graceful", f"signal_{signum}")
    marker("exited", "signal_exit")
    os._exit(0)


signal.signal(signal.SIGTERM, terminate)

if args.mode == "detach-child" and args.role == "child":
    marker("before-detach", "before_setsid")
    os.setsid()
    marker("after-detach", "after_setsid")

marker("started", "started")

child = None
if args.depth > 0:
    child_role = "child" if args.role == "parent" else "grandchild"
    child_mode = "linger" if args.mode == "parent-exits" else args.mode
    child = subprocess.Popen(
        [
            sys.executable,
            str(Path(__file__).resolve()),
            "--role",
            child_role,
            "--marker-dir",
            str(marker_dir),
            "--depth",
            str(args.depth - 1),
            "--mode",
            child_mode,
            "--normal-delay",
            str(args.normal_delay),
        ]
    )

if args.mode == "parent-exits":
    time.sleep(args.normal_delay)
    marker("exited", "normal_exit_parent_early")
    raise SystemExit(0)

if args.mode == "normal":
    if child is not None:
        child.wait()
    time.sleep(args.normal_delay)
    marker("exited", "normal_exit")
    raise SystemExit(0)

while True:
    time.sleep(0.1)
