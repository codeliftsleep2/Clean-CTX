//! Pinned-schema continuation fixtures, not live captures.
use serde_json::{Value, json};
use std::io::{self, Write};

pub(super) fn fixture_tool(mode: &str, request: &Value) -> bool {
    if !mode.starts_with("b3-") {
        return false;
    }
    let args = &request["params"]["arguments"];
    let later = args.get("cursor").is_some()
        || args["offset"].as_u64().is_some_and(|n| n > 0)
        || args["path_offset"].as_u64().is_some_and(|n| n > 0);
    if mode == "b3-retry" {
        let log = std::path::PathBuf::from(std::env::var_os("CLEAN_CTX_A2_FIXTURE_LOG").unwrap());
        let calls = std::fs::read_to_string(log.with_extension("requests"))
            .unwrap()
            .lines()
            .filter(|line| serde_json::from_str::<Value>(line).unwrap()["method"] == "tools/call")
            .count();
        if calls == 1 || later {
            println!(
                "{}",
                json!({"jsonrpc":"2.0","id":request["id"],"error":{"code":-32603,"message":"retry audit failure"}})
            );
            io::stdout().flush().unwrap();
            return true;
        }
    }
    let mut body = match mode {
        "b3-search" => {
            json!({"qn_rule":super::b1_support::QN_RULE,"cols":["name","label","lines","in","out"],
            "groups":[{"qn_prefix":"fixture","file":"api.ts","rows":[[if later {"second"} else {"first"},"Function","1-3",0,0]]}],
            "returned":1,"total":2,"has_more":!later,"truncated":!later})
        }
        "b3-trace" => json!({"function":args["function_name"],"direction":args["direction"],
            "callers_total":2,"callers_total_relation":"eq","callees_total":0,"callees_total_relation":"eq",
            "callers":{"qn_rule":super::b1_support::QN_RULE,"cols":["name","hop"],"groups":[{"qn_prefix":"fixture","rows":[[if later {"ancestor"} else {"caller"},if later {2} else {1}]]}]},
            "callees":{"qn_rule":super::b1_support::QN_RULE,"cols":["name","hop"],"groups":[]},
            "has_more":!later,"truncated":!later}),
        "b3-coverage" => {
            if request["params"]["name"] == "index_status" {
                emit(
                    request,
                    json!({"project":args["project"],"root_path":std::env::current_dir().unwrap(),"status":"ready"}),
                    false,
                );
                return true;
            }
            let paths = args["paths"].as_array().unwrap();
            let offset = args["path_offset"].as_u64().unwrap_or(0) as usize;
            let end = (offset + 20).min(paths.len());
            let mut value = json!({"project":args["project"],"signal":"best_effort","metadata":{"recording_status":"unknown"},
                "paths":paths[offset..end].iter().map(|p| json!({"requested_path":p,"status":"metadata_match"})).collect::<Vec<_>>(),
                "path_returned":end-offset,"path_total":paths.len(),"path_has_more":end<paths.len(),
                "scopes":[],"scope_total":0,"scope_returned":0,"scope_truncated":false,"has_more":false});
            if end < paths.len() {
                value["path_next_offset"] = json!(end);
            }
            emit(request, value, false);
            return true;
        }
        _ => json!({"columns":["alias","flag","missing"],"rows":[["same",false,null]],
            "returned":1,"total":2,"total_relation":"eq","has_more":!later,"truncated":!later}),
    };
    if !later || mode == "b3-repeat" {
        body["truncation_reason"] = json!("page_limit");
        if mode == "b3-search" {
            body["next_offset"] = json!(1);
        } else {
            body["next_cursor"] = json!("opaque-page-two");
        }
    }
    if mode == "b3-repeat" {
        body["has_more"] = json!(true);
        body["truncated"] = json!(true);
    }
    if mode == "b3-total" && later {
        body["total"] = json!(3);
    }
    if mode == "b3-empty" {
        body["rows"] = json!([]);
        body["returned"] = json!(0);
    }
    if mode == "b3-columns" && later {
        body["columns"][0] = json!("changed");
    }
    if mode == "b3-stale" && later {
        emit(request, json!({"error":"stale cursor"}), true);
        return true;
    }
    if mode == "b3-budget" {
        body["rows"][0][0] = json!("x".repeat(2_200_000));
    }
    if mode == "b3-deadline" {
        std::thread::sleep(std::time::Duration::from_millis(150));
        body["total"] = json!(3);
        body["has_more"] = json!(true);
        body["truncated"] = json!(true);
        body["truncation_reason"] = json!("page_limit");
        body["next_cursor"] = json!(if later {
            "page-three"
        } else {
            "opaque-page-two"
        });
    }
    emit(request, body, false);
    true
}
fn emit(request: &Value, body: Value, error: bool) {
    println!(
        "{}",
        json!({"jsonrpc":"2.0","id":request["id"],"result":{"isError":error,"content":[{"type":"text","text":body.to_string()}]}})
    );
    io::stdout().flush().unwrap();
}
