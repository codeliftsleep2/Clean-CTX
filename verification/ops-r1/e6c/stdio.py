"""Real compiled MCP client. Run only through run.py's external watchdog/Job Object."""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
import queue
import subprocess
import threading
import time
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[3]
APP = "import type {Routes} from '@angular/router';export class UserResolver{};export const routes:Routes=[{path:'users',resolve:{user:UserResolver}}];"
BUDGET = dict(elapsed_ms=10000, source_bytes=2097152, support_bytes=2097152,
              retained_bytes=67108864, active_expensive_tasks=2, traversal_units=20000,
              traversal_edges=4000, validation_bytes=8388608, validation_checks=20000,
              response_bytes=1048576, finalization=dict(elapsed_ms=2000,
              validation_bytes=4194304, validation_checks=10000, response_bytes=1048576))


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding="utf-8")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root, name, text):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def fixture(root, case):
    write(root, "app.ts", APP)
    options = {"module": "esnext", "moduleResolution": "bundler"}
    if case != "implicit-node":
        options["types"] = ["node"]
    write(root, "tsconfig.json", json.dumps({"compilerOptions": options}))
    write(root, "node_modules/@angular/router/package.json", json.dumps({"name": "@angular/router", "version": "20.0.0", "exports": {".": {"types": "./index.d.ts"}}}))
    write(root, "node_modules/@angular/router/index.d.ts", "export type Routes=unknown[];")
    write(root, "node_modules/@types/node/package.json", json.dumps({"name": "@types/node", "version": "0.0.0-synthetic", "types": "index.d.ts"}))
    write(root, "node_modules/@types/node/index.d.ts", "declare class NodeProbe {}")
    if case in ("irrelevant-malformed", "implicit-node"):
        write(root, "node_modules/@types/node/index.d.ts", "declare class Malformed {")
    if case == "relevant-malformed":
        write(root, "node_modules/@angular/router/index.d.ts", "export interface Routes {")
    if case == "conflicting":
        write(root, "node_modules/@angular/router/index.d.ts", "export type Routes=unknown[];export type Routes=unknown[];")
    if case == "competitor":
        write(root, "app.ts", APP + "declare module '@angular/router' {export interface Route {conflict:string;}}")
    if case == "unknown":
        write(root, "app.ts", "const routes=[{path:'users',resolve:{...other}}];")
    if case == "inventory":
        for i in range(8):
            write(root, f"feature{i}/component.ts", f"export const unrelated{i}=1;")
        write(root, "skip-me/hidden.ts", APP)
    if case == "deadline":
        write(root, "app.ts", APP + "\n" + "\n".join(f"export const padding{i}={i};" for i in range(8192)))


