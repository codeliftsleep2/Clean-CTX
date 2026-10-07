#!/usr/bin/env python3
import argparse
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--stdout-marker", required=True)
parser.add_argument("--stderr-marker", required=True)
parser.add_argument("--exit-code", required=True, type=int)
args = parser.parse_args()

print("ordinary stdout diagnostic before")
print(args.stdout_marker)
print("ordinary stdout diagnostic after")
print("ordinary stderr diagnostic before", file=sys.stderr)
print(args.stderr_marker, file=sys.stderr)
print("ordinary stderr diagnostic after", file=sys.stderr)
raise SystemExit(args.exit_code)
