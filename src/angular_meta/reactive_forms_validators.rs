//! Conservative validator extraction for Angular Reactive Forms.

use super::{ConstructionKind, ImportedForms, find_candidates, is_identifier};
use crate::angular_meta::util::{consume_call_expression, find_matching_brace, split_top_level};

pub(super) fn validators_from_value(
    value: &str,
    imports: &ImportedForms,
    builder_aliases: &[String],
) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.starts_with('[') {
        let Some(close) = find_matching_brace(trimmed, '[') else {
            return Vec::new();
        };
        let items = split_top_level(&trimmed[1..close], ',');
        return collect_validator_list(items.into_iter().skip(1));
    }
    let candidates = find_candidates(trimmed, imports, builder_aliases);
    let Some(control) = candidates
        .iter()
        .find(|candidate| candidate.kind == ConstructionKind::Control)
    else {
        return Vec::new();
    };
    let Some((_, body)) = consume_call_expression(trimmed, control.open_paren) else {
        return Vec::new();
    };
    validators_from_control_body(&body)
}

pub(super) fn validators_from_control_body(body: &str) -> Vec<String> {
    collect_validator_list(split_top_level(body, ',').into_iter().skip(1))
}

fn collect_validator_list(expressions: impl Iterator<Item = String>) -> Vec<String> {
    let mut validators = Vec::new();
    for expression in expressions {
        collect_validator_expression(&expression, &mut validators);
    }
    validators
}

fn collect_validator_expression(expression: &str, validators: &mut Vec<String>) {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return;
    }
    if trimmed.starts_with('[') {
        if let Some(close) = find_matching_brace(trimmed, '[') {
            for item in split_top_level(&trimmed[1..close], ',') {
                collect_validator_expression(&item, validators);
            }
        }
        return;
    }
    if let Some(compose) = trimmed.strip_prefix("Validators.compose") {
        if let Some(open) = compose.find('(') {
            if let Some((_, body)) = consume_call_expression(compose, open) {
                collect_validator_expression(&body, validators);
            }
        }
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("Validators.") {
        let name: String = rest
            .chars()
            .take_while(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '$'))
            .collect();
        push_rule(validators, &name);
        return;
    }
    let callable = trimmed
        .split_once('(')
        .map(|(name, _)| name.trim())
        .unwrap_or(trimmed)
        .rsplit('.')
        .next()
        .unwrap_or(trimmed)
        .trim();
    if is_identifier(callable) && !matches!(callable, "null" | "undefined" | "true" | "false") {
        push_rule(validators, callable);
    }
}

fn push_rule(validators: &mut Vec<String>, rule: &str) {
    if !rule.is_empty() && !validators.iter().any(|existing| existing == rule) {
        validators.push(rule.to_string());
    }
}