class Client:
    def __init__(self, binary, output, configured, ceiling):
        self.output = output
        output.mkdir(parents=True, exist_ok=False)
        launch = output / "launch"
        launch.mkdir()
        config = dict(cbm={"enabled": False, "auto_launch": False}, persistence={"enabled": False},
                      cache={"enabled": False}, proxy={"auto_start": False}, tokenizer="cl100k",
                      exclude_patterns=["skip-me"])
        save(launch / ".clean-ctx.json", config)
        save(launch / "operator.json", {"schema": "clean-ctx/resolver-evidence-operator-v1", "ceiling": ceiling})
        env = dict(os.environ, CLEAN_CTX_PROJECT_ROOT=str(launch))
        for key in ("CLEAN_CTX_PCG_PROFILE", "CLEAN_CTX_PCG_DOCUMENT_PROFILE", "CLEAN_CTX_RESOLVER_EVIDENCE_PROFILE"):
            env.pop(key, None)
        if configured:
            env["CLEAN_CTX_RESOLVER_EVIDENCE_PROFILE"] = str(launch / "operator.json")
        self.stderr = (output / "stderr.txt").open("wb")
        self.process = subprocess.Popen([str(binary)], cwd=launch, env=env, stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=self.stderr)
        self.responses = queue.Queue()
        self.sent = {}
        self.receipts = []
        self.counter = 0
        self.pending = {}
        self.stopped = threading.Event()
        self.samples = []

        def receive():
            for line in self.process.stdout:
                self.responses.put((time.perf_counter(), line))
            self.responses.put((time.perf_counter(), None))

        def sample():
            spec = importlib.util.spec_from_file_location("memory", ROOT / "verification/ops-r1/p0/run_bounded.py")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            read, release = module.memory_reader(self.process.pid)
            try:
                while not self.stopped.is_set():
                    try:
                        self.samples.append(dict(seconds=time.perf_counter(), working_set_bytes=read()))
                    except OSError:
                        break
                    self.stopped.wait(0.01)
            finally:
                release()

        self.reader = threading.Thread(target=receive, daemon=True)
        self.sampler = threading.Thread(target=sample, daemon=True)
        self.reader.start()
        self.sampler.start()

    def send(self, method, params, name, notification=False):
        self.counter += 1
        identity = self.counter
        request = dict(jsonrpc="2.0", method=method, params=params)
        if not notification:
            request["id"] = identity
        raw = (json.dumps(request, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")
        (self.output / f"{identity}.request.json").write_bytes(raw)
        self.sent[identity] = (time.perf_counter(), name, len(raw))
        print(f"send {identity} {name}",flush=True)
        self.process.stdin.write(raw)
        self.process.stdin.flush()
        return identity

    def receive(self, identity):
        while identity not in self.pending:
            ended, raw = self.responses.get(timeout=15)
            assert raw is not None, "MCP stdout closed"
            value = json.loads(raw)
            key = value["id"]
            assert key in self.sent, value
            started, name, size = self.sent[key]
            (self.output / f"{key}.response.json").write_bytes(raw)
            self.pending[key] = value
            self.receipts.append(dict(id=key, case=name, request_bytes=size, wire_bytes=len(raw),
                                      started_seconds=started,ended_seconds=ended,
                                      roundtrip_us=int((ended - started) * 1e6)))
        print(f"received {identity}",flush=True)
        return self.pending.pop(identity)

    def rpc(self, method, params, name):
        return self.receive(self.send(method, params, name))

    def close(self):
        self.process.stdin.close()
        termination = "normal"
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            termination = "terminated after completed receipts"
            self.process.kill()
            self.process.wait(timeout=5)
        self.stopped.set()
        self.sampler.join(timeout=2)
        self.reader.join(timeout=2)
        self.stderr.close()
        assert "Failed to parse" not in (self.output/"stderr.txt").read_text(encoding="utf-8"),"Invalid launch configuration"
        assert "Persistence enabled:" not in (self.output/"stderr.txt").read_text(encoding="utf-8"),"Unexpected persistence activation"
        assert "[cbm-stderr]" not in (self.output/"stderr.txt").read_text(encoding="utf-8"),"Unexpected CBM child"
        save(self.output / "receipts.json", self.receipts)
        save(self.output / "memory.json", self.samples)
        save(self.output / "lifecycle.json", dict(exit_code=self.process.returncode, termination=termination,
             sampled_process_peak=max((s["working_set_bytes"] for s in self.samples), default=0)))


def arguments(root):
    return dict(workspaceRoot=str(root), type="resolver_registrations", contract_version=2,
                direction="forward", selector_basis="route_literal", route_path="users",
                source_files=["app.ts"], budget=copy.deepcopy(BUDGET))


def main(binary, output, profile):
    assert not output.exists()
    output.mkdir(parents=True)
    schemas = {name: json.loads((ROOT / f"verification/ops-r1/e6b/schemas/{name}.schema.json").read_text(encoding="utf-8"))
               for name in ["request", "response", "batch-request", "batch-response"]}
    ceiling=copy.deepcopy(BUDGET)
    if profile=="focused":
        ceiling.update(elapsed_ms=2000,source_bytes=262144,support_bytes=262144,retained_bytes=16777216,active_expensive_tasks=1,traversal_edges=2000,validation_bytes=1048576,response_bytes=262144)
        ceiling["finalization"].update(elapsed_ms=400,validation_bytes=524288,response_bytes=262144)
    manifests = []
    for configured in (False, True):
        client = Client(binary, output / ("enabled" if configured else "disabled"), configured, ceiling)
        try:
            init = client.rpc("initialize", dict(protocolVersion="2025-11-25", capabilities={}, clientInfo={"name": "MaxHeadRoom-E6C", "version": "1"}), "initialize")
            assert init["result"]["protocolVersion"] == "2025-11-25"
            client.send("notifications/initialized", {}, "initialized", True)
            tools = client.rpc("tools/list", {}, "tools-list")["result"]["tools"]
            tool = next(t for t in tools if t["name"] == "workspace_query")
            assert ("resolverEvidenceCapabilities" in tool) == configured
            if not configured:
                root = output / "fixtures/disabled"
                fixture(root, "present")
                response = client.rpc("tools/call", {"name": "workspace_query", "arguments": arguments(root)}, "disabled-refusal")
                assert "error" in response
                continue
            assert tool["inputSchema"].get("type") == "object", "MCP input schema must be an object schema"
            for case in ["present", "reverse", "absent", "implicit-node", "irrelevant-malformed", "relevant-malformed", "conflicting", "competitor", "unknown", "inventory", "capped", "withheld", "minimum", "deadline"]:
                root = output / "fixtures" / case
                fixture(root, case)
                args = arguments(root)
                if case=="reverse":
                    args.update(direction="reverse",selector_basis="written_reference",resolver_name="UserResolver")
                    args.pop("route_path")
                if case in ("absent", "inventory"):
                    args["route_path"] = "missing"
                if case == "inventory":
                    args.pop("source_files")
                if case == "capped":
                    args["budget"]["source_bytes"] *= 2
                if case == "withheld":
                    args["budget"]["response_bytes"] = args["budget"]["finalization"]["response_bytes"] = 100000
                if case == "minimum":
                    args["budget"]["response_bytes"] = args["budget"]["finalization"]["response_bytes"] = 1
                if case == "deadline":
                    args["budget"]["elapsed_ms"] = 2
                    args["budget"]["finalization"]["elapsed_ms"] = 0
                jsonschema.validate(args, schemas["request"])
                jsonschema.validate(args, tool["inputSchema"])
                response = client.rpc("tools/call", {"name": "workspace_query", "arguments": args}, case)
                if "error" in response:
                    assert case in ("minimum", "deadline"), response
                    assert response["error"]["code"] == -32004
                else:
                    value = response["result"]["structuredContent"]
                    jsonschema.validate(value, schemas["response"])
                    jsonschema.validate(value, tool["outputSchema"])
                    assert json.loads(response["result"]["content"][0]["text"]) == value
                    assert value["canonical_registrations"] == []
                    if case=="inventory":
                        assert len(value["examined_scope"]["enumerated"])==9
                    if case in ("absent","inventory"):
                        assert value["search_conclusion"]["state"]=="absent_within_closed_predicate"
                    if case=="unknown":assert value["search_conclusion"]["state"]=="not_observed_unknown"
                    if case=="withheld":assert value["search_conclusion"]["state"]=="observed_not_returned"
                    print(f"validated {case}",flush=True)
                    if case in ("present", "reverse", "implicit-node", "irrelevant-malformed"):
                        assert value["interpretations"][0]["classification"] == "strongly_supported"
                    if case in ("relevant-malformed", "conflicting", "competitor"):
                        assert value["matching_records"]["returned"] == 1 and value["interpretations"][0]["classification"] == "partial"
                for path in sorted(root.rglob("*")):
                    if path.is_file():
                        manifests.append(dict(case=case, path=str(path.relative_to(root)), sha256=sha(path), bytes=path.stat().st_size))
            changed=output/"fixtures/mutation"
            fixture(changed,"present")
            write(changed,"source-before.txt",APP)
            original_hash=sha(changed/"app.ts")
            updated=APP.replace("path:'users'","path:'changed'")
            write(changed,"app.ts",updated)
            write(changed,"source-after.txt",updated)
            mutated=arguments(changed);mutated["route_path"]="changed"
            response=client.rpc("tools/call",{"name":"workspace_query","arguments":mutated},"source-mutation")
            value=response["result"]["structuredContent"]
            assert value["observations"][0]["source"]["sha256"]==sha(changed/"app.ts")!=original_hash
            write(changed,"support-before.txt",(changed/"node_modules/@angular/router/index.d.ts").read_text())
            write(changed,"node_modules/@angular/router/index.d.ts","export interface Routes {")
            write(changed,"support-after.txt","export interface Routes {")
            response=client.rpc("tools/call",{"name":"workspace_query","arguments":mutated},"support-mutation")
            value=response["result"]["structuredContent"]
            assert value["matching_records"]["returned"]==1 and value["interpretations"][0]["classification"]=="partial"
            for path in sorted(changed.rglob("*")):
                if path.is_file():manifests.append(dict(case="mutation",path=str(path.relative_to(changed)),sha256=sha(path),bytes=path.stat().st_size))
            root = output / "fixtures/present"
            for i in range(3):
                response = client.rpc("tools/call", {"name": "workspace_query", "arguments": arguments(root)}, f"warm-{i}")
                assert response["result"]["structuredContent"]["matching_records"]["returned"] == 1
            keys = [client.send("tools/call", {"name": "workspace_query", "arguments": arguments(root)}, f"concurrent-{i}") for i in range(2)]
            for key in keys:
                value = client.receive(key)["result"]["structuredContent"]
                assert value["matching_records"]["returned"] == 1 and value["execution"]["used"]["source_bytes"] == len(APP.encode()) + 1
            item = arguments(root)
            for field in ("workspaceRoot", "budget"):
                item.pop(field)
            item["id"] = "a"
            other = copy.deepcopy(item)
            other["id"] = "b"
            batch = dict(workspaceRoot=str(root), budget=copy.deepcopy(BUDGET), queries=[item, other])
            jsonschema.validate(batch, tool["inputSchema"])
            response = client.rpc("tools/call", {"name": "workspace_query", "arguments": batch}, "batch")
            value = response["result"]["structuredContent"]
            jsonschema.validate(value, schemas["batch-response"])
            assert json.loads(response["result"]["content"][0]["text"]) == value
            assert [r["id"] for r in value["results"]] == ["a", "b"]
            assert value["execution"]["used"]["source_bytes"] == 2 * (len(APP.encode()) + 1)
            mixed = copy.deepcopy(batch)
            mixed["queries"][1] = {"id": "legacy", "type": "has_cycle"}
            assert client.rpc("tools/call", {"name": "workspace_query", "arguments": mixed}, "mixed")["error"]["code"] == -32602
            key = client.send("tools/call", {"name": "workspace_query", "arguments": arguments(root)}, "cancelled")
            client.send("notifications/cancelled", {"requestId": key, "reason": "controlled E6-C cancellation"}, "cancel-notification", True)
            response = client.receive(key)
            assert "error" in response or response["result"]["structuredContent"]["execution"]["state"] == "cancelled"
        finally:
            client.close()
    save(output / "fixture-identities.json", manifests)
    save(output / "executable.json", dict(path=str(binary), sha256=sha(binary)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--profile",choices=["full","focused"],default="full")
    options = parser.parse_args()
    main(options.binary, options.output, options.profile)
