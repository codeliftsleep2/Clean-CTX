# Plan: Adding Rust Language Support to Clean-CTX

## Executive Summary

This document outlines the implementation plan for adding Rust language support to Clean-CTX. Rust is a natural choice for the first new language support because:

1. **Dogfooding**: Clean-CTX itself is written in Rust - we can immediately test on our own codebase
2. **tree-sitter-rust availability**: Version `0.20.4` is compatible with our pinned tree-sitter 0.20.10 ABI
3. **Structural patterns**: Rust's `impl` blocks, `trait`s, `struct`s, `enum`s map well to our existing capture pipeline
4. **Baseline metrics**: We can get immediate compression statistics on our own source code

## Current Architecture Analysis

### Language Support Pipeline

```
File Extension
      ↓
language_for_extension()
      ↓
(Language, Query) pair
      ↓
run_capture_pipeline()
      ↓
Vec<CapEntry> (captures sorted by position)
      ↓
build_output_lines() OR IRCompiler
      ↓
Compressed Output
```

### Key Integration Points

| Component | File | Purpose |
|-----------|------|---------|
| Language detection | `src/compression/language.rs` | Maps extension → (Language, Query) |
| Tree-sitter queries | `src/queries.rs` | Query strings per language |
| Modifier stripping | `src/compaction/modifiers.rs` | Keyword lists for compaction |
| Class extraction | `src/compaction/class.rs` | Class/struct name extraction |
| Method extraction | `src/compaction/method.rs` | Method signature compaction |
| Import extraction | `src/compaction/import.rs` | Use/import statement handling |
| IR Language Layer | `src/ir/layers/` | Language-specific IR emission |
| Diff builder | `src/diff/builder.rs` | Snapshot construction |

## Rust-Specific Considerations

### Rust AST Node Types (tree-sitter-rust 0.20.4)

Key node types we need to capture:

| Rust Construct | tree-sitter Node Type | Capture Name |
|----------------|----------------------|--------------|
| `struct Foo { }` | `struct_item` | `struct.root` |
| `enum Bar { }` | `enum_item` | `enum.root` |
| `trait Baz { }` | `trait_item` | `trait.root` |
| `impl Foo { }` | `impl_item` | `impl.root` |
| `fn method()` | `function_item` | `method.root` |
| `type Alias = ...` | `type_item` | `type.root` |
| `use crate::...` | `use_declaration` | `import.root` |
| `field: Type` | `field_declaration` | `field.root` |
| `return expr` | `return_expression` | `return.root` |
| `if expr { }` | `if_expression` | `if.root` |
| `for i in ...` | `for_expression` | `for.root` |
| `while cond { }` | `while_expression` | `while.root` |
| `match expr { }` | `match_expression` | `match.root` |
| `macro_name!()` | `macro_invocation` | `macro.root` |

### Rust-Specific Challenges

1. **No classes**: Rust uses `struct` + `impl` blocks instead of classes
2. **Traits**: Rust's trait system has no direct equivalent in TS/C#
3. **Ownership annotations**: `&self`, `&mut self`, `self` in method signatures
4. **Generic bounds**: `where T: Trait + Clone` syntax
5. **Module system**: `mod` declarations, `pub` visibility
6. **Macros**: `macro_rules!` and procedural macros
7. **Lifetimes**: `'a`, `'static` annotations

### Rust Modifier Keywords

```rust
// Modifiers to strip at Low fidelity
MODIFIERS_LOW_RS = [
    "pub ", "pub(crate) ", "pub(super) ",
    "async ", "const ", "unsafe ",
    "extern ", "dyn ", "move ",
    "mut ", "ref ",
]

// Modifiers to keep at Medium fidelity (semantic meaning)
MODIFIERS_MEDIUM_RS = [
    "pub ", "pub(crate) ", "pub(super) ",
    "unsafe ",  // Critical safety annotation
]
```

## Implementation Plan

### Phase 1: Core Integration (MVP)

**Goal**: Basic Rust file compression working end-to-end

#### Step 1.1: Add tree-sitter-rust Dependency
**File**: `Cargo.toml`
```toml
# SAFETY: Must match tree-sitter 0.20.x ABI.
tree-sitter-rust = "=0.20.4"
```

