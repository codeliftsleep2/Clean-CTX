//! Angular TestBed + Vitest source-shape extraction.
//!
//! This detector is independent of Angular decorators. It deliberately uses
//! conservative string scanning and the shared meta-layer delimiter helpers;
//! it does not parse or evaluate TypeScript expressions.

use std::collections::BTreeSet;
use std::path::Path;

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Marker vocabulary emitted by the Angular testing meta-layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestKind {
    Describe,
    Test,
    TestBed,
    Spy,
}

impl PhiMarker for TestKind {
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Describe => "Φdescribe:",
            Self::Test => "Φit:",
            Self::TestBed => "ΦtestBed:",
            Self::Spy => "Φspy:",
        }
    }

    fn expansion(self) -> &'static str {
        match self {
            Self::Describe => "describe suite",
            Self::Test => "test case",
            Self::TestBed => "TestBed configuration",
            Self::Spy => "Vitest spy/mock",
        }
    }

    fn all_in_expand_order() -> &'static [Self] {
        &[Self::Describe, Self::TestBed, Self::Spy, Self::Test]
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "Φdescribe" => Some(Self::Describe),
            "Φit" => Some(Self::Test),
            "ΦtestBed" => Some(Self::TestBed),
            "Φspy" => Some(Self::Spy),
            _ => None,
        }
    }

    fn token(self) -> &'static str {
        match self {
            Self::Describe => "Φdescribe",
            Self::Test => "Φit",
            Self::TestBed => "ΦtestBed",
            Self::Spy => "Φspy",
        }
    }
}

/// Structured testing facts extracted from one TypeScript source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestingShape {
    describe_count: usize,
    test_count: usize,
    tests: Vec<String>,
    test_bed: Vec<String>,
    spies: Vec<String>,
    suites: Vec<String>,
}

impl TestingShape {
    pub fn is_empty(&self) -> bool {
        self.describe_count == 0
            && self.test_count == 0
            && self.test_bed.is_empty()
            && self.spies.is_empty()
    }

    /// Render an independent, fidelity-gated testing section.
    pub fn render(&self, fidelity: Fidelity) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut lines = vec![format!(
            "{}[describes={} tests={}]",
            TestKind::Describe.marker_prefix(),
            self.describe_count,
            self.test_count
        )];
        if fidelity != Fidelity::Low {
            lines.extend(
                self.tests
                    .iter()
                    .map(|name| format!("{}{}", TestKind::Test.marker_prefix(), name)),
            );
            lines.extend(
                self.test_bed
                    .iter()
                    .map(|summary| format!("{}{}", TestKind::TestBed.marker_prefix(), summary)),
            );
        }
        if matches!(
            fidelity,
            Fidelity::High | Fidelity::Edit | Fidelity::Verbatim
        ) {
            lines.extend(
                self.spies
                    .iter()
                    .map(|spy| format!("{}{}", TestKind::Spy.marker_prefix(), spy)),
            );
            lines.extend(
                self.suites
                    .iter()
                    .map(|suite| format!("{}{}", TestKind::Describe.marker_prefix(), suite)),
            );
        }
        let mut rendered = String::from("// --- Φ Testing Meta ---\n");
        for line in lines {
            rendered.push_str(&line);
            rendered.push('\n');
        }
        rendered
    }
}

/// Testing eligibility is path-aware but independent of Angular decorators.
pub fn is_testing_source(source: &str, path: &Path) -> bool {
    path.to_string_lossy().ends_with(".spec.ts")
        || source.contains("describe(")
        || source.contains("TestBed")
        || source.contains("vi.fn(")
}

/// Extract generic testing markers. The caller owns path-based eligibility;
/// every emitted fact still requires an actual source shape.
pub fn extract_testing_shape(source: &str, _fidelity: Fidelity) -> Option<TestingShape> {
    let describes = collect_literal_calls(source, "describe");
    let tests = collect_literal_calls(source, "it");
    let test_bed = extract_test_bed_summaries(source);
    let spies = extract_vitest_spies(source);
    let shape = TestingShape {
        describe_count: count_calls(source, "describe"),
        test_count: count_calls(source, "it"),
        tests: tests.into_iter().map(|call| call.value).collect(),
        test_bed,
        spies,
        suites: describe_hierarchy(describes),
    };
    (!shape.is_empty()).then_some(shape)
}

