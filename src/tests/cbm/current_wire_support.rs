//! B1-only bodies derived from pinned mcp.c; old A fixtures are unchanged.
use serde_json::{Value, json};
use std::io::{self, Write};

pub(super) const QN_RULE: &str = "qn = qn_prefix == \"\" ? name : qn_prefix + \".\" + name";

pub(super) fn fixture_tool(mode: &str, request: &Value) -> bool {
    if !mode.starts_with("b1-") {
        return false;
    }
    let body = match mode {
        "b1-search" => json!({"qn_rule":QN_RULE,
            "cols":["out","name","lines","label","in"],
            "groups":[{"qn_prefix":"fixture.src.api","file":"src/api.ts",
                "rows":[[0,"run","2-8","Function",3]]}],
            "returned":1,"total":1,"has_more":false,"truncated":false}),
        "b1-empty-search" => json!({"qn_rule":QN_RULE,
            "cols":["name","label","lines","in","out"],"groups":[],"returned":0,"total":0,"has_more":false,"truncated":false}),
        "b1-missing-group" => json!({"cols":["name","label","lines","in","out"]}),
        "b1-bad-search-row" => json!({"qn_rule":QN_RULE,
            "cols":["name","label","lines","in","out"],
            "groups":[{"qn_prefix":"fixture","file":"api.ts","rows":[["run"]]}]}),
        "b1-query" => json!({"columns":["alias","flag","missing"],
            "rows":[["0",false,null]],"returned":1,"total":1,"total_relation":"eq"}),
        "b1-bad-query-column" => json!({"columns":[7],"rows":[["run"]]}),
        "b1-bad-query-row" => json!({"columns":["name"],"rows":["invalid"]}),
        "b1-missing-query-table" => json!({"total":0}),
        "b1-trace" => json!({"function":"fixture.run","direction":"both",
            "callers_total":2,"callers_total_relation":"eq","callees_total":1,"callees_total_relation":"eq",
            "callers":{"qn_rule":QN_RULE,"cols":["hop","name"],"groups":[
                {"qn_prefix":"fixture","rows":[[1,"caller"],[2,"ancestor"]]}]},
            "callees":{"qn_rule":QN_RULE,"cols":["name","hop"],"groups":[
                {"qn_prefix":"fixture","rows":[["leaf",1]]}]}}),
        "b1-missing-trace-leg" => json!({"function":"fixture.run","direction":"both"}),
        "b1-architecture" => json!({
            "packages":{"cols":["name","nodes","fan_in","fan_out"],"rows":[["api",4,0,1]]},
            "boundaries":{"cols":["from","to","calls"],"rows":[["api","data",2]]}}),
        "b1-soft-text" => {
            println!(
                "{}",
                json!({"jsonrpc":"2.0","id":request["id"],"result":{
                "isError":true,"content":[{"type":"text","text":"project not indexed"}]}})
            );
            io::stdout().flush().unwrap();
            return true;
        }
        _ => panic!("unknown B1 fixture mode: {mode}"),
    };
    println!(
        "{}",
        json!({"jsonrpc":"2.0","id":request["id"],"result":{
        "content":[{"type":"text","text":body.to_string()}]}})
    );
    io::stdout().flush().unwrap();
    true
}
