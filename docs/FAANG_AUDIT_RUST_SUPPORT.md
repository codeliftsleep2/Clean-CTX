# FAANG Audit: Rust Language Support Implementation

**Audit Date:** 2026-06-10
**Scope:** Complete Rust language support added to Clean-CTX
**Auditor:** Cline (automated)
**Severity Scale:** CRITICAL → HIGH → MODERATE → LOW → INFO

---

## Executive Summary

The Rust language support implementation has **4 CRITICAL gaps**, **2 HIGH issues**, and several MODERATE/LOW findings. The CRITICAL issues mean that **Rust files will produce incomplete or incorrect IR output**, and the **diff/delta pipeline will silently produce wrong results** for Rust files. The compression pipeline (text output) is mostly functional but has one gap in the text-delta handler.

---

## CRITICAL Findings

### C-01: IR Compiler Does Not Emit `DefClass` for Rust Type Captures
**File:** `src/ir/compiler.rs` (line 171-326)
**Impact:** Rust structs/enums/traits produce NO class definitions in IR; all Rust methods are silently dropped.

The IR compiler's `compile` method has a `match cap.name.as_str()` that only handles:
- `"class.root"` → emits `DefClass`, sets `current_class`
- `"method.root"` → emits `DefMethod` (requires `current_class`)
- `"field.root"` → emits `DefField` (requires `current_class`)
- `"import.root"` → emits `Import`
- Control flow captures → accumulates flags

**Rust captures (`struct.root`, `enum.root`, `trait.root`, `impl.root`) fall through to the `_` catch-all**, which only invokes language layers but does NOT:
1. Emit `DefClass` instructions
2. Set `current_class` / `layer_context.current_class`

**Consequence:** When a subsequent `method.root` capture (Rust `function_item`) is encountered, `self.current_class` is `None`, so the F-29 guard **skips the method entirely** (`continue` on line 217). **No Rust methods appear in the IR output.**

**Fix Required:** Add Rust type captures to the IR compiler's core emission loop:
```rust
"struct.root" | "enum.root" | "trait.root" => {
    let class_id = self.next_id("C");
    instructions.push(CoreOp::DefClass(class_id.clone(), cap.text.clone()));
    self.current_class = Some(class_id.clone());
    layer_context.current_class = Some(class_id.clone());
    layer_context.current_class_name = Some(cap.raw_text.clone());
    layer_context.current_class_bare_name = Some(cap.text.clone());
    // Register in symbol table
    layer_context.symbol_table_mut().register(
        class_id.clone(), cap.text.clone(), SymbolKind::Class, file_id,
    );
    // Invoke language layers
    for ll in self.language_layers.iter_mut() {
        let layer_ops = ll.process_capture(&cap.name, &cap.raw_text, &mut layer_context);
        instructions.extend(layer_ops);
    }
}
"impl.root" => {
    // Impl blocks: invoke language layers (trait impl extraction)
    // but don't create a new class context
    for ll in self.language_layers.iter_mut() {
        let layer_ops = ll.process_capture(&cap.name, &cap.raw_text, &mut layer_context);
        instructions.extend(layer_ops);
    }
}
```

---

### C-02: `compile_file_ir` Missing Rust Layer Registration
**File:** `src/mcp/tools.rs` (line 1541-1549)
**Impact:** `RustLayer` is never instantiated; Rust-specific IR ops (UNSAFE, trait impls) are never emitted.

The `compile_file_ir` function has:
```rust
match extension {
    "ts" | "js" => {
        compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    }
    "cs" => {
        compiler.add_language_layer(Box::new(CSharpLayer::new()));
    }
    _ => {}  // <-- Rust falls through here!
}
```

**Fix Required:** Add Rust arm:
```rust
"rs" => {
    compiler.add_language_layer(Box::new(
        crate::ir::layers::rust::RustLayer::new()
    ));
}
```

---

### C-03: `compress_text_body` Missing Rust Capture Handling
**File:** `src/mcp/tools.rs` (line 855-866)
**Impact:** Text-delta handler produces incorrect output for Rust files.

The closure in `compress_text_body` only handles:
```rust
if capture_name == "class.root" {
    Some(extract_class_name(raw))
} else if capture_name == "method.root" {
    Some(extract_method_sig(raw, f))
} ...
```

Rust captures (`struct.root`, `enum.root`, `trait.root`, `impl.root`, `mod.root`, `type.root`) are NOT handled. They fall through to `compact_expression(raw, f)`, which may produce suboptimal output.

**Fix Required:** Add Rust capture handling identical to `pipeline.rs`:
```rust
"struct.root" | "enum.root" | "trait.root" | "impl.root" => {
    Some(extract_rust_struct_name(raw))
}
"mod.root" => Some(compact_import(raw, f)),
"type.root" => Some(compact_expression(raw, f)),
```

