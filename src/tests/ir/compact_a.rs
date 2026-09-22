//! Test-only COMPACT-A prototype. It is not reachable from production.

use crate::compression::Fidelity;
use crate::ir::normalize_control_full;
use serde_json::{json, Map, Value};

const KEYS: &[(&str, &str)] = &[
    ("schema", "s"),
    ("schema_version", "v"),
    ("file", "f"),
    ("id", "i"),
    ("source_path", "p"),
    ("ir_version", "iv"),
    ("mode", "m"),
    ("fidelity", "q"),
    ("exact_body_method_ids", "bm"),
    ("source_requirement", "sr"),
    ("classes", "c"),
    ("interfaces", "n"),
    ("imports", "$"),
    ("type_aliases", "t"),
    ("calls", "k"),
    ("semantic_edges", "e"),
    ("navigation", "g"),
    ("kind", "d"),
    ("name", "a"),
    ("synthetic", "y"),
    ("methods", "M"),
    ("fields", "F"),
    ("parameters", "P"),
    ("return_type", "r"),
    ("type", "x"),
    ("body", "b"),
    ("body_start", "bs"),
    ("body_end", "be"),
    ("occurrence", "o"),
    ("relation", "R"),
    ("subject", "u"),
    ("object", "j"),
    ("layer", "l"),
    ("domain", "D"),
    ("entity_type", "E"),
    ("call_evidence", "ce"),
    ("caller_method_id", "cm"),
    ("callee_written_name", "cw"),
    ("explicit_argument_count", "ac"),
    ("has_spread", "sp"),
    ("callee_resolution", "cr"),
    ("modifier_occurrences", "mo"),
    ("control_summary_occurrences", "co"),
    ("pattern_fact_occurrences", "po"),
    ("legacy_flag_occurrences", "lo"),
    ("injection_occurrences", "io"),
    ("control_flow", "cf"),
    ("data_flow", "df"),
    ("side_effects", "se"),
    ("execution_contexts", "ec"),
    ("patterns", "pt"),
    ("args", "ar"),
    ("occurrence_groups", "og"),
    ("semantic_edge_provenance", "ep"),
    ("locator", "L"),
    ("owner", "O"),
    ("member", "N"),
    ("field", "H"),
    ("semantics", "S"),
    ("collection", "C"),
    ("endpoint_fields", "ef"),
];

fn encode_key(key: &str) -> String {
    KEYS.iter()
        .find_map(|(full, short)| (*full == key).then_some((*short).to_string()))
        .unwrap_or_else(|| format!("~{key}"))
}

fn decode_key(key: &str) -> Result<String, String> {
    if let Some((full, _)) = KEYS.iter().find(|(_, short)| *short == key) {
        return Ok((*full).to_string());
    }
    key.strip_prefix('~')
        .map(str::to_string)
        .ok_or_else(|| format!("unknown COMPACT-A key: {key}"))
}

fn transform(value: &Value, decode: bool) -> Result<Value, String> {
    match value {
        Value::Object(object) => {
            let mut output = Map::new();
            for (key, value) in object {
                let key = if decode {
                    decode_key(key)?
                } else {
                    encode_key(key)
                };
                if output
                    .insert(key.clone(), transform(value, decode)?)
                    .is_some()
                {
                    return Err(format!("duplicate decoded key: {key}"));
                }
            }
            Ok(Value::Object(output))
        }
        Value::Array(values) => Ok(Value::Array(
            values
                .iter()
                .map(|value| transform(value, decode))
                .collect::<Result<_, _>>()?,
        )),
        scalar => Ok(scalar.clone()),
    }
}

fn normalize(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), normalize(&object[key])))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(normalize).collect()),
        scalar => scalar.clone(),
    }
}

fn roundtrip(oracle: &Value) -> Value {
    let encoded = transform(oracle, false).unwrap();
    let wire = serde_json::to_string(&encoded).unwrap();
    let reparsed: Value = serde_json::from_str(&wire).unwrap();
    transform(&reparsed, true).unwrap()
}

