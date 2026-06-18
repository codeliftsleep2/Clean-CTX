# IR-to-LLM Integration Plan

**Status:** Final — FAANG-reviewed
**Version:** 3
**Last Updated:** 2026-06-17

## Problem Statement

The current architecture has two parallel pipelines:
- **Text pipeline** → compacted text with `$c`, `⊕guard`, `Φcmp:` markers → sent as `content[0].text` → the LLM reads this
- **IR pipeline** → `CoreOp[]` instructions → encoded as JSON arrays in hidden `"ir"` field → the LLM **never sees this**

The IR pipeline contains richer structural information (class relationships, method flags, Angular/Spring metadata, pattern recognition), but none of it reaches the LLM's context window. We need to **flip the architecture** so the LLM consumes the structured IR output directly.

## Target Architecture

```
Source ──→ IR Pipeline ──→ Flat CoreOp[] ──→ Hierarchical IR ──→ Compact LLM Text ──→ content[0].text (LLM input)
                                                                                       └── "pretty": text pipeline (human debug)
                                                                                       └── "ir": hierarchical JSON (structured)
```

The text pipeline becomes a secondary concern — a human-readable "pretty print" of the same data, not the primary LLM input.

### Architectural Invariants

1. **The renderer is a STATELESS projection.** All stateful operations (deltas, persistence, context tracking) operate on the flat `CoreOp[]` stream internally. The LLM-optimized text is generated fresh each time from the latest IR. Never diff the rendered text — diff the underlying CoreOp stream.
2. **The alias IDs (`C1`, `M1`) are NOT stable across compiles.** The rendered text uses class/method **names** as the canonical identifiers, never internal alias IDs. This ensures the LLM sees stable references across tool calls.
3. **Delta transport continues unchanged.** The `delta_code_context` handler compares flat `CoreOp[]` arrays. The renderer is only called at the final output step. No changes to the delta algorithm.

---

## Notation Reference

All LLM-visible text uses single-character structural markers. Every marker is documented in the `// SCHEMA v2` header that opens every output.

### Structural Markers

| Marker | Meaning | Source IR |
|--------|---------|-----------|
| `//` | Comment, boundary, schema header | Always present |
| `X` | Extends (parent class) | `CoreOp::Extends` |
| `I` | Implements (interface/trait) | `CoreOp::Implements` |
| `F` | Field declaration | `CoreOp::DefField` |
| `M` | Method declaration | `CoreOp::DefMethod` |
| `P` | Compressed pattern | `CoreOp::Pattern` |
| `$` | Import declaration | `CoreOp::Import` |
| `T` | Type alias | `CoreOp::TypeAlias` |
| `→` | Scoped modifier (params, return type, flags) | Method body |
| `fl:` | Method-level flag list | `CoreOp::Flags` |
| `cl:` | Class-level flag list | `CoreOp::ClassFlags` |
| `@` | Framework meta-layer annotation | `CoreOp::TypeAlias` where alias starts with `@` |

### Meta-Layer `@` Markers

| Marker | Meaning | Framework |
|--------|---------|-----------|
| `@cmp` | Angular `@Component` | Angular |
| `@sel` | Component CSS selector | Angular |
| `@svc` | `@Injectable` / `@Service` | Both |
| `@mod` | `@NgModule` | Angular |
| `@dir` | `@Directive` | Angular |
| `@pipe` | `@Pipe` | Angular |
| `@in` | `@Input()` property binding | Angular |
| `@out` | `@Output()` event emitter | Angular |
| `@mdl` | `model()` signal (Anglar 17+) | Angular |
| `@rest` | `@RestController` | Spring |
| `@repo` | `@Repository` | Spring |
| `@ctrl` | `@Controller` | Spring |
| `@cfg` | `@Configuration` | Spring |
| `@comp` | `@Component` (Spring stereotype) | Spring |
| `@bean` | `@Bean` method-level | Spring |
| `@wire` | `@Autowired` field injection | Spring |
| `@val` | `@Value` property injection | Spring |
| `@map` | `@RequestMapping` / endpoint mappings | Spring |