#### Step 1.2: Define Rust Tree-sitter Queries
**File**: `src/queries.rs`

Add `RS_QUERY` constant:
```rust
pub const RS_QUERY: &str = r#"
    ; Core structural captures
    (struct_item) @struct.root
    (enum_item) @enum.root
    (trait_item) @trait.root
    (impl_item) @impl.root
    (function_item) @method.root
    (type_item) @type.root
    (field_declaration) @field.root
    (use_declaration) @import.root
    
    ; Control flow captures
    (return_expression) @return.root
    (if_expression) @if.root
    (for_expression) @for.root
    (while_expression) @while.root
    (match_expression) @match.root
    
    ; Macro captures
    (macro_invocation) @macro.root
"#;
```

#### Step 1.3: Update Language Detection
**File**: `src/compression/language.rs`

```rust
/// Returns `true` if the source text looks like Rust.
pub fn looks_like_rust(source: &str) -> bool {
    source.contains("fn ")
        || source.contains("struct ")
        || source.contains("impl ")
        || source.contains("trait ")
        || source.contains("use ")
        || source.contains("pub ")
        || source.contains("mod ")
}

pub fn detect_language(source: &str) -> (Language, &'static str) {
    if looks_like_csharp(source) {
        (tree_sitter_c_sharp::language(), queries::CS_QUERY)
    } else if looks_like_rust(source) {
        (tree_sitter_rust::language(), queries::RS_QUERY)
    } else {
        (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
    }
}

pub fn language_for_extension(extension: &str) -> Option<(Language, &'static str)> {
    match extension {
        "ts" => Some((tree_sitter_typescript::language_typescript(), queries::TS_QUERY)),
        "cs" => Some((tree_sitter_c_sharp::language(), queries::CS_QUERY)),
        "rs" => Some((tree_sitter_rust::language(), queries::RS_QUERY)),
        _ => None,
    }
}
```

#### Step 1.4: Add Rust Modifier Lists
**File**: `src/compaction/modifiers.rs`

```rust
/// Modifiers stripped at Low fidelity for Rust.
pub(crate) const MODIFIERS_LOW_RS: &[&str] = &[
    "pub ", "pub(crate) ", "pub(super) ",
    "async ", "const ", "unsafe ",
    "extern ", "dyn ", "move ",
    "mut ", "ref ",
];

/// Modifiers stripped at Medium fidelity for Rust.
pub(crate) const MODIFIERS_MEDIUM_RS: &[&str] = &[
    "pub ", "pub(crate) ", "pub(super) ",
];

/// Modifiers stripped from Rust struct/trait/enum declarations.
pub(crate) const MODIFIERS_STRUCT_RS: &[&str] = &[
    "pub ", "pub(crate) ", "pub(super) ",
    "abstract ", "final ",
];
```

#### Step 1.5: Update Compaction for Rust Structs
**File**: `src/compaction/class.rs`

The `extract_class_name` function needs to handle Rust structs, enums, and traits:

```rust
/// Extract name from Rust struct/enum/trait declaration.
/// Input:  "pub struct MyStruct<T> where T: Clone { ... }"
/// Output: "MyStruct" (Low), "MyStruct:T:Clone" (Medium)
pub fn extract_rust_struct_name(text: &str) -> String {
    let decl = text.lines().next().unwrap_or(text);
    let decl = decl.split('{').next().unwrap_or(decl).trim();
    
    // Strip modifiers
    let rest = strip_modifiers(decl, MODIFIERS_STRUCT_RS);
    
    // Strip struct/enum/trait keyword
    let rest = rest.strip_prefix("struct ")
        .or_else(|| rest.strip_prefix("enum "))
        .or_else(|| rest.strip_prefix("trait "))
        .unwrap_or(&rest)
        .trim();
    
    // Extract name (up to `<` or whitespace)
    let name = rest.split(['<', ' ']).next().unwrap_or(rest);
    name.to_string()
}
```

#### Step 1.6: Update Method Signature Extraction
**File**: `src/compaction/method.rs`

