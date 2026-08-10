// src/angular_meta/markers.rs
//
// `Φ` (Phi) marker construction & expansion.
//
// # Mirrors `compression::markers` shape
//
// Just like the `⊕` behavior markers (`⊕guard`, `⊕loop`, `⊕⇒`, `⊕!`)
// the compression pipeline emits a marker token and the decompressor
// expands it back to a human-readable form, the `Φ` meta markers
// (`Φcmp:`, `Φsvc:`, `Φin:`, `Φout:`, …) are a parallel vocabulary
// for framework-annotation. They are:
//   - emitted by `run_meta_layer` after the compacted class entry
//   - collapsed by `expand_phi_in_line` on decompression
//
// # Marker vocabulary (single source of truth: [`PhiLineKind`])
//
// | Marker    | Expansion (Phase 1)                          |
// |-----------|----------------------------------------------|
// | Φcmp:     | @Component                                   |
// | Φdir:     | @Directive                                   |
// | Φpipe:    | @Pipe                                        |
// | Φsvc:     | @Injectable                                  |
// | Φmod:     | @NgModule                                    |
// | Φin:      | @Input  (or `input()` signal with ` signal` suffix) |
// | Φout:     | @Output (or `output()` signal with ` signal` suffix) |
// | Φmodel:   | model() signal (two-way binding signal)      |
// | Φinjects: | constructor injection (private/protected)   |
// | Φgraph:   | cross-file dependency graph (Phase 3 only)   |
// | ΦBUNDLE   | file-triplet bundle (Phase 2 only)           |
// | ΦMAP      | workspace meta-map footer (Phase 2 only)     |
// | Φtpl:     | template extraction                          |
// | Φsty:     | style extraction                             |
//
// # Why a single-character prefix?
//
// `Φ` (U+03A6) is visually distinct from `$` (opcode), `⊕` (behavior),
// and `α/β/γ` (path). It cannot be confused for an identifier in
// TypeScript / JavaScript / C# source, so the markers are safe to
// interleave with the existing notation without escaping.
//
// # Adding a new marker
//
// 1. Add a variant to [`PhiLineKind`] (the single source of truth).
// 2. Add the `marker_prefix` and `expansion` arms in the `PhiLineKind` impl.
// 3. If the marker has a builder, create a struct + [`PhiLine`] impl.
// 4. Add a thin `build_*` wrapper if needed for call-site convenience.
//
// The `expand_phi_in_line` and `expand_phi` functions are **generic** over
// `PhiLineKind` and need no manual updates.

// ---------------------------------------------------------------------------
// PhiLineKind — single source of truth for the marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind. This enum is the **single source of truth**
/// for the marker vocabulary — adding a new marker means adding one variant
/// here plus its `marker_prefix` / `expansion` arms (and optionally a
/// [`PhiLine`] impl if it has a builder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhiLineKind {
    Component,
    Service,
    Module,
    Directive,
    Pipe,
    Input,
    Output,
    Model,
    Injects,
    Graph,
    Bundle,
    Map,
    Template,
    Style,
    // --- ANGULAR_HTML_COMPRESSION_PLAN: template detail markers ---
    /// `[prop]="expr"` property binding on a template element.
    TemplateBinding,
    /// `*ngIf`, `*ngFor`, etc. structural directive on a template element.
    TemplateDirective,
    /// Custom element tag with its inputs/outputs (e.g. `<app-user-card>`).
    TemplateComponent,
}