### Method Flags

| Flag | Meaning |
|------|---------|
| `IF` | Contains if/branching |
| `LOOP` | Contains for/while/loop |
| `RET` | Contains return statement |
| `THROW` | Contains throw/panic |
| `ASYNC` | Async/promise function |
| `GEN` | Generator function |
| `EXPORT` | Public/exported visibility |
| `STATIC` | Static member |
| `PRIVATE` | Private visibility |
| `PROTECTED` | Protected visibility |
| `ABSTRACT` | Abstract class/method |
| `UNSAFE` | Unsafe block (Rust) |
| `CTOR` | Constructor injection pattern |
| `OBSERVABLE` | Observable/Promise return |
| `GETTER` `SETTER` | Accessor pattern |
| `OVERRIDE` | Override pattern |

### Overloaded Method Disambiguation

When a class has multiple methods with the same name (Java/C# overloading, TypeScript union signatures), the renderer appends `+N` where N is the parameter count to disambiguate:

| Rendered Output | Meaning |
|----------------|---------|
| `M create(+1)` | `create` with 1 parameter |
| `M create(+3)` | `create` with 3 parameters |
| `M doWork(+0)` | `doWork` with no parameters |

This ensures every method name in the rendered output is unique within its class without using internal alias IDs.

---

## Output Format

### Fidelity Levels

The renderer uses `Fidelity` to control verbosity:

| Level | Fields | Methods | Flags | Meta | Example |
|-------|--------|---------|-------|------|---------|
| **Low** | Space-separated, same line | Single line, minimal | Always shown | Always shown | `F a:$s b:$n` |
| **Medium** | One per line | Single line, params shown | Always shown | Always shown | `F a:$s\nF b:$n` |
| **High** | One per line, types verbose | Multi-line if needed | Always shown | Always shown | Same as Medium |

### Single File (compress_code_context)

**TypeScript file:**
```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags
// ── UserListComponent ──
@cmp UserListComponent
@sel app-user-list
X BaseListComponent
I OnInit OnDestroy
F users:$s[] selectedUser:$n
M ngOnInit  → fl:IF
M trackById  → p:index:$n user:$s  → fl:RET
@in selectedUser
@out userSelected
// ── α1 (src/app/user-list.component.ts) ──
$ ./core [OnInit, OnDestroy]
$ ./user.service [UserService]
T UserId = $n
```

**Rust file:**
```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags
// ── User ──
F id:$n name:$s email:$s
// ── Role ──
P EMPTY_CTOR
// ── UserService ──
X Repository<User>
F users:HashMap cache:RwLock
M new  → fl:CTOR
M get_user  → p:id:$n  → fl:ASYNC
M create_user  → p:user:User  → fl:RET
// ── α1 (src/services/user_service.rs) ──
$ std::collections [HashMap]
$ tokio::sync [RwLock]
T UserId = $n
```

**Java/Spring file with overloaded methods:**
```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags
// ── UserController ──
@rest UserController
@map [GET /users POST /users]
X BaseController
F userService:UserService
M getAll  → fl:RET
M find(+1)  → p:id:$n  → fl:RET
M find(+3)  → p:name:$n age:$n role:$s  → fl:RET,IF
@wire userService
// ── α1 (src/main/java/com/app/UserController.java) ──
$ org.springframework.web.bind.annotation [*]
$ com.app.service [UserService]
```

### Workspace (compress_workspace)

```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags
// ── α1 (src/app/user-list/user-list.component.ts) ──
// ── UserListComponent ──
@cmp UserListComponent
@sel app-user-list
X BaseListComponent
I OnInit
F users:$s[]
M ngOnInit  → fl:IF

// ── α2 (src/app/user.service.ts) ──
@svc UserService
F http:HttpClient
M getUsers  → fl:RET,ASYNC

// ── α3 (src/app/app.module.ts) ──
@mod AppModule
@mod AppModule decl [UserListComponent]
@mod AppModule imp [BrowserModule]
@mod AppModule exp [AppComponent]
```

**Note on cross-file references:** Within-file alias resolution (class `A` extends class `B` where `B` is defined later in the same file) is handled by `resolve_forward_aliases()`. Cross-file alias resolution (class `Foo` in `α1` extends class `Bar` in `α2`) is **not yet implemented** and tracked as a future enhancement.

---

## Wire Format (MCP Response)

### compress_code_context

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "// SCHEMA v2\n// ── UserListComponent ──\n@cmp UserListComponent\n@sel app-user-list\n..."
      }
    ],
    "ir": {
      "file": "α1",
      "v": 1,
      "encoding": "hierarchical",
      "ir": { "c": [{ ... }] }
    },
    "pretty": "// --- Token Optimization Report ---\n// Raw Tokens: 1215 | Retained Tokens: 116 ...\n§PATHMAP\nα1 = src/app/user-list.component.ts",
    "v": 1,
    "file": "α1"
  }
}
```

### provide_code_context — FullCompress strategy

Same as above — LLM content in `content[0].text`, text pipeline + pathmap in `pretty`, hierarchical IR in `ir`.

### provide_code_context — DeltaTransport strategy

When delta mode is active:
- `content[0].text` = the delta wire format (as currently)
- `pretty` = **re-rendered hierarchical text from the merged IR** (fresh full picture), cached per-edit-session
- `ir` = current hierarchical IR JSON

The `"pretty"` field always includes the `§PATHMAP` section when applicable.

**Caching:** The rendered text is cached in `McpState.llm_text_cache: HashMap<String, String>`. When a delta is applied, the cache for that file is invalidated. On the next `provide_code_context` delta call, the renderer runs only once (cache miss), then subsequent calls use the cached value until the next edit.

---

## Architecture Diagram

```
                 ┌──────────────┐
                 │   Source     │
                 │   Code       │
                 └──────┬───────┘
                        │
              ┌─────────▼─────────┐
              │  tree-sitter      │
              │  Capture Pipeline │
              └─────────┬─────────┘
                        │
              ┌─────────▼─────────┐
              │  IRCompiler       │
              │  (4 layers)       │
              │  L1: Core IR      │
              │  L2: Language     │
              │  L3: Meta         │
              │  L4: Patterns     │
              └─────────┬─────────┘
                        │
                 ┌──────▼──────┐
                 │  Flat       │
                 │  CoreOp[]   │
                 └──────┬──────┘
                        │
              ┌─────────▼─────────┐
              │  ir_to_hierarchical│ (FIXED: pattern scoping by ID)
              └─────────┬─────────┘
                        │
              ┌─────────▼──────────┐
              │  render_for_llm()   │◄── NEW: stateless projection
              │                     │     (uses names, +N disambiguation)
              └─────────┬──────────┘
                        │
              ┌─────────▼──────────┐
              │  "content[0].text"  │──→ LLM context window
              │  "pretty"           │──→ Human debugging (cached)
              │  "ir"               │──→ Structured JSON
              └────────────────────┘
