use crate::compression::Fidelity;
use crate::ir::normalize_control_full;
use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::compact_a_graph_tests::{decode_graph, encode_graph};
use super::compact_a_tests::{
    BodyFrame, decode_body_frames, decode_scoped_declarations, encode_body_frame,
    encode_scoped_declarations,
};

fn body_frames(oracle: &Value) -> Vec<BodyFrame> {
    ["classes", "interfaces"]
        .into_iter()
        .flat_map(|family| oracle[family].as_array().unwrap())
        .flat_map(|owner| owner["methods"].as_array().unwrap())
        .filter_map(|method| {
            Some(BodyFrame {
                method_id: method["id"].as_str()?.into(),
                start: method["body_start"].as_u64()?,
                end: method["body_end"].as_u64()?,
                body: method["body"].as_str()?.into(),
            })
        })
        .collect()
}

fn navigation_index(oracle: &Value) -> Value {
    let di = oracle["classes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|owner| {
            let groups = owner["injection_occurrences"].as_array().unwrap();
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
    let mut behavior = Vec::new();
    for method in ["classes", "interfaces"]
        .into_iter()
        .flat_map(|family| oracle[family].as_array().unwrap())
        .flat_map(|owner| owner["methods"].as_array().unwrap())
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
            if !method[field].as_array().unwrap().is_empty() {
                behavior.push(json!([method["id"], tag, method[field]]));
            }
        }
    }
    let edges = oracle["semantic_edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["layer"] != "builtin")
        .map(|edge| {
            json!({
                "occurrence": edge["occurrence"], "relation": edge["relation"],
                "subject_name": edge["subject"]["name"],
                "subject_file": edge["subject"]["file"],
                "object_name": edge["object"]["name"],
                "object_file": edge["object"]["file"], "layer": edge["layer"],
            })
        })
        .collect::<Vec<_>>();
    json!({ "D": di, "V": behavior, "E": edges })
}

pub(super) fn encode(oracle: &Value) -> (Value, Vec<u8>) {
    let declarations = encode_scoped_declarations(oracle);
    let graph = encode_graph(oracle);
    let envelope = json!({
        "A": 1,
        "h": [oracle["schema"], oracle["schema_version"], oracle["file"], oracle["mode"]],
        "d": declarations,
        "g": graph,
        "n": navigation_index(oracle),
        "i": oracle["imports"],
        "t": oracle["type_aliases"],
    });
    let bodies = body_frames(oracle)
        .iter()
        .flat_map(encode_body_frame)
        .collect();
    (envelope, bodies)
}

fn occurrence_navigation(owner_kind: &str, owner: &Value) -> Vec<Value> {
    const FAMILIES: [&str; 4] = [
        "modifier_occurrences",
        "control_summary_occurrences",
        "pattern_fact_occurrences",
        "legacy_flag_occurrences",
    ];
    owner["methods"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|method| {
            FAMILIES.map(|family| {
                json!({
                    "locator": {
                        "owner": { "kind": owner_kind, "id": owner["id"] },
                        "member": { "kind": "method", "id": method["id"] },
                        "field": { "kind": "occurrence_group_array", "name": family },
                    },
                    "semantics": {
                        "outer_array": "ordered_occurrences",
                        "inner_array": "one_occurrence_group",
                        "duplicates_significant": true,
                        "empty_groups_significant": true,
                    }
                })
            })
        })
        .collect()
}

