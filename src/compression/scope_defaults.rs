// src/compression/scope_defaults.rs
//
// Phase III (Idea #7 — Structural Deduplication with Scope Defaults).
//
// Detects common method patterns within each class and emits them as
// scope defaults, reducing repetition in classes with many similar methods.
//
// ## How it works
//
// 1. The assembled body is a semicolon-separated list of tokens
// 2. Method definitions ($ctor, $e, $a, etc.) carry class_id and method_id
// 3. Return types ($r) and flags (FLAGS) carry method_id
// 4. If ≥2 methods in the same class share the same return type and/or flags,
//    those are emitted as a `$dft` line and omitted from individual methods
// 5. Methods that differ from defaults keep their explicit values
//
// ## Scope default syntax
//
//   `$dft r=$b fl=IF`      — return type default = $b, flags default = IF
//   `$dft fl=$1`            — flags default only
//   `$dft` alone            — no specific defaults (shouldn't be emitted)
//
// ## Example
//
//   Input:  `Foo;$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF;$ctor C1 M2 $s data;$r M2 $b;FLAGS M2 IF`
//   Output: `$dft r=$b fl=IF;Foo;$ctor C1 M1 $s payload;$ctor C1 M2 $s data`

use crate::compression::Fidelity;

/// Apply structural deduplication to the assembled body (Low fidelity only).
/// Higher fidelities have more verbose output where scope defaults would not
/// provide meaningful savings.
#[allow(dead_code)]
pub fn apply_scope_defaults(body: &str, fidelity: Fidelity) -> String {
    if fidelity != Fidelity::Low {
        return body.to_string();
    }
    if body.is_empty() {
        return String::new();
    }

    let tokens: Vec<&str> = body.split(';').collect();

    // Step 1: Build method_id → class_id mapping from method definitions.
    // Method def tokens start with $ctor, $e, $a, $st, $k, $nw, etc.
    // Format: "$ctor CLASS_ID METHOD_ID [params...]"
    let mut method_to_class: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for token in &tokens {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        match parts[0] {
            "$ctor" | "$e" | "$a" | "$st" | "$k" | "$nw" => {
                let class_id = parts[1].to_string();
                let method_id = parts[2].to_string();
                method_to_class.insert(method_id, class_id);
            }
            _ => {}
        }
    }

    if method_to_class.is_empty() {
        return body.to_string();
    }

    // Step 2: Collect return types and flags by class.
    let mut class_return_types: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut class_flags: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for token in &tokens {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
        match parts[0] {
            "$r" if parts.len() >= 3 => {
                let method_id = parts[1].to_string();
                if let Some(class_id) = method_to_class.get(&method_id) {
                    class_return_types
                        .entry(class_id.clone())
                        .or_default()
                        .push((method_id, parts[2].to_string()));
                }
            }
            "FLAGS" if parts.len() >= 3 => {
                let method_id = parts[1].to_string();
                if let Some(class_id) = method_to_class.get(&method_id) {
                    class_flags
                        .entry(class_id.clone())
                        .or_default()
                        .push((method_id, parts[2..].join(" ")));
                }
            }
            _ => {}
        }
    }

    // Step 3: For each class, find common defaults (≥2 methods sharing same value).
    let mut class_defaults: std::collections::HashMap<String, (Option<String>, Option<String>)> =
        std::collections::HashMap::new();

    let mut all_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
    for cid in method_to_class.values() {
        all_classes.insert(cid.clone());
    }

    for class_id in &all_classes {
        let mut default_return = None;
        let mut default_flags = None;

        if let Some(rts) = class_return_types.get(class_id) {
            if rts.len() >= 2 {
                let mut freq: std::collections::HashMap<&str, usize> =
                    std::collections::HashMap::new();
                for (_, rt) in rts {
                    *freq.entry(rt.as_str()).or_insert(0) += 1;
                }
                for (val, &count) in &freq {
                    if count >= 2 {
                        default_return = Some(val.to_string());
                        break;
                    }
                }
            }
        }

        if let Some(fls) = class_flags.get(class_id) {
            if fls.len() >= 2 {
                let mut freq: std::collections::HashMap<&str, usize> =
                    std::collections::HashMap::new();
                for (_, fl) in fls {
                    *freq.entry(fl.as_str()).or_insert(0) += 1;
                }
                for (val, &count) in &freq {
                    if count >= 2 {
                        default_flags = Some(val.to_string());
                        break;
                    }
                }
            }
        }

        if default_return.is_some() || default_flags.is_some() {
            class_defaults.insert(class_id.clone(), (default_return, default_flags));
        }
    }

    if class_defaults.is_empty() {
        return body.to_string();
    }

    // Step 4: Build output, inserting $dft and stripping defaulted tokens.
    let mut output_parts: Vec<String> = Vec::new();
    let mut class_dft_emitted: std::collections::HashSet<String> = std::collections::HashSet::new();

    for token in &tokens {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();

        match parts[0] {
            "$ctor" | "$e" | "$a" | "$st" | "$k" | "$nw" if parts.len() >= 3 => {
                let method_id = parts[2];
                if let Some(cid) = method_to_class.get(method_id) {
                    if !class_dft_emitted.contains(cid) {
                        if let Some((drt, dfl)) = class_defaults.get(cid) {
                            let mut dft_line = "$dft".to_string();
                            if let Some(rt) = drt {
                                dft_line.push_str(&format!(" r={}", rt));
                            }
                            if let Some(fl) = dfl {
                                dft_line.push_str(&format!(" fl={}", fl));
                            }
                            output_parts.push(dft_line);
                            class_dft_emitted.insert(cid.clone());
                        }
                    }
                }
                output_parts.push(trimmed.to_string());
            }
            "$r" if parts.len() >= 3 => {
                let method_id = parts[1];
                if let Some(cid) = method_to_class.get(method_id) {
                    if let Some((Some(drt), _)) = class_defaults.get(cid) {
                        if parts[2] == drt.as_str() {
                            continue; // Covered by $dft
                        }
                    }
                }
                output_parts.push(trimmed.to_string());
            }
            "FLAGS" if parts.len() >= 3 => {
                let method_id = parts[1];
                if let Some(cid) = method_to_class.get(method_id) {
                    if let Some((_, Some(dfl))) = class_defaults.get(cid) {
                        let flag_str = parts[2..].join(" ");
                        if flag_str == *dfl {
                            continue; // Covered by $dft
                        }
                    }
                }
                output_parts.push(trimmed.to_string());
            }
            _ => {
                output_parts.push(trimmed.to_string());
            }
        }
    }

    output_parts.join(";")
}

#[cfg(test)]
#[path = "../tests/compression/scope_defaults.rs"]
mod tests;