```

---

## Pre-Requisite Fixes (Required Before Phase 1)

### Fix A: Fix `ir_to_hierarchical()` Pattern Scoping

**File:** `src/ir/hierarchical.rs`

**Problem:** The `ir_to_hierarchical()` function assigns `CoreOp::Pattern` ops to the current class/method based on positional scope (last processed `current_class_idx` / `current_method_idx`). The `CompressingPatternRecognizer` may emit patterns at positions in the instruction stream that don't match their parent scope, causing patterns to be assigned to the wrong class.

**Fix:** Parse pattern args to find the correct parent by class/method ID instead of using positional scope:

```rust
CoreOp::Pattern(name, args) => {
    // Parse args to find class_id and method_id
    // PatternOp::to_tuple() format: [pattern_name, class_id?, method_id?, ...args]
    let class_id = args.first().cloned();
    let method_id = args.get(1).cloned();
    
    if let Some(cid) = class_id {
        if let Some(c_idx) = find_class_by_id(&classes, &cid) {
            if let Some(mid) = method_id {
                if let Some(m_idx) = classes[c_idx].methods.iter().position(|m| m.id == mid) {
                    classes[c_idx].methods[m_idx].patterns.push(PatternEntry { name, args });
                }
            } else {
                classes[c_idx].patterns.push(PatternEntry { name, args });
            }
        }
    }
}
```

**File:** `src/ir/patterns.rs`

Also ensure `PatternOp::to_tuple()` consistently emits `[class_id, method_id, ...args]` for method-level patterns and `[class_id, ...args]` for class-level patterns. Currently:
- Constructor: `["CTOR", class_id, method_id, dep1, ...]`
- Getter: `["GETTER", class_id, method_id, property]`
- Setter: `["SETTER", class_id, method_id, property]`
- Observable: `["OBSERVABLE", class_id, method_id, return_type]`
- Promise: `["PROMISE", class_id, method_id, return_type]`
- Override: `["OVERRIDE", class_id, method_id]`
- EmptyConstructor: `["EMPTY_CTOR", class_id, method_id]`

All have `class_id` at index 0 and `method_id` at index 1. The fix in `ir_to_hierarchical()` extracts these to find the correct class/method scope.

**Tests:** Add test cases with pattern-rich instruction streams to verify correct scope assignment.

---

### Fix B: Fix Method Overloading Disambiguation

**File:** `src/ir/render_llm.rs` (to be created)

**Problem:** When a class has multiple methods with the same name (Java overloading), the rendered text would produce duplicate `M methodName` lines, making them ambiguous.

**Fix:** In the renderer, when emitting methods for a class, count occurrences of each method name and append `+N` (parameter count) for duplicates:

```rust
let mut method_name_counts: HashMap<String, usize> = HashMap::new();
let mut method_name_index: HashMap<String, usize> = HashMap::new();

