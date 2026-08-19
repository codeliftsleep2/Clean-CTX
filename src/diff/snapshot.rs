// src/diff/snapshot.rs
//
// Data structures representing a structural snapshot of a source file.

/// Per-file structural snapshot. Two snapshots can be diffed to produce
/// an AST-level change-set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapturedStructure {
    pub imports: Vec<String>,
    pub classes: Vec<CapturedClass>,
    /// Fields that appeared outside of any class context.
    pub orphan_fields: Vec<String>,
    /// Methods/functions declared at top level (outside any class).
    /// Without this, TypeScript files with only top-level functions
    /// produced zero methods — any change to them was a false negative.
    pub orphan_methods: Vec<CapturedMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedClass {
    /// Display name (e.g. "FooService" or "FooService:BaseService,IFoo").
    pub name: String,
    /// Class-level metadata: base class / interface list (e.g.
    /// `: BaseService, IFoo`). Captured separately from `name` so a
    /// change to the inheritance list is detected even when the class
    /// name itself is unchanged. F-04 diff audit: previously the base
    /// class / interface list was stripped by `extract_class_name` and
    /// lost before the diff ran, so changing `class Foo : BaseA` to
    /// `class Foo : BaseB` reported the class as unchanged.
    pub class_meta: String,
    pub fields: Vec<String>,
    pub methods: Vec<CapturedMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedMethod {
    /// The compact signature text (e.g. "process(id:string):boolean").
    pub sig: String,
    /// Behavior markers attached to the method (⊕guard, ⊕loop, etc.).
    pub markers: Vec<String>,
    /// Normalized method body text (whitespace-collapsed). `None` when the
    /// body could not be extracted (e.g. abstract/interface methods, or
    /// test fixtures that don't set it).
    ///
    /// This field exists so the diff comparator can detect **body-only**
    /// changes (logic fixes) that leave the signature and markers
    /// untouched. Previously `diff_snapshots` compared only `sig` and
    /// `markers`, so a method whose body changed was reported as
    /// `Unchanged` — a false negative for `diff_commits`.
    pub body: Option<String>,
}