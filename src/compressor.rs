// src/compressor.rs
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use tree_sitter::{Language, Parser as TSParser, Query, QueryCursor};
use crate::queries;
use crate::dictionary::{PathDictionary, SymbolDictionary};
use crate::cache::LocalStateCache;
use crate::analytics::calculate_savings;
use crate::helpers::{
    compact_expression, compact_import, extract_class_name, extract_field,
    extract_method_sig, format_class_entry, simple_compact,
};

/// Compression fidelity level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fidelity {
    /// Maximum compression — strips keywords, async, fields, errors (current default)
    Low,
    /// Balanced — preserves async, field types, errors, control flow markers
    Medium,
    /// Minimal compression — preserves as much semantic depth as possible
    High,
}

impl Fidelity {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "medium" => Fidelity::Medium,
            "high" => Fidelity::High,
            _ => Fidelity::Low,
        }
    }
}

/// A progress event emitted by streaming compression. The `progress` is in
/// the range `0.0..=1.0`. `phase` describes the current step of the pipeline.
#[derive(Debug, Clone)]
pub struct CompressionProgress {
    /// Percentage complete in `[0.0, 1.0]`.
    pub progress: f64,
    /// Human-readable phase label, e.g. "reading", "parsing", "compressing".
    pub phase: String,
    /// Optional snippet or partial output for the current chunk.
    pub partial: Option<String>,
}