// First pass: count occurrences
for method in &class.methods {
    *method_name_counts.entry(method.name.clone()).or_insert(0) += 1;
}

// Second pass: emit with +N for duplicates
for method in &class.methods {
    let entry = method_name_index.entry(method.name.clone()).or_insert(0);
    *entry += 1;
    let count = method_name_counts[&method.name];
    
    if count > 1 {
        // Disambiguate with +N (parameter count)
        let param_count = method.params.len();
        emit!("M {}(+{})", method.name, param_count);
    } else {
        emit!("M {}", method.name);
    }
}
```

**Note:** This fix lives entirely in the renderer. No changes to the IR pipeline or hierarchical format are needed.

---

### Fix C: Add Version Stamp to Binary Wire Format

**File:** `src/ir/binary_wire.rs`

**Problem:** After Phase 2, all persisted Angular/Spring TYPE ops will use the abbreviated `@` format. Old persistence entries still use the long format (`NG_COMPONENT_Foo`). On replay, old and new deltas would produce inconsistent TYPE ops.

**Fix:** Add a single version byte to the beginning of `encode()` output:

```rust
/// Binary wire format version:
/// 0x01 = Original (long TYPE op names like "NG_COMPONENT_Foo")
/// 0x02 = Abbreviated (@-prefixed TYPE ops like "@cmp")
const BINARY_WIRE_VERSION: u8 = 0x02;

pub fn encode(ir: &CompiledIR) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(BINARY_WIRE_VERSION);
    // ... rest of encoding
}

