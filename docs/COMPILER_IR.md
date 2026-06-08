# Clean-CTX — Compiler IR: Structured State Protocol

**Version:** 0.1.0 (Proposed)
**Last updated:** 2026-06-08 (Phase G marked complete)
**Status:** Phase A Complete - Phase B Complete - Phase C Complete - Phase D Complete - Phase E Complete - Phase F Complete - Phase G Complete

> **Living document.** This spec defines the evolution from text-based compression to a structured intermediate representation (IR) with delta-based state transport. It serves as the implementation guideline for the Compiler IR subsystem.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [Architecture Overview](#2-architecture-overview)
3. [The Pipeline](#3-the-pipeline)
4. [Phase A: IR Core](#4-phase-a-ir-core)
5. [Phase B: Global Symbol Table](#5-phase-b-global-symbol-table)
6. [Phase C: Delta Transport](#6-phase-c-delta-transport)
7. [Phase D: State Replay](#7-phase-d-state-replay)
8. [Phase E: IR / Pretty Separation](#8-phase-e-ir--pretty-separation)
9. [Phase F: Layered Encoding](#9-phase-f-layered-encoding)
10. [Phase G: Integration & MCP Tools](#10-phase-g-integration--mcp-tools)
11. [Phase H: Positional Encoding & Advanced Compression](#11-phase-h-positional-encoding--advanced-compression)
12. [Migration Map](#12-migration-map)
13. [Wire Protocol Reference](#13-wire-protocol-reference)
14. [Static Schema Definition](#14-static-schema-definition)
15. [Phase Dependencies & Timeline](#15-phase-dependencies--timeline)

---

## 1. Motivation

The current Clean-CTX system operates as a **one-shot text compression pipeline**:

```
Source → Tree-sitter AST → Fidelity Filter → Opcode Encode → Text Output
```

Every compression is a full re-parse. The diff system computes deltas between structural snapshots but emits them as human-readable diff lines, not machine-replayable instruction streams. The client (LLM) must re-read the full compressed output on every interaction, even for a single method change.

**The core limitation:** all state is serialized as text strings. There is no concept of incremental state application.

### What Changes

| Dimension | Current | Compiler IR |
|-----------|---------|-------------|
| Output format | Compressed text | Structured instruction stream |
| Diff output | Human-readable text diff | Machine-applicable delta ops |
| State model | Stateless (re-parse each time) | Stateful (apply deltas to state) |
| Transport | Full output per call | Full first, deltas after |
| Symbol tracking | Per-file dictionaries | Cross-stage global symbol table |
| Layer separation | Monolithic pipeline | 4-layer encoding architecture |
| Fidelity | Compile-time filter | Render-time filter (same IR) |

### Expected Token Savings

| Scenario | Current (Low) | IR Full | IR Delta |
|---|---|---|---|
| First compression (32-line file) | 26 tokens | ~20 tokens | N/A |
| First compression (438-line file) | 75 tokens | ~55 tokens | N/A |
| Subsequent edit (1 method changed) | 75 tokens (full re-compress) | 75 tokens | ~8-12 tokens |
| Subsequent edit (1 line changed) | 75 tokens | 75 tokens | ~5-8 tokens |
| 50-edit session (cumulative) | 3,750 tokens | 3,750 tokens | ~200 tokens |

---

## 2. Architecture Overview

### 4-Layer Encoding Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Application Patterns + Positional Encoding    │
│  Pattern recognition, key stripping, positional tuples  │
│                                                         │
│  ["DEF_C","C1"] ["DEF_M","C1","M1"] ["SIG","M1","P1",   │
│   "$s","payload"]                                       │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Meta-Layer (Framework-Specific)               │
│  Angular Φ markers, React patterns, NgRx patterns       │
│                                                         │
│  ["NG_COMPONENT","C1",{selector:"app-root",...}]        │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Language Layer (TS, C#, etc.)                 │
│  Language-specific ops that map to Core IR              │
│                                                         │
│  ["TS_ASYNC","M1"] ["TS_GENERICS","C1",["T"]]           │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Core IR (Language-Agnostic)                   │
│  Universal instruction set — every language compiles    │
│  down to these operations                               │
│                                                         │
│  DEF_C, DEF_M, DEF_F, SIG, RET, FLAGS, EXT, IMP, IMP    │
└─────────────────────────────────────────────────────────┘
```

### The Pipeline

```
     Source Code
          │
          ▼
┌─────────────────────┐
│   Tree-sitter AST   │  Existing capture pipeline (unchanged)
│   Parse + Extract   │
└─────────┬───────────┘
          │ CapEntry captures
          ▼
┌─────────────────────┐
│   IR Compiler       │  NEW: translates captures → Vec<CoreOp>
│   (Core + Lang +    │  Runs all 4 layers
│    Meta + Pattern)  │
└─────────┬───────────┘
          │ CompiledIR { instructions, version }
          │
     ┌────┴────────────────────┐
     ▼                         ▼
┌─────────────┐     ┌──────────────────┐
│ IR → Wire   │     │ IR → Pretty Text │
│ (delta,     │     │ (backward-compat │
│  transport) │     │  output)         │
└──────┬──────┘     └────────┬─────────┘
       │                     │
       ▼                     ▼
┌─────────────┐     ┌──────────────────┐
│ Delta       │     │ Human-Readable   │
│ Transport   │     │ Compressed       │
│ Protocol    │     │ Output           │
└──────┬──────┘     └──────────────────┘
       │
       ▼
┌─────────────────────┐
│   State Replay      │  Apply delta ops to client state
│   (Apply + Render)  │
└─────────────────────┘
```

---

## 3. The Pipeline

### 3.1 Compilation (Source → IR)

The IR compiler reuses the existing tree-sitter capture pipeline (`run_capture_pipeline`) but replaces the text-formatting orchestration (`build_output_lines`) with instruction emission:

```
1. Tree-sitter parse (existing — no change)
2. Capture walk (existing — no change)
3. Core IR emission (NEW — replaces build_output_lines)
4. Language layer translation (NEW — TS/C# specific ops)
5. Meta-layer pass (REFACTORED — angular_meta implements MetaLayer trait)
6. Pattern recognition (NEW — Layer 4 pattern compression)
7. Register in global symbol table (NEW — cross-stage tracking)
```

### 3.2 Delta Computation (IR → Delta)

Instead of computing text diffs between `CapturedStructure` snapshots, the delta engine computes instruction-level deltas between `CompiledIR` states:

```
1. Index both IRs by symbol (opcode + primary key)
2. Diff the indices:
   - Symbols in current but not baseline → additions
   - Symbols in baseline but not current → deletions
   - Symbols in both but different instructions → modifications
3. Emit DeltaOps envelope
```

### 3.3 State Replay (Delta → Updated State)

The client applies delta ops to its local state machine:

```
1. Validate version chain (from_version must match)
2. Apply deletions (remove instructions + unregister symbols)
3. Apply modifications (in-place replacement)
4. Apply additions (append + register symbols)
5. Bump version
6. Render if needed (IR → pretty text at requested fidelity)
```

---

## 4. Phase A: IR Core

### Goal

Define the IR instruction types and build the compiler that translates tree-sitter captures into structured instructions.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/mod.rs` | Module root, public API exports |
| `src/ir/opcodes.rs` | `CoreOp` enum, opcode constants, arity table |
| `src/ir/compiler.rs` | `IRCompiler` struct, `compile()` method |
| `src/ir/render.rs` | `ir_to_text()` — pretty output from IR |
| `src/ir/wire.rs` | `ir_to_wire()` — JSON serialization |
| `src/ir/mod.rs` (tests) | Unit tests for compiler + render |

### Core Instruction Types

```rust
// src/ir/opcodes.rs

/// Core IR opcodes — the universal instruction set.
/// Every language compiles down to these operations.
/// Serialized as positional JSON arrays: [opcode, ...operands]
pub enum CoreOp {
    // ── Structural Definitions ──────────────────────────
    /// ["DEF_C", class_id, original_name]
    DefClass(String, String),
    
    /// ["DEF_M", class_id, method_id, original_name]
    DefMethod(String, String, String),
    
    /// ["DEF_F", class_id, field_id, original_name]
    DefField(String, String, String),
    
    /// ["DEF_I", interface_id, original_name]
    DefInterface(String, String),

    // ── Signatures & Types ──────────────────────────────
    /// ["SIG", method_id, param_id, type_opcode, param_name]
    Param(String, String, String, String),
    
    /// ["RET", method_id, type_opcode]
    Return(String, String),
    
    /// ["FIELD_T", field_id, type_opcode]
    FieldType(String, String),

    // ── Control Flow & Behavior ─────────────────────────
    /// ["FLAGS", target_id, [flag1, flag2, ...]]
    /// Replaces ⊕guard, ⊕loop, ⊕⇒, ⊕! markers
    Flags(String, Vec<String>),
    
    /// ["FLAGS_C", class_id, [flag1, flag2, ...]]
    /// Class-level flags: EXPORT, ABSTRACT, etc.
    ClassFlags(String, Vec<String>),

    // ── Relationships ───────────────────────────────────
    /// ["EXT", child_id, parent_id]
    Extends(String, String),
    
    /// ["IMPL", class_id, interface_id]
    Implements(String, String),
    
    /// ["INJECTS", class_id, [dep1, dep2, ...]]
    Injects(String, Vec<String>),

    // ── Imports ─────────────────────────────────────────
    /// ["IMP", alias, module, named_export]
    Import(String, String, String),

    // ── Type Aliases (runtime-assigned) ─────────────────
    /// ["TYPE", alias, original_type]
    TypeAlias(String, String),
}

/// Core flag constants
pub const FLAG_IF: &str = "IF";
pub const FLAG_LOOP: &str = "LOOP";
pub const FLAG_RET: &str = "RET";
pub const FLAG_THROW: &str = "THROW";
pub const FLAG_ASYNC: &str = "ASYNC";
pub const FLAG_GEN: &str = "GEN";
pub const FLAG_EXPORT: &str = "EXPORT";
pub const FLAG_STATIC: &str = "STATIC";
pub const FLAG_PRIVATE: &str = "PRIVATE";
pub const FLAG_PROTECTED: &str = "PROTECTED";
pub const FLAG_ABSTRACT: &str = "ABSTRACT";

/// Built-in type opcodes (subset of existing PRIMITIVE_OPCODES)
pub const TYPE_STRING: &str = "$s";
pub const TYPE_NUMBER: &str = "$n";
pub const TYPE_BOOLEAN: &str = "$b";
pub const TYPE_VOID: &str = "$v";
pub const TYPE_TRUE: &str = "$T";
pub const TYPE_FALSE: &str = "$F";
pub const TYPE_NULL: &str = "$nl";
pub const TYPE_UNDEFINED: &str = "$ud";
```

### IR Compiler

```rust
// src/ir/compiler.rs

use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::Fidelity;
use super::opcodes::CoreOp;

/// The compiled IR for a single file.
#[derive(Debug, Clone)]
pub struct CompiledIR {
    /// File identifier (path alias)
    pub file_id: String,
    /// Ordered instruction stream
    pub instructions: Vec<CoreOp>,
    /// Monotonic version number
    pub version: u64,
}

/// IR Compiler — translates tree-sitter captures into Core IR instructions.
pub struct IRCompiler {
    /// Running instruction counter for ID generation
    id_counter: u32,
}

impl IRCompiler {
    pub fn new() -> Self {
        Self { id_counter: 0 }
    }
    
    /// Compile source code into IR.
    /// Reuses the existing capture pipeline but emits CoreOp instructions
    /// instead of formatted text strings.
    pub fn compile(
        &mut self,
        source: &str,
        file_id: &str,
        language: tree_sitter::Language,
        query_string: &str,
        fidelity: Fidelity,
    ) -> Result<CompiledIR, Box<dyn std::error::Error>> {
        let captures = run_capture_pipeline(
            language,
            query_string,
            source,
            fidelity,
            |capture_name, raw, f| {
                // Same capture processing as existing pipeline
                match capture_name {
                    "class.root" => Some(raw.to_string()),
                    "method.root" => Some(raw.to_string()),
                    "field.root" => Some(raw.to_string()),
                    _ => Some(raw.to_string()),
                }
            },
        )?;
        
        let mut instructions = Vec::new();
        
        for cap in &captures {
            match cap.name.as_str() {
                "class.root" => {
                    let class_id = self.next_id("C");
                    instructions.push(CoreOp::DefClass(
                        class_id.clone(),
                        cap.text.clone(),
                    ));
                }
                "method.root" => {
                    // Parse method signature to extract name, params, return
                    let method_id = self.next_id("M");
                    // Emit DefMethod + Param + Return instructions
                    self.emit_method_ir(&mut instructions, &method_id, &cap.text);
                }
                "field.root" => {
                    let field_id = self.next_id("F");
                    instructions.push(CoreOp::DefField(
                        field_id.clone(),
                        cap.text.clone(),
                    ));
                }
                // Control flow captures → FLAGS
                "if.root" => {
                    // Attach to most recent method
                    if let Some(last_method) = self.find_last_method(&instructions) {
                        instructions.push(CoreOp::Flags(
                            last_method,
                            vec![FLAG_IF.to_string()],
                        ));
                    }
                }
                "for.root" | "while.root" => {
                    if let Some(last_method) = self.find_last_method(&instructions) {
                        instructions.push(CoreOp::Flags(
                            last_method,
                            vec![FLAG_LOOP.to_string()],
                        ));
                    }
                }
                "return.root" => {
                    if let Some(last_method) = self.find_last_method(&instructions) {
                        instructions.push(CoreOp::Flags(
                            last_method,
                            vec![FLAG_RET.to_string()],
                        ));
                    }
                }
                "throw.root" => {
                    if let Some(last_method) = self.find_last_method(&instructions) {
                        instructions.push(CoreOp::Flags(
                            last_method,
                            vec![FLAG_THROW.to_string()],
                        ));
                    }
                }
                _ => {}
            }
        }
        
        Ok(CompiledIR {
            file_id: file_id.to_string(),
            instructions,
            version: 1,
        })
    }
    
    fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}{}", prefix, self.id_counter)
    }
    
    fn emit_method_ir(
        &mut self,
        instructions: &mut Vec<CoreOp>,
        method_id: &str,
        raw_sig: &str,
    ) {
        // Parse "processComplexData(payload:string):boolean"
        // Into: DefMethod + Param + Return
        // TODO: implement signature parser
        instructions.push(CoreOp::DefMethod(
            String::new(), // class_id (set by caller)
            method_id.to_string(),
            raw_sig.to_string(),
        ));
    }
    
    fn find_last_method(&self, instructions: &[CoreOp]) -> Option<String> {
        instructions.iter().rev().find_map(|op| {
            if let CoreOp::DefMethod(_, id, _) = op {
                Some(id.clone())
            } else {
                None
            }
        })
    }
}
```

### IR → Wire Serialization

```rust
// src/ir/wire.rs

use super::opcodes::CoreOp;
use super::compiler::CompiledIR;
use serde_json::{json, Value};

/// Serialize a single CoreOp to its positional tuple representation.
pub fn op_to_tuple(op: &CoreOp) -> Vec<String> {
    match op {
        CoreOp::DefClass(id, name) => vec!["DEF_C".into(), id.clone(), name.clone()],
        CoreOp::DefMethod(cid, mid, name) => vec!["DEF_M".into(), cid.clone(), mid.clone(), name.clone()],
        CoreOp::DefField(cid, fid, name) => vec!["DEF_F".into(), cid.clone(), fid.clone(), name.clone()],
        CoreOp::DefInterface(id, name) => vec!["DEF_I".into(), id.clone(), name.clone()],
        CoreOp::Param(mid, pid, ty, name) => vec!["SIG".into(), mid.clone(), pid.clone(), ty.clone(), name.clone()],
        CoreOp::Return(mid, ty) => vec!["RET".into(), mid.clone(), ty.clone()],
        CoreOp::FieldType(fid, ty) => vec!["FIELD_T".into(), fid.clone(), ty.clone()],
        CoreOp::Flags(tid, flags) => {
            let mut v = vec!["FLAGS".into(), tid.clone()];
            v.extend(flags.iter().cloned());
            v
        }
        CoreOp::ClassFlags(cid, flags) => {
            let mut v = vec!["FLAGS_C".into(), cid.clone()];
            v.extend(flags.iter().cloned());
            v
        }
        CoreOp::Extends(child, parent) => vec!["EXT".into(), child.clone(), parent.clone()],
        CoreOp::Implements(cid, iid) => vec!["IMPL".into(), cid.clone(), iid.clone()],
        CoreOp::Injects(cid, deps) => {
            let mut v = vec!["INJECTS".into(), cid.clone()];
            v.extend(deps.iter().cloned());
            v
        }
        CoreOp::Import(alias, module, named) => vec!["IMP".into(), alias.clone(), module.clone(), named.clone()],
        CoreOp::TypeAlias(alias, original) => vec!["TYPE".into(), alias.clone(), original.clone()],
    }
}

/// Serialize a CompiledIR to the wire format.
pub fn ir_to_wire(ir: &CompiledIR) -> Value {
    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();
    
    json!({
        "file": ir.file_id,
        "v": ir.version,
        "ir": tuples
    })
}
```

### Completion Criteria

- [x] `src/ir/mod.rs` created with module declarations
- [x] `CoreOp` enum defined with all 14 instruction types
- [x] `op_to_tuple()` serializes every `CoreOp` variant to positional tuple
- [x] `IRCompiler::compile()` processes captures and emits `Vec<CoreOp>`
- [x] `CompiledIR` struct defined (file_id, instructions, version)
- [x] `ir_to_wire()` serializes `CompiledIR` to JSON value
- [x] Unit tests: compile a sample TypeScript file → verify IR instruction count and types
- [x] Unit tests: `op_to_tuple()` round-trip for every `CoreOp` variant
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 5. Phase B: Global Symbol Table

### Goal

Unified symbol registry that subsumes `SymbolDictionary` and `PathDictionary` into a cross-stage, version-tracked registry.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/symbol_table.rs` | `GlobalSymbolTable`, `SymbolEntry`, `SymbolKind` |
| Tests for symbol table | Registration, lookup, versioning, unregister |

### Symbol Table Design

```rust
// src/ir/symbol_table.rs

use std::collections::HashMap;

/// What kind of symbol this is
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Class,
    Method,
    Field,
    Interface,
    Param,
    Import,
    Type,
}

/// A single symbol entry in the global registry
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    /// Machine alias (e.g., "C1", "M3", "P2")
    pub alias: String,
    /// Original name (e.g., "SampleService", "processComplexData")
    pub original: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Which file defines this symbol
    pub file_id: String,
    /// Version when first registered
    pub version_first: u64,
    /// Version when last modified
    pub version_last: u64,
}

/// Cross-stage global symbol table.
/// Tracks all symbols across all files, with version-based change tracking.
pub struct GlobalSymbolTable {
    /// Monotonically increasing version counter
    version: u64,
    
    /// alias → SymbolEntry
    symbols: HashMap<String, SymbolEntry>,
    
    /// original_name → alias (reverse index)
    reverse: HashMap<String, String>,
    
    /// file_id → set of symbol aliases defined in that file
    file_members: HashMap<String, Vec<String>>,
    
    /// Next alias counter per kind (C1, C2, ... M1, M2, ... F1, F2, ...)
    counters: HashMap<SymbolKind, u32>,
}

impl GlobalSymbolTable {
    pub fn new() -> Self {
        Self {
            version: 0,
            symbols: HashMap::new(),
            reverse: HashMap::new(),
            file_members: HashMap::new(),
            counters: HashMap::new(),
        }
    }
    
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Bump version (called after each delta application)
    pub fn bump_version(&mut self) -> u64 {
        self.version += 1;
        self.version
    }
    
    /// Generate the next alias for a given kind
    pub fn next_alias(&mut self, kind: SymbolKind) -> String {
        let counter = self.counters.entry(kind).or_insert(0);
        *counter += 1;
        let prefix = match kind {
            SymbolKind::Class => "C",
            SymbolKind::Method => "M",
            SymbolKind::Field => "F",
            SymbolKind::Interface => "I",
            SymbolKind::Param => "P",
            SymbolKind::Import => "IM",
            SymbolKind::Type => "T",
        };
        format!("{}{}", prefix, counter)
    }
    
    /// Register a new symbol
    pub fn register(
        &mut self,
        alias: String,
        original: String,
        kind: SymbolKind,
        file_id: &str,
    ) {
        let entry = SymbolEntry {
            alias: alias.clone(),
            original: original.clone(),
            kind,
            file_id: file_id.to_string(),
            version_first: self.version,
            version_last: self.version,
        };
        self.reverse.insert(original, alias.clone());
        self.symbols.insert(alias.clone(), entry);
        self.file_members
            .entry(file_id.to_string())
            .or_default()
            .push(alias);
    }
    
    /// Unregister a symbol (for delta deletions)
    pub fn unregister(&mut self, alias: &str) -> Option<SymbolEntry> {
        if let Some(entry) = self.symbols.remove(alias) {
            self.reverse.remove(&entry.original);
            if let Some(members) = self.file_members.get_mut(&entry.file_id) {
                members.retain(|a| a != alias);
            }
            Some(entry)
        } else {
            None
        }
    }
    
    /// Update a symbol's version (for delta modifications)
    pub fn touch(&mut self, alias: &str) {
        if let Some(entry) = self.symbols.get_mut(alias) {
            entry.version_last = self.version;
        }
    }
    
    /// Look up a symbol by alias
    pub fn get(&self, alias: &str) -> Option<&SymbolEntry> {
        self.symbols.get(alias)
    }
    
    /// Look up a symbol by original name
    pub fn get_by_original(&self, original: &str) -> Option<&SymbolEntry> {
        self.reverse.get(original).and_then(|a| self.symbols.get(a))
    }
    
    /// Get all symbols for a file
    pub fn get_file_symbols(&self, file_id: &str) -> Vec<&SymbolEntry> {
        self.file_members
            .get(file_id)
            .map(|aliases| {
                aliases.iter()
                    .filter_map(|a| self.symbols.get(a))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all symbols modified in a version range
    pub fn get_changed_since(&self, since_version: u64) -> Vec<&SymbolEntry> {
        self.symbols.values()
            .filter(|e| e.version_last > since_version)
            .collect()
    }
}
```

### Completion Criteria

- [x] `GlobalSymbolTable` struct implemented with all methods
- [x] `SymbolEntry` and `SymbolKind` defined
- [x] `next_alias()` generates correct prefixed aliases (C1, M1, F1, etc.)
- [x] `register()` / `unregister()` / `touch()` maintain all indexes
- [x] `get()` / `get_by_original()` / `get_file_symbols()` / `get_changed_since()` work correctly
- [x] Version bumping works (each `bump_version()` increments monotonically)
- [x] Unit tests: register 10 symbols across 3 files → verify lookup by alias, original, and file
- [x] Unit tests: unregister removes from all indexes
- [x] Unit tests: `get_changed_since()` returns correct subset
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 6. Phase C: Delta Transport

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Instruction-level diffing between two `CompiledIR` states, producing a structured delta envelope for transport.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/delta.rs` | `IRDelta`, `DeltaOps`, `ModOp`, `DeltaComputer` |
| `src/tests/ir/delta.rs` | 26 tests: add/modify/remove detection, version chain, JSON round-trip, edge cases |

### Delta Wire Format

```rust
// src/ir/delta.rs

use super::opcodes::CoreOp;
use super::compiler::CompiledIR;

/// A structured delta between two IR states.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IRDelta {
    /// Target file (path alias)
    pub file: String,
    /// Baseline version this delta applies to
    pub from_version: u64,
    /// Version after applying this delta
    pub to_version: u64,
    /// Operations grouped by type
    pub ops: DeltaOps,
}

/// Grouped delta operations.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeltaOps {
    /// New instructions to insert
    #[serde(rename = "+")]
    pub adds: Vec<Vec<String>>,
    /// In-place modifications
    #[serde(rename = "~")]
    pub mods: Vec<ModOp>,
    /// Instructions to remove (matched by opcode + primary key)
    #[serde(rename = "-")]
    pub dels: Vec<Vec<String>>,
}

/// A modification operation: match by key, replace with new instruction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModOp {
    /// The instruction to match (opcode + id = primary key)
    #[serde(rename = "k")]
    pub key: Vec<String>,
    /// The full replacement instruction
    #[serde(rename = "r")]
    pub replace: Vec<String>,
}

/// Delta computation engine.
/// Compares two CompiledIR states and produces an IRDelta.
pub struct DeltaComputer;

impl DeltaComputer {
    pub fn new() -> Self {
        Self
    }
    
    /// Compute the delta between baseline and current IR.
    /// Returns None if both IRs are identical.
    pub fn compute(
        &self,
        baseline: &CompiledIR,
        current: &CompiledIR,
    ) -> Option<IRDelta> {
        let base_indexed = index_instructions(&baseline.instructions);
        let cur_indexed = index_instructions(&current.instructions);
        
        let mut ops = DeltaOps::default();
        
        // Additions: in current but not baseline
        for (key, insn) in &cur_indexed {
            if !base_indexed.contains_key(key) {
                ops.adds.push(op_to_tuple(insn));
            }
        }
        
        // Removals: in baseline but not current
        for (key, insn) in &base_indexed {
            if !cur_indexed.contains_key(key) {
                ops.dels.push(op_to_tuple(insn));
            }
        }
        
        // Modifications: in both but different
        for (key, base_insn) in &base_indexed {
            if let Some(cur_insn) = cur_indexed.get(key) {
                if op_to_tuple(base_insn) != op_to_tuple(cur_insn) {
                    ops.mods.push(ModOp {
                        key: key_tuple(base_insn),
                        replace: op_to_tuple(cur_insn),
                    });
                }
            }
        }
        
        // Return None if no changes
        if ops.adds.is_empty() && ops.mods.is_empty() && ops.dels.is_empty() {
            return None;
        }
        
        Some(IRDelta {
            file: current.file_id.clone(),
            from_version: baseline.version,
            to_version: current.version,
            ops,
        })
    }
}

/// Index instructions by their primary key (opcode + identifying operands).
fn index_instructions(instructions: &[CoreOp]) -> std::collections::BTreeMap<String, CoreOp> {
    instructions.iter().map(|op| {
        let key = primary_key(op);
        (key, op.clone())
    }).collect()
}

/// Extract the primary key from an instruction.
/// Used for matching in deltas.
fn primary_key(op: &CoreOp) -> String {
    match op {
        CoreOp::DefClass(id, _) => format!("DEF_C:{}", id),
        CoreOp::DefMethod(cid, mid, _) => format!("DEF_M:{}:{}", cid, mid),
        CoreOp::DefField(cid, fid, _) => format!("DEF_F:{}:{}", cid, fid),
        CoreOp::DefInterface(id, _) => format!("DEF_I:{}", id),
        CoreOp::Param(mid, pid, _, _) => format!("SIG:{}:{}", mid, pid),
        CoreOp::Return(mid, _) => format!("RET:{}", mid),
        CoreOp::FieldType(fid, _) => format!("FIELD_T:{}", fid),
        CoreOp::Flags(tid, _) => format!("FLAGS:{}", tid),
        CoreOp::ClassFlags(cid, _) => format!("FLAGS_C:{}", cid),
        CoreOp::Extends(child, _) => format!("EXT:{}", child),
        CoreOp::Implements(cid, iid) => format!("IMPL:{}:{}", cid, iid),
        CoreOp::Injects(cid, _) => format!("INJECTS:{}", cid),
        CoreOp::Import(alias, _, _) => format!("IMP:{}", alias),
        CoreOp::TypeAlias(alias, _) => format!("TYPE:{}", alias),
    }
}

/// Extract the key tuple from an instruction (for ModOp matching).
fn key_tuple(op: &CoreOp) -> Vec<String> {
    match op {
        CoreOp::DefClass(id, _) => vec!["DEF_C".into(), id.clone()],
        CoreOp::DefMethod(cid, mid, _) => vec!["DEF_M".into(), cid.clone(), mid.clone()],
        CoreOp::DefField(cid, fid, _) => vec!["DEF_F".into(), cid.clone(), fid.clone()],
        CoreOp::Param(mid, pid, _, _) => vec!["SIG".into(), mid.clone(), pid.clone()],
        CoreOp::Return(mid, _) => vec!["RET".into(), mid.clone()],
        // ... other variants follow same pattern
        _ => op_to_tuple(op),
    }
}
```

### Completion Criteria

- [x] `IRDelta` struct with `DeltaOps` (adds, mods, dels) defined and serializable
- [x] `ModOp` struct with key/replace fields
- [x] `DeltaComputer::compute()` correctly identifies additions, removals, modifications
- [x] `primary_key()` generates unique keys for every instruction type
- [x] `key_tuple()` extracts the match key for modifications
- [x] Delta is `None` when IRs are identical (no unnecessary deltas)
- [x] Version chain: `from_version` = baseline.version, `to_version` = current.version
- [x] Unit tests: add a method → delta has 1 add op
- [x] Unit tests: remove a method → delta has 1 del op (and its SIG/RET)
- [x] Unit tests: modify a method signature → delta has 1 mod op
- [x] Unit tests: unchanged IR → delta is None
- [x] JSON serialization produces correct `+`/`~`/`-` keys
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 7. Phase D: State Replay

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Client-side state machine that applies delta ops to reconstruct IR state, with version-based catch-up support.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/replay.rs` | `ContextState`, `FileState`, `DeltaError` |
| `src/tests/ir/replay.rs` | 39 tests: FileState ops, ContextState apply, version validation, error cases, sequential deltas, render, multi-file, full replay cycle |

### State Machine Design

```rust
// src/ir/replay.rs

use std::collections::HashMap;
use super::delta::IRDelta;
use super::compiler::CompiledIR;
use super::render::ir_to_text;
use super::wire::op_to_tuple;
use super::delta::{primary_key_from_tuple, key_tuple_from_tuple};
use crate::compression::Fidelity;

/// Errors during delta application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    UnknownFile(String),
    VersionMismatch { expected: u64, got: u64 },
    SymbolNotFound(String),
    DuplicateSymbol(String),
}

/// Per-file IR state with indexed instruction stream.
#[derive(Debug, Clone)]
pub struct FileState {
    /// Ordered instruction stream (each instruction is a positional tuple)
    pub instructions: Vec<Vec<String>>,
    /// Index: primary_key → instruction index in `instructions`
    pub index: HashMap<String, usize>,
    /// Version when this file was last modified
    pub version: u64,
}

impl FileState {
    pub fn new(version: u64) -> Self { ... }
    pub fn from_compiled(ir: &CompiledIR) -> Self { ... }
    pub fn remove_by_key(&mut self, key_tuple: &[String]) -> bool { ... }
    pub fn replace_by_key(&mut self, key_tuple: &[String], replacement: &[String]) -> bool { ... }
    pub fn append(&mut self, instruction: Vec<String>) { ... }
    pub fn contains_key(&self, key_tuple: &[String]) -> bool { ... }
    fn rebuild_index(&mut self) { ... }
}

/// Top-level context state — tracks all files and their IR states.
#[derive(Debug, Clone)]
pub struct ContextState {
    files: HashMap<String, FileState>,
    version: u64,
}

impl ContextState {
    pub fn new() -> Self { ... }
    pub fn load_ir(&mut self, ir: CompiledIR) { ... }
    pub fn apply(&mut self, delta: IRDelta) -> Result<u64, DeltaError> { ... }
    pub fn render_pretty(&self, file_id: &str, fidelity: Fidelity) -> Option<String> { ... }
    pub fn get_ir(&self, file_id: &str) -> Option<&Vec<Vec<String>>> { ... }
    pub fn version(&self) -> u64 { ... }
    pub fn has_file(&self, file_id: &str) -> bool { ... }
    pub fn file_version(&self, file_id: &str) -> Option<u64> { ... }
    pub fn instruction_count(&self, file_id: &str) -> Option<usize> { ... }
    pub fn remove_file(&mut self, file_id: &str) -> bool { ... }
    pub fn file_ids(&self) -> Vec<String> { ... }
}
```

### Completion Criteria

- [x] `FileState` with instructions, index, and version tracking
- [x] `ContextState` with per-file management
- [x] `apply()` validates version chain before applying
- [x] Apply order: deletions → modifications → additions
- [x] `remove_by_key()` correctly removes and rebuilds index
- [x] `replace_by_key()` correctly replaces and updates index
- [x] `append()` adds instruction and updates index
- [x] `render_pretty()` delegates to `ir_to_text()`
- [x] `load_ir()` bootstraps state from full CompiledIR
- [x] Error cases: unknown file, version mismatch, missing symbol
- [x] Unit tests: apply add delta → new instruction present
- [x] Unit tests: apply remove delta → instruction absent
- [x] Unit tests: apply modify delta → instruction updated
- [x] Unit tests: version mismatch → DeltaError::VersionMismatch
- [x] Unit tests: unknown file → DeltaError::UnknownFile
- [x] Unit tests: sequential deltas (v1→v2→v3) apply correctly
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 8. Phase E: IR / Pretty Separation

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Two completely independent render paths from the same canonical IR. The IR is the source of truth; pretty output is derived.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/render.rs` | `ir_to_text()` — fidelity-aware rendering |
| Tests for render | Round-trip tests, fidelity comparison |

### Render Design

```rust
// src/ir/render.rs

use crate::compression::Fidelity;
use super::opcodes::*;

/// Render IR instructions to human-readable text.
/// Fidelity controls what information is included, not the compilation.
/// Same IR → different fidelity → different output.
pub fn ir_to_text(instructions: &[Vec<String>], fidelity: Fidelity) -> String {
    let mut output = String::new();
    let mut current_class: Option<String> = None;
    
    for insn in instructions {
        match insn[0].as_str() {
            "DEF_C" => {
                let name = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                match fidelity {
                    Fidelity::Low => {
                        if current_class.is_some() {
                            output.push(';');
                        }
                        output.push_str(&format!("$c {}", name));
                        current_class = Some(name.to_string());
                    }
                    Fidelity::Medium => {
                        if current_class.is_some() {
                            output.push('\n');
                        }
                        output.push_str(&format!("class {} {{\n", name));
                        current_class = Some(name.to_string());
                    }
                    Fidelity::High => {
                        if current_class.is_some() {
                            output.push('\n');
                        }
                        output.push_str(&format!("class {} {{\n", name));
                        current_class = Some(name.to_string());
                    }
                }
            }
            "DEF_M" => {
                let name = insn.get(3).map(|s| s.as_str()).unwrap_or("?");
                let indent = match fidelity {
                    Fidelity::High => "  ",
                    _ => "",
                };
                if fidelity == Fidelity::Low {
                    output.push_str(&format!("{}();", name));
                } else {
                    output.push_str(&format!("{}{}()", indent, name));
                }
            }
            "SIG" => {
                let param_name = insn.get(4).map(|s| s.as_str()).unwrap_or("?");
                let type_op = insn.get(3).map(|s| s.as_str()).unwrap_or("$v");
                match fidelity {
                    Fidelity::Low => {
                        // Low: just type
                        output.push_str(&format!("{}:{},", param_name, type_op));
                    }
                    _ => {
                        output.push_str(&format!("{}:{},", param_name, type_op));
                    }
                }
            }
            "RET" => {
                let type_op = insn.get(2).map(|s| s.as_str()).unwrap_or("$v");
                output.push_str(&format!("):{}", type_op));
                if fidelity != Fidelity::Low {
                    output.push('\n');
                }
            }
            "FLAGS" => {
                let flags: Vec<&str> = insn[2..].iter().map(|s| s.as_str()).collect();
                let markers = flags_to_markers(&flags);
                match fidelity {
                    Fidelity::Low => {
                        output.push_str(&format!(" {}", markers.join(" ")));
                    }
                    Fidelity::Medium => {
                        output.push_str(&format!(" {}", markers.join(" ")));
                    }
                    Fidelity::High => {
                        output.push_str(&format!(" {{ {} }}", markers.join(" ")));
                    }
                }
            }
            "IMP" => {
                let module = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                let named = insn.get(3).map(|s| s.as_str()).unwrap_or("*");
                match fidelity {
                    Fidelity::Low => {
                        output.push_str(&format!("$im {}.$fm{};", named, module));
                    }
                    _ => {
                        output.push_str(&format!("import {{ {} }} from '{}';\n", named, module));
                    }
                }
            }
            _ => {}
        }
    }
    
    output
}

/// Convert IR flags back to ⊕ markers for backward-compatible display.
fn flags_to_markers(flags: &[&str]) -> Vec<String> {
    flags.iter().map(|f| {
        match *f {
            "IF" => "⊕guard".to_string(),
            "LOOP" => "⊕loop".to_string(),
            "RET" => "⊕⇒".to_string(),
            "THROW" => "⊕!".to_string(),
            other => format!("⊕{}", other),
        }
    }).collect()
}
```

### Key Insight: Backward Compatibility

The `ir_to_text()` function can produce **byte-identical output** to the current `compress_code_context` tool. This means:

1. **Existing tools keep working** — `compress_code_context` just calls `ir_to_text(compiler.compile(...))`
2. **New tools expose IR** — `compress_code_context_ir` calls `ir_to_wire(compiler.compile(...))`
3. **Same compilation, two outputs** — no duplicate parsing logic

### Completion Criteria

- [x] `ir_to_text()` renders IR to human-readable text at all 3 fidelity levels
- [x] Low fidelity: compact output with opcodes and ⊕ markers
- [x] Medium fidelity: full signatures with keywords
- [x] High fidelity: indented with behavior markers in braces
- [x] `flags_to_markers()` converts IR flags back to ⊕ notation
- [x] Import rendering at all fidelity levels
- [x] Class/method/field rendering at all fidelity levels
- [x] Round-trip test: compile sample → render → matches expected output format
- [x] Fidelity comparison: same IR at Low vs Medium vs High shows progressive detail
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 9. Phase F: Layered Encoding

### Goal

Separate language-specific and framework-specific logic into pluggable layers that emit additional ops on top of the Core IR.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/layers/mod.rs` | `LanguageLayer`, `MetaLayer` traits |
| `src/ir/layers/typescript.rs` | `TypeScriptLayer` implementation |
| `src/ir/layers/csharp.rs` | `CSharpLayer` implementation |
| `src/ir/layers/angular.rs` | `AngularMetaLayer` (refactored from `angular_meta/`) |
| `src/ir/layers/patterns.rs` | `PatternRecognizer` — Layer 4 |

### Layer Traits

```rust
// src/ir/layers/mod.rs

use super::opcodes::CoreOp;

/// Language-specific IR layer (Layer 2).
/// Translates language-specific captures into additional IR instructions.
pub trait LanguageLayer {
    /// Language name (e.g., "typescript", "csharp")
    fn name(&self) -> &str;
    
    /// Process a capture and emit additional IR instructions.
    /// Called for each capture from the tree-sitter pipeline.
    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp>;
    
    /// Post-processing: emit any cross-cutting instructions
    /// after all captures have been processed.
    fn finalize(&mut self, context: &mut LayerContext) -> Vec<CoreOp> {
        Vec::new()
    }
}

/// Framework-specific IR layer (Layer 3).
/// Extracts framework patterns (decorators, annotations, etc.)
pub trait MetaLayer {
    /// Framework name (e.g., "angular", "react", "ngrx")
    fn name(&self) -> &str;
    
    /// Extract framework-specific ops from the full source and class list.
    fn extract(
        &mut self,
        source: &str,
        classes: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp>;
}

/// Pattern recognizer (Layer 4).
/// Identifies common patterns and compresses them to single ops.
pub trait PatternRecognizer {
    /// Analyze the instruction stream and compress recognized patterns.
    fn recognize(&self, instructions: &[CoreOp]) -> Vec<CoreOp>;
}

/// Context passed to layer processing functions
pub struct LayerContext {
    /// Current class ID (set when processing a class capture)
    pub current_class: Option<String>,
    /// Current method ID (set when processing a method capture)
    pub current_method: Option<String>,
    /// Global symbol table reference
    pub symbol_table: GlobalSymbolTable,
}
```

### TypeScript Layer

```rust
// src/ir/layers/typescript.rs

pub struct TypeScriptLayer;

impl LanguageLayer for TypeScriptLayer {
    fn name(&self) -> &str { "typescript" }
    
    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();
        
        match capture_name {
            "class.root" => {
                // Extract extends/implements from raw text
                if let Some((base, interfaces)) = extract_ts_class_head(raw_text) {
                    if let Some(class_id) = &context.current_class {
                        if let Some(base_id) = base {
                            ops.push(CoreOp::Extends(
                                class_id.clone(),
                                base_id,
                            ));
                        }
                        for iface in interfaces {
                            ops.push(CoreOp::Implements(
                                class_id.clone(),
                                iface,
                            ));
                        }
                    }
                }
            }
            "method.root" => {
                // Extract async, generics from raw text
                if raw_text.contains("async") {
                    if let Some(method_id) = &context.current_method {
                        ops.push(CoreOp::Flags(
                            method_id.clone(),
                            vec![FLAG_ASYNC.to_string()],
                        ));
                    }
                }
            }
            _ => {}
        }
        
        ops
    }
}
```

### Angular Meta-Layer (Layer 3)

```rust
// src/ir/layers/angular.rs

pub struct AngularMetaLayer;

impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &str { "angular" }
    
    fn extract(
        &mut self,
        source: &str,
        classes: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp> {
        // This wraps the existing angular_meta::run_meta_layer logic
        // but emits CoreOp instructions instead of Φ marker text
        let mut ops = Vec::new();
        
        // Delegate to existing decorator extraction
        // Convert Φ markers to CoreOp instructions
        // e.g., Φcmp: → CoreOp::MetaOp("NG_COMPONENT", ...)
        
        ops
    }
}
```

### Completion Criteria

- [x] `LanguageLayer` trait defined with `process_capture()` and `finalize()`
- [x] `MetaLayer` trait defined with `extract()`
- [x] `PatternRecognizer` trait defined with `recognize()`
- [x] `LayerContext` struct with current_class, current_method, symbol_table, source, fidelity
- [x] `TypeScriptLayer` implements `LanguageLayer` — extracts extends/implements/async/static/export
- [x] `CSharpLayer` implements `LanguageLayer` — extracts inheritance, interfaces, flags
- [x] `AngularMetaLayer` implements `MetaLayer` — wraps existing angular_meta logic, emits CoreOps
- [x] `CodePatternRecognizer` implements `PatternRecognizer` — detects CTOR, OBSERVABLE, getter/setter
- [x] `IRCompiler` module declarations updated to expose `layers` submodule
- [x] Unit tests: TypeScript extends → `EXT` op in output
- [x] Unit tests: TypeScript implements → `IMPL` ops in output
- [x] Unit tests: TS async/static flags emitted correctly
- [x] Unit tests: C# inheritance with colon syntax → `EXT` + `IMPL` ops
- [x] Unit tests: C# abstract/public class flags emitted
- [x] Unit tests: Constructor injection pattern → `CTOR` flag
- [x] Unit tests: Observable pattern → `OBSERVABLE` flag
- [x] Unit tests: Getter/Setter pattern → `GETTER`/`SETTER` flags
- [x] Unit tests: Unrecognized patterns pass through unchanged
- [x] Unit tests: Angular meta-layer returns empty for non-Angular source
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] All 504 existing tests pass with 0 failures

---

## 10. Phase G: Integration & MCP Tools

### Goal

Wire the IR system into the MCP tool interface, maintaining backward compatibility.

### Deliverables

| File | Description |
|------|-------------|
| `src/mcp/tools.rs` (modified) | New tool definitions for IR operations |
| `src/ir/mod.rs` (updated) | Public API for MCP integration |

### New MCP Tools

#### `compress_code_context` (upgraded, backward compatible)

```json
// Input: same as today
{
  "filePath": "/path/to/file.ts",
  "fidelity": "low"
}

// Output: IR + pretty (new shape, but includes pretty for compat)
{
  "pretty": "// compressed text output (existing format)...",
  "ir": [
    ["DEF_C", "C1", "SampleService"],
    ["DEF_M", "C1", "M1", "processComplexData"],
    ["SIG", "M1", "P1", "$s", "payload"],
    ["RET", "M1", "$b"],
    ["FLAGS", "M1", "IF"]
  ],
  "v": 1,
  "file": "α1"
}
```

#### `delta_code_context` (new — replaces text-based diff)

```json
// Input: file that changed
{
  "filePath": "/path/to/file.ts"
}

// Output: delta envelope
{
  "file": "α1",
  "from": 5,
  "to": 6,
  "ops": {
    "+": [
      ["DEF_M", "C1", "M3", "newMethod"],
      ["SIG", "M3", "P1", "$s", "input"],
      ["RET", "M3", "$b"]
    ],
    "~": [
      {"k": ["DEF_M", "C1", "M1"], "r": ["DEF_M", "C1", "M1", "renamedMethod"]}
    ],
    "-": [
      ["DEF_M", "C1", "M4"]
    ]
  }
}
```

#### `apply_delta` (new — client-side state update)

```json
// Input: delta + current state version
{
  "delta": { ... },
  "currentVersion": 5
}

// Output: confirmation + updated pretty
{
  "ok": true,
  "newVersion": 6,
  "pretty": "// re-rendered after delta application..."
}
```

**Status: ✅ Complete** — implemented 2026-06-08

### Completion Criteria

- [x] `compress_code_context` tool output includes `ir` field alongside `pretty`
- [x] `delta_code_context` tool computes and returns `IRDelta` envelope
- [x] `apply_delta` tool applies delta and returns updated state
- [x] Backward compatibility: existing `compress_code_context` `pretty` field is byte-identical to current output
- [x] MCP tool schemas updated for new tools
- [x] Integration test: compress → edit → delta → apply → render matches expected
- [x] `cargo clippy --all-targets -- -D warnings` passes

### Implementation Summary

**Files modified:**

| File | Change |
|------|--------|
| `src/mcp/tools.rs` | Added `delta_code_context` and `apply_delta` tool definitions + handlers; upgraded `compress_code_context` to also emit `ir` field; added `compile_file_ir()` helper. Refactored into per-tool handler functions. |
| `src/mcp/state.rs` | Added `ir_context: ContextState` field to `McpState` for in-session IR delta tracking. |
| `src/ir/layers/angular.rs` | Fixed pre-existing clippy warnings (`needless_borrow`). |
| `src/ir/layers/patterns.rs` | Fixed pre-existing clippy warnings (`collapsible_match`, `needless_range_loop`). |
| `src/tests/ir/layers/patterns.rs` | Fixed pre-existing dead-code warning. |

**Files added:**

| File | Purpose |
|------|---------|
| `src/tests/ir/integration.rs` | Phase G end-to-end tests: full pipeline, state replay, wire format, delta computer with state. |
| `src/tests/ir/mod.rs` | Test module wiring for the new integration tests. |

**Key design decisions:**

1. **`McpState.ir_context`** — The new `ContextState` is held inside the existing per-session `McpState` (alongside `dict`, `cache`, `config`) so handlers can mutate it via `&mut McpState` — no signature changes to the existing dispatch chain.
2. **Backward compatibility** — The upgraded `compress_code_context` keeps the original `compress_file()` text output and *adds* `ir`, `pretty`, `v`, and `file` fields. Old clients that only read `content` see byte-identical text.
3. **`delta_code_context` first call** — When no baseline IR exists, the handler compiles the current IR and stores it as the baseline. Subsequent calls compute the `IRDelta` envelope using the existing `DeltaComputer`.
4. **`apply_delta` validation** — The handler does a pre-flight version check against `currentVersion` to fail fast on stale deltas, then delegates to `ContextState::apply()` for the actual mutation.
5. **`compile_file_ir` helper** — Detects the language from the file extension, looks up the tree-sitter query via `language_for_extension`, runs the existing `IRCompiler::compile`, and returns a `CompiledIR` ready to load or diff.

---

## 11. Phase H: Positional Encoding & Advanced Compression

### Goal

Maximum compression via key stripping and pattern recognition.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/positional.rs` | Positional encoding/decoding |
| `src/ir/patterns.rs` | Pattern recognition + compression |

### Positional Encoding

Strip all keys, rely on opcode positional semantics:

```rust
// Before (Layer 1 IR):
["DEF_M", "C1", "M1", "processComplexData"]

// After (Layer 4 positional — keys removed):
// Client knows DEF_M arity=3: [class_id, method_id, name]
// No keys transmitted — position implies meaning
```

The positional encoding is **not a separate format** — it's the same JSON tuples, but the client knows the schema and doesn't need keys. The static schema (Phase 14) provides the arity table.

### Pattern Compression

```rust
/// Recognize common code patterns and compress to single ops
pub struct PatternRecognizer;

impl PatternRecognizer {
    pub fn recognize(&self, instructions: &[CoreOp]) -> Vec<CoreOp> {
        let mut output = Vec::new();
        let mut i = 0;
        
        while i < instructions.len() {
            // Pattern: Constructor injection
            // DEF_M + SIG(P:ServiceType) + INJECTS → PAT_CTOR
            if let Some(pat) = self.try_ctor_pattern(&instructions[i..]) {
                output.push(pat);
                i += /* skip consumed instructions */;
                continue;
            }
            
            // Pattern: Observable stream
            // DEF_M + RET($P) + FLAGS(ASYNC) → PAT_OBSERVABLE
            if let Some(pat) = self.try_observable_pattern(&instructions[i..]) {
                output.push(pat);
                i += /* skip consumed instructions */;
                continue;
            }
            
            // No pattern matched — pass through
            output.push(instructions[i].clone());
            i += 1;
        }
        
        output
    }
}
```

### Completion Criteria

- [ ] Positional encoding can serialize/deserialize IR without keys
- [ ] Client can decode positional format using static schema
- [ ] Pattern recognizer detects constructor injection pattern
- [ ] Pattern recognizer detects observable stream pattern
- [ ] Pattern compression reduces instruction count for recognized patterns
- [ ] Unit tests: inject constructor → single PAT_CTOR op
- [ ] Unit tests: unrecognized patterns pass through unchanged
- [ ] `cargo clippy --all-targets -- -D warnings` passes

---

## 12. Migration Map

How each existing module maps to the new IR system:

| Existing Component | Role in IR System | Migration Path |
|---|---|---|
| `compression/opcodes.rs` (32 primitives) | Core IR type constants | Becomes `ir/opcodes.rs` — primitives become `CoreOp` type constants |
| `compression/markers.rs` (⊕guard etc.) | Replaced by `FLAGS` opcodes | `FLAGS("M1", ["IF","LOOP","RET"])` replaces `⊕guard ⊕loop ⊕⇒` |
| `compaction/*` | IR emitters | Each function emits `Vec<CoreOp>` instead of formatted strings |
| `diff/snapshot.rs` (CapturedStructure) | Replaced by `CompiledIR` | `CapturedClass` → `Vec<CoreOp>` instructions |
| `diff/differ.rs` (diff_snapshots) | Replaced by `DeltaComputer` | Text diff → instruction-level delta |
| `diff/action.rs` (DiffAction) | Replaced by `IRDelta` ops | `DiffKind::Added` → `DeltaOp::Add(CoreOp)` |
| `dictionary/symbol.rs` | Subsumed by `GlobalSymbolTable` | Runtime opcodes → typed aliases |
| `dictionary/path.rs` | Subsumed by `GlobalSymbolTable` | File aliases → file membership tracking |
| `angular_meta/*` | Layer 3 Meta-Layer | Implements `MetaLayer` trait |
| `decompression/*` | Replaced by `ir_to_text()` | Text expansion → IR rendering |
| `compression/pipeline.rs` | Wrapped by `IRCompiler` | One-shot compression → compiled state |
| `cache.rs` (LocalStateCache) | Enhanced by version tracking | Hash check → version check |

---

## 13. Wire Protocol Reference

### Instruction Tuple Format

Every instruction is a JSON array where position determines meaning:

```
[op0, op1, op2, ...]
 ^     ^     ^
 |     |     └── operand (type depends on opcode)
 |     └── target/class/method id
 └── opcode (always first element)
```

### Complete Opcode Reference

| Opcode | Arity | Operands | Example |
|--------|------:|----------|---------|
| `DEF_C` | 3 | id, name | `["DEF_C","C1","SampleService"]` |
| `DEF_M` | 4 | class_id, id, name | `["DEF_M","C1","M1","process"]` |
| `DEF_F` | 4 | class_id, id, name | `["DEF_F","C1","f1","payload"]` |
| `DEF_I` | 3 | id, name | `["DEF_I","I1","IMyService"]` |
| `SIG` | 5 | method_id, param_id, type, name | `["SIG","M1","P1","$s","payload"]` |
| `RET` | 3 | method_id, type | `["RET","M1","$b"]` |
| `FIELD_T` | 3 | field_id, type | `["FIELD_T","f1","$n"]` |
| `FLAGS` | 3+ | target_id, flag1, ... | `["FLAGS","M1","IF","LOOP"]` |
| `FLAGS_C` | 3+ | class_id, flag1, ... | `["FLAGS_C","C1","EXPORT"]` |
| `EXT` | 3 | child_id, parent_id | `["EXT","C1","C2"]` |
| `IMPL` | 3 | class_id, iface_id | `["IMPL","C1","I1"]` |
| `INJECTS` | 3+ | class_id, dep1, ... | `["INJECTS","C1","S1","S2"]` |
| `IMP` | 4 | alias, module, named | `["IMP","$im","rxjs","map"]` |
| `TYPE` | 3 | alias, original | `["TYPE","$uid","UserId"]` |

### Delta Wire Format

```json
{
  "file": "<path_alias>",
  "from": <baseline_version>,
  "to": <current_version>,
  "ops": {
    "+": [ [<instruction>, ...], ... ],
    "~": [ {"k": [<key_tuple>], "r": [<replacement>]}, ... ],
    "-": [ [<instruction>, ...], ... ]
  }
}
```

### Type Opcode Reference

| Opcode | Type | Inherited From |
|--------|------|----------------|
| `$s` | string | `PRIMITIVE_OPCODES["$s"]` |
| `$n` | number | `PRIMITIVE_OPCODES["$n"]` |
| `$b` | boolean | `PRIMITIVE_OPCODES["$b"]` |
| `$v` | void | `PRIMITIVE_OPCODES["$v"]` |
| `$T` | true | `PRIMITIVE_OPCODES["$T"]` |
| `$F` | false | `PRIMITIVE_OPCODES["$F"]` |
| `$nl` | null | `PRIMITIVE_OPCODES["$nl"]` |
| `$ud` | undefined | `PRIMITIVE_OPCODES["$ud"]` |

### Flag Reference

| Flag | Meaning | Replaces |
|------|---------|----------|
| `IF` | Conditional branch (if/switch) | `⊕guard` |
| `LOOP` | Loop construct (for/while/do) | `⊕loop` |
| `RET` | Return statement | `⊕⇒` |
| `THROW` | Throw/exception | `⊕!` |
| `ASYNC` | Async function | — (keyword preserved) |
| `GEN` | Generator function | — |
| `EXPORT` | Exported symbol | `$e` (existing opcode) |
| `STATIC` | Static member | `$st` (existing opcode) |
| `PRIVATE` | Private visibility | `$pv` (existing opcode) |
| `PROTECTED` | Protected visibility | `$pd` (existing opcode) |
| `ABSTRACT` | Abstract class/method | — |

---

## 14. Static Schema Definition

Sent once per session, defines the opcode vocabulary for all subsequent communication:

```json
{
  "schema_version": 1,
  "opcodes": {
    "DEF_C":    { "arity": 3, "args": ["id", "name"] },
    "DEF_M":    { "arity": 4, "args": ["class_id", "id", "name"] },
    "DEF_F":    { "arity": 4, "args": ["class_id", "id", "name"] },
    "DEF_I":    { "arity": 3, "args": ["id", "name"] },
    "SIG":      { "arity": 5, "args": ["method_id", "param_id", "type", "name"] },
    "RET":      { "arity": 3, "args": ["method_id", "type"] },
    "FIELD_T":  { "arity": 3, "args": ["field_id", "type"] },
    "FLAGS":    { "arity": -1, "args": ["target_id", "flags..."] },
    "FLAGS_C":  { "arity": -1, "args": ["class_id", "flags..."] },
    "EXT":      { "arity": 3, "args": ["child_id", "parent_id"] },
    "IMPL":     { "arity": 3, "args": ["class_id", "iface_id"] },
    "INJECTS":  { "arity": -1, "args": ["class_id", "deps..."] },
    "IMP":      { "arity": 4, "args": ["alias", "module", "named"] },
    "TYPE":     { "arity": 3, "args": ["alias", "original"] }
  },
  "flags": [
    "IF", "LOOP", "RET", "THROW", "ASYNC", "GEN",
    "EXPORT", "STATIC", "PRIVATE", "PROTECTED", "ABSTRACT"
  ],
  "types": ["$s", "$n", "$b", "$v", "$T", "$F", "$nl", "$ud"],
  "primitives": {
    "$e": "export", "$c": "class", "$a": "async", "$P": "Promise",
    "$b": "boolean", "$s": "string", "$n": "number", "$v": "void",
    "$T": "true", "$F": "false", "$pu": "public", "$pv": "private",
    "$pd": "protected", "$st": "static", "$nw": "new", "$r": "return",
    "$t": "throw", "$E": "Error", "$k": "const", "$l": "let",
    "$i": "if", "$fr": "for", "$w": "while", "$h": "this",
    "$x": "extends", "$m": "implements", "$im": "import", "$fm": "from",
    "$if": "interface", "$ty": "type", "$fn": "function", "$ctor": "constructor",
    "$ud": "undefined", "$nl": "null"
  }
}
```

`arity: -1` indicates variadic instruction length.

---

## 15. Phase Dependencies & Timeline

```
Phase A: IR Core ─────────────────────────────────────┐
  (instruction types, compiler, render)                │
                                                       ├──→ Phase E: IR/Pretty Separation
Phase B: Global Symbol Table ──────────────────────────┤     (two render paths)
  (unified registry)                                   │
                                                       ├──→ Phase F: Layered Encoding
Phase C: Delta Transport ──────────────────────────────┤     (language + meta layers)
  (diff engine, delta envelope)                        │
                                                       │
Phase D: State Replay ─────────────────────────────────┘
  (apply, version chain, catch-up)                     │
                                                       │
Phase G: Integration & MCP Tools ──────────────────────┤
  (tool definitions, backward compat)                  │
                                                       │
Phase H: Positional Encoding & Patterns ───────────────┘
  (maximum compression, pattern recognition)
```

### Estimated Effort

| Phase | Effort | Depends On |
|-------|--------|------------|
| **A: IR Core** | 3-4 days | Nothing |
| **B: Global Symbol Table** | 2-3 days | A |
| **C: Delta Transport** | 2-3 days | A, B |
| **D: State Replay** | 2-3 days | C |
| **E: IR/Pretty Separation** | 2-3 days | A |
| **F: Layered Encoding** | 3-4 days | A, B |
| **G: Integration & MCP Tools** | 2-3 days | C, D, E |
| **H: Positional Encoding** | 2-3 days | G |
| **Total** | **18-26 days** | — |

### Minimum Viable Protocol

Phases **A → C → D → G** form the minimum viable path to a working delta transport protocol. Phases E, F, H are enhancements that can be delivered in parallel or after the core is working.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.