/// Infer conservative `TestArtifact --Tests--> Component` relationships.
pub fn extract_testing_semantic_edges(source: &str, path: &Path) -> Vec<SemanticEdge> {
    if !is_testing_source(source, path) {
        return Vec::new();
    }
    let explicit = create_component_candidates(source);
    let filename = component_from_spec_path(path);

    let targets: Vec<String> = if explicit.is_empty() {
        filename.into_iter().collect()
    } else if let Some(filename_target) = filename {
        if explicit.len() == 1 && explicit.contains(&filename_target) {
            vec![filename_target]
        } else {
            return Vec::new();
        }
    } else {
        explicit.into_iter().collect()
    };
    if targets.is_empty() {
        return Vec::new();
    }

    let artifact = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("typescript-test");
    targets
        .into_iter()
        .map(|target| SemanticEdge {
            relation: SemanticRelation::Tests,
            subject: EntityRef::new("angular", "TestArtifact", artifact),
            object: EntityRef::new("angular", "Component", target),
            layer: "angular",
        })
        .collect()
}

pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<TestKind>(line)
}

#[derive(Debug)]
struct LiteralCall {
    value: String,
    start: usize,
    body_end: Option<usize>,
}

fn collect_literal_calls(source: &str, name: &str) -> Vec<LiteralCall> {
    let needle = format!("{name}(");
    call_positions(source, &needle)
        .into_iter()
        .filter_map(|start| {
            let open = start + name.len();
            let (value, _) = first_literal_argument(source, open + 1)?;
            let close = find_matching(source, open, '(', ')')?;
            let body_end = callback_body_end(source, open, close);
            Some(LiteralCall {
                value,
                start,
                body_end,
            })
        })
        .collect()
}

fn count_calls(source: &str, name: &str) -> usize {
    call_positions(source, &format!("{name}(")).len()
}

fn call_positions(source: &str, needle: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(needle) {
        let start = offset + relative;
        let boundary_ok = start == 0
            || !source.as_bytes()[start - 1].is_ascii_alphanumeric()
                && source.as_bytes()[start - 1] != b'_'
                && source.as_bytes()[start - 1] != b'.';
        if boundary_ok && !crate::angular_meta::util::is_inside_comment_or_string(source, start) {
            positions.push(start);
        }
        offset = start + needle.len();
    }
    positions
}