fn navigation(decoded: &Value) -> Value {
    let occurrence_groups = decoded["classes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|owner| occurrence_navigation("class", owner))
        .chain(
            decoded["interfaces"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|owner| occurrence_navigation("interface", owner)),
        )
        .collect::<Vec<_>>();
    let semantic_edge_provenance = decoded["semantic_edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| {
            json!({
                "locator": {
                    "collection": "semantic_edges", "occurrence": edge["occurrence"],
                    "relation": edge["relation"], "layer": edge["layer"],
                    "subject": {
                        "domain": edge["subject"]["domain"],
                        "entity_type": edge["subject"]["entity_type"],
                        "name": edge["subject"]["name"],
                    },
                    "object": {
                        "domain": edge["object"]["domain"],
                        "entity_type": edge["object"]["entity_type"],
                        "name": edge["object"]["name"],
                    },
                },
                "endpoint_fields": {
                    "subject_file": { "endpoint": "subject", "field": "file" },
                    "object_file": { "endpoint": "object", "field": "file" },
                }
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": "clean-ctx/control-full-navigation",
        "schema_version": 1,
        "occurrence_groups": occurrence_groups,
        "semantic_edge_provenance": semantic_edge_provenance,
    })
}

pub(super) fn decode(envelope: &Value, body_wire: &[u8]) -> Result<Value, String> {
    let declarations = decode_scoped_declarations(&envelope["d"]);
    let graph = decode_graph(&envelope["g"]);
    let expected_ids = envelope["h"][3]["exact_body_method_ids"]
        .as_array()
        .ok_or("missing exact body method IDs")?
        .iter()
        .map(|id| id.as_str().ok_or("non-string exact body method ID"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut frames = BTreeMap::new();
    for frame in decode_body_frames(body_wire)? {
        let method_id = frame.method_id.clone();
        if frames.insert(method_id.clone(), frame).is_some() {
            return Err(format!("duplicate body frame: {method_id}"));
        }
    }
    if frames.len() != expected_ids.len()
        || expected_ids
            .iter()
            .any(|method_id| !frames.contains_key(*method_id))
    {
        return Err("body frames do not match exact body method IDs".into());
    }
    let mut decoded = json!({
        "schema": envelope["h"][0], "schema_version": envelope["h"][1],
        "file": envelope["h"][2], "mode": envelope["h"][3],
        "classes": declarations["classes"], "interfaces": declarations["interfaces"],
        "imports": envelope["i"], "type_aliases": envelope["t"],
        "calls": graph["calls"], "semantic_edges": graph["semantic_edges"],
    });
    for family in ["classes", "interfaces"] {
        for owner in decoded[family].as_array_mut().unwrap() {
            for method in owner["methods"].as_array_mut().unwrap() {
                if let Some(frame) = frames.get(method["id"].as_str().unwrap()) {
                    method["body"] = json!(frame.body);
                    method["body_start"] = json!(frame.start);
                    method["body_end"] = json!(frame.end);
                }
            }
        }
    }
    decoded["navigation"] = navigation(&decoded);
    Ok(decoded)
}

#[test]
fn scoped_a1_envelope_roundtrips_the_complete_normalized_oracle() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let (encoded, bodies) = encode(&oracle);
    let wire = serde_json::to_string(&encoded).unwrap();
    let reparsed = serde_json::from_str(&wire).unwrap();
    let decoded = decode(&reparsed, &bodies).unwrap();

    assert_eq!(decoded, oracle);
    assert_eq!(
        decoded["classes"][0]["methods"][0]["body"],
        "{ return lookup(key); }"
    );
    assert_eq!(decoded["classes"][0]["methods"][0]["body_start"], 41);
    assert_eq!(decoded["navigation"], oracle["navigation"]);
    assert_eq!(
        encoded["n"]["D"][0],
        json!(["C1", [["Repo", "Repo", "Clock"], ["Repo"]]])
    );
    assert_eq!(encoded["n"]["V"], json!([]));
    let mut behavior_oracle = oracle.clone();
    behavior_oracle["classes"][0]["methods"][0]["control_summary_occurrences"] = json!([["RET"]]);
    let behavior_index = navigation_index(&behavior_oracle);
    assert_eq!(behavior_index["V"][0], json!(["M1", "cs", [["RET"]]]));
    assert_eq!(encoded["n"]["E"][0]["subject_file"], "C:/repo/owner.ts");
    assert_eq!(encoded["n"]["E"][0]["object_file"], "C:/repo/repo.ts");
}

#[test]
fn scoped_a1_envelope_rejects_missing_duplicate_and_unexpected_bodies() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let (encoded, bodies) = encode(&oracle);
    assert!(decode(&encoded, &[]).is_err());

    let mut duplicate = bodies.clone();
    duplicate.extend_from_slice(&bodies);
    assert_eq!(
        decode(&encoded, &duplicate).unwrap_err(),
        "duplicate body frame: M1"
    );

    let unexpected = encode_body_frame(&BodyFrame {
        method_id: "M404".into(),
        start: 0,
        end: 2,
        body: "{}".into(),
    });
    assert_eq!(
        decode(&encoded, &unexpected).unwrap_err(),
        "body frames do not match exact body method IDs"
    );
}