#[derive(Debug, PartialEq)]
pub(super) struct BodyFrame {
    pub(super) method_id: String,
    pub(super) start: u64,
    pub(super) end: u64,
    pub(super) body: String,
}

pub(super) fn encode_body_frame(frame: &BodyFrame) -> Vec<u8> {
    let body = frame.body.as_bytes();
    let mut output = format!(
        "B {} {} {} {}\n",
        frame.method_id,
        frame.start,
        frame.end,
        body.len()
    )
    .into_bytes();
    output.extend_from_slice(body);
    output.push(b'\n');
    output
}

pub(super) fn decode_body_frames(mut input: &[u8]) -> Result<Vec<BodyFrame>, String> {
    let mut frames = Vec::new();
    while !input.is_empty() {
        let header_end = input
            .iter()
            .position(|byte| *byte == b'\n')
            .ok_or("truncated body header")?;
        let header =
            std::str::from_utf8(&input[..header_end]).map_err(|_| "invalid header UTF-8")?;
        let columns = header.split_whitespace().collect::<Vec<_>>();
        if columns.len() != 5 || columns[0] != "B" {
            return Err("invalid body header".into());
        }
        let start = columns[2]
            .parse::<u64>()
            .map_err(|_| "invalid body start")?;
        let end = columns[3].parse::<u64>().map_err(|_| "invalid body end")?;
        let length = columns[4]
            .parse::<usize>()
            .map_err(|_| "invalid body length")?;
        input = &input[header_end + 1..];
        if input.len() < length + 1 {
            return Err("truncated body bytes".into());
        }
        if input[length] != b'\n' {
            return Err("missing body frame terminator".into());
        }
        let body = std::str::from_utf8(&input[..length])
            .map_err(|_| "invalid body UTF-8")?
            .to_string();
        frames.push(BodyFrame {
            method_id: columns[1].to_string(),
            start,
            end,
            body,
        });
        input = &input[length + 1..];
    }
    Ok(frames)
}