impl PhiLineKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φcmp:"`).
    /// For prefix-less tokens (`ΦBUNDLE`, `ΦMAP`) the colon is omitted.
    pub fn marker_prefix(self) -> &'static str {
        match self {
            Self::Component => "Φcmp:",
            Self::Service => "Φsvc:",
            Self::Module => "Φmod:",
            Self::Directive => "Φdir:",
            Self::Pipe => "Φpipe:",
            Self::Input => "Φin:",
            Self::Output => "Φout:",
            Self::Model => "Φmodel:",
            Self::Injects => "Φinjects:",
            Self::Graph => "Φgraph:",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
            Self::Template => "Φtpl:",
            Self::Style => "Φsty:",
            Self::TemplateBinding => "Φtbind:",
            Self::TemplateDirective => "Φtdir:",
            Self::TemplateComponent => "Φtcmp:",
        }
    }

    /// The human-readable expansion (e.g. `"@Component"`). Does NOT
    /// include the trailing space that `expand_phi_in_line` adds.
    pub fn expansion(self) -> &'static str {
        match self {
            Self::Component => "@Component",
            Self::Service => "@Injectable",
            Self::Module => "@NgModule",
            Self::Directive => "@Directive",
            Self::Pipe => "@Pipe",
            Self::Input => "@Input",
            Self::Output => "@Output",
            Self::Model => "@Model",
            Self::Injects => "@Inject",
            Self::Graph => "@Graph",
            Self::Bundle => "@Bundle",
            Self::Map => "@Map",
            Self::Template => "@Template",
            Self::Style => "@Style",
            Self::TemplateBinding => "@TemplateBinding",
            Self::TemplateDirective => "@TemplateDirective",
            Self::TemplateComponent => "@TemplateComponent",
        }
    }

    /// All variants in a canonical order. Longer prefixes are placed
    /// before shorter ones to prevent partial-match issues in string
    /// replacement (defensive — the current vocabulary has no overlaps,
    /// but this ordering is cheap insurance).
    pub fn all_in_expand_order() -> &'static [PhiLineKind] {
        &[
            Self::Injects,    // Φinjects:  (9 chars)
            Self::Component,  // Φcmp:      (5 chars)
            Self::Directive,  // Φdir:      (5 chars)
            Self::Module,     // Φmod:      (5 chars)
            Self::Pipe,       // Φpipe:     (6 chars)
            Self::Service,    // Φsvc:      (5 chars)
            Self::Model,      // Φmodel:    (7 chars)
            Self::Input,      // Φin:       (4 chars)
            Self::Output,     // Φout:      (5 chars)
            Self::Graph,      // Φgraph:    (7 chars)
            Self::Template,        // Φtpl:      (5 chars)
            Self::Style,           // Φsty:      (5 chars)
            Self::TemplateBinding, // Φtbind:    (7 chars)
            Self::TemplateDirective, // Φtdir:    (6 chars)
            Self::TemplateComponent, // Φtcmp:    (6 chars)
            Self::Bundle,          // ΦBUNDLE    (8 chars)
            Self::Map,             // ΦMAP       (5 chars)
        ]
    }

    /// Look up a [`PhiLineKind`] by its marker token string (without
    /// the trailing colon/binding). Returns `None` for unknown tokens.
    pub fn from_token(token: &str) -> Option<PhiLineKind> {
        match token {
            "Φcmp" => Some(Self::Component),
            "Φdir" => Some(Self::Directive),
            "Φpipe" => Some(Self::Pipe),
            "Φsvc" => Some(Self::Service),
            "Φmod" => Some(Self::Module),
            "Φin" => Some(Self::Input),
            "Φout" => Some(Self::Output),
            "Φmodel" => Some(Self::Model),
            "Φinjects" => Some(Self::Injects),
            "Φgraph" => Some(Self::Graph),
            "ΦBUNDLE" => Some(Self::Bundle),
            "ΦMAP" => Some(Self::Map),
            "Φtpl" => Some(Self::Template),
            "Φsty" => Some(Self::Style),
            "Φtbind" => Some(Self::TemplateBinding),
            "Φtdir" => Some(Self::TemplateDirective),
            "Φtcmp" => Some(Self::TemplateComponent),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    /// Used by tests (`src/tests/angular_meta/markers.rs`).
    #[allow(dead_code)]
    pub fn token(self) -> &'static str {
        match self {
            Self::Component => "Φcmp",
            Self::Service => "Φsvc",
            Self::Module => "Φmod",
            Self::Directive => "Φdir",
            Self::Pipe => "Φpipe",
            Self::Input => "Φin",
            Self::Output => "Φout",
            Self::Model => "Φmodel",
            Self::Injects => "Φinjects",
            Self::Graph => "Φgraph",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
            Self::Template => "Φtpl",
            Self::Style => "Φsty",
            Self::TemplateBinding => "Φtbind",
            Self::TemplateDirective => "Φtdir",
            Self::TemplateComponent => "Φtcmp",
        }
    }
}