Handle Rust function signatures with ownership annotations:

```rust
/// Extract compact Rust method signature.
/// Input:  "pub async fn get_user(&self, id: u32) -> Result<User, Error>"
/// Low:    "get_user(id)"
/// Medium: "async get_user(&self,id):Result<User,Error>"
pub fn extract_rust_method_sig(text: &str, fidelity: Fidelity) -> String {
    let sig_line = text.lines().next().unwrap_or(text);
    let sig_line = sig_line.split('{').next().unwrap_or(sig_line).trim();
    
    match fidelity {
        Fidelity::Low => compact_rust_method_low(sig_line),
        Fidelity::Medium => compact_rust_method_medium(sig_line),
        Fidelity::High => sig_line.to_string(),
    }
}

fn compact_rust_method_low(sig: &str) -> String {
    let s = strip_modifiers(sig, MODIFIERS_LOW_RS);
    // Strip "fn " keyword
    let s = s.strip_prefix("fn ").unwrap_or(&s).trim();
    // Extract name and params
    let name = s.split(['(', '<']).next().unwrap_or(&s);
    let params = extract_rust_param_names(&s);
    format!("{}({})", name, params.join(","))
}

fn extract_rust_param_names(sig: &str) -> Vec<String> {
    let Some(open) = sig.find('(') else { return Vec::new(); };
    let close = sig.rfind(')').unwrap_or(sig.len());
    if open >= close { return Vec::new(); }
    
    let params_str = &sig[open + 1..close];
    if params_str.trim().is_empty() { return Vec::new(); }
    
    params_str
        .split(',')
        .map(|p| {
            // Handle "name: Type" and "self", "&self", "&mut self"
            let name_part = p.split(':').next().unwrap_or(p).trim();
            // Strip default values
            let name_part = name_part.split('=').next().unwrap_or(name_part).trim();
            name_part.to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}
```

#### Step 1.7: Update Import Extraction
**File**: `src/compaction/import.rs`

Handle Rust `use` statements:

```rust
/// Compact a Rust use declaration.
/// Input:  "use crate::models::{User, UserService};"
/// Low:    "User,UserService"
/// Medium: "use crate::models::{User,UserService}"
pub fn compact_rust_import(text: &str, fidelity: Fidelity) -> String {
    let line = text.lines().next().unwrap_or(text).trim();
    
    match fidelity {
        Fidelity::Low => extract_rust_import_names(line),
        Fidelity::Medium => line.replace("::{", "::{").replace(" }", "}"),
        Fidelity::High => line.to_string(),
    }
}

fn extract_rust_import_names(line: &str) -> String {
    // Handle grouped imports: use path::{A, B, C}
    if let (Some(open), Some(close)) = (line.find('{'), line.find('}')) {
        if open < close {
            return line[open + 1..close]
                .split(',')
                .map(|s| {
                    // Handle "Foo as Bar" aliases
                    s.split(" as ").last().unwrap_or(s).trim().to_string()
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(",");
        }
    }
    
    // Simple import: use path::Name
    if let Some(after_use) = line.strip_prefix("use ") {
        let path = after_use.trim_end_matches(';').trim();
        // Get the last segment
        path.split("::").last().unwrap_or(path).to_string()
    } else {
        String::new()
    }
}
```

### Phase 2: IR Layer Integration

**Goal**: Language-specific IR emission for Rust

#### Step 2.1: Create Rust Language Layer
**New File**: `src/ir/layers/rust.rs`

