//! Source-derived B2 observations; these are not live provider captures.
use serde_json::{Value, json};
use std::io::{self, Write};

pub(super) fn fixture_tool(mode: &str, request: &Value) -> bool {
    if !mode.starts_with("b2-") {
        return false;
    }
    let args = &request["params"]["arguments"];
    let tool = request["params"]["name"].as_str().unwrap();
    let mut body = match tool {
        "query_graph" => json!({"columns":["name"],"rows":[["run"]],
            "returned":1,"total":1,"total_relation":"eq","has_more":false,"truncated":false}),
        "search_graph" => json!({"qn_rule":super::b1_support::QN_RULE,
            "cols":["name","label","lines","in","out"],
            "groups":[{"qn_prefix":"fixture","file":"api.ts","rows":[["run","Function","1-3",0,0]]}],
            "returned":1,"total":1,"total_relation":"eq","has_more":false,"truncated":false}),
        "trace_path" => json!({"function":args["function_name"],"direction":"both",
            "callers_total":0,"callers_total_relation":"eq",
            "callees_total":0,"callees_total_relation":"eq","truncated":false,
            "callers":{"qn_rule":super::b1_support::QN_RULE,"cols":["name","hop"],"groups":[]},
            "callees":{"qn_rule":super::b1_support::QN_RULE,"cols":["name","hop"],"groups":[]}}),
        "index_repository" => json!({"project":"test-project","status":"indexed",
            "root_path":args["repo_path"],"mode":"fast","coverage":{"signal":"best_effort"}}),
        "index_status" => {
            json!({"project":args["project"],"status":"ready","root_path":std::env::current_dir().unwrap(),
            "indexed_at":"generation-1","coverage":{"signal":"best_effort","status":"clean"}})
        }
        "check_index_coverage" => json!({"project":args["project"],"signal":"best_effort",
            "metadata":{"recording_status":"unknown","generation_matches":false,"hash_records_complete":false},
            "paths":args["paths"].as_array().unwrap().iter().map(|p| json!({"requested_path":p,"status":"metadata_match"})).collect::<Vec<_>>(),
            "path_total":args["paths"].as_array().unwrap().len(),"path_returned":args["paths"].as_array().unwrap().len(),"path_has_more":false,
            "scopes":[],"scope_total":0,"scope_returned":0,"scope_truncated":false,"has_more":false}),
        "detect_changes" => {
            json!({"project":args["project"],"changed_total":0,"changed_has_more":false})
        }
        _ => panic!("unexpected B2 tool: {tool}"),
    };
    match mode {
        "b2-more" => {
            body["has_more"] = json!(true);
            body["next_cursor"] = json!("opaque");
        }
        "b2-gte" => body["total_relation"] = json!("gte"),
        "b2-truncated" => {
            body["truncated"] = json!(true);
            body["reason"] = json!("output_budget");
        }
        "b2-count" => body["total"] = json!(2),
        "b2-running" => {
            body.as_object_mut().unwrap().remove("status");
            body["state"] = json!("running");
        }
        "b2-degraded" => body["status"] = json!("degraded"),
        "b2-unknown-index" => {
            body.as_object_mut().unwrap().remove("status");
        }
        "b2-wrong-project" => body["project"] = json!("other-project"),
        "b2-wrong-root" => body["root_path"] = json!("/unapproved/root"),
        "b2-coverage-more" => {
            if tool == "check_index_coverage" {
                body["path_has_more"] = json!(true);
            }
        }
        "b2-unknown-page" => {
            for key in ["returned", "total", "total_relation"] {
                body.as_object_mut().unwrap().remove(key);
            }
        }
        "b2-failed-index" => {
            if tool == "index_repository" {
                body["status"] = json!("error");
                body["serving_index_preserved"] = json!(true);
            }
        }
        "b2-good" => {}
        _ => panic!("unknown B2 mode"),
    }
    println!(
        "{}",
        json!({"jsonrpc":"2.0","id":request["id"],"result":{
        "isError":mode == "b2-failed-index" && tool == "index_repository", "content":[{"type":"text","text":body.to_string()}]}})
    );
    io::stdout().flush().unwrap();
    true
}
