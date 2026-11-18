// MSTest + Moq semantic compression.
//
// Class captures are produced by the tree-sitter C# pipeline. This extractor
// deliberately stays string-based inside those structural boundaries, matching
// the other .NET meta-layer extractors.

use std::collections::BTreeSet;

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

use super::MetaBlock;
use super::markers::{
    build_fixture_line, build_mock_line, build_setup_line, build_test_class_line, build_test_line,
    build_verify_line,
};
use super::semantic::extract_class_name_from_class;

/// Extract MSTest and Moq markers from one tree-sitter class capture.
pub fn extract_testing(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    if !has_attribute(class_source, "TestClass") {
        return None;
    }

    let class_name = extract_class_name_from_class(class_source)?;
    let mut lines = vec![build_test_class_line(&class_name)];

    if fidelity != Fidelity::Low {
        lines.extend(extract_test_methods(class_source));
        lines.extend(extract_mocks(class_source));
    }

    if fidelity == Fidelity::High {
        lines.extend(extract_setups(class_source));
        lines.extend(extract_verifications(class_source));
        lines.extend(extract_fixtures(class_source));
    }

    Some(MetaBlock { lines })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TestSubjectCandidate {
    pub(crate) test_class: String,
    pub(crate) subject: String,
}

enum ConstructedSubject {
    None,
    Unique(String),
    Ambiguous,
}

/// Publish the source-proven TestClass declaration without manufacturing a SUT.
pub(crate) fn extract_testing_declaration(class_source: &str) -> Option<SemanticEdge> {
    if !is_test_class_declaration(class_source) {
        return None;
    }
    let class_name = extract_class_name_from_class(class_source)?;
    let entity = EntityRef::new("dotnet", "TestClass", class_name);
    Some(SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: entity.clone(),
        object: entity,
        layer: "dotnet",
        call_evidence: None,
    })
}

/// Collect candidate evidence. Exact declaration authority is joined later.
pub(crate) fn extract_testing_subject_candidate(
    class_source: &str,
) -> Option<TestSubjectCandidate> {
    if !is_test_class_declaration(class_source) {
        return None;
    }
    let class_name = extract_class_name_from_class(class_source)?;
    let constructed = infer_constructed_sut(class_source, &class_name);
    let naming_candidate = (!class_name.ends_with("IntegrationTests")
        && !class_name.ends_with("UnitTests"))
    .then(|| class_name.strip_suffix("Tests"))
    .flatten();
    let subject = match constructed {
        ConstructedSubject::Unique(subject) => subject,
        ConstructedSubject::Ambiguous => return None,
        ConstructedSubject::None => naming_candidate
            .filter(|name| !name.is_empty())
            .map(str::to_string)?,
    };
    Some(TestSubjectCandidate {
        test_class: class_name,
        subject,
    })
}

pub(crate) fn test_subject_candidates_in_source(source: &str) -> Vec<TestSubjectCandidate> {
    let Some(language) = crate::compression::language::safe_csharp_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    collect_test_candidates(tree.root_node(), source.as_bytes(), &mut candidates);
    candidates
}

fn collect_test_candidates(
    node: tree_sitter::Node<'_>,
    source: &[u8],
    candidates: &mut Vec<TestSubjectCandidate>,
) {
    if node.kind() == "class_declaration"
        && let Ok(class_source) = node.utf8_text(source)
        && let Some(candidate) = extract_testing_subject_candidate(class_source)
    {
        candidates.push(candidate);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_test_candidates(child, source, candidates);
    }
}

fn is_test_class_declaration(source: &str) -> bool {
    let Some(language) = crate::compression::language::safe_csharp_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser.parse(source, None) else {
        return false;
    };
    first_class_has_attribute(tree.root_node(), source.as_bytes(), "TestClass")
}

fn first_class_has_attribute(node: tree_sitter::Node<'_>, source: &[u8], name: &str) -> bool {
    if node.kind() == "class_declaration" {
        let mut cursor = node.walk();
        return node.named_children(&mut cursor).any(|child| {
            child.kind() == "attribute_list"
                && child
                    .utf8_text(source)
                    .is_ok_and(|attributes| has_attribute(attributes, name))
        });
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| first_class_has_attribute(child, source, name))
}