pub fn decode(bytes: &[u8]) -> Option<CompiledIR> {
    if bytes.is_empty() { return None; }
    let version = bytes[0];
    match version {
        0x01 => decode_v1(&bytes[1..]),
        0x02 => decode_v2(&bytes[1..]),
        _ => None, // Unknown version → reject
    }
}
```

The `decode_v1` function handles the old format (backward compatible). The `decode_v2` function handles the new abbreviated format. If old persisted data is replayed, it gets decoded in the old format — no mixing.

**Tests:** Add round-trip tests for both V1 and V2 binary formats.

---

### Fix D: LLM Text Caching in McpState

**File:** `src/mcp/state.rs`

**Problem:** In delta mode, re-rendering the full hierarchical text on every call adds latency.

**Fix:** Add a `llm_text_cache: HashMap<String, String>` to `McpState`:

```rust
pub struct McpState {
    // ... existing fields
    /// Cache of rendered LLM text, keyed by path alias.
    /// Invalidated when a delta is applied to the file.
    pub llm_text_cache: HashMap<String, String>,
}
```

On `handle_provide_code_context` delta mode:
1. Check `llm_text_cache` for the file's path alias
2. If cache hit → use cached value for `"pretty"`
3. If cache miss → re-render, store in cache
4. When a delta is applied via `apply_delta`, remove the file from `llm_text_cache`

On `handle_restore_context`, clear the entire cache.

---

## Phases (Revised)

### Pre-Phase 0: Apply Pre-Requisite Fixes

| Fix | File | Description | Priority |
|-----|------|-------------|----------|
| Fix A | `ir/hierarchical.rs` | Fix pattern scoping by ID, not position | **BLOCKER** |
| Fix B | `ir/render_llm.rs` | Method overloading disambiguation (+N) | **BLOCKER** (in Phase 1) |
| Fix C | `ir/binary_wire.rs` | Version stamp for backward compat | **HIGH** |
| Fix D | `mcp/state.rs` | LLM text caching for delta mode | **MEDIUM** |

---

### Phase 1: Create `ir/render_llm.rs`

**Files:** `src/ir/render_llm.rs` (NEW), `src/ir/mod.rs` (add module)

**Purpose:** Convert `HierarchicalIR` → compact LLM-optimized text.

**Function signature:**
```rust
pub fn render_hierarchical_for_llm(hir: &HierarchicalIR) -> String
```

**Includes Fix B** (method overloading disambiguation): When a method name appears multiple times in the same class, append `+N` where N is the number of parameters.

**Behavior by fidelity:**
| Step | Low | Medium | High |
|------|-----|--------|------|
| Header | `// SCHEMA v2` | Same | Same |
| Class boundary | `// ── Name ──` | Same | Same |
| `@` annotations | Always | Always | Always |
| `X` / `I` | Always | Always | Always |
| `F` fields | Space-separated on one line | One per line | One per line |
| `M` method | `M name → fl:FLAGS` | `M name → p:x:$t → fl:FLAGS` | Same as Medium |
| `P` pattern | Always | Always | Always |
| `$` imports | Always | Always | Always |
| `T` type aliases | Always | Always | Always |
| `cl:` class flags | Shown | Shown | Shown |
| Overloaded `M` | `M name(+N)` | `M name(+N) → ...` | Same as Medium |
| Whitespace | Minimal | Readable | Readable |

**Alias ID exclusion:** The renderer uses class/method **names** only, never internal alias IDs (`C1`, `M1`). This ensures stable output across compiles. Overloaded methods use `+N` disambiguation (see Fix B).

**Tests:** `src/tests/ir/render_llm.rs` — covers all class types (TS, Rust, Java, Angular, Spring), all fidelity levels, edge cases (empty classes, no methods, no fields, no meta), and overloaded methods.

---

### Phase 2: Fix `ir_to_hierarchical()` Pattern Scoping

**File:** `src/ir/hierarchical.rs`

**Applies Fix A.** Update the `Pattern` op case in `ir_to_hierarchical()` to parse pattern args and look up the correct parent class/method by ID rather than relying on positional scope.

**Tests:** Add pattern-scoping test cases to `src/tests/ir/hierarchical.rs`.

---

### Phase 3: Fix Binary Wire Versioning

**File:** `src/ir/binary_wire.rs`

**Applies Fix C.** Add version stamp byte to `encode()`/`decode()`. Support V1 (long TYPE ops) for backward compat and V2 (abbreviated `@` ops).

**Tests:** Add `binary_wire::tests::round_trip_v1_v2_compat` and `binary_wire::tests::rejects_unknown_version`.

---

### Phase 4: Abbreviate Meta-Layer TYPE Ops

**Files:** `src/ir/layers/angular.rs`, `src/ir/layers/spring.rs`

Change the `TypeAlias` values from verbose strings to compact `@`-prefixed notation:

| Current | New | Rationale |
|---------|-----|-----------|
| `"NG_COMPONENT_Foo"` | `"@cmp"` | 16 chars → 4 chars, 75% reduction |
| `"NG_SEL_Foo"` | `"@sel"` | 12 chars → 4 chars |
| `"NG_SERVICE_Foo"` | `"@svc"` | 16 chars → 4 chars |
| `"NG_MODULE_Foo"` | `"@mod"` | 14 chars → 4 chars |
| `"NG_DIRECTIVE_Foo"` | `"@dir"` | 16 chars → 4 chars |
| `"NG_PIPE_Foo"` | `"@pipe"` | 12 chars → 5 chars |
| `"NG_INPUT_Foo_bar"` | `"@in"` | 16+ chars → 3 chars |
| `"NG_OUTPUT_Foo_bar"` | `"@out"` | 17+ chars → 4 chars |
| `"NG_MODEL_Foo_bar"` | `"@mdl"` | 16+ chars → 4 chars |
| `"SP_REST_Foo"` | `"@rest"` | 12 chars → 4 chars |
| `"SP_SERVICE_Foo"` | `"@svc"` | 13 chars → 4 chars |
| `"SP_REPO_Foo"` | `"@repo"` | 11 chars → 5 chars |
| `"SP_CTRL_Foo"` | `"@ctrl"` | 11 chars → 5 chars |
| `"SP_CFG_Foo"` | `"@cfg"` | 9 chars → 4 chars |
| `"SP_COMP_Foo"` | `"@comp"` | 11 chars → 5 chars |
| `"SP_BEAN_Foo_bar"` | `"@bean"` | 14+ chars → 5 chars |
| `"SP_AUTOWIRED_Foo_bar"` | `"@wire"` | 19+ chars → 5 chars |
| `"SP_VALUE_Foo_bar"` | `"@val"` | 14+ chars → 4 chars |
| `"SP_MAP_Foo"` | `"@map"` | 10 chars → 4 chars |

The second operand (the class name or metadata value) remains unchanged. The `@` prefix ensures these are visually distinct from structural ops, and the `// SCHEMA v2` header with `@=meta` disambiguates from TypeScript decorator syntax.

**Tests:** Update `src/tests/ir/layers/mod.rs`, Angular layer tests, and Spring layer tests to expect new abbreviated TypeAlias values.

---

### Phase 5: Update `ir/layers/angular.rs` — Abbreviated Ops

**File:** `src/ir/layers/angular.rs`

Change `parse_phi_line()` to emit abbreviated TypeAlias ops. Each Φ marker prefix maps directly to an `@` prefix:

| Φ prefix | `@` marker |
|----------|-----------|
| `cmp` | `@cmp` |
| `svc` | `@svc` |
| `mod` | `@mod` (+ `decl`/`imp`/`exp` stored as separate TYPE ops) |
| `dir` | `@dir` |
| `pipe` | `@pipe` |
| `in` | `@in` |
| `out` | `@out` |
| `injects` | Not emitted as `@` — constructor DI is captured by `P CTOR` pattern |
| `model` | `@mdl` |

The Angular meta-layer `extract` method currently returns TYPE ops with format `NG_COMPONENT_Foo = "Foo"`. After this change, it returns `@cmp = "Foo"`.

---

### Phase 6: Update `ir/layers/spring.rs` — Abbreviated Ops

**File:** `src/ir/layers/spring.rs`

Same pattern as Phase 5:

| Φ prefix | `@` marker |
|----------|-----------|
| `rest` | `@rest` (+ `@map` for mappings) |
| `svc` | `@svc` |
| `repo` | `@repo` |
| `ctrl` | `@ctrl` (+ `@map` for mappings) |
| `cfg` | `@cfg` |
| `comp` | `@comp` |
| `bean` | `@bean` |
| `autowired` | `@wire` |
| `value` | `@val` |

---

### Phase 7: Add LLM Text Caching to McpState

**File:** `src/mcp/state.rs`

**Applies Fix D.** Add `llm_text_cache: HashMap<String, String>` field. Wire invalidation on delta apply and restore.

---

### Phase 8: Rewrite MCP Handlers — IR-First Response