---

### C-04: `diff/builder.rs` Missing Rust Capture Handling
**File:** `src/diff/builder.rs` (line 62-75)
**Impact:** AST diff system silently produces wrong results for Rust files.

Same issue as C-03. The closure in `try_build_with` only handles `"class.root"`, `"method.root"`, `"field.root"`, `"import.root"`. Rust captures fall through to `compact_expression`.

**Additionally**, the fallback logic (line 37-44) only tries TS ↔ C#:
```rust
let (other_lang, other_query) = if query_string == queries::TS_QUERY {
    (tree_sitter_c_sharp::language(), queries::CS_QUERY)
} else {
    (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
};
```

If Rust is detected but yields no captures (edge case), it falls back to TypeScript instead of trying Rust.

**Fix Required:**
1. Add Rust capture handling to the closure
2. Update fallback logic to include Rust ↔ TS ↔ C# three-way fallback

---

## HIGH Findings

### H-01: `extract_rust_struct_name` Doesn't Handle `impl` Blocks
**File:** `src/compaction/class.rs` (line 135-158)
**Impact:** `impl` blocks produce `"impl"` as the class name in text output.

The function strips modifiers then tries `struct `, `enum `, `trait ` prefixes. For `impl MyStruct { ... }`:
1. After stripping modifiers: `rest = "impl MyStruct"`
2. `strip_prefix("struct ")` → None
3. `strip_prefix("enum ")` → None
4. `strip_prefix("trait ")` → None
5. Falls through to `unwrap_or(&rest)` → `"impl MyStruct"`
6. `name = rest.split(['<', ' ']).next()` → `"impl"`

**Result:** The text output shows `class impl { ... }` which is meaningless.

**Fix Required:** Add `"impl "` to the strip_prefix chain:
```rust
let rest = rest
    .strip_prefix("struct ")
    .or_else(|| rest.strip_prefix("enum "))
    .or_else(|| rest.strip_prefix("trait "))
    .or_else(|| rest.strip_prefix("impl "))
    .unwrap_or(&rest)
    .trim();
```

For impl blocks, also extract the self type and any trait name for a meaningful output like `MyStruct` or `MyStruct:Display`.

---

### H-02: `RustLayer::process_capture` Dead Code for Struct/Enum/Trait
**File:** `src/ir/layers/rust.rs` (line 242-258)
**Impact:** The `struct.root`/`enum.root`/`trait.root` branch in `process_capture` checks `context.current_class`, but since C-01 means `current_class` is never set for Rust types, this code is unreachable.

Even after fixing C-01, the branch's logic is redundant — it emits `ClassFlags` for the current class, but the IR compiler already emits `DefClass` with the class ID. The layer should instead focus on extracting derives, generic params, and visibility.