fn has_attribute(source: &str, name: &str) -> bool {
    source.contains(&format!("[{name}]")) || source.contains(&format!("[{name}("))
}

fn extract_test_methods(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (attribute, data_driven) in [("TestMethod", false), ("DataTestMethod", true)] {
        let mut offset = 0;
        let needle = format!("[{attribute}]");
        while let Some(relative) = source[offset..].find(&needle) {
            let start = offset + relative;
            let tail = &source[start + needle.len()..];
            if let Some((name, signature_start)) = following_method_name(tail) {
                let rows = if data_driven {
                    tail[..signature_start].matches("[DataRow").count()
                } else {
                    0
                };
                found.push((start, build_test_line(&name, rows)));
            }
            offset = start + needle.len();
        }
    }
    found.sort_by_key(|(position, _)| *position);
    found.into_iter().map(|(_, line)| line).collect()
}

fn following_method_name(source: &str) -> Option<(String, usize)> {
    let visibility = ["public ", "internal ", "protected ", "private "]
        .iter()
        .filter_map(|needle| source.find(needle).map(|position| (position, needle.len())))
        .min_by_key(|(position, _)| *position)?;
    let signature = &source[visibility.0 + visibility.1..];
    let open = signature.find('(')?;
    let name = signature[..open].split_whitespace().last()?.trim();
    is_identifier(name).then(|| (name.to_string(), visibility.0))
}

fn extract_mocks(source: &str) -> Vec<String> {
    let mut mocks = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find("Mock<") {
        let start = offset + relative + "Mock<".len();
        if let Some(end) = find_matching_angle(source, start) {
            let dependency = source[start..end].trim();
            if !dependency.is_empty() {
                mocks.insert(dependency.to_string());
            }
            offset = end + 1;
        } else {
            break;
        }
    }
    mocks
        .into_iter()
        .map(|mock| build_mock_line(&mock))
        .collect()
}

fn extract_setups(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(".Setup(") {
        let open = offset + relative + ".Setup".len();
        let Some(close) = find_matching(source, open, '(', ')') else {
            break;
        };
        let member = extract_lambda_member(&source[open + 1..close]);
        let tail = &source[close + 1..];
        let chain_end = tail.find(';').unwrap_or(tail.len());
        let tail = &tail[..chain_end];
        let returns = [".ReturnsAsync(", ".Returns("]
            .iter()
            .filter_map(|needle| tail.find(needle).map(|position| (position, *needle)))
            .min_by_key(|(position, _)| *position);
        if let (Some(member), Some((position, needle))) = (member, returns) {
            let return_open = close + 1 + position + needle.len() - 1;
            if let Some(return_close) = find_matching(source, return_open, '(', ')') {
                if let Some(hint) = concise_return_hint(&source[return_open + 1..return_close]) {
                    lines.push(build_setup_line(&member, &hint));
                }
            }
        }
        offset = close + 1;
    }
    lines
}

fn extract_verifications(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(".Verify(") {
        let open = offset + relative + ".Verify".len();
        let Some(close) = find_matching(source, open, '(', ')') else {
            break;
        };
        let arguments = &source[open + 1..close];
        if let Some(member) = extract_lambda_member(arguments) {
            lines.push(build_verify_line(&member, static_times(arguments)));
        }
        offset = close + 1;
    }
    lines
}

fn extract_fixtures(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for attribute in ["TestInitialize", "ClassInitialize", "TestCleanup"] {
        let needle = format!("[{attribute}]");
        let mut offset = 0;
        while let Some(relative) = source[offset..].find(&needle) {
            let start = offset + relative;
            if let Some((name, _)) = following_method_name(&source[start + needle.len()..]) {
                lines.push(build_fixture_line(&name));
            }
            offset = start + needle.len();
        }
    }
    lines
}

fn extract_lambda_member(expression: &str) -> Option<String> {
    let body = expression.split_once("=>")?.1.trim_start();
    let body = body.strip_prefix("await ").unwrap_or(body);
    let open = body.find('(').unwrap_or(body.len());
    let path = body[..open].trim();
    let member = path.rsplit('.').next()?.trim();
    is_identifier(member).then(|| member.to_string())
}

