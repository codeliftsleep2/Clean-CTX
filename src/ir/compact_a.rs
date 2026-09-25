//! Experimental CONTROL-FULL text codecs and the non-authoritative A2 snapshot.
//!
//! This codec is derived only from normalized CONTROL-FULL. Positional rows
//! retain canonical typed identities and exact UTF-8 body frames. A1 remains a
//! research codec. A2 is written as compatibility/diagnostic `pretty_text` by
//! the delta persistence path, but restore and replay never decode or trust it:
//! physical binary `0x04`, `dv:2` history, and aligned semantic-edge snapshots
//! are the durable authority. Neither format is model-visible content.

use serde_json::{Value, json};
use std::fmt::Write;

pub const LEGEND: &str = "S A1 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution] E[occurrence,relation,S(domain,type,name,file),O(domain,type,name,file),layer,call_evidence]. N.D[owner_id,ordered_core_injection_groups]; NO_CORE_INJECTION_OCCURRENCES is authoritative; constructor parameters are signatures and NEVER injection evidence; duplicates significant. N.V tagged rows: mod,cs,pf,lf,pt,cf,df,se,ec. N.E framework relation direction is subject -> object; subject_file and object_file are independent. B[method_id,start,end,utf8_bytes]";

pub const FILE_CONTEXT_LEGEND: &str = "S A2 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution]. Workspace graph edges are retrieved with workspace_query. N.D and N.V index existing local facts. B[method_id,start,end,utf8_bytes]";
pub const FILE_CONTEXT_SCHEMA: &str = "clean-ctx/file-context";
pub const FILE_CONTEXT_VERSION: u64 = 1;

fn file_navigation(normalized: &Value) -> Value {
    let injections = normalized["classes"]
        .as_array()
        .expect("normalized classes")
        .iter()
        .filter(|owner| {
            !owner["injection_occurrences"]
                .as_array()
                .expect("normalized DI")
                .is_empty()
        })
        .map(|owner| json!([owner["id"], "inj"]))
        .collect::<Vec<_>>();
    let behavior = behavior_navigation(normalized)
        .into_iter()
        .map(|row| json!([row[0], row[1]]))
        .collect::<Vec<_>>();
    json!({ "D": injections, "V": behavior })
}

fn method_row(method: &Value) -> Value {
    json!([
        method["id"],
        method["name"],
        method["parameters"],
        method["return_type"],
        method["modifier_occurrences"],
        method["control_summary_occurrences"],
        method["pattern_fact_occurrences"],
        method["legacy_flag_occurrences"],
        method["patterns"],
        method["control_flow"],
        method["data_flow"],
        method["side_effects"],
        method["execution_contexts"]
    ])
}

fn methods(owner: &Value) -> Vec<Value> {
    owner["methods"]
        .as_array()
        .expect("normalized methods")
        .iter()
        .map(method_row)
        .collect()
}

fn entity_row(entity: &Value) -> Value {
    json!([
        entity["domain"],
        entity["entity_type"],
        entity["name"],
        entity["file"]
    ])
}

fn behavior_navigation(normalized: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    for method in ["classes", "interfaces"]
        .into_iter()
        .flat_map(|family| normalized[family].as_array().expect("normalized owners"))
        .flat_map(|owner| owner["methods"].as_array().expect("normalized methods"))
    {
        for (tag, field) in [
            ("mod", "modifier_occurrences"),
            ("cs", "control_summary_occurrences"),
            ("pf", "pattern_fact_occurrences"),
            ("lf", "legacy_flag_occurrences"),
            ("pt", "patterns"),
            ("cf", "control_flow"),
            ("df", "data_flow"),
            ("se", "side_effects"),
            ("ec", "execution_contexts"),
        ] {
            if !method[field]
                .as_array()
                .expect("normalized behavior")
                .is_empty()
            {
                rows.push(json!([method["id"], tag, method[field]]));
            }
        }
    }
    rows
}