// ---------------------------------------------------------------------------
// PhiLine trait — each marker type renders itself
// ---------------------------------------------------------------------------

/// A marker line that can render itself to its `Φ…` string form.
///
/// Implementing this trait for a marker struct makes the marker's
/// formatting self-contained. The `build_*` free functions become thin
/// wrappers that create the struct and call `.render()`.
pub trait PhiLine {
    /// The kind of this marker.
    /// Used by tests to verify marker round-trips.
    #[allow(dead_code)]
    fn kind(&self) -> PhiLineKind;

    /// Render the full marker line (e.g. `"Φcmp:Foo sel=app-foo"`).
    fn render(&self) -> String;
}

// ---------------------------------------------------------------------------
// ComponentFields — parsed fields of a @Component decorator
// ---------------------------------------------------------------------------

/// Parsed fields of a `@Component({...})` decorator object. All
/// fields are optional; missing fields are simply not emitted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentFields {
    pub selector: Option<String>,
    pub template_url: Option<String>,
    pub template: Option<String>,
    pub style_urls: Option<Vec<String>>,
    pub styles: Option<String>,
}

// ---------------------------------------------------------------------------
// Marker structs + PhiLine impls
// ---------------------------------------------------------------------------

/// A `Φcmp:<ClassName> [sel=… tpl=… sty=…]` marker line.
pub struct ComponentLine<'a> {
    pub class_name: &'a str,
    pub fields: &'a ComponentFields,
}

impl PhiLine for ComponentLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Component
    }

    fn render(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(sel) = &self.fields.selector {
            parts.push(format!("sel={}", sel));
        }
        if let Some(tpl) = &self.fields.template_url {
            parts.push(format!("tpl={}", tpl));
        } else if let Some(tpl) = &self.fields.template {
            parts.push(format!("tpl={}", tpl));
        }
        if let Some(sty) = &self.fields.style_urls {
            if sty.len() == 1 {
                parts.push(format!("sty={}", sty[0]));
            } else if !sty.is_empty() {
                parts.push(format!("sty=[{}]", sty.join(",")));
            }
        } else if let Some(sty) = &self.fields.styles {
            parts.push(format!("sty={}", sty));
        }
        if parts.is_empty() {
            format!("Φcmp:{}", self.class_name)
        } else {
            format!("Φcmp:{} {}", self.class_name, parts.join(" "))
        }
    }
}

/// A `Φsvc:<ClassName> [scope=…]` marker line.
pub struct ServiceLine<'a> {
    pub class_name: &'a str,
    pub provided_in: Option<&'a str>,
}

impl PhiLine for ServiceLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Service
    }

    fn render(&self) -> String {
        match self.provided_in {
            Some(scope) => format!("Φsvc:{} scope={}", self.class_name, scope),
            None => format!("Φsvc:{}", self.class_name),
        }
    }
}

/// A `Φmod:<ClassName> [decl=… imp=… exp=…]` marker line.
pub struct ModuleLine<'a> {
    pub class_name: &'a str,
    pub decl: &'a [String],
    pub imp: &'a [String],
    pub exp: &'a [String],
}

impl PhiLine for ModuleLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Module
    }

    fn render(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.decl.is_empty() {
            parts.push(format!("decl=[{}]", self.decl.join(",")));
        }
        if !self.imp.is_empty() {
            parts.push(format!("imp=[{}]", self.imp.join(",")));
        }
        if !self.exp.is_empty() {
            parts.push(format!("exp=[{}]", self.exp.join(",")));
        }
        if parts.is_empty() {
            format!("Φmod:{}", self.class_name)
        } else {
            format!("Φmod:{} {}", self.class_name, parts.join(" "))
        }
    }
}

