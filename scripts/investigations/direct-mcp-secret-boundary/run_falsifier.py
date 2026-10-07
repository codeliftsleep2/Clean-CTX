#!/usr/bin/env python3
import json
import os
import secrets
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PRODUCER = ROOT / "synthetic_producer.py"
SERVER = ROOT / "secret_boundary_server.py"
EVIDENCE = (ROOT / "evidence").resolve()
if ROOT not in EVIDENCE.parents:
    raise RuntimeError(f"unsafe evidence path: {EVIDENCE}")
if EVIDENCE.exists():
    shutil.rmtree(EVIDENCE)
EVIDENCE.mkdir(parents=True)

nonce = secrets.token_hex(12).upper()
markers = {
    "control": f"CLEAN_CTX_AMH_CONTROL_SECRET_{nonce}",
    "stdout": f"CLEAN_CTX_AMH_STDOUT_SECRET_{nonce}",
    "stderr": f"CLEAN_CTX_AMH_STDERR_SECRET_{nonce}",
}
(EVIDENCE / "fixture-owned-markers.json").write_text(
    json.dumps(markers, indent=2), encoding="utf-8"
)

# Control: a host-owned tool result necessarily contains producer output before
# any later PostToolUse-style transformation can inspect it.
control = subprocess.run(
    [
        sys.executable,
        str(PRODUCER),
        "--stdout-marker",
        markers["control"],
        "--stderr-marker",
        markers["control"],
        "--exit-code",
        "0",
    ],
    capture_output=True,
    text=True,
    check=False,
)
control_result = {"stdout": control.stdout, "stderr": control.stderr, "exitCode": control.returncode}
(EVIDENCE / "control-host-tool-result.json").write_text(
    json.dumps(control_result, indent=2), encoding="utf-8"
)

audit_path = EVIDENCE / "server-audit.jsonl"
environment = os.environ.copy()
environment.update(
    {
        "CLEAN_CTX_AMH_AUDIT_PATH": str(audit_path),
        "CLEAN_CTX_AMH_STDOUT_MARKER": markers["stdout"],
        "CLEAN_CTX_AMH_STDERR_MARKER": markers["stderr"],
    }
)
server = subprocess.Popen(
    [sys.executable, str(SERVER)],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    env=environment,
)
assert server.stdin is not None and server.stdout is not None and server.stderr is not None
wire = []


def request(message: dict, expect_response=True):
    serialized = json.dumps(message, separators=(",", ":"))
    server.stdin.write(serialized + "\n")
    server.stdin.flush()
    if not expect_response:
        return None
    response_line = server.stdout.readline()
    if not response_line:
        raise RuntimeError("MCP probe server ended before responding")
    wire.append(json.loads(response_line))
    return wire[-1]


request(
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "amh-boundary-client", "version": "0.0.0"},
        },
    }
)
request({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}, False)
request({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
for message_id, mode in enumerate(
    ("success", "failed_producer", "redaction_failure"), start=3
):
    request(
        {
            "jsonrpc": "2.0",
            "id": message_id,
            "method": "tools/call",
            "params": {
                "name": "run_secret_boundary_probe",
                "arguments": {"mode": mode},
            },
        }
    )

server.stdin.close()
return_code = server.wait(timeout=10)
server_stderr = server.stderr.read()
(EVIDENCE / "mcp-wire-responses.json").write_text(
    json.dumps(wire, indent=2), encoding="utf-8"
)
(EVIDENCE / "server-stderr.txt").write_text(server_stderr, encoding="utf-8")

wire_text = json.dumps(wire, sort_keys=True)
audit_text = audit_path.read_text(encoding="utf-8")
controlled_files = [
    path
    for path in EVIDENCE.rglob("*")
    if path.is_file() and path.name not in {"fixture-owned-markers.json", "control-host-tool-result.json"}
]
controlled_leaks = []
for path in controlled_files:
    text = path.read_text(encoding="utf-8")
    if markers["stdout"] in text or markers["stderr"] in text:
        controlled_leaks.append(str(path.relative_to(EVIDENCE)))

calls = {item["id"]: item["result"] for item in wire if item.get("id") in {3, 4, 5}}
summary = {
    "python": sys.version.split()[0],
    "platform": sys.platform,
    "server_exit_code": return_code,
    "control_host_result_contains_control_marker": markers["control"] in json.dumps(control_result),
    "success_content_contains_marker": markers["stdout"] in json.dumps(calls[3].get("content")) or markers["stderr"] in json.dumps(calls[3].get("content")),
    "success_structured_content_contains_marker": markers["stdout"] in json.dumps(calls[3].get("structuredContent")) or markers["stderr"] in json.dumps(calls[3].get("structuredContent")),
    "failed_producer_is_error": calls[4].get("isError") is True,
    "failed_producer_result_contains_marker": markers["stdout"] in json.dumps(calls[4]) or markers["stderr"] in json.dumps(calls[4]),
    "redaction_failure_is_error": calls[5].get("isError") is True,
    "redaction_failure_withheld_raw": calls[5].get("structuredContent", {}).get("rawEvidenceWithheld") is True,
    "redaction_failure_result_contains_marker": markers["stdout"] in json.dumps(calls[5]) or markers["stderr"] in json.dumps(calls[5]),
    "serialized_mcp_responses_contain_marker": markers["stdout"] in wire_text or markers["stderr"] in wire_text,
    "server_audit_contains_marker": markers["stdout"] in audit_text or markers["stderr"] in audit_text,
    "server_stderr_contains_marker": markers["stdout"] in server_stderr or markers["stderr"] in server_stderr,
    "controlled_artifact_leaks": controlled_leaks,
}
summary["all_protocol_assertions_passed"] = (
    return_code == 0
    and summary["control_host_result_contains_control_marker"]
    and not summary["success_content_contains_marker"]
    and not summary["success_structured_content_contains_marker"]
    and summary["failed_producer_is_error"]
    and not summary["failed_producer_result_contains_marker"]
    and summary["redaction_failure_is_error"]
    and summary["redaction_failure_withheld_raw"]
    and not summary["redaction_failure_result_contains_marker"]
    and not summary["serialized_mcp_responses_contain_marker"]
    and not summary["server_audit_contains_marker"]
    and not summary["server_stderr_contains_marker"]
    and not controlled_leaks
)
(EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
print(json.dumps(summary, indent=2))
raise SystemExit(0 if summary["all_protocol_assertions_passed"] else 1)
