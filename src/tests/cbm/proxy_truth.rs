//! B4 model-facing compression truth, including positional grouped cells.
use crate::cbm::json_compress::compress_cbm_response;
use serde_json::{Value, json};
fn rpc(result: Value) -> String {
    json!({"jsonrpc":"2.0","id":17,"result":result}).to_string()
}
fn compressed_body(body: Value) -> Value {
    let raw = rpc(json!({"content":[{"type":"text","text":body.to_string()}]}));
    serde_json::from_str(&compress_cbm_response(&raw).unwrap().compressed_text)
        .expect("protected provider JSON must preserve its structure")
}
#[test]
fn cbm_proxy_truth_grouped_cells_preserve_null_empty_false_zero_and_order() {
    let body = json!({"cols":["name","optional","flag","degree"],"groups":[{"qn_prefix":"","file":"api.ts","rows":[["run",null,false,0],["",null,true,1]]}],"returned":2,"total":2,"has_more":false,"truncated":false,"next_cursor":null});
    assert_eq!(compressed_body(body.clone()), body);
}
#[test]
fn cbm_proxy_truth_query_aliases_duplicate_rows_and_strings_are_exact() {
    let body = json!({"columns":["aliased","second"],"rows":[["  value:\nquoted \"name\"  ",""],["same","same"]],"total":2,"total_relation":"gte","has_more":true,"next_cursor":" opaque\t token "});
    assert_eq!(compressed_body(body.clone()), body);
}
#[test]
fn cbm_proxy_truth_coverage_unknown_empty_and_generation_flags_survive() {
    let body = json!({"project":"fixture","status":"clean","signal":"best_effort","metadata":{"generation":null,"generation_matches":false,"recording_status":"unknown","hash_records_complete":false},"paths":[],"scopes":[],"scope_truncated":false,"has_more":false});
    assert_eq!(compressed_body(body.clone()), body);
}
#[test]
fn cbm_proxy_truth_distinct_keys_never_collide() {
    let body = json!({"error":null,"edges":[],"data":{"status":"unknown"},"dependencies":[],"content":[],"change_type":"modified"});
    assert_eq!(compressed_body(body.clone()), body);
}
#[test]
fn cbm_proxy_truth_multiple_blocks_and_structured_content_are_retained() {
    let result = json!({"isError":false,"content":[{"type":"text","text":"first"},{"type":"text","text":"second"}],"structuredContent":{"total":9,"total_relation":"gte","has_more":true},"_meta":{"generation":null}});
    let raw = rpc(result.clone());
    let out = compress_cbm_response(&raw).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&out.compressed_text).unwrap(),
        result
    );
}
#[test]
fn cbm_proxy_truth_tree_transport_whitespace_is_verbatim() {
    let tree = "groups:\n\tname\thop\n\talpha beta\t1\nnext_cursor: opaque  token\n";
    let out =
        compress_cbm_response(&rpc(json!({"content":[{"type":"text","text":tree}]}))).unwrap();
    assert_eq!(out.compressed_text, tree);
}
#[test]
fn cbm_proxy_truth_rpc_code_data_and_error_message_are_retained() {
    let raw=json!({"jsonrpc":"2.0","id":17,"error":{"code":-32001,"message":"refused","data":{"generation":null,"retryable":false}}}).to_string();
    let out = compress_cbm_response(&raw).unwrap();
    assert!(out.compressed_text.contains("-32001"));
    assert!(out.compressed_text.contains("retryable"));
    assert_eq!(out.cbm_error.as_deref(), Some("refused"));
}
#[test]
fn cbm_proxy_truth_soft_error_is_classified_and_retained() {
    let result = json!({"isError":true,"content":[{"type":"text","text":"project not indexed"}],"structuredContent":{"status":"failed","serving_index_preserved":true}});
    let out = compress_cbm_response(&rpc(result.clone())).unwrap();
    assert!(out.cbm_error.is_some());
    assert_eq!(
        serde_json::from_str::<Value>(&out.compressed_text).unwrap(),
        result
    );
}