```rust
use super::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{CoreOp, FLAG_ASYNC, FLAG_UNSAFE, FLAG_PUB};

pub struct RustLayer;

impl RustLayer {
    pub fn new() -> Self { Self }
    
    /// Extract trait implementations from impl blocks.
    fn extract_impl_relationships(impl_head: &str) -> (Option<String>, Vec<String>) {
        let mut self_type = None;
        let mut traits = Vec::new();
        
        // Parse "impl<T> Trait for Type"
        if let Some(for_pos) = impl_head.find(" for ") {
            let trait_part = impl_head[..for_pos].trim();
            let type_part = impl_head[for_pos + 5..].trim();
            
            // Extract type (up to where or {)
            self_type = Some(type_part.split_whitespace()
                .next().unwrap_or(type_part).to_string());
            
            // Extract trait (after "impl<T>" or "impl")
            let after_impl = trait_part.strip_prefix("impl")
                .unwrap_or(trait_part).trim();
            // Skip generic parameters
            let trait_name = if let Some(gt_pos) = after_impl.find('>') {
                after_impl[gt_pos + 1..].trim()
            } else {
                after_impl
            };
            if !trait_name.is_empty() {
                traits.push(trait_name.to_string());
            }
        }
        
        (self_type, traits)
    }
    
    /// Extract visibility and safety flags.
    fn extract_class_flags(impl_head: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if impl_head.contains("unsafe") {
            flags.push(FLAG_UNSAFE.to_string());
        }
        if impl_head.contains("pub ") || impl_head.starts_with("pub ") {
            flags.push(FLAG_PUB.to_string());
        }
        flags
    }
    
    /// Extract method flags.
    fn extract_method_flags(raw_sig: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if raw_sig.contains("async") {
            flags.push(FLAG_ASYNC.to_string());
        }
        if raw_sig.contains("unsafe") {
            flags.push(FLAG_UNSAFE.to_string());
        }
        if raw_sig.contains("pub ") {
            flags.push(FLAG_PUB.to_string());
        }
        flags
    }
}

impl LanguageLayer for RustLayer {
    fn name(&self) -> &str { "rust" }
    
    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();
        
        match capture_name {
            "impl.root" => {
                let (self_type, traits) = Self::extract_impl_relationships(raw_text);
                if let Some(class_id) = &context.current_class {
                    // Emit Implements for trait implementations
                    for trait_name in &traits {
                        let trait_alias = context
                            .symbol_table
                            .alias_for(trait_name)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| trait_name.clone());
                        ops.push(CoreOp::Implements(class_id.clone(), trait_alias));
                    }
                    
                    let flags = Self::extract_class_flags(raw_text);
                    if !flags.is_empty() {
                        ops.push(CoreOp::ClassFlags(class_id.clone(), flags));
                    }
                }
            }
            "method.root" => {
                let method_flags = Self::extract_method_flags(raw_text);
                if let Some(method_id) = &context.current_method {
                    if !method_flags.is_empty() {
                        ops.push(CoreOp::Flags(method_id.clone(), method_flags));
                    }
                }
            }
            _ => {}
        }
        
        ops
    }
}

impl Default for RustLayer {
    fn default() -> Self { Self::new() }
}
```

#### Step 2.2: Register Rust Layer
**File**: `src/ir/layers/mod.rs`

```rust
pub mod rust;
```

**File**: Wherever IRCompiler is instantiated (likely `src/mcp/tools.rs` or similar):

```rust
use crate::ir::layers::rust::RustLayer;

// When language is Rust:
compiler.add_language_layer(Box::new(RustLayer::new()));
```

### Phase 3: Testing and Validation

**Goal**: Comprehensive test coverage

#### Step 3.1: Create Rust Test Files
**New Directory**: `src/test_files/rust/`

Create sample Rust files:
- `src/test_files/rust/simple_struct.rs` - Basic struct with impl
- `src/test_files/rust/traits.rs` - Trait definitions and implementations
- `src/test_files/rust/enums.rs` - Enum definitions with match
- `src/test_files/rust/macros.rs` - Macro invocations
- `src/test_files/rust/async_fn.rs` - Async functions
- `src/test_files/rust/nested.rs` - Complex nested structures

#### Step 3.2: Write Unit Tests
**New File**: `src/tests/compaction/rust.rs`

```rust
#[cfg(test)]
mod rust_compaction_tests {
    use crate::compaction::{extract_class_name, extract_method_sig};
    use crate::compression::Fidelity;
    
    #[test]
    fn test_extract_struct_name() {
        let input = "pub struct UserService { ... }";
        assert_eq!(extract_rust_struct_name(input), "UserService");
    }
    
    #[test]
    fn test_extract_method_sig() {
        let input = "pub async fn get_user(&self, id: u32) -> Result<User, Error> { ... }";
        assert_eq!(
            extract_rust_method_sig(input, Fidelity::Low),
            "get_user(id)"
        );
    }
    
    #[test]
    fn test_extract_use_import() {
        let input = "use crate::models::{User, UserService};";
        assert_eq!(extract_rust_import_names(input), "User,UserService");
    }
}
```

