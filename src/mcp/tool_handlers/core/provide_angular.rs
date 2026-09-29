// Angular template specialization for provide_code_context.

use super::common::ContentKind;
use super::provide::outcome::ProvidedContext;
use crate::mcp::McpState;
use crate::mcp::tools::parse_tokenizer_arg;
use serde_json::Value;

pub(super) fn try_evaluate_angular_template(
    params: &Value,
    state: &McpState,
    resolved_path: &str,
    source: &str,
) -> Option<ProvidedContext> {
    if !resolved_path.to_lowercase().ends_with(".component.html") {
        return None;
    }

    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();
    let fidelity = match explicit_fidelity {
        Some(s) => match crate::compression::Fidelity::parse(s) {
            Ok(f) => f,
            Err(_) => {
                // Template editing intent → High fidelity.
                if explicit_intent == Some("edit") {
                    crate::compression::Fidelity::High
                } else {
                    crate::compression::Fidelity::Medium
                }
            }
        },
        None => {
            // Template editing intent → High fidelity.
            if explicit_intent == Some("edit") {
                crate::compression::Fidelity::High
            } else {
                crate::compression::Fidelity::Medium
            }
        }
    };
    // Verbatim fidelity: return the raw template source byte-exact.
    // The plan's fidelity table promises "Full raw source, byte-exact
    // entire document" — the template compressor would otherwise
    // produce a compressed skeleton while `contract_fields` reports
    // `verbatim_document`/`["document"]` (self-reporting contract leak).
    if fidelity == crate::compression::Fidelity::Verbatim {
        return Some(ProvidedContext::new(
            source.to_string(),
            serde_json::json!({
                "strategy": "full", "fidelity": "verbatim",
                "is_angular": true, "template_compressed": false,
                "content_kind": ContentKind::VerbatimDocument, "byte_exact": ["document"],
                "degradation": null
            }),
        ));
    }
    let lines =
        crate::angular_meta::template_compress::compress_template_with_prime_ng(source, fidelity);
    let body = lines.join("\n");
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();
    let economic = crate::mcp::content_economics::select_with_local_tokenizer(
        source,
        body,
        tokenizer_kind,
        tokenizer_ref,
    );
    let raw_tokens = economic.raw_tokens;
    let comp_tokens = economic.selected_tokens();
    let raw_passthrough = matches!(
        economic.selected,
        crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
    );
    let body = economic.text;
    state.record_compression(
        resolved_path,
        raw_tokens,
        comp_tokens,
        &format!("{:?}", fidelity).to_lowercase(),
        true,
        "full",
        None,
        if raw_passthrough {
            "raw_passthrough"
        } else {
            "angular_template"
        },
    );

    // Angular templates use a separate, non-canonical representation. They
    // contribute session statistics but must not create an empty canonical-IR
    // persistence row. A durable template contract requires its own approved
    // semantic representation.

    // The Angular template compressor emits structural markers, never
    // verbatim method bodies — so the self-reporting contract must not
    // claim `["method_bodies"]` even at Edit fidelity (it would be a
    // Gap 5/3/6 contract leak: the LLM would attempt replace_in_file
    // SEARCH against bodies that don't exist in template output).
    let (content_kind, byte_exact) = (ContentKind::Skeleton, Vec::<&'static str>::new());
    Some(ProvidedContext::new(
        body,
        serde_json::json!({
            "strategy": "full", "fidelity": format!("{:?}", fidelity).to_lowercase(),
            "is_angular": true, "template_compressed": !raw_passthrough,
            "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
            "byte_exact": if raw_passthrough { serde_json::json!(["document"]) } else { serde_json::to_value(byte_exact).unwrap_or_default() },
            "degradation": null
        }),
    ))
}