fn first_literal_argument(source: &str, mut start: usize) -> Option<(String, usize)> {
    while source
        .as_bytes()
        .get(start)
        .is_some_and(u8::is_ascii_whitespace)
    {
        start += 1;
    }
    let delimiter = *source.as_bytes().get(start)?;
    if !matches!(delimiter, b'\'' | b'"' | b'`') {
        return None;
    }
    let mut escaped = false;
    for index in start + 1..source.len() {
        let byte = source.as_bytes()[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == delimiter {
            let value = &source[start + 1..index];
            if delimiter == b'`' && value.contains("${") {
                return None;
            }
            return Some((value.to_string(), index + 1));
        }
    }
    None
}

fn callback_body_end(source: &str, open: usize, close: usize) -> Option<usize> {
    let arguments = &source[open + 1..close];
    let arrow = arguments.find("=>")? + open + 1;
    let brace = source[arrow + 2..close].find('{')? + arrow + 2;
    find_matching(source, brace, '{', '}')
}

fn describe_hierarchy(calls: Vec<LiteralCall>) -> Vec<String> {
    let mut paths: Vec<(String, usize)> = Vec::new();
    let mut result = Vec::new();
    for call in calls {
        paths.retain(|(_, end)| call.start < *end);
        let name = if paths.is_empty() {
            call.value.clone()
        } else {
            format!(
                "{} > {}",
                paths
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(" > "),
                call.value
            )
        };
        result.push(name.clone());
        if let Some(end) = call.body_end {
            paths.push((call.value, end));
        }
    }
    result
}

fn extract_test_bed_summaries(source: &str) -> Vec<String> {
    let needle = "TestBed.configureTestingModule(";
    call_positions(source, needle)
        .into_iter()
        .filter_map(|start| {
            let open = start + needle.len() - 1;
            let close = find_matching(source, open, '(', ')')?;
            let object_start = source[open + 1..close].find('{')? + open + 1;
            let object_end = find_matching(source, object_start, '{', '}')?;
            let object = &source[object_start + 1..object_end];
            let providers = extract_array_property(object, "providers");
            let imports = extract_array_property(object, "imports");
            let mut parts = Vec::new();
            if !providers.is_empty() {
                parts.push(format!("providers=[{}]", providers.join(",")));
            }
            if !imports.is_empty() {
                parts.push(format!("imports=[{}]", imports.join(",")));
            }
            (!parts.is_empty()).then(|| parts.join(" "))
        })
        .collect()
}

fn extract_array_property(object: &str, property: &str) -> Vec<String> {
    let mut offset = 0;
    while let Some(relative) = object[offset..].find(property) {
        let start = offset + relative;
        let after = start + property.len();
        let tail = object[after..].trim_start();
        if !tail.starts_with(':') {
            offset = after;
            continue;
        }
        let colon = object[after..].find(':').unwrap_or(0) + after;
        let Some(open_relative) = object[colon + 1..].find('[') else {
            return Vec::new();
        };
        let open = colon + 1 + open_relative;
        let Some(close) = find_matching(object, open, '[', ']') else {
            return Vec::new();
        };
        return stable_array_values(&object[open + 1..close]);
    }
    Vec::new()
}

fn stable_array_values(array: &str) -> Vec<String> {
    let mut values = Vec::new();
    for entry in crate::angular_meta::util::split_top_level(array, ',') {
        let entry = entry.trim();
        let candidate = if entry.starts_with('{') {
            extract_provider_token(entry)
        } else {
            identifier_like(entry).then(|| entry.to_string())
        };
        if let Some(candidate) = candidate
            && !values.contains(&candidate)
        {
            values.push(candidate);
        }
    }
    values
}

fn extract_provider_token(entry: &str) -> Option<String> {
    let provide = entry.find("provide")? + "provide".len();
    let colon = entry[provide..].find(':')? + provide;
    let token = entry[colon + 1..].split([',', '}']).next()?.trim();
    identifier_like(token).then(|| token.to_string())
}

fn extract_vitest_spies(source: &str) -> Vec<String> {
    let mut spies = BTreeSet::new();
    for start in call_positions(source, "vi.fn(") {
        let name = enclosing_object_assignment(source, start)
            .unwrap_or_else(|| assignment_target_before(source, start));
        if let Some(name) = name {
            spies.insert(name.to_string());
        }
    }
    for start in call_positions(source, "vi.spyOn(") {
        let open = start + "vi.spyOn".len();
        let Some(close) = find_matching(source, open, '(', ')') else {
            continue;
        };
        let args = crate::angular_meta::util::split_top_level(&source[open + 1..close], ',');
        if args.len() >= 2
            && identifier_path(args[0].trim())
            && let Some((method, _)) = first_literal_argument(&args[1], 0)
            && identifier_like(&method)
        {
            spies.insert(format!("{}.{}", args[0].trim(), method));
        }
    }
    for start in call_positions(source, "vi.mock(") {
        let open = start + "vi.mock".len();
        if let Some((module, _)) = first_literal_argument(source, open + 1) {
            spies.insert(format!("module:{module}"));
        }
    }
    spies.into_iter().collect()
}

fn enclosing_object_assignment(source: &str, call_start: usize) -> Option<Option<&str>> {
    let open = crate::angular_meta::util::find_enclosing_brace(source, call_start)?;
    let close = find_matching(source, open, '{', '}')?;
    if close < call_start {
        return None;
    }

    let before_brace = source[..open].trim_end();
    match before_brace.as_bytes().last() {
        Some(b'=') => Some(assignment_target_before(source, before_brace.len() - 1)),
        Some(b':' | b'(' | b',' | b'[') => Some(None),
        _ => None,
    }
}

fn assignment_target_before(source: &str, end: usize) -> Option<&str> {
    let statement_start = source[..end]
        .rfind(['\n', ';'])
        .map_or(0, |index| index + 1);
    source[statement_start..end]
        .split('=')
        .next()?
        .split_whitespace()
        .last()
        .filter(|name| identifier_like(name))
}

fn create_component_candidates(source: &str) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    for start in call_positions(source, "TestBed.createComponent(") {
        let argument = start + "TestBed.createComponent(".len();
        let tail = source[argument..].trim_start();
        let candidate = tail
            .split(|byte: char| byte == ')' || byte == ',' || byte.is_whitespace())
            .next()
            .unwrap_or("");
        if type_name(candidate) {
            candidates.insert(candidate.to_string());
        }
    }
    candidates
}

fn component_from_spec_path(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;
    let stem = filename.strip_suffix(".component.spec.ts")?;
    if stem.is_empty() {
        return None;
    }
    let mut class_name = String::new();
    for segment in stem.split(['-', '_', '.']) {
        if segment.is_empty() || !segment.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            return None;
        }
        let mut chars = segment.chars();
        class_name.extend(chars.next()?.to_uppercase());
        class_name.extend(chars);
    }
    class_name.push_str("Component");
    Some(class_name)
}

fn identifier_like(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn identifier_path(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(identifier_like)
}

fn type_name(value: &str) -> bool {
    identifier_like(value)
        && value
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
}

fn find_matching(source: &str, open: usize, opening: char, closing: char) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open).copied()? != opening as u8 {
        return None;
    }
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => index = skip_quoted(bytes, index)?,
            b'`' => index = skip_template(bytes, index)?,
            byte if byte == opening as u8 => depth += 1,
            byte if byte == closing as u8 => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let delimiter = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] == delimiter {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn skip_template(bytes: &[u8], start: usize) -> Option<usize> {
    skip_quoted(bytes, start)
}
