use clean_ctx::compression::Fidelity;
use clean_ctx::ir::hierarchical::try_ir_to_hierarchical;
use clean_ctx::ir::{
    hierarchical_wire_to_ir, render_control_full, render_hierarchical_for_llm,
    render_hierarchical_for_llm_focused,
};
use clean_ctx::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use clean_ctx::tokenizer::{create_tokenizer, TokenizerKind};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

fn fidelity(value: &str) -> Fidelity {
    match value {
        "low" => Fidelity::Low,
        "medium" => Fidelity::Medium,
        "high" => Fidelity::High,
        "edit" => Fidelity::Edit,
        "verbatim" => Fidelity::Verbatim,
        _ => panic!("unsupported fidelity: {value}"),
    }
}

fn tokenizer(value: &str) -> TokenizerKind {
    match value {
        "cl100k" => TokenizerKind::Cl100k,
        "o200k" => TokenizerKind::O200k,
        _ => panic!("only real cl100k/o200k counters are accepted"),
    }
}

fn count(args: &[String]) {
    let text = fs::read_to_string(&args[3]).expect("payload file");
    let tokenizer = create_tokenizer(tokenizer(&args[2])).expect("tokenizer");
    println!("{}", tokenizer.count_tokens(&text));
}

fn fixed_envelope(args: &[String]) {
    let text = fs::read_to_string(&args[2]).expect("CONTROL-FULL payload");
    let (header, remainder) = text.split_once('\n').expect("CONTROL-FULL header");
    let (json_text, footer) = remainder
        .split_once("\n§PATHMAP")
        .map(|(json, pathmap)| (json, Some(pathmap)))
        .unwrap_or((remainder, None));
    let mut payload: Value = serde_json::from_str(json_text).expect("CONTROL-FULL JSON");
    let object = payload.as_object_mut().expect("CONTROL-FULL object");
    for family in [
        "classes",
        "interfaces",
        "imports",
        "type_aliases",
        "calls",
        "semantic_edges",
    ] {
        object.insert(family.to_string(), Value::Array(Vec::new()));
    }
    if let Some(mode) = object.get_mut("mode").and_then(Value::as_object_mut) {
        mode.insert(
            "exact_body_method_ids".to_string(),
            Value::Array(Vec::new()),
        );
    }
    let mut output = format!(
        "{header}\n{}",
        serde_json::to_string_pretty(&payload).expect("fixed envelope")
    );
    if let Some(pathmap) = footer {
        output.push_str("\n§PATHMAP");
        output.push_str(pathmap);
    }
    fs::write(&args[3], output).expect("fixed-envelope output");
}

fn render_a3(args: &[String]) {
    let text = fs::read_to_string(&args[2]).expect("CONTROL-FULL payload");
    let (_, remainder) = text.split_once('\n').expect("CONTROL-FULL header");
    let normalized = serde_json::Deserializer::from_str(remainder)
        .into_iter::<Value>()
        .next()
        .expect("CONTROL-FULL JSON value")
        .expect("CONTROL-FULL JSON");
    let payload =
        clean_ctx::ir::compact_a3::document::encode_cold(&normalized).expect("A3 cold encoding");
    fs::write(&args[3], payload).expect("A3 output");
}

fn write_a3_legend(args: &[String]) {
    fs::write(
        &args[2],
        format!(
            "{}\n{}\n",
            clean_ctx::ir::compact_a3::COLD_PREAMBLE,
            clean_ctx::ir::compact_a3::COLD_LEGEND
        ),
    )
    .expect("A3 legend output");
}