**New File**: `src/tests/compression/rust.rs`

```rust
#[cfg(test)]
mod rust_compression_tests {
    use crate::compression::language::language_for_extension;
    
    #[test]
    fn test_rs_extension_detection() {
        let result = language_for_extension("rs");
        assert!(result.is_some());
    }
}
```

**New File**: `src/tests/ir/layers/rust.rs`

```rust
#[cfg(test)]
mod rust_layer_tests {
    use crate::ir::layers::rust::RustLayer;
    
    #[test]
    fn test_impl_relationships() {
        let input = "impl<T> Repository<T> for PostgresRepo";
        let (self_type, traits) = RustLayer::extract_impl_relationships(input);
        assert_eq!(self_type, Some("PostgresRepo".to_string()));
        assert_eq!(traits, vec!["Repository".to_string()]);
    }
}
```

### Phase 4: Baseline Metrics

**Goal**: Get compression statistics on Clean-CTX's own codebase

#### Step 4.1: Run Compression on Self
After implementation, run:

```bash
# Compress individual source files
cargo run -- --file src/compression/pipeline.rs --fidelity low
cargo run -- --file src/ir/compiler.rs --fidelity medium

# Compress entire workspace
cargo run -- --workspace . --fidelity low
```

#### Step 4.2: Document Results
Create `docs/RUST_BASELINE_METRICS.md` with:

| File | Raw Tokens | Compressed (Low) | Compressed (Medium) | Savings % |
|------|-----------|------------------|---------------------|-----------|
| src/compression/pipeline.rs | ? | ? | ? | ?% |
| src/ir/compiler.rs | ? | ? | ? | ?% |
| src/mcp/tools.rs | ? | ? | ? | ?% |
| **Overall** | ? | ? | ? | ?% |

## Estimated Effort

| Phase | Effort | Description |
|-------|--------|-------------|
| Phase 1 | 2-3 hours | Core integration with tree-sitter |
| Phase 2 | 1-2 hours | IR layer implementation |
| Phase 3 | 1-2 hours | Test coverage |
| Phase 4 | 30 minutes | Baseline metrics |
| **Total** | **5-8 hours** | Full Rust support |

## Success Criteria

1. ✅ `.rs` files are recognized and compressed
2. ✅ Structs, enums, traits are extracted correctly
3. ✅ Method signatures are compacted at all fidelity levels
4. ✅ `use` statements are handled properly
5. ✅ IR layer emits correct Rust-specific operations
6. ✅ All existing tests continue to pass
7. ✅ New Rust-specific tests pass
8. ✅ Compression stats documented for own codebase

## Future Enhancements (Post-MVP)

1. **Lifetime compression**: Compress `'a`, `'static` annotations
2. **Generic bounds compression**: Compress `where T: Trait + Clone`
3. **Macro expansion**: Optionally expand common macros
4. **Cargo.toml integration**: Detect Rust projects via Cargo.toml
5. **Workspace-level Rust**: Handle multi-crate workspaces

## Resolved Design Decisions

### 1. `mod` Declarations — Two Behaviors

**External module declarations** (`mod foo;`) are structural elements that define the module graph. They map to `use` in TS/C# and should be captured at all fidelity levels.

**Inline module blocks** (`mod tests { ... }`) are fidelity-gated:
- **Low/Medium**: Drop test modules entirely (noise for LLM context)
- **High**: Keep module signature, compress body (like a class body)

```
mod user_service;        → @import.root (structural, always kept)
mod tests { ... }        → dropped at Low/Medium, compressed at High
```

### 2. Procedural Macros — Φ Marker Pattern

`#[derive(...)]` maps to the existing Angular meta-layer `Φ` marker pattern — it's semantic metadata about a type.