/// A `Φdir:<ClassName> [sel=…]` marker line.
pub struct DirectiveLine<'a> {
    pub class_name: &'a str,
    pub selector: Option<&'a str>,
}

impl PhiLine for DirectiveLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Directive
    }

    fn render(&self) -> String {
        match self.selector {
            Some(sel) => format!("Φdir:{} sel={}", self.class_name, sel),
            None => format!("Φdir:{}", self.class_name),
        }
    }
}

/// A `Φpipe:<ClassName> [name=…]` marker line.
pub struct PipeLine<'a> {
    pub class_name: &'a str,
    pub name: Option<&'a str>,
}

impl PhiLine for PipeLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Pipe
    }

    fn render(&self) -> String {
        match self.name {
            Some(n) => format!("Φpipe:{} name={}", self.class_name, n),
            None => format!("Φpipe:{}", self.class_name),
        }
    }
}

/// A `Φin:<fieldName> [alias=…]` marker line.
pub struct InputLine<'a> {
    pub field_name: &'a str,
    pub alias: Option<&'a str>,
}

impl PhiLine for InputLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Input
    }

    fn render(&self) -> String {
        match self.alias {
            Some(a) => format!("Φin:{} alias={}", self.field_name, a),
            None => format!("Φin:{}", self.field_name),
        }
    }
}

/// A `Φout:<fieldName> [alias=…]` marker line.
pub struct OutputLine<'a> {
    pub field_name: &'a str,
    pub alias: Option<&'a str>,
}

impl PhiLine for OutputLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Output
    }

    fn render(&self) -> String {
        match self.alias {
            Some(a) => format!("Φout:{} alias={}", self.field_name, a),
            None => format!("Φout:{}", self.field_name),
        }
    }
}

/// A `Φmodel:<fieldName> [alias=…]` marker line for a `model()` signal
/// field (Angular 17.1+). Models are two-way binding signals that bridge
/// input/output behavior.
///
/// The alias parameter refers to the public-facing binding name, which
/// may differ from the field name if the signal was created with one.
pub struct ModelLine<'a> {
    pub field_name: &'a str,
    pub alias: Option<&'a str>,
}

impl PhiLine for ModelLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Model
    }

    fn render(&self) -> String {
        match self.alias {
            Some(a) => format!("Φmodel:{} alias={}", self.field_name, a),
            None => format!("Φmodel:{}", self.field_name),
        }
    }
}

/// A `Φinjects:[<Type>,…]` marker line for constructor parameters with
/// `private` / `protected` access modifiers.
pub struct InjectsLine<'a> {
    pub types: &'a [String],
}

impl PhiLine for InjectsLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Injects
    }

    fn render(&self) -> String {
        format!("Φinjects:[{}]", self.types.join(","))
    }
}

// ---------------------------------------------------------------------------
// build_* free functions — thin wrappers preserving the public API
// ---------------------------------------------------------------------------

/// Build a `Φcmp:<ClassName> [sel=… tpl=… sty=…]` marker line from
/// the parsed fields of a `@Component` decorator object.
///
/// Unknown fields are silently dropped (forward-compat with future
/// Angular versions). The class name is always emitted even if no
/// other fields were parsed.
pub fn build_component_line(class_name: &str, fields: &ComponentFields) -> String {
    ComponentLine { class_name, fields }.render()
}

/// Build a `Φsvc:<ClassName> [scope=…]` marker line from a parsed
/// `@Injectable` decorator.
pub fn build_service_line(class_name: &str, provided_in: Option<&str>) -> String {
    ServiceLine { class_name, provided_in }.render()
}

/// Build a `Φmod:<ClassName> [decl=… imp=… exp=…]` marker line from a
/// parsed `@NgModule` decorator.
pub fn build_module_line(class_name: &str, decl: &[String], imp: &[String], exp: &[String]) -> String {
    ModuleLine { class_name, decl, imp, exp }.render()
}