fn method_record(method: &Value) -> Value {
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

fn decode_method(record: &Value) -> Value {
    json!({
        "id": record[0], "name": record[1], "parameters": record[2],
        "return_type": record[3], "modifier_occurrences": record[4],
        "control_summary_occurrences": record[5], "pattern_fact_occurrences": record[6],
        "legacy_flag_occurrences": record[7], "patterns": record[8],
        "body": null, "body_start": null, "body_end": null,
        "control_flow": record[9], "data_flow": record[10],
        "side_effects": record[11], "execution_contexts": record[12]
    })
}

pub(super) fn encode_scoped_declarations(oracle: &Value) -> Value {
    let classes = oracle["classes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|class| {
            json!([
                class["id"],
                class["name"],
                class["synthetic"],
                class["methods"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(method_record)
                    .collect::<Vec<_>>(),
                class["fields"],
                class["modifier_occurrences"],
                class["class_flag_occurrences"],
                class["extends"],
                class["implements"],
                class["injection_occurrences"],
                class["patterns"]
            ])
        })
        .collect::<Vec<_>>();
    let interfaces = oracle["interfaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|interface| {
            json!([
                interface["id"],
                interface["name"],
                interface["methods"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(method_record)
                    .collect::<Vec<_>>(),
                interface["fields"],
                interface["modifier_occurrences"],
                interface["extends"]
            ])
        })
        .collect::<Vec<_>>();
    json!({ "A": 1, "c": classes, "i": interfaces })
}

pub(super) fn decode_scoped_declarations(encoded: &Value) -> Value {
    let classes = encoded["c"].as_array().unwrap().iter().map(|record| json!({
        "kind": "class", "id": record[0], "name": record[1], "synthetic": record[2],
        "methods": record[3].as_array().unwrap().iter().map(decode_method).collect::<Vec<_>>(),
        "fields": record[4], "modifier_occurrences": record[5],
        "class_flag_occurrences": record[6], "extends": record[7],
        "implements": record[8], "injection_occurrences": record[9], "patterns": record[10]
    })).collect::<Vec<_>>();
    let interfaces = encoded["i"].as_array().unwrap().iter().map(|record| json!({
        "kind": "interface", "id": record[0], "name": record[1],
        "methods": record[2].as_array().unwrap().iter().map(decode_method).collect::<Vec<_>>(),
        "fields": record[3], "modifier_occurrences": record[4], "extends": record[5]
    })).collect::<Vec<_>>();
    json!({ "classes": classes, "interfaces": interfaces })
}

fn declaration_projection(oracle: &Value) -> Value {
    let mut projection =
        json!({ "classes": oracle["classes"], "interfaces": oracle["interfaces"] });
    for family in ["classes", "interfaces"] {
        for owner in projection[family].as_array_mut().unwrap() {
            for method in owner["methods"].as_array_mut().unwrap() {
                method["body"] = Value::Null;
                method["body_start"] = Value::Null;
                method["body_end"] = Value::Null;
            }
        }
    }
    projection
}

#[test]
fn compact_a_roundtrips_to_normalized_semantics_not_json_bytes() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let encoded = transform(&oracle, false).unwrap();
    let wire = serde_json::to_string(&encoded).unwrap();
    let decoded = roundtrip(&oracle);

    assert_eq!(normalize(&decoded), normalize(&oracle));
    assert_ne!(serde_json::to_string_pretty(&oracle).unwrap(), wire);
    assert_eq!(
        decoded["classes"][0]["methods"][0]["body"],
        "{ return lookup(key); }"
    );
    assert_eq!(decoded["classes"][0]["methods"][0]["body_start"], 41);
    assert_eq!(decoded["classes"][0]["methods"][0]["body_end"], 64);
    assert_eq!(
        decoded["classes"][0]["injection_occurrences"],
        json!([["Repo", "Repo", "Clock"], ["Repo"]])
    );
}

#[test]
fn normalization_ignores_object_order_but_preserves_array_order_and_bytes() {
    assert_eq!(
        normalize(&json!({"a": 1, "b": 2})),
        normalize(&json!({"b": 2, "a": 1}))
    );
    assert_ne!(
        normalize(&json!({"a": [1, 2]})),
        normalize(&json!({"a": [2, 1]}))
    );
    assert_ne!(
        normalize(&json!({"body": "x\r\ny"})),
        normalize(&json!({"body": "x\ny"}))
    );
}

#[test]
fn compact_a_roundtrips_every_structured_fidelity_and_focused_edit_bodies() {
    let (ir, mut hierarchy, edges) = super::tests::fixture();
    hierarchy.classes[0].methods[1].body = Some("{ return second(); }".into());
    hierarchy.classes[0].methods[1].body_start = Some(70);
    hierarchy.classes[0].methods[1].body_end = Some(90);

    for fidelity in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
    ] {
        let oracle = normalize_control_full(
            &ir.file_id,
            "C:/repo/owner.ts",
            ir.version,
            fidelity,
            &hierarchy,
            &edges,
        );
        assert_eq!(normalize(&roundtrip(&oracle)), normalize(&oracle));
    }

    let mut focused = hierarchy.clone();
    focused.classes[0].methods[1].body = None;
    focused.classes[0].methods[1].body_start = None;
    focused.classes[0].methods[1].body_end = None;
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &focused,
        &edges,
    );
    let decoded = roundtrip(&oracle);
    assert_eq!(decoded["mode"]["exact_body_method_ids"], json!(["M1"]));
    assert_eq!(
        decoded["classes"][0]["methods"][0]["body"],
        "{ return lookup(key); }"
    );
    assert!(decoded["classes"][0]["methods"][1]["body"].is_null());
    assert_eq!(normalize(&decoded), normalize(&oracle));
}

