//! Import and local `FormBuilder` alias detection for Reactive Forms.

use super::is_identifier;
use crate::angular_meta::util::{
    consume_call_expression, extract_decl_name, is_inside_comment_or_string, split_top_level,
};

#[derive(Debug, Default)]
pub(super) struct ImportedForms {
    pub(super) builders: Vec<String>,
    pub(super) groups: Vec<String>,
    pub(super) controls: Vec<String>,
    pub(super) arrays: Vec<String>,
}

impl ImportedForms {
    pub(super) fn is_empty(&self) -> bool {
        self.builders.is_empty()
            && self.groups.is_empty()
            && self.controls.is_empty()
            && self.arrays.is_empty()
    }
}

pub(super) fn extract_forms_imports(source: &str) -> ImportedForms {
    let mut imports = ImportedForms::default();
    let mut search_from = 0usize;
    while let Some(relative) = source[search_from..].find("@angular/forms") {
        let package_pos = search_from + relative;
        let Some(import_pos) = source[..package_pos].rfind("import") else {
            search_from = package_pos + 1;
            continue;
        };
        if is_inside_comment_or_string(source, import_pos) {
            search_from = package_pos + 1;
            continue;
        }
        let prefix = &source[import_pos..package_pos];
        if !prefix.contains("from") || prefix.contains(';') {
            search_from = package_pos + 1;
            continue;
        }
        if let (Some(open), Some(close)) = (prefix.find('{'), prefix.rfind('}')) {
            if close > open {
                for item in split_top_level(&prefix[open + 1..close], ',') {
                    collect_import(&item, &mut imports);
                }
            }
        }
        search_from = package_pos + "@angular/forms".len();
    }
    imports
}

fn collect_import(item: &str, imports: &mut ImportedForms) {
    let words: Vec<&str> = item.split_whitespace().collect();
    let Some(imported) = words.first().copied() else {
        return;
    };
    let local = if words.get(1) == Some(&"as") {
        words.get(2).copied().unwrap_or(imported)
    } else {
        imported
    };
    match imported {
        "FormBuilder" => push_unique(&mut imports.builders, local),
        "FormGroup" => push_unique(&mut imports.groups, local),
        "FormControl" => push_unique(&mut imports.controls, local),
        "FormArray" => push_unique(&mut imports.arrays, local),
        _ => {}
    }
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if is_identifier(value) && !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

pub(super) fn has_construction_hint(source: &str, imports: &ImportedForms) -> bool {
    source.contains(".group(")
        || source.contains(".control(")
        || source.contains(".array(")
        || imports
            .groups
            .iter()
            .chain(&imports.controls)
            .chain(&imports.arrays)
            .any(|name| source.contains(&format!("new {name}")))
}

pub(super) fn extract_builder_aliases(source: &str, builder_types: &[String]) -> Vec<String> {
    let mut aliases = constructor_aliases(source, builder_types);
    let mut search_from = 0usize;
    while let Some(relative) = source[search_from..].find("inject") {
        let pos = search_from + relative;
        search_from = pos + "inject".len();
        if is_inside_comment_or_string(source, pos) {
            continue;
        }
        let rest = &source[search_from..];
        let Some(open_relative) = rest.find('(') else {
            break;
        };
        if !rest[..open_relative].trim().is_empty() {
            continue;
        }
        let open = search_from + open_relative;
        let Some((_, argument)) = consume_call_expression(source, open) else {
            continue;
        };
        if builder_types
            .iter()
            .any(|builder| builder == argument.trim())
        {
            if let Some(name) = extract_decl_name(&source[..pos]) {
                push_unique(&mut aliases, &name);
            }
        }
    }
    aliases
}

fn constructor_aliases(source: &str, builder_types: &[String]) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut search_from = 0usize;
    while let Some(relative) = source[search_from..].find("constructor") {
        let pos = search_from + relative;
        search_from = pos + "constructor".len();
        if is_inside_comment_or_string(source, pos) {
            continue;
        }
        let Some(open_relative) = source[search_from..].find('(') else {
            break;
        };
        let open = search_from + open_relative;
        let Some((_, params)) = consume_call_expression(source, open) else {
            continue;
        };
        for param in split_top_level(&params, ',') {
            let pieces = split_top_level(&param, ':');
            if pieces.len() < 2 {
                continue;
            }
            let ty = pieces[1].trim().trim_end_matches('?');
            if builder_types.iter().any(|builder| builder == ty) {
                if let Some(name) = last_identifier(&pieces[0]) {
                    push_unique(&mut aliases, name);
                }
            }
        }
    }
    aliases
}

fn last_identifier(text: &str) -> Option<&str> {
    text.split_whitespace()
        .last()
        .map(|word| word.trim_end_matches('?'))
        .filter(|word| is_identifier(word))
}