**Fix Required:** After fixing C-01, refactor `RustLayer::process_capture` to:
1. Emit `ClassFlags` with visibility + unsafe + derives
2. Store derives in `LayerContext` for meta-layer consumption
3. Remove the duplicate `unsafe trait` check (it's already in `extract_method_flags`)

---

## MODERATE Findings

### M-01: `looks_like_rust` Heuristic Too Broad
**File:** `src/compression/language.rs` (line 35-43)
**Impact:** False positives in content-based language detection (diff path).

```rust
pub fn looks_like_rust(source: &str) -> bool {
    source.contains("fn ") || source.contains("struct ")
        || source.contains("impl ") || source.contains("trait ")
        || source.contains("use ") || source.contains("pub ")
        || source.contains("mod ")
}
```

- `use ` appears in Python (`import ... as ...`), TypeScript (`import { use } from ...`)
- `mod ` appears in CSS (`@media`), Python
- `pub ` is fairly Rust-specific but could match prose/comments

**Fix Required:** Strengthen the heuristic:
```rust
pub fn looks_like_rust(source: &str) -> bool {
    // Require multiple Rust-specific signals to reduce false positives
    let has_fn = source.contains("fn ");
    let has_struct = source.contains("struct ");
    let has_impl = source.contains("impl ");
    let has_trait = source.contains("trait ");
    let has_pub = source.contains("pub ");
    let has_use = source.contains("use ");
    let has_mod = source.contains("mod ");
    
    // At least two Rust-specific signals, or one strong signal
    let strong = has_impl || has_trait;
    let signals = [has_fn, has_struct, has_impl, has_trait, has_pub, has_use, has_mod]
        .iter().filter(|&&x| x).count();
    
    strong || signals >= 2
}
```

---

### M-02: `build_output_lines` Treats `impl.root` as a Class
**File:** `src/compression/pipeline.rs` (line 283)
**Impact:** `impl` blocks produce `class impl_name {` in Medium/High fidelity output.

The match arm `"class.root" | "struct.root" | "enum.root" | "trait.root" | "impl.root"` calls `format_class_entry`, which wraps the name with `class ... {`. For Rust, this is semantically wrong — `impl` blocks are not classes.

**Fix Required:** Either:
1. Use a different format for Rust captures (e.g., `impl MyStruct {`)
2. Or at minimum, don't prepend `class` for Rust-specific captures

---

### M-03: `extract_impl_relationships` Ignores `self_type` Return Value
**File:** `src/ir/layers/rust.rs` (line 261)
**Impact:** The self type from `impl Trait for Type` is discarded.

```rust
let (_self_type, traits) = Self::extract_impl_relationships(raw_text);
```

The `self_type` is extracted but thrown away. This means the IR doesn't know which type implements the trait, only that a trait is implemented. The `Implements` op is emitted as `Implements(class_id, trait_alias)`, but `class_id` is the current class from the compiler, not the actual self type.

This is actually correct behavior for the current architecture (the current class IS the self type), but the naming is confusing and the code should document this.

---

## LOW Findings

### L-01: Tool Descriptions Not Updated
**File:** `src/mcp/tools.rs` (line 46, 66)
- `compress_code_context`: `"Absolute path to .ts or .cs file."` → should include `.rs`
- `compress_workspace`: `"Compresses all TypeScript/C# files"` → should include Rust

### L-02: No Integration Tests for Rust Compression
There are no end-to-end tests that compress a real Rust source file and verify the output. The existing tests only verify:
- Language detection
- Modifier stripping
- Name extraction
- IR layer unit functions

Missing tests:
- Full pipeline compression of a Rust file
- IR compilation of a Rust file
- Diff of a Rust file
- Delta transport of a Rust file

### L-03: `RustLayer` Doesn't Extract Generic Parameters
The `RustTypeEntry` struct has a `generic_params` field, but it's never populated during `process_capture`. Generic bounds like `<T: Clone + Debug>` are lost.

### L-04: `RustLayer` Doesn't Extract Where Clauses
Where clauses (`where T: Clone`) are not parsed or stored anywhere.

### L-05: No `#[cfg(...)]` Handling
Conditional compilation attributes are not parsed or represented in the IR.

### L-06: `RustLayer` Doesn't Handle `type.root` Captures
Type aliases (`type Result<T> = std::result::Result<T, Error>`) are not processed by the Rust layer.

---

## Regression Risk Assessment

| Finding | Regression Risk | Current Impact |
|---------|----------------|----------------|
| C-01 | **CRITICAL** | IR output for Rust files is empty (no classes, no methods) |
| C-02 | **CRITICAL** | RustLayer never runs; UNSAFE/trait flags missing |
| C-03 | **CRITICAL** | Text-delta handler produces wrong output for Rust |
| C-04 | **CRITICAL** | Diff system produces wrong output for Rust |
| H-01 | HIGH | `impl` blocks show as `class impl` in text output |
| H-02 | HIGH | Rust layer struct/enum/trait handling is dead code |
| M-01 | MODERATE | False positives in content-based detection |
| M-02 | MODERATE | `impl` blocks formatted as classes |
| M-03 | LOW | Confusing but functionally correct |
| L-01 | LOW | Misleading tool descriptions |
| L-02 | LOW | No regression safety net |
| L-03-L06 | LOW | Missing features (not regressions) |

---

## Recommended Fix Priority

1. **C-01** (IR compiler Rust captures) — Foundation fix; everything depends on this
2. **C-02** (RustLayer registration) — One-line fix; enables all Rust IR features
3. **C-03** (compress_text_body) — Enables text-delta for Rust
4. **C-04** (diff builder) — Enables diff for Rust
5. **H-01** (extract_rust_struct_name impl) — Fixes text output quality
6. **H-02** (RustLayer dead code) — Code quality; depends on C-01
7. **M-01** (heuristic) — Prevents false positives
8. **M-02** (build_output_lines) — Output quality
9. **L-01-L06** — Polish items

---

## Positive Findings

The following aspects of the implementation are well-done:

- ✅ **tree-sitter-rust dependency** correctly pinned at ABI-compatible version
- ✅ **RS_QUERY** covers all major Rust AST nodes
- ✅ **Language detection** correctly prioritizes C# > Rust > TypeScript
- ✅ **`language_for_extension`** correctly maps `.rs` to Rust grammar
- ✅ **Compaction modifiers** are well-structured with Low/Medium/Struct variants
- ✅ **`extract_impl_relationships`** handles nested generics correctly
- ✅ **`extract_self_kind`** covers all four variants
- ✅ **`extract_derives`** works correctly
- ✅ **Test coverage** for the unit functions that exist (13 IR tests, 10 detection tests)
- ✅ **All 821 tests pass** (no regressions to existing functionality)