#[test]
fn compact_a_roundtrips_delta_and_durable_regeneration_shapes() {
    let delta = json!({
        "schema": "clean-ctx/control-full-delta",
        "schema_version": 2,
        "file_id": "alpha",
        "fidelity": "edit",
        "delta": { "operations": [["replace_body", "M1", "{\r\n  λ();\r\n}", 41, 58]] },
        "semantic_edges_after_apply": [{
            "relation": "Calls",
            "subject": { "domain": "builtin", "entity_type": "Method", "name": "run", "file": "a.ts" },
            "object": { "domain": "builtin", "entity_type": "Method", "name": "λ", "file": null },
            "layer": "core"
        }]
    });
    assert_eq!(normalize(&roundtrip(&delta)), normalize(&delta));
    assert_eq!(
        roundtrip(&delta)["delta"]["operations"][0][2],
        "{\r\n  λ();\r\n}"
    );

    let (ir, hierarchy, edges) = super::tests::fixture();
    let durable = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let persisted = serde_json::to_string(&roundtrip(&durable)).unwrap();
    let restored: Value = serde_json::from_str(&persisted).unwrap();
    let replayed = roundtrip(&restored);
    assert_eq!(normalize(&replayed), normalize(&durable));
}

#[test]
fn compact_a_escapes_schema_growth_without_silent_loss() {
    let future = json!({
        "schema": "clean-ctx/control-full",
        "future_family": { "future_field": [3, 2, 2, 1] }
    });
    let encoded = transform(&future, false).unwrap();
    assert_eq!(
        encoded["~future_family"]["~future_field"],
        json!([3, 2, 2, 1])
    );
    assert_eq!(roundtrip(&future), future);
}

#[test]
fn scoped_a1_body_frames_are_byte_exact_and_canonical_method_keyed() {
    let frames = vec![
        BodyFrame {
            method_id: "M11".into(),
            start: 41,
            end: 79,
            body: "{\r\n  const λ = \"B M99 0 0 4\";\r\n}\r\n".into(),
        },
        BodyFrame {
            method_id: "M13".into(),
            start: 80,
            end: 82,
            body: "{}".into(),
        },
    ];
    let wire = frames
        .iter()
        .flat_map(encode_body_frame)
        .collect::<Vec<_>>();
    let decoded = decode_body_frames(&wire).unwrap();
    assert_eq!(decoded, frames);
    assert_eq!(decoded[0].body.as_bytes(), frames[0].body.as_bytes());
}

#[test]
fn scoped_a1_body_frames_reject_truncation_and_length_corruption() {
    let frame = BodyFrame {
        method_id: "M11".into(),
        start: 1,
        end: 4,
        body: "λ()".into(),
    };
    let mut truncated = encode_body_frame(&frame);
    truncated.pop();
    assert_eq!(
        decode_body_frames(&truncated).unwrap_err(),
        "truncated body bytes"
    );

    let corrupt = b"B M11 1 4 1\n\xce\xbb()\n";
    assert_eq!(
        decode_body_frames(corrupt).unwrap_err(),
        "missing body frame terminator"
    );
}

#[test]
fn scoped_a1_declarations_roundtrip_typed_ids_and_occurrence_groups() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let encoded = encode_scoped_declarations(&oracle);
    let wire = serde_json::to_string(&encoded).unwrap();
    let reparsed: Value = serde_json::from_str(&wire).unwrap();
    let decoded = decode_scoped_declarations(&reparsed);

    assert_eq!(
        normalize(&decoded),
        normalize(&declaration_projection(&oracle))
    );
    assert_eq!(decoded["classes"][0]["id"], "C1");
    assert_eq!(decoded["classes"][0]["methods"][0]["id"], "M1");
    assert_eq!(
        decoded["classes"][0]["methods"][0]["parameters"][0]["id"],
        "P1"
    );
    assert_eq!(
        decoded["classes"][0]["injection_occurrences"],
        json!([["Repo", "Repo", "Clock"], ["Repo"]])
    );
}
