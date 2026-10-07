#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path

PRODUCER = Path(__file__).with_name("synthetic_producer.py")
AUDIT_PATH = Path(os.environ["CLEAN_CTX_AMH_AUDIT_PATH"])
STDOUT_MARKER = os.environ["CLEAN_CTX_AMH_STDOUT_MARKER"]
STDERR_MARKER = os.environ["CLEAN_CTX_AMH_STDERR_MARKER"]


def contains_marker(value) -> bool:
    serialized = json.dumps(value, sort_keys=True)
    return STDOUT_MARKER in serialized or STDERR_MARKER in serialized


def write_audit(event: dict) -> None:
    with AUDIT_PATH.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(event, sort_keys=True) + "\n")


def sanitized(text: str) -> str:
    return text.replace(STDOUT_MARKER, "[REDACTED_STDOUT]").replace(
        STDERR_MARKER, "[REDACTED_STDERR]"
    )


def run_probe(mode: str) -> dict:
    exit_code = 17 if mode == "failed_producer" else 0
    completed = subprocess.run(
        [
            sys.executable,
            str(PRODUCER),
            "--stdout-marker",
            STDOUT_MARKER,
            "--stderr-marker",
            STDERR_MARKER,
            "--exit-code",
            str(exit_code),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    raw_stdout_contains = STDOUT_MARKER in completed.stdout
    raw_stderr_contains = STDERR_MARKER in completed.stderr
    assert raw_stdout_contains and raw_stderr_contains

    if mode == "redaction_failure":
        result = {
            "content": [
                {
                    "type": "text",
                    "text": "Secret-boundary probe failed closed: sanitization failed; raw evidence withheld.",
                }
            ],
            "structuredContent": {
                "status": "redaction_failure",
                "rawEvidenceWithheld": True,
            },
            "isError": True,
        }
        assert not contains_marker(result)
        write_audit(
            {
                "mode": mode,
                "raw_stdout_contains_marker": raw_stdout_contains,
                "raw_stderr_contains_marker": raw_stderr_contains,
                "raw_evidence_withheld": True,
                "result_contains_marker": False,
            }
        )
        return result

    clean_stdout = sanitized(completed.stdout)
    clean_stderr = sanitized(completed.stderr)
    assert STDOUT_MARKER not in clean_stdout and STDERR_MARKER not in clean_stderr
    status = "producer_failed" if completed.returncode else "ok"
    result = {
        "content": [
            {
                "type": "text",
                "text": (
                    f"status={status}; exit_code={completed.returncode}\n"
                    f"stdout:\n{clean_stdout}"
                    f"stderr:\n{clean_stderr}"
                ),
            }
        ],
        "structuredContent": {
            "status": status,
            "exitCode": completed.returncode,
            "stdout": clean_stdout,
            "stderr": clean_stderr,
            "redactionApplied": True,
        },
        "isError": completed.returncode != 0,
    }
    assert not contains_marker(result)
    write_audit(
        {
            "mode": mode,
            "producer_exit_code": completed.returncode,
            "raw_stdout_contains_marker": raw_stdout_contains,
            "raw_stderr_contains_marker": raw_stderr_contains,
            "sanitized_stdout_contains_marker": False,
            "sanitized_stderr_contains_marker": False,
            "result_contains_marker": False,
        }
    )
    return result


def response(message_id, result=None, error=None):
    value = {"jsonrpc": "2.0", "id": message_id}
    if error is not None:
        value["error"] = error
    else:
        value["result"] = result
    serialized = json.dumps(value, separators=(",", ":"))
    assert STDOUT_MARKER not in serialized and STDERR_MARKER not in serialized
    print(serialized, flush=True)


for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    message_id = message.get("id")
    if method == "initialize":
        response(
            message_id,
            {
                "protocolVersion": "2025-11-25",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "amh-secret-boundary-probe", "version": "0.0.0"},
            },
        )
    elif method == "notifications/initialized":
        continue
    elif method == "tools/list":
        response(
            message_id,
            {
                "tools": [
                    {
                        "name": "run_secret_boundary_probe",
                        "description": "Run one closed synthetic secrecy-boundary probe.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "mode": {
                                    "type": "string",
                                    "enum": ["success", "failed_producer", "redaction_failure"],
                                }
                            },
                            "required": ["mode"],
                            "additionalProperties": False,
                        },
                        "outputSchema": {
                            "type": "object",
                            "properties": {
                                "status": {"type": "string"},
                                "exitCode": {"type": "integer"},
                                "stdout": {"type": "string"},
                                "stderr": {"type": "string"},
                                "redactionApplied": {"type": "boolean"},
                                "rawEvidenceWithheld": {"type": "boolean"},
                            },
                            "required": ["status"],
                        },
                    }
                ]
            },
        )
    elif method == "tools/call":
        params = message.get("params", {})
        if params.get("name") != "run_secret_boundary_probe":
            response(message_id, error={"code": -32602, "message": "Unknown tool"})
            continue
        mode = params.get("arguments", {}).get("mode")
        if mode not in {"success", "failed_producer", "redaction_failure"}:
            response(message_id, error={"code": -32602, "message": "Invalid closed probe mode"})
            continue
        response(message_id, run_probe(mode))
    else:
        response(message_id, error={"code": -32601, "message": "Method not found"})