**Files:** `src/mcp/tool_handlers.rs`, `src/mcp/tools.rs`, `src/mcp/tool_helpers.rs`

**`handle_compress_code_context` changes:**
1. Compile to IR as before (`compile_file_ir`)
2. Store IR in context state (unchanged)
3. Convert flat IR → hierarchical via `ir_to_hierarchical()`
4. Render hierarchical → compact text via `render_hierarchical_for_llm()`
5. Set compact text as `content[0].text`
6. Set old text pipeline output as `"pretty"` (includes pathmap)
7. Set hierarchical JSON as `"ir"` field
8. The `"ir"` wire field uses hierarchical encoding (not positional/named)
9. Store rendered text in `llm_text_cache` for delta mode

**`handle_provide_code_context` changes:**
- **FullCompress**: Same transformation as above
- **DeltaTransport**: After computing delta, check `llm_text_cache`. If cache hit → skip re-render; if cache miss → re-render and cache. Set in `"pretty"` field. Content stays as delta wire format.

**New helper in `tool_helpers.rs`:**
```rust
pub fn render_file_ir_for_llm(
    file_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<String, Box<dyn std::error::Error>>
```
Orchestrates: read source → compile_ir → ir_to_hierarchical → render_hierarchical_for_llm

**Tool schema updates:**
- `compress_code_context` description updated to document IR-first content
- `provide_code_context` description updated
- All tool descriptions mention `"pretty"` for debug output

---

### Phase 9: Regression & Backward-Compat Tests

**Files:** `src/tests/mcp/tools.rs`, `src/tests/ir/render_llm.rs` (NEW)

**Regression tests:**
1. `compress_code_context` response still has `content`, `"ir"`, `"file"`, `"v"` fields
2. `content[0].text` is present and non-empty
3. `"pretty"` field is present and contains pathmap marker `§PATHMAP`
4. `"ir"` field has `"encoding": "hierarchical"`
5. All existing layer integration tests still pass
6. Delta transport still returns valid deltas
7. Binary wire V1/V2 round-trip compatibility

**New render tests:**
1. All class types render correctly (TS, Rust, Java, Angular, Spring)
2. All fidelity levels produce expected verbosity
3. Empty class produces `// ── Name ──` with no children
4. Class with only meta annotations
5. Class with extends + implements
6. Method with params + flags + patterns
7. Overloaded methods produce `+N` disambiguation
8. Workspace output with multiple files

---

### Phase 10: Expand Micro-Opcode Table (Lower Priority)

**File:** `src/compression/micro_opcodes.rs`

Add micro-opcodes for the text pipeline (used only for `"pretty"` output):

| Opcode | Pattern | Replacement | Purpose |
|--------|---------|-------------|---------|
| `§I` | `⊕guard` | `§I` | Condition/if marker |
| `§L` | `⊕loop` | `§L` | Loop marker |
| `§E` | `⊕⇒` | `§E` | Throw/Return marker |

These reduce `"pretty"` output size but have no impact on the LLM-consumed content. This phase is deferred until all higher-priority phases are complete and stable.

---

## Design Decisions

### D1: Why not inject `@` ops directly into the text pipeline?
The text pipeline output format uses `$c`, `⊕guard`, `Φcmp:` as prefix-style markers. Adding `@` there would create format confusion. The IR pipeline is the correct place for structural annotations. The `@` prefix is an IR-level concern, emitted as `CoreOp::TypeAlias` and rendered by the hierarchical renderer.

### D2: Why use `$` for imports instead of `I`?
`I` is already used for `Implements`. Using `$` is consistent with the existing text pipeline notation (`$im`, `$fm`) and avoids ambiguity.

### D3: Why remove alias IDs from the rendered text?
Alias IDs (`C1`, `M1`) are internal compiler counters reset per compile. A file edited between calls could see different IDs for the same class. Class/method names are stable identifiers that the LLM can reference across sessions. The hierarchical JSON in the `"ir"` field still contains the IDs for structured processing. Overloaded methods use `+N` disambiguation instead.