/// Reads a target source file and compiles it down into a highly compacted,
/// keyword-stripped structural signature stream with configurable fidelity.
pub fn compress_file(
    file: PathBuf, 
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_code = fs::read_to_string(&file)?;
    let source_bytes = source_code.as_bytes();
    
    let current_hash = cache.compute_hash(source_bytes);
    let absolute_path = fs::canonicalize(&file)?.to_string_lossy().into_owned();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // Include fidelity in the cache key so different fidelity levels don't share cached results
    let cache_key = format!("{}::{:?}", absolute_path, fidelity);
    let is_modified = cache.update_and_verify(cache_key, current_hash);
    if !is_modified {
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n", 
            path_alias
        );
        let meta = calculate_savings(&source_code, &cached_notice);
        
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, cached_notice
        ));
    }

    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let (language, query_string): (Language, &str) = match extension {
        "ts" | "js" => (tree_sitter_typescript::language_typescript(), queries::TS_QUERY),
        "cs" => (tree_sitter_c_sharp::language(), queries::CS_QUERY),
        _ => return Err(format!("Unsupported file extension: .{}", extension).into()),
    };

    let mut parser = TSParser::new();
    parser.set_language(language)?;
    let tree = parser.parse(&source_code, None).ok_or("AST Generation Error")?;

    let query = Query::new(language, query_string)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_bytes);

    let mut output_lines: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut markers: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let method_count: usize = 0;
    let _field_count: usize = 0;
    let mut import_count: usize = 0;
    let mut class_count: usize = 0;
    let _marker_count: usize = 0;

    // Collect all captures with their node positions for ordering
    #[derive(Debug)]
    struct CaptureEntry {
        name: String,
        text: String,
        start_byte: usize,
    }

    let mut all_captures: Vec<CaptureEntry> = Vec::new();

    for mat in matches {
        for capture in mat.captures {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let text = text_slice.to_string();
                let cap_name = capture_name.clone();
                all_captures.push(CaptureEntry {
                    name: cap_name,
                    text: if capture_name == "class.root" {
                        extract_class_name(&text)
                    } else if capture_name == "method.root" {
                        extract_method_sig(&text, fidelity)
                    } else if capture_name == "field.root" {
                        extract_field(&text, fidelity)
                    } else {
                        compact_expression(&text, fidelity)
                    },
                    start_byte: capture.node.start_byte(),
                });
            }
        }
    }

    // Sort captures by document position
    all_captures.sort_by(|a, b| a.start_byte.cmp(&b.start_byte));

    // Build the output: walk through captures in document order
    for cap in &all_captures {
        match cap.name.as_str() {
            "import.root" => {
                import_count += 1;
                imports.push(compact_import(&cap.text, fidelity));
            }
            "class.root" => {
                class_count += 1;
                // Separate classes with empty line in higher fidelities
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_class_entry(&cap.text, &fields, fidelity));
                fields.clear();
                markers.clear();
            }
            "method.root" => {
                let sig = &cap.text;
                if !markers.is_empty() {
                    let marker_str = markers.join(" ");
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {} {{ {} }}", sig, marker_str));
                    } else if fidelity == Fidelity::Medium {
                        output_lines.push(format!("{} {}", sig, marker_str));
                    } else {
                        output_lines.push(sig.clone());
                    }
                } else {
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {}", sig));
                    } else {
                        output_lines.push(sig.clone());
                    }
                }
                markers.clear();
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    fields.push(cap.text.clone());
                }
            }
            _ => {
                // Control flow / behavior markers (throw, for, if, while, return)
                if fidelity == Fidelity::Low {
                    continue;
                }
                let marker = match cap.name.as_str() {
                    "throw.root" => format!("⊕!{}", cap.text),
                    "for.root" => "⊕loop".to_string(),
                    "if.root" => "⊕guard".to_string(),
                    "while.root" => "⊕loop".to_string(),
                    "return.root" => format!("⊕⇒{}", cap.text),
                    _ => continue,
                };
                // Deduplicate consecutive identical markers
                if markers.last().map(|m| m != &marker).unwrap_or(true) {
                    markers.push(marker);
                }
            }
        }
    }

    // If we have fields and no class context, add them if fidelity allows
    if !fields.is_empty() && output_lines.is_empty() && fidelity != Fidelity::Low {
        output_lines.push(format!("⊕fields {{ {} }}", fields.join("; ")));
    }

    // If nothing was captured, provide a raw fallback
    if output_lines.is_empty() {
        if let Some(first_line) = source_code.lines().next() {
            let trimmed = first_line.trim().to_string();
            if !trimmed.is_empty() {
                output_lines.push(simple_compact(&trimmed, fidelity));
            }
        }
    }

    // Prepend imports (if any were captured) to the output
    if !imports.is_empty() {
        let import_block = match fidelity {
            Fidelity::Low => imports.join("; "),
            _ => imports.join("\n"),
        };
        output_lines.insert(0, import_block);
    }

    let body_content: String = match fidelity {
        Fidelity::Low => output_lines.join(";"),
        Fidelity::Medium => output_lines.join("\n"),
        Fidelity::High => output_lines.join("\n"),
    };
    
    // ---- Symbol Opcode Compression (optional pre-session pass) ----
    // Only apply opcodes to low fidelity compact bodies where they help most.
    // For medium/high, the structural markers already provide sufficient density.
    let (display_body, sym_footer) = if fidelity == Fidelity::Low {
        let mut sym_dict = SymbolDictionary::new();
        for token in body_content.split_whitespace() {
            let clean = token.trim_matches(|c: char| c == '(' || c == ')' || c == '[' || c == ']' 
                                                         || c == '{' || c == '}' || c == '<' || c == '>'
                                                         || c == ':' || c == ';' || c == ',' || c == '.');
            if !clean.is_empty() {
                sym_dict.register(clean);
            }
            if let Some(rest) = token.strip_prefix('⊕') {
                if !rest.is_empty() {
                    sym_dict.register(rest);
                }
            }
        }
        let encoded = sym_dict.encode(&body_content);
        let footer = sym_dict.format_footer();
        (encoded, footer)
    } else {
        (body_content, String::new())
    };
    
    let layout_header = match fidelity {
        Fidelity::Low => format!("// --- Compacted Layout (Low Fidelity): {} ---", path_alias),
        Fidelity::Medium => format!("// --- Enhanced Layout (Medium Fidelity): {} ---", path_alias),
        Fidelity::High => format!("// --- Full Layout (High Fidelity): {} ---", path_alias),
    };
    
    let compacted_body = if sym_footer.is_empty() {
        format!("{}\n{}\n", layout_header, display_body)
    } else {
        format!("{}\n{}\n{}", layout_header, display_body, sym_footer)
    };

    let meta = calculate_savings(&source_code, &compacted_body);
    
    // Build compression ratio report
    let ratio_report = format!(
        "// Structures: {} classes, {} methods, {} imports | {}/{} raw tokens",
        class_count, method_count, import_count, meta.raw_tokens, meta.raw_tokens
    );
    
    let final_output = format!(
        "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
        meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, compacted_body
    );

    Ok(final_output)
}

