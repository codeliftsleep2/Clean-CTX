//! Exact additive written-registration input and output shapes.
use serde_json::{Value, json};

fn span() -> Value {
    json!({"type":"array","minItems":2,"maxItems":2,"items":{"type":"integer","minimum":0}})
}

fn declaration() -> Value {
    json!({"type":"object","properties":{
        "source":{"type":"string"},"source_hash":{"type":"string"},
        "start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":0},
        "name":{"type":"string"},"kind":{"type":"string"}
    },"required":["source","source_hash","start","end","name","kind"],"additionalProperties":false})
}

fn route() -> Value {
    let mut value = declaration();
    let _ = value["properties"].as_object_mut().unwrap().remove("name");
    let _ = value["properties"].as_object_mut().unwrap().remove("kind");
    value["properties"]["root"] = declaration();
    value["required"] = json!(["source", "source_hash", "start", "end", "root"]);
    value
}

pub(in crate::mcp) fn records() -> Value {
    json!({"type":"array","items":{"type":"object","properties":{
        "route":route(),"route_path":{"type":"string"},"key":{"type":"string","minLength":1},
        "resolver":declaration(),"evidence":{"type":"object","properties":{
            "source":{"type":"string"},"source_hash":{"type":"string"},
            "field_pair":span(),"field_key":span(),"object":span(),"pair":span(),"key":span(),"value":span()
        },"required":["source","source_hash","field_pair","field_key","object","pair","key","value"],"additionalProperties":false}
    },"required":["route","route_path","key","resolver","evidence"],"additionalProperties":false}})
}

pub(in crate::mcp) fn omissions(selected: bool) -> Value {
    json!({"type":"array","items":{"type":"object","properties":{
        "source":{"type":"string"},"source_hash":{"type":"string"},
        "start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":0},
        "route":route(),"route_path":{"type":"string"},"resolver":declaration(),
        "reason":{"enum":["duplicate_resolve_field","duplicate_mapping_key","unsupported_mapping_key",
            "unsupported_mapping_value","unresolved_mapping_target","ambiguous_mapping_target",
            "type_only_mapping_target","mapping_proof_not_established"]},
        "attribution":{"enum":if selected { vec!["selected_route","selected_resolver"] } else { vec!["candidate_source"] }}
    },"required":["source","source_hash","start","end","reason","attribution"],"additionalProperties":false,
    "allOf":[
        {"if":{"properties":{"attribution":{"const":"selected_route"}}},"then":{"required":["route","route_path"]}},
        {"if":{"properties":{"attribution":{"const":"selected_resolver"}}},"then":{"required":["resolver"]}}
    ]}})
}

pub(in crate::mcp) fn result() -> Value {
    json!({"type":"object","properties":{
        "format_version":{"const":1},"direction":{"enum":["forward","reverse"]},
        "registrations":records(),"semantic_omissions":omissions(true),"candidate_source_omissions":omissions(false),
        "count":{"type":"integer","minimum":0},"coverage":{"type":"object"}
    },"required":["format_version","direction","registrations","semantic_omissions","candidate_source_omissions","count","coverage"]})
}

pub(super) fn input_variant(require_root: bool) -> Value {
    let mut required = vec!["type", "direction"];
    if require_root {
        required.push("workspaceRoot");
    }
    json!({"properties":{"type":{"const":"resolver_registrations"}},"required":required,
    "oneOf":[
        {"properties":{"direction":{"const":"forward"}},"required":["route_path"],"not":{"required":["resolver_name"]}},
        {"properties":{"direction":{"const":"reverse"},"resolver_name":{"type":"string","minLength":1}},"required":["resolver_name"],"not":{"required":["route_path"]}}
    ]})
}