### D4: How does `"pretty"` work in delta mode?
In delta transport, the client receives a diff instead of full content. The server still has the merged IR state internally. The `"pretty"` field is re-rendered from the **current** merged state (after applying the newest delta). The rendered text is **cached** in `McpState.llm_text_cache` so it's only re-rendered once per edit session.

### D5: Is there a version migration path?
Old clients that read `content[0].text` will see the new hierarchical format instead of the old text format. The content is still valid text — just structurally different. Old clients that read the `"ir"` field will see hierarchical JSON instead of positional JSON. The new `"encoding": "hierarchical"` field allows clients to detect the format. The `"pretty"` field preserves the old format for debugging.

### D6: Why fix pattern scoping and binary wire versioning before abbreviating TYPE ops?
The pattern scoping fix (Fix A) ensures patterns are correctly assigned to classes/methods regardless of the instruction stream ordering. Without this fix, the hierarchical renderer could show patterns under the wrong class — making the LLM see incorrect structural relationships. The binary wire versioning (Fix C) ensures that persisted data from before the TYPE op change can still be decoded. Without this fix, replaying old sessions after the abbreviation change would produce corrupted TYPE ops.

---

## Implementation Order (Revised)

```
Phase 0a: Fix A - Pattern scoping in ir_to_hierarchical()       ← BLOCKER
Phase 0b: Fix C - Binary wire version stamp                     ← HIGH
Phase 1:  Create ir/render_llm.rs (includes Fix B disambiguation)
Phase 2:  Abbreviate meta-layer TYPE ops (angular + spring)
Phase 3:  Update angular.rs + spring.rs layers
Phase 4:  Fix D - LLM text caching in McpState
Phase 5:  Rewrite MCP handlers (flip architecture)
Phase 6:  Regression & backward-compat tests
Phase 7:  Micro-opcode expansion (optional, deferred)
```

---

## Testing Strategy

| Test Suite | What it covers | Phase |
|-----------|---------------|-------|
| `ir::hierarchical::tests::*` | Pattern scoping by ID (Fix A) | Phase 0a |
| `ir::binary_wire::tests::*` | V1/V2 round-trip compat (Fix C) | Phase 0b |
| `ir::render_llm::tests::*` | Hierarchical IR → compact text, +N disambiguation | Phase 1 |
| `ir::layers::angular::tests::*` | Abbreviated TYPE ops | Phase 2 |
| `ir::layers::spring::tests::*` | Abbreviated TYPE ops | Phase 2 |
| `ir::tests::layers_integration::*` | Full IR pipeline with new TYPE ops | Phase 2 |
| `mcp::state::tests::*` | LLM text cache (Fix D) | Phase 4 |
| `mcp::tools::tests::*` | Response format, field presence | Phase 5 |
| `mcp::tool_helpers::tests::*` | `render_file_ir_for_llm()` helper | Phase 5 |
| `mcp::regression::*` | Backward compatibility | Phase 6 |
| Existing test suite (1224 tests) | No regressions | Every phase |

---

## Known Limitations

1. **Cross-file alias resolution** — The `resolve_forward_aliases()` function only handles within-file references. When class `Foo` in file `α1` extends class `Bar` in file `α2`, the IR does not yet resolve this across files. The workspace-level alias resolution is tracked as a future enhancement (post-Phase 7).
2. **`"pretty"` field size in delta mode** — The re-rendered full text in delta mode is the same size as a full compress. This is acceptable because `"pretty"` is optional debugging output — clients can skip reading it. The caching (Fix D) ensures it's only rendered once per edit session.
3. **Schema versioning** — The `// SCHEMA v2` header is a simple version tag. Future schema changes will increment this and update the legend table. No automated migration will be attempted — each version is self-describing via the header.
4. **Pattern+TypeAlias ordering in rendered text** — The renderer emits patterns (`P`) and type aliases (`T`, `@`) in the order they appear in the `HierarchicalIR`, which follows the original instruction stream order. Patterns and type aliases emitted by the Angular/Spring meta layers (which run after the main compile loop) will appear after all structural instructions. This is acceptable because the LLM can reference any symbol regardless of document position.