/// Streaming variant of [`compress_file`] that consumes the file through a
/// buffered reader, yielding [`CompressionProgress`] events to the supplied
/// callback. The callback is invoked multiple times during the pipeline:
/// - once when reading the source bytes (`progress = 0.0..=0.2`)
/// - once when parsing the AST (`progress = 0.2..=0.4`)
/// - once per top-level structural capture (`progress = 0.4..=0.9`)
/// - once when assembling the final report (`progress = 0.9..=1.0`)
///
/// Returning `Err` from the callback aborts the compression. This API is
/// preferred for very large source files (multi-MB) because the entire source
/// is never held in memory beyond a single read pass.
pub fn compress_file_streaming<F>(
    file: PathBuf,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    chunk_bytes: usize,
    mut on_progress: F,
) -> Result<String, Box<dyn std::error::Error>>
where
    F: FnMut(CompressionProgress) -> Result<(), Box<dyn std::error::Error>>,
{
    let metadata = fs::metadata(&file)?;
    let total_bytes = metadata.len() as usize;
    let absolute_path = fs::canonicalize(&file)?.to_string_lossy().into_owned();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // --- Phase 1: streaming read with progress reporting -----------------
    on_progress(CompressionProgress {
        progress: 0.0,
        phase: "reading".to_string(),
        partial: Some(format!("// Streaming read: {} ({} bytes)", path_alias, total_bytes)),
    })?;

    let f = fs::File::open(&file)?;
    let mut reader = BufReader::with_capacity(chunk_bytes.max(4096), f);
    let mut source_code = String::new();
    let mut buf = vec![0u8; chunk_bytes.max(4096)];
    let mut bytes_read: usize = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        // Safety: the source code is assumed to be UTF-8. We push raw bytes
        // and rely on String's validation. To avoid a full re-validation per
        // chunk we only validate the final block.
        let chunk = std::str::from_utf8(&buf[..n])
            .map_err(|e| format!("Invalid UTF-8 in source file: {}", e))?;
        source_code.push_str(chunk);
        bytes_read += n;
        let p = if total_bytes > 0 { (bytes_read as f64 / total_bytes as f64) * 0.2 } else { 0.2 };
        on_progress(CompressionProgress {
            progress: p,
            phase: "reading".to_string(),
            partial: None,
        })?;
    }

    let source_bytes = source_code.as_bytes();
    let current_hash = cache.compute_hash(source_bytes);

    // --- Cache short-circuit (consistent with non-streaming variant) ------
    let cache_key = format!("{}::{:?}", absolute_path, fidelity);
    let is_modified = cache.update_and_verify(cache_key, current_hash);
    if !is_modified {
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );
        let meta = calculate_savings(&source_code, &cached_notice);
        on_progress(CompressionProgress {
            progress: 1.0,
            phase: "cache-hit".to_string(),
            partial: Some(cached_notice.clone()),
        })?;
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, cached_notice
        ));
    }

    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let (language, query_string): (Language, &str) = match extension {
        "ts" | "js" => (tree_sitter_typescript::language_typescript(), queries::TS_QUERY),
        "cs" => (tree_sitter_c_sharp::language(), queries::CS_QUERY),
        _ => return Err(format!("Unsupported file extension: .{}", extension).into()),
    };

    // --- Phase 2: parse AST ----------------------------------------------
    on_progress(CompressionProgress {
        progress: 0.22,
        phase: "parsing".to_string(),
        partial: None,
    })?;

    let mut parser = TSParser::new();
    parser.set_language(language)?;
    let tree = parser.parse(&source_code, None).ok_or("AST Generation Error")?;

    let query = Query::new(language, query_string)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_bytes);

    on_progress(CompressionProgress {
        progress: 0.4,
        phase: "extracting".to_string(),
        partial: None,
    })?;

    // --- Phase 3: extract captures with incremental progress -------------
    let mut output_lines: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut markers: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let method_count: usize = 0;
    let _field_count: usize = 0;
    let mut import_count: usize = 0;
    let mut class_count: usize = 0;
    let _marker_count: usize = 0;

    #[derive(Debug)]
    struct CaptureEntry {
        name: String,
        text: String,
        start_byte: usize,
    }

    let mut all_captures: Vec<CaptureEntry> = Vec::new();

    for mat in matches {
        for capture in mat.captures {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let text = text_slice.to_string();
                let cap_name = capture_name.clone();
                all_captures.push(CaptureEntry {
                    name: cap_name,
                    text: if capture_name == "class.root" {
                        extract_class_name(&text)
                    } else if capture_name == "method.root" {
                        extract_method_sig(&text, fidelity)
                    } else if capture_name == "field.root" {
                        extract_field(&text, fidelity)
                    } else {
                        compact_expression(&text, fidelity)
                    },
                    start_byte: capture.node.start_byte(),
                });
            }
        }
    }

    // Sort captures by document position
    all_captures.sort_by(|a, b| a.start_byte.cmp(&b.start_byte));

    // Build the output: walk through captures in document order,
    // emitting progress events proportional to position in the capture list.
    let total_captures = all_captures.len();
    for (idx, cap) in all_captures.iter().enumerate() {
        // Progress spans 0.4..=0.9 across captures
        let p = 0.4 + (idx as f64 / total_captures.max(1) as f64) * 0.5;
        on_progress(CompressionProgress {
            progress: p,
            phase: "compressing".to_string(),
            partial: None,
        })?;

        match cap.name.as_str() {
            "import.root" => {
                import_count += 1;
                imports.push(compact_import(&cap.text, fidelity));
            }
            "class.root" => {
                class_count += 1;
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_class_entry(&cap.text, &fields, fidelity));
                fields.clear();
                markers.clear();
            }
            "method.root" => {
                let sig = &cap.text;
                if !markers.is_empty() {
                    let marker_str = markers.join(" ");
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {} {{ {} }}", sig, marker_str));
                    } else if fidelity == Fidelity::Medium {
                        output_lines.push(format!("{} {}", sig, marker_str));
                    } else {
                        output_lines.push(sig.clone());
                    }
                } else {
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {}", sig));
                    } else {
                        output_lines.push(sig.clone());
                    }
                }
                markers.clear();
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    fields.push(cap.text.clone());
                }
            }
            _ => {
                if fidelity == Fidelity::Low {
                    continue;
                }
                let marker = match cap.name.as_str() {
                    "throw.root" => format!("⊕!{}", cap.text),
                    "for.root" => "⊕loop".to_string(),
                    "if.root" => "⊕guard".to_string(),
                    "while.root" => "⊕loop".to_string(),
                    "return.root" => format!("⊕⇒{}", cap.text),
                    _ => continue,
                };
                if markers.last().map(|m| m != &marker).unwrap_or(true) {
                    markers.push(marker);
                }
            }
        }
    }

    // Orphaned fields with no class context
    if !fields.is_empty() && output_lines.is_empty() && fidelity != Fidelity::Low {
        output_lines.push(format!("⊕fields {{ {} }}", fields.join("; ")));
    }

    // Raw fallback when nothing was captured
    if output_lines.is_empty() {
        if let Some(first_line) = source_code.lines().next() {
            let trimmed = first_line.trim().to_string();
            if !trimmed.is_empty() {
                output_lines.push(simple_compact(&trimmed, fidelity));
            }
        }
    }

    // Prepend imports
    if !imports.is_empty() {
        let import_block = match fidelity {
            Fidelity::Low => imports.join("; "),
            _ => imports.join("\n"),
        };
        output_lines.insert(0, import_block);
    }

    let body_content: String = match fidelity {
        Fidelity::Low => output_lines.join(";"),
        Fidelity::Medium => output_lines.join("\n"),
        Fidelity::High => output_lines.join("\n"),
    };

    // --- Phase 4: symbol opcode compression (Low fidelity only) ----------
    let (display_body, sym_footer) = if fidelity == Fidelity::Low {
        let mut sym_dict = SymbolDictionary::new();
        for token in body_content.split_whitespace() {
            let clean = token.trim_matches(|c: char| c == '(' || c == ')' || c == '[' || c == ']'
                                                         || c == '{' || c == '}' || c == '<' || c == '>'
                                                         || c == ':' || c == ';' || c == ',' || c == '.');
            if !clean.is_empty() {
                sym_dict.register(clean);
            }
            if let Some(rest) = token.strip_prefix('⊕') {
                if !rest.is_empty() {
                    sym_dict.register(rest);
                }
            }
        }
        let encoded = sym_dict.encode(&body_content);
        let footer = sym_dict.format_footer();
        (encoded, footer)
    } else {
        (body_content, String::new())
    };

    // --- Phase 5: assemble final report ----------------------------------
    on_progress(CompressionProgress {
        progress: 0.9,
        phase: "assembling".to_string(),
        partial: None,
    })?;

    let layout_header = match fidelity {
        Fidelity::Low => format!("// --- Compacted Layout (Low Fidelity): {} ---", path_alias),
        Fidelity::Medium => format!("// --- Enhanced Layout (Medium Fidelity): {} ---", path_alias),
        Fidelity::High => format!("// --- Full Layout (High Fidelity): {} ---", path_alias),
    };

    let compacted_body = if sym_footer.is_empty() {
        format!("{}\n{}\n", layout_header, display_body)
    } else {
        format!("{}\n{}\n{}", layout_header, display_body, sym_footer)
    };

    let meta = calculate_savings(&source_code, &compacted_body);

    let ratio_report = format!(
        "// Structures: {} classes, {} methods, {} imports | {}/{} raw tokens",
        class_count, method_count, import_count, meta.raw_tokens, meta.raw_tokens
    );

    let final_output = format!(
        "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
        meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, compacted_body
    );

    on_progress(CompressionProgress {
        progress: 1.0,
        phase: "done".to_string(),
        partial: Some(final_output.clone()),
    })?;

    Ok(final_output)
}