//! Reusable per-source comment and string membership index.

/// Immutable index of byte regions that the shared meta-layer scanner treats
/// as comments, strings, or template literals.
///
#[derive(Debug, Default)]
pub struct LexicalRegions {
    spans: Vec<(usize, usize)>,
}

impl LexicalRegions {
    /// Scans `source` once and records the half-open byte ranges where
    /// [`super::is_inside_comment_or_string`] returns `true`.
    pub fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut spans = Vec::new();
        let mut span_start = None;
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let mut in_string: Option<u8> = None;
        let mut skip_next = false;

        for (index, &byte) in bytes.iter().enumerate() {
            let was_non_code = in_line_comment || in_block_comment || in_string.is_some();
            if skip_next {
                skip_next = false;
            } else if in_line_comment {
                if byte == b'\n' {
                    in_line_comment = false;
                }
            } else if in_block_comment {
                if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    in_block_comment = false;
                    skip_next = true;
                }
            } else if let Some(quote) = in_string {
                if byte == b'\\' {
                    skip_next = true;
                } else if byte == quote {
                    in_string = None;
                }
            } else if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
                in_line_comment = true;
            } else if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
                in_block_comment = true;
            } else if matches!(byte, b'\'' | b'"' | b'`') {
                in_string = Some(byte);
            }

            let is_non_code = in_line_comment || in_block_comment || in_string.is_some();
            if !was_non_code && is_non_code {
                span_start = Some(index + 1);
            } else if was_non_code && !is_non_code {
                spans.push((
                    span_start
                        .take()
                        .expect("a non-code region must have a recorded start"),
                    index + 1,
                ));
            }
        }
        if let Some(span_start) = span_start {
            spans.push((span_start, source.len() + 1));
        }

        Self { spans }
    }

    pub fn contains(&self, pos: usize) -> bool {
        self.spans
            .binary_search_by(|(start, end)| {
                if pos < *start {
                    std::cmp::Ordering::Greater
                } else if pos >= *end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }
}