fn static_times(arguments: &str) -> Option<usize> {
    if arguments.contains("Times.Once()") {
        return Some(1);
    }
    if arguments.contains("Times.Never()") {
        return Some(0);
    }
    let start = arguments.find("Times.Exactly(")? + "Times.Exactly(".len();
    let end = arguments[start..].find(')')? + start;
    arguments[start..end].trim().parse().ok()
}

fn concise_return_hint(expression: &str) -> Option<String> {
    let collapsed = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let hint = collapsed.trim();
    if hint.is_empty() || hint.contains("=>") || hint.contains(';') {
        None
    } else {
        Some(hint.to_string())
    }
}

fn infer_constructed_sut(source: &str, class_name: &str) -> ConstructedSubject {
    let mut candidates = BTreeSet::new();
    for body in setup_bodies(source, class_name) {
        let mut offset = 0;
        while let Some(relative) = body[offset..].find("= new ") {
            let equals = offset + relative;
            let Some(lhs) = body[..equals].split_whitespace().last() else {
                offset = equals + "= new ".len();
                continue;
            };
            let lhs = lhs.trim();
            let lhs = lhs.strip_prefix("this.").unwrap_or(lhs);
            let type_start = equals + "= new ".len();
            let Some(type_length) = body[type_start..]
                .find(|character: char| character == '(' || character.is_whitespace())
            else {
                break;
            };
            let type_end = type_start + type_length;
            let constructed = body[type_start..type_end].trim();
            if lhs.starts_with('_')
                && is_identifier(lhs)
                && is_type_name(constructed)
                && has_persistent_field(source, lhs)
            {
                candidates.insert(constructed.to_string());
            }
            offset = type_end;
        }
    }
    match candidates.len() {
        0 => ConstructedSubject::None,
        1 => ConstructedSubject::Unique(candidates.into_iter().next().expect("one candidate")),
        _ => ConstructedSubject::Ambiguous,
    }
}

fn setup_bodies<'a>(source: &'a str, class_name: &str) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    for marker in [format!("{class_name}("), "[TestInitialize]".to_string()] {
        let mut offset = 0;
        while let Some(relative) = source[offset..].find(&marker) {
            let start = offset + relative + marker.len();
            if let Some(open_relative) = source[start..].find('{') {
                let open = start + open_relative;
                if let Some(close) = find_matching(source, open, '{', '}') {
                    bodies.push(&source[open + 1..close]);
                    offset = close + 1;
                    continue;
                }
            }
            break;
        }
    }
    bodies
}

fn has_persistent_field(source: &str, field: &str) -> bool {
    let Some(language) = crate::compression::language::safe_csharp_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser.parse(source, None) else {
        return false;
    };
    let Some(class) = first_class_node(tree.root_node()) else {
        return false;
    };
    let Some(body) = class.child_by_field_name("body") else {
        return false;
    };
    let mut cursor = body.walk();
    body.named_children(&mut cursor).any(|declaration| {
        declaration.kind() == "field_declaration"
            && declaration.utf8_text(source.as_bytes()).is_ok_and(|text| {
                text.split(|character: char| character != '_' && !character.is_ascii_alphanumeric())
                    .any(|token| token == field)
            })
    })
}

fn first_class_node(node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    if node.kind() == "class_declaration" {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).find_map(first_class_node)
}

fn find_matching(source: &str, open: usize, opening: char, closing: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut string_delimiter = None;
    let mut escaped = false;
    for (relative, character) in source[open..].char_indices() {
        if let Some(delimiter) = string_delimiter {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                string_delimiter = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            string_delimiter = Some(character);
        } else if character == opening {
            depth += 1;
        } else if character == closing {
            depth -= 1;
            if depth == 0 {
                return Some(open + relative);
            }
        }
    }
    None
}

fn find_matching_angle(source: &str, start: usize) -> Option<usize> {
    let mut depth = 1usize;
    for (relative, character) in source[start..].char_indices() {
        match character {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + relative);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn is_type_name(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
        && value
            .chars()
            .all(|character| character == '_' || character == '.' || character.is_alphanumeric())
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/testing.rs"]
mod tests;