fn render_prod(args: &[String]) {
    let response: Value = serde_json::from_str(
        &fs::read_to_string(&args[2]).expect("compress/restore/replay response"),
    )
    .expect("response JSON");
    let wire = response
        .pointer("/result/ir")
        .or_else(|| response.pointer("/result/structuredContent/ir"))
        .expect("hierarchical result.ir");
    let ir = hierarchical_wire_to_ir(wire).expect("decode hierarchical IR");
    let hierarchy = try_ir_to_hierarchical(&ir).expect("checked hierarchy");
    let fidelity = fidelity(&args[4]);
    let focus = args
        .get(5)
        .filter(|value| !value.is_empty())
        .map(|value| value.split(',').map(str::to_string).collect::<HashSet<_>>());
    let rendered = if focus.is_some() {
        render_hierarchical_for_llm_focused(&hierarchy, fidelity, focus.as_ref())
    } else {
        render_hierarchical_for_llm(&hierarchy, fidelity)
    };
    let source_path = response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .and_then(|text| text.split_once('\n'))
        .map(|(_, payload)| payload.split("\n§PATHMAP").next().unwrap_or(payload))
        .and_then(|payload| serde_json::from_str::<Value>(payload).ok())
        .and_then(|payload| {
            payload
                .pointer("/file/source_path")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| args[3].clone());
    let footer = format!(
        "// ── {} ({}) ──\n§PATHMAP\n  {} = {}",
        ir.file_id, source_path, ir.file_id, source_path
    );
    let candidate = format!("{}\n{}", rendered.trim(), footer);
    let output = if args[7] == "provide-fallback" {
        let source = fs::read_to_string(&args[3]).expect("historical source");
        let counter = create_tokenizer(tokenizer(&args[6])).expect("tokenizer");
        if counter.count_tokens(&candidate) > counter.count_tokens(&source) {
            source
        } else {
            candidate
        }
    } else {
        candidate
    };
    fs::write(&args[8], output).expect("CONTROL-PROD output");
}

fn render_oracle(args: &[String]) {
    let response: Value =
        serde_json::from_str(&fs::read_to_string(&args[2]).expect("structured response"))
            .expect("response JSON");
    let wire = response
        .pointer("/result/ir")
        .or_else(|| response.pointer("/result/structuredContent/ir"))
        .expect("hierarchical result.ir or result.structuredContent.ir");
    let mut ir = hierarchical_wire_to_ir(wire).expect("decode hierarchical IR");
    let focus = args[5]
        .split(',')
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<HashSet<_>>();
    if !focus.is_empty() {
        let hierarchy = try_ir_to_hierarchical(&ir).expect("checked hierarchy");
        let method_ids = clean_ctx::ir::focus::resolve_focus_method_ids(&hierarchy, &focus)
            .expect("resolve focus against typed owners");
        clean_ctx::ir::focus::retain_focused_bodies(&mut ir, &method_ids);
    }
    let hierarchy = try_ir_to_hierarchical(&ir).expect("checked focused hierarchy");
    let edges_value = response
        .pointer("/result/semantic_edges")
        .or_else(|| response.pointer("/result/structuredContent/semantic_edges"))
        .or_else(|| response.pointer("/result/_meta/semantic_edges"))
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let edges = edges_value
        .as_array()
        .expect("semantic edge array")
        .iter()
        .map(edge_from_json)
        .collect::<Vec<_>>();
    let source_path = &args[3];
    let payload = render_control_full(
        &ir.file_id,
        source_path,
        ir.version,
        fidelity(&args[4]),
        &hierarchy,
        &edges,
    );
    let footer = format!(
        "// ── {} ({}) ──\n§PATHMAP\n  {} = {}",
        ir.file_id, source_path, ir.file_id, source_path
    );
    fs::write(&args[6], format!("{payload}\n{footer}")).expect("CONTROL-FULL oracle output");
}

fn leaked_str(value: &Value, field: &str) -> &'static str {
    Box::leak(
        value[field]
            .as_str()
            .unwrap_or_else(|| panic!("missing semantic edge field: {field}"))
            .to_owned()
            .into_boxed_str(),
    )
}

fn entity_from_json(value: &Value) -> EntityRef {
    let mut entity = EntityRef::new(
        leaked_str(value, "domain"),
        leaked_str(value, "entity_type"),
        value["name"].as_str().expect("entity name"),
    );
    if let Some(file) = value["file"].as_str() {
        entity = entity.with_file(file.to_owned());
    }
    entity
}

fn edge_from_json(value: &Value) -> SemanticEdge {
    SemanticEdge {
        relation: serde_json::from_value::<SemanticRelation>(value["relation"].clone())
            .expect("semantic relation"),
        subject: entity_from_json(&value["subject"]),
        object: entity_from_json(&value["object"]),
        layer: leaked_str(value, "layer"),
        call_evidence: value
            .get("call_evidence")
            .filter(|evidence| !evidence.is_null())
            .map(|evidence| {
                serde_json::from_value::<CallEvidence>(evidence.clone()).expect("call evidence")
            }),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("count") if args.len() == 4 => count(&args),
        Some("fixed") if args.len() == 4 => fixed_envelope(&args),
        Some("prod") if args.len() == 9 => render_prod(&args),
        Some("oracle") if args.len() == 7 => render_oracle(&args),
        Some("a3") if args.len() == 4 => render_a3(&args),
        Some("a3-legend") if args.len() == 3 => write_a3_legend(&args),
        _ => panic!(
            "usage: measure count <cl100k|o200k> <file> | measure fixed <control-full.txt> <output> | measure a3 <control-full.txt> <output> | measure a3-legend <output> | measure prod <response.json> <source> <fidelity> <focus-csv-or-empty> <selection-tokenizer> <provide-fallback|renderer> <output> | measure oracle <response.json> <source> <fidelity> <focus-csv-or-empty> <output>"
        ),
    }
}
