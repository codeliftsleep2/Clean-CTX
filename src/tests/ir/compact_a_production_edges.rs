//! Production-pipeline edge cases for the test-only COMPACT-A1 candidate.

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::hierarchical::try_ir_to_hierarchical;
use crate::ir::normalize_control_full;
use serde_json::Value;

use super::compact_a_envelope_tests::{decode, encode};

fn compile_oracle(source: &str, extension: &str, path: &str) -> Value {
    let (language, query) = crate::compression::language::language_for_extension(extension)
        .expect("enabled production language");
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile_focused(
            source,
            "edge-cases",
            Some(path),
            language,
            query,
            Fidelity::Edit,
            None,
            None,
        )
        .expect("production compilation");
    let hierarchy = try_ir_to_hierarchical(&ir).expect("checked hierarchy");
    normalize_control_full(
        &ir.file_id,
        path,
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &compiler.semantic_edges,
    )
}

fn assert_a1_roundtrip(oracle: &Value) -> Value {
    let (encoded, bodies) = encode(oracle);
    let wire = serde_json::to_string(&encoded).expect("encode envelope");
    let reparsed = serde_json::from_str(&wire).expect("decode envelope JSON");
    let decoded = decode(&reparsed, &bodies).expect("decode A1");
    assert_eq!(&decoded, oracle);
    decoded
}

#[cfg(feature = "typescript")]
#[test]
fn a1_roundtrips_nested_and_multiple_angular_arrow_callables() {
    let source = r#"
import { Component } from '@angular/core';
import { map, tap } from 'rxjs/operators';

@Component({ selector: 'app-edge', template: '' })
export class EdgeComponent {
  constructor(private repo: Repository, readonly clock: Clock) {}

  load = () => this.source.pipe(
    map(x => this.transform(x)),
    tap(x => this.audit(x)),
  ).subscribe({
    next: value => this.save(value),
    error: error => this.log(error),
  });

  cancel = () => queueMicrotask(() => this.abort());
  refresh = (...items: string[]) => this.external(...items);
}
"#;
    let decoded = assert_a1_roundtrip(&compile_oracle(source, "ts", "C:/repo/edge.component.ts"));

    let methods = decoded["classes"][0]["methods"].as_array().unwrap();
    let method_id = |name: &str| {
        methods
            .iter()
            .find(|method| method["name"] == name)
            .unwrap_or_else(|| panic!("missing arrow {name}"))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let load = method_id("load");
    let cancel = method_id("cancel");
    let refresh = method_id("refresh");
    assert_ne!(load, cancel);
    assert_ne!(cancel, refresh);

    let calls = decoded["calls"].as_array().unwrap();
    let callees = |caller: &str| {
        calls
            .iter()
            .filter(|call| call["caller_method_id"] == caller)
            .map(|call| call["callee_written_name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        callees(&load),
        [
            "pipe",
            "map",
            "transform",
            "tap",
            "audit",
            "subscribe",
            "save",
            "log"
        ]
    );
    assert_eq!(callees(&cancel), ["queueMicrotask", "abort"]);
    let spread = calls
        .iter()
        .find(|call| {
            call["caller_method_id"] == refresh && call["callee_written_name"] == "external"
        })
        .expect("spread call");
    assert_eq!(spread["has_spread"], true);
    assert_eq!(spread["callee_resolution"], "unresolved");
    assert!(
        decoded["semantic_edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["relation"] == "Injects"
                    && edge["subject"]["name"] == "EdgeComponent"
                    && edge["object"]["name"] == "Repository"
            })
    );
}

#[cfg(feature = "csharp")]
#[test]
fn a1_roundtrips_csharp_overloads_nested_types_and_lambda_calls() {
    let source = r#"
using System;
using Microsoft.AspNetCore.Mvc;
[ApiController]
[Route("api/edge")]
class EdgeController : ControllerBase {
  public enum State { Ready, Busy }
  [HttpGet("text")]
  public string Run(string value) {
    Func<string, string> map = x => Normalize(x);
    return map(value);
  }
  [HttpGet("number")]
  public string Run(int value) {
    Action first = () => Audit(value);
    Action second = () => Notify(value);
    first(); second();
    return value.ToString();
  }
  private string Normalize(string value) => value.Trim();
}
"#;
    let decoded = assert_a1_roundtrip(&compile_oracle(source, "cs", "C:/repo/Service.cs"));
    let service = decoded["classes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|owner| owner["name"] == "EdgeController")
        .expect("EdgeController owner");
    let runs = service["methods"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|method| method["name"] == "Run")
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 2);
    assert_ne!(runs[0]["id"], runs[1]["id"]);
    assert!(runs.iter().all(|method| method["body"].is_string()));
    assert!(runs.iter().all(|method| method["body_start"].is_number()));
    let run_ids = runs
        .iter()
        .map(|method| method["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let calls = decoded["calls"].as_array().unwrap();
    for callee in ["Normalize", "Audit", "Notify"] {
        assert!(
            calls.iter().any(|call| {
                run_ids.contains(&call["caller_method_id"].as_str().unwrap())
                    && call["callee_written_name"] == callee
            }),
            "missing C# lambda/nested call {callee}"
        );
    }
    assert!(
        decoded["semantic_edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["relation"] == "HasRoute" && edge["subject"]["name"] == "EdgeController"
            })
    );
    assert!(
        decoded["semantic_edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["relation"] == "ControllerAction"
                    && edge["subject"]["name"] == "EdgeController"
            })
    );
}