fn envelope(normalized: &Value) -> Value {
    let classes = normalized["classes"]
        .as_array()
        .expect("normalized classes")
        .iter()
        .map(|owner| {
            json!([
                owner["id"],
                owner["name"],
                owner["synthetic"],
                methods(owner),
                owner["fields"],
                owner["modifier_occurrences"],
                owner["class_flag_occurrences"],
                owner["extends"],
                owner["implements"],
                owner["injection_occurrences"],
                owner["patterns"]
            ])
        })
        .collect::<Vec<_>>();
    let interfaces = normalized["interfaces"]
        .as_array()
        .expect("normalized interfaces")
        .iter()
        .map(|owner| {
            json!([
                owner["id"],
                owner["name"],
                methods(owner),
                owner["fields"],
                owner["modifier_occurrences"],
                owner["extends"]
            ])
        })
        .collect::<Vec<_>>();
    let calls = normalized["calls"]
        .as_array()
        .expect("normalized calls")
        .iter()
        .map(|call| {
            json!([
                call["occurrence"],
                call["caller_method_id"],
                call["callee_written_name"],
                call["explicit_argument_count"],
                call["has_spread"],
                call["callee_resolution"]
            ])
        })
        .collect::<Vec<_>>();
    let edges = normalized["semantic_edges"]
        .as_array()
        .expect("normalized edges")
        .iter()
        .map(|edge| {
            json!([
                edge["occurrence"],
                edge["relation"],
                entity_row(&edge["subject"]),
                entity_row(&edge["object"]),
                edge["layer"],
                edge["call_evidence"]
            ])
        })
        .collect::<Vec<_>>();
    let di = normalized["classes"]
        .as_array()
        .expect("normalized classes")
        .iter()
        .map(|owner| {
            let groups = owner["injection_occurrences"]
                .as_array()
                .expect("normalized DI");
            json!([
                owner["id"],
                if groups.is_empty() {
                    json!("NO_CORE_INJECTION_OCCURRENCES")
                } else {
                    json!(groups)
                }
            ])
        })
        .collect::<Vec<_>>();
    let edge_navigation = normalized["semantic_edges"]
        .as_array()
        .expect("normalized edges")
        .iter()
        .filter(|edge| edge["layer"] != "builtin")
        .map(|edge| {
            json!({
                "occurrence": edge["occurrence"], "relation": edge["relation"],
                "subject_name": edge["subject"]["name"], "subject_file": edge["subject"]["file"],
                "object_name": edge["object"]["name"], "object_file": edge["object"]["file"],
                "layer": edge["layer"]
            })
        })
        .collect::<Vec<_>>();
    json!({
        "A": 1,
        "h": [normalized["schema"], normalized["schema_version"], normalized["file"], normalized["mode"]],
        "d": { "A": 1, "c": classes, "i": interfaces },
        "g": { "K": calls, "E": edges },
        "n": { "D": di, "V": behavior_navigation(normalized), "E": edge_navigation },
        "i": normalized["imports"], "t": normalized["type_aliases"]
    })
}

fn body_wire(normalized: &Value) -> String {
    let mut wire = String::new();
    for method in ["classes", "interfaces"]
        .into_iter()
        .flat_map(|family| normalized[family].as_array().expect("normalized owners"))
        .flat_map(|owner| owner["methods"].as_array().expect("normalized methods"))
    {
        let Some(body) = method["body"].as_str() else {
            continue;
        };
        writeln!(
            wire,
            "B {} {} {} {}",
            method["id"].as_str().expect("method ID"),
            method["body_start"].as_u64().expect("exact body start"),
            method["body_end"].as_u64().expect("exact body end"),
            body.len()
        )
        .expect("writing to String cannot fail");
        wire.push_str(body);
        wire.push('\n');
    }
    wire
}

pub fn render(normalized: &Value) -> String {
    format!(
        "// COMPACT-A A1; decodes to normalized CONTROL-FULL v2\n{}\n{}\n§BODIES\n{}",
        LEGEND,
        serde_json::to_string(&envelope(normalized)).expect("A1 envelope is serializable"),
        body_wire(normalized)
    )
}

/// Render the file-local A2 boundary. Workspace graph edges remain in the
/// authoritative index/auxiliary state and are fetched through workspace_query.
pub fn render_file_context(normalized: &Value) -> String {
    let mut file_envelope = envelope(normalized);
    file_envelope["A"] = json!(2);
    file_envelope["h"][0] = json!(FILE_CONTEXT_SCHEMA);
    file_envelope["h"][1] = json!(FILE_CONTEXT_VERSION);
    file_envelope["g"]
        .as_object_mut()
        .expect("A2 graph")
        .remove("E");
    file_envelope["n"] = file_navigation(normalized);
    format!(
        "// COMPACT-A A2; file-local; workspace graph via workspace_query\n{}\n{}\n§BODIES\n{}",
        FILE_CONTEXT_LEGEND,
        serde_json::to_string(&file_envelope).expect("A2 envelope is serializable"),
        body_wire(normalized)
    )
}