**Compression behavior:**
| Fidelity | `#[derive(Debug, Clone)]` | `#[tokio::main]` | `#[cfg(...)]` |
|----------|--------------------------|-------------------|---------------|
| Low | Dropped | Dropped | Dropped |
| Medium | `⊕derive:Debug,Clone` | `⊕tokio:main` | `⊕cfg:...` |
| High | Full expansion | Full expansion | Full expansion |

**IR layer**: Derive list is metadata on the struct node — captured as a field on the IR struct entry, not expanded inline. Special cases: `#[test]` drops with test module, `#[cfg(...)]` preserved as marker at Medium+.

### 3. `unsafe` Blocks — Semantic Signal, First-Class Flag

`unsafe` is a semantic signal, not just a modifier. It maps to the existing `⊕` marker system (same as `⊕guard` or `⊕loop`).

**Compression behavior:**
- `unsafe fn foo()` → `⊕unsafe` marker on method entry
- `unsafe { ... }` block → `⊕unsafe` inline marker at Medium, dropped at Low
- `unsafe impl Trait for Type` → `⊕unsafe` on impl entry

**IR layer**: Boolean flag on Rust method/impl entries:

```rust
pub struct RustMethodEntry {
    pub is_unsafe: bool,
    pub is_async: bool,
    pub visibility: RustVisibility,
    pub self_kind: SelfKind, // None | Ref | RefMut | Owned
}
```

**Dashboard metric**: The IR captures `unsafe` as a first-class flag, enabling `"this workspace contains N unsafe blocks"` in the stats dashboard — genuinely useful for LLM context.

### 4. Complex Generic Type Parameters — Depth-Based Truncation

Three approaches by fidelity:

| Fidelity | Behavior | Example |
|----------|----------|---------|
| Low | Drop all generic params entirely | `fn process<T: Iterator<Item=Result<Row,Error>>>` → `fn process` |
| Medium | Keep top-level, collapse at depth 2, emit `…` for deeper | `HashMap<String, Vec<Arc<Mutex<...>>>>` → `HashMap<String, Vec<…>>`. Bounds: `T:3bounds` |
| High | Full expansion, string table compresses repeated types | `Arc`, `Mutex`, `Vec` get short codes via Huffman |

**Configurable**: `rust.generic_depth` in `.clean-ctx.json` (default: 2).

### 5. Trait Impls vs Inherent Impls — Structural Distinction

`impl Foo { }` (inherent) and `impl Bar for Foo { }` (trait) are fundamentally different.

- **`impl Trait for Type`**: Structural element with its own IR node type. Captured at all fidelity levels with trait name preserved. Enables Phase 3 blast radius — if `UserService` implements `Repository`, changes to `Repository` trait have blast radius to `UserService`.
- **`impl Type`** (inherent): Follows normal class body compression rules.

---

## Finalized IR Data Structures

```rust
// Rust visibility enum
pub enum RustVisibility {
    Public,       // pub
    Crate,        // pub(crate)
    Super,        // pub(super)
    Private,      // (default)
}

// Self kind for methods
pub enum SelfKind {
    None,         // associated function (no self)
    Ref,          // &self
    RefMut,       // &mut self
    Owned,        // self (by value)
}

// Struct/Enum/Trait entry
pub struct RustTypeEntry {
    pub kind: RustTypeKind,  // Struct | Enum | Trait
    pub visibility: RustVisibility,
    pub derives: Vec<String>,
    pub generic_params: Option<String>,  // compressed at medium fidelity
    pub is_unsafe: bool,  // for unsafe trait
}

// Method entry
pub struct RustMethodEntry {
    pub is_unsafe: bool,
    pub is_async: bool,
    pub visibility: RustVisibility,
    pub self_kind: SelfKind,
    pub generic_bounds: Option<String>,  // compressed at medium fidelity
}

// Impl block entry
pub struct RustImplEntry {
    pub trait_name: Option<String>,  // None for inherent impl
    pub self_type: String,
    pub is_unsafe: bool,
    pub generic_params: Option<String>,
}
```

---

*Document created: 2026-06-10*
*Updated: 2026-06-10 with design decisions*
*Status: Ready for Implementation*