/// Build a `Φdir:<ClassName> [sel=…]` marker line from a parsed
/// `@Directive` decorator.
pub fn build_directive_line(class_name: &str, selector: Option<&str>) -> String {
    DirectiveLine { class_name, selector }.render()
}

/// Build a `Φpipe:<ClassName> [name=…]` marker line from a parsed
/// `@Pipe` decorator.
pub fn build_pipe_line(class_name: &str, name: Option<&str>) -> String {
    PipeLine { class_name, name }.render()
}

/// Build a `Φin:<fieldName> [alias=…]` marker line for an `@Input()`
/// field.
pub fn build_input_line(field_name: &str, alias: Option<&str>) -> String {
    InputLine { field_name, alias }.render()
}

/// Build a `Φout:<fieldName> [alias=…]` marker line for an `@Output()`
/// field.
pub fn build_output_line(field_name: &str, alias: Option<&str>) -> String {
    OutputLine { field_name, alias }.render()
}

/// Build a `Φmodel:<fieldName> [alias=…]` marker line for a `model()`
/// signal field (Angular 17.1+).
pub fn build_model_line(field_name: &str, alias: Option<&str>) -> String {
    ModelLine { field_name, alias }.render()
}

/// Build a `Φinjects:[<Type>,…]` marker line for constructor
/// parameters with `private` / `protected` access modifiers. The
/// Phase 1 implementation emits class names only (no `α` aliases yet
/// — that is Phase 3).
pub fn build_injects_line(types: &[String]) -> String {
    InjectsLine { types }.render()
}

// ---------------------------------------------------------------------------
// Expansion — generic over PhiLineKind (no manual table updates needed)
// ---------------------------------------------------------------------------

/// Expand every recognised `Φ…` marker in a line back to its
/// decorator form. Used by the decompressor.
///
/// This is the round-trip counterpart to the `build_*` helpers
/// above. It is **deliberately conservative**: only the marker
/// prefix is rewritten (`Φcmp:` → `@Component`); the trailing
/// `key=value` attributes are left untouched because they are
/// already human-readable in the compressed form.
///
/// Unknown `Φ` tokens (e.g. a future Phase marker) are passed
/// through unchanged.
///
/// Adding a new marker to the vocabulary only requires updating
/// [`PhiLineKind`] — this function is generic and needs no edits.
pub fn expand_phi_in_line(line: &str) -> String {
    let mut s = line.to_string();
    for &kind in PhiLineKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        if s.contains(prefix) {
            // Expansion includes a trailing space so that
            // `Φcmp:Foo` → `@Component Foo` (space separates
            // decorator from name).
        s = s.replace(prefix, &format!("{} ", kind.expansion()));
        }
    }
    // Chain the registered sub-layer expansions (RxJS, NgRx, Signals,
    // Routing) via the [`PHI_EXPANDERS`](crate::angular_meta::phi::PHI_EXPANDERS)
    // registry. The Angular Ecosystem Deepening markers are block-scoped
    // to their own `// --- Φ … Meta ---` sections, so they never collide
    // with the Angular decorator markers above.
    //
    // Adding a new sub-layer (e.g. React) only requires registering its
    // `expand_phi_in_line` in `phi.rs` — no edit needed here.
    for expander in crate::angular_meta::phi::PHI_EXPANDERS {
        s = expander(&s);
    }
    s
}

/// Expand a single `Φ` marker token to its decorator form. Returns
/// `None` for unknown markers so the caller can pass them through.
///
/// Adding a new marker to the vocabulary only requires updating
/// [`PhiLineKind`] — this function is generic and needs no edits.
/// Used by tests (`src/tests/angular_meta/markers.rs`).
#[allow(dead_code)]
pub fn expand_phi(token: &str) -> Option<&'static str> {
    PhiLineKind::from_token(token).map(|k| k.expansion())
}

#[cfg(test)]
#[path = "../tests/angular_meta/markers.rs"]
mod tests;