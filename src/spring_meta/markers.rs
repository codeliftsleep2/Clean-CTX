// src/spring_meta/markers.rs
//
// `Φ` (Phi) marker construction & expansion for Spring Boot.
//
// # Mirrors `angular_meta::markers` shape
//
// Just like the Angular Meta-Layer emits `Φ` markers for decorators,
// the Spring Boot Meta-Layer emits `Φ` markers for Spring annotations.
// They are:
//   - emitted by `run_meta_layer` after the compacted class entry
//   - collapsed by `expand_phi_in_line` on decompression
//
// # Marker vocabulary (single source of truth: [`PhiLineKind`])
//
// | Marker    | Expansion (Phase 1)                          |
// |-----------|----------------------------------------------|
// | Φrest:    | @RestController                              |
// | Φctrl:    | @Controller                                  |
// | Φsvc:     | @Service                                     |
// | Φrepo:    | @Repository                                  |
// | Φconf:    | @Configuration                               |
// | Φmap:     | @RequestMapping / @GetMapping / etc.         |
// | Φaut:     | @Autowired (field-level)                     |
// | Φval:     | @Value (field-level)                         |
// | Φbean:    | @Bean (method-level)                         |
// | Φprop:    | @ConfigurationProperties                     |
// | Φgraph:   | cross-file dependency graph (Phase 3 only)   |
// | ΦBUNDLE   | layer bundle (Phase 2 only)                  |
// | ΦMAP      | workspace meta-map footer (Phase 2 only)     |
// | Φpropf:   | properties file extraction (Phase 2 only)    |
//
// # Why a single-character prefix?
//
// `Φ` (U+03A6) is visually distinct from `$` (opcode), `⊕` (behavior),
// and `α/β/γ` (path). It cannot be confused for an identifier in
// Java source, so the markers are safe to interleave with the existing
// notation without escaping.

// ---------------------------------------------------------------------------
// PhiLineKind — single source of truth for the marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind. This enum is the **single source of truth**
/// for the marker vocabulary — adding a new marker means adding one variant
/// here plus its `marker_prefix` / `expansion` arms (and optionally a
/// [`PhiLine`] impl if it has a builder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhiLineKind {
    RestController,
    Controller,
    Service,
    Repository,
    Configuration,
    RequestMapping,
    Autowired,
    Value,
    Bean,
    ConfigurationProperties,
    Graph,
    Bundle,
    Map,
    PropertiesFile,
}

impl PhiLineKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φrest:"`).
    /// For prefix-less tokens (`ΦBUNDLE`, `ΦMAP`) the colon is omitted.
    /// Used by `expand_phi_in_line` (production when `spring_boot` is
    /// enabled) and by tests (`src/tests/spring_meta/markers_tests.rs`).
    #[cfg_attr(not(feature = "spring_boot"), allow(dead_code))]
    pub fn marker_prefix(self) -> &'static str {
        match self {
            Self::RestController => "Φrest:",
            Self::Controller => "Φctrl:",
            Self::Service => "Φsvc:",
            Self::Repository => "Φrepo:",
            Self::Configuration => "Φconf:",
            Self::RequestMapping => "Φmap:",
            Self::Autowired => "Φaut:",
            Self::Value => "Φval:",
            Self::Bean => "Φbean:",
            Self::ConfigurationProperties => "Φprop:",
            Self::Graph => "Φgraph:",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
            Self::PropertiesFile => "Φpropf:",
        }
    }

    /// The human-readable expansion (e.g. `"@RestController"`). Does NOT
    /// include the trailing space that `expand_phi_in_line` adds.
    pub fn expansion(self) -> &'static str {
        match self {
            Self::RestController => "@RestController",
            Self::Controller => "@Controller",
            Self::Service => "@Service",
            Self::Repository => "@Repository",
            Self::Configuration => "@Configuration",
            Self::RequestMapping => "@RequestMapping",
            Self::Autowired => "@Autowired",
            Self::Value => "@Value",
            Self::Bean => "@Bean",
            Self::ConfigurationProperties => "@ConfigurationProperties",
            Self::Graph => "@Graph",
            Self::Bundle => "@Bundle",
            Self::Map => "@Map",
            Self::PropertiesFile => "@PropertiesFile",
        }
    }

    /// All variants in a canonical order. Longer prefixes are placed
    /// before shorter ones to prevent partial-match issues in string
    /// replacement (defensive — the current vocabulary has no overlaps,
    /// but this ordering is cheap insurance).
    /// Used by `expand_phi_in_line` and by tests.
    #[cfg_attr(not(feature = "spring_boot"), allow(dead_code))]
    pub fn all_in_expand_order() -> &'static [PhiLineKind] {
        &[
            Self::RestController,          // Φrest:   (6 chars)
            Self::Controller,              // Φctrl:   (6 chars)
            Self::Repository,              // Φrepo:   (6 chars)
            Self::Configuration,           // Φconf:   (6 chars)
            Self::RequestMapping,          // Φmap:    (5 chars)
            Self::Service,                 // Φsvc:    (5 chars)
            Self::Autowired,               // Φaut:    (5 chars)
            Self::Value,                   // Φval:    (5 chars)
            Self::Bean,                    // Φbean:   (6 chars)
            Self::ConfigurationProperties, // Φprop: (6 chars)
            Self::Graph,                   // Φgraph:  (7 chars)
            Self::PropertiesFile,          // Φpropf:  (7 chars)
            Self::Bundle,                  // ΦBUNDLE  (8 chars)
            Self::Map,                     // ΦMAP     (5 chars)
        ]
    }

    /// Look up a [`PhiLineKind`] by its marker token string (without
    /// the trailing colon/binding). Returns `None` for unknown tokens.
    /// Consumed only by `expand_phi` below, which is test-facing.
    #[allow(dead_code)]
    pub fn from_token(token: &str) -> Option<PhiLineKind> {
        match token {
            "Φrest" => Some(Self::RestController),
            "Φctrl" => Some(Self::Controller),
            "Φsvc" => Some(Self::Service),
            "Φrepo" => Some(Self::Repository),
            "Φconf" => Some(Self::Configuration),
            "Φmap" => Some(Self::RequestMapping),
            "Φaut" => Some(Self::Autowired),
            "Φval" => Some(Self::Value),
            "Φbean" => Some(Self::Bean),
            "Φprop" => Some(Self::ConfigurationProperties),
            "Φgraph" => Some(Self::Graph),
            "ΦBUNDLE" => Some(Self::Bundle),
            "ΦMAP" => Some(Self::Map),
            "Φpropf" => Some(Self::PropertiesFile),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    /// Used by tests (`src/tests/spring_meta/markers_tests.rs`).
    #[allow(dead_code)]
    pub fn token(self) -> &'static str {
        match self {
            Self::RestController => "Φrest",
            Self::Controller => "Φctrl",
            Self::Service => "Φsvc",
            Self::Repository => "Φrepo",
            Self::Configuration => "Φconf",
            Self::RequestMapping => "Φmap",
            Self::Autowired => "Φaut",
            Self::Value => "Φval",
            Self::Bean => "Φbean",
            Self::ConfigurationProperties => "Φprop",
            Self::Graph => "Φgraph",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
            Self::PropertiesFile => "Φpropf",
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

    /// Render the full marker line (e.g. `"Φrest:UserController map=[GET /api/users]"`).
    fn render(&self) -> String;
}

// ---------------------------------------------------------------------------
// Marker structs + PhiLine impls
// ---------------------------------------------------------------------------

/// A `Φrest:<ClassName> [map=…]` marker line for @RestController.
pub struct RestControllerLine<'a> {
    pub class_name: &'a str,
    pub mappings: &'a [RequestMappingMapping],
}

impl PhiLine for RestControllerLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::RestController
    }

    fn render(&self) -> String {
        if self.mappings.is_empty() {
            format!("Φrest:{}", self.class_name)
        } else {
            let maps: Vec<String> = self.mappings.iter().map(|m| m.to_string()).collect();
            format!("Φrest:{} map=[{}]", self.class_name, maps.join(","))
        }
    }
}

/// A `Φctrl:<ClassName> [map=…]` marker line for @Controller.
pub struct ControllerLine<'a> {
    pub class_name: &'a str,
    pub mappings: &'a [RequestMappingMapping],
}

impl PhiLine for ControllerLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Controller
    }

    fn render(&self) -> String {
        if self.mappings.is_empty() {
            format!("Φctrl:{}", self.class_name)
        } else {
            let maps: Vec<String> = self.mappings.iter().map(|m| m.to_string()).collect();
            format!("Φctrl:{} map=[{}]", self.class_name, maps.join(","))
        }
    }
}

/// A `Φsvc:<ClassName>` marker line for @Service.
pub struct ServiceLine<'a> {
    pub class_name: &'a str,
}

impl PhiLine for ServiceLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Service
    }

    fn render(&self) -> String {
        format!("Φsvc:{}", self.class_name)
    }
}

/// A `Φrepo:<ClassName>` marker line for @Repository.
pub struct RepositoryLine<'a> {
    pub class_name: &'a str,
}

impl PhiLine for RepositoryLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Repository
    }

    fn render(&self) -> String {
        format!("Φrepo:{}", self.class_name)
    }
}

/// A `Φconf:<ClassName>` marker line for @Configuration.
pub struct ConfigurationLine<'a> {
    pub class_name: &'a str,
}

impl PhiLine for ConfigurationLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Configuration
    }

    fn render(&self) -> String {
        format!("Φconf:{}", self.class_name)
    }
}

/// A `Φmap:<ClassName> map=[…]` marker line for @RequestMapping methods.
pub struct RequestMappingLine<'a> {
    pub class_name: &'a str,
    pub mappings: &'a [RequestMappingMapping],
}

impl PhiLine for RequestMappingLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::RequestMapping
    }

    fn render(&self) -> String {
        if self.mappings.is_empty() {
            format!("Φmap:{}", self.class_name)
        } else {
            let maps: Vec<String> = self.mappings.iter().map(|m| m.to_string()).collect();
            format!("Φmap:{} map=[{}]", self.class_name, maps.join(","))
        }
    }
}

/// A `Φaut:<fieldName>` marker line for @Autowired field.
pub struct AutowiredLine<'a> {
    pub field_name: &'a str,
}

impl PhiLine for AutowiredLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Autowired
    }

    fn render(&self) -> String {
        format!("Φaut:{}", self.field_name)
    }
}

/// A `Φval:<fieldName>` marker line for @Value field.
pub struct ValueLine<'a> {
    pub field_name: &'a str,
}

impl PhiLine for ValueLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Value
    }

    fn render(&self) -> String {
        format!("Φval:{}", self.field_name)
    }
}

/// A `Φbean:<methodName>` marker line for @Bean method.
pub struct BeanLine<'a> {
    pub method_name: &'a str,
}

impl PhiLine for BeanLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::Bean
    }

    fn render(&self) -> String {
        format!("Φbean:{}", self.method_name)
    }
}

/// A `Φprop:<className>` marker line for @ConfigurationProperties.
pub struct ConfigurationPropertiesLine<'a> {
    pub class_name: &'a str,
}

impl PhiLine for ConfigurationPropertiesLine<'_> {
    fn kind(&self) -> PhiLineKind {
        PhiLineKind::ConfigurationProperties
    }

    fn render(&self) -> String {
        format!("Φprop:{}", self.class_name)
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// A request mapping (HTTP method + path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMappingMapping {
    pub method: Option<String>,
    pub path: String,
}

impl std::fmt::Display for RequestMappingMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref m) = self.method {
            write!(f, "{} {}", m, self.path)
        } else {
            write!(f, "{}", self.path)
        }
    }
}
// ---------------------------------------------------------------------------
// build_* free functions — thin wrappers preserving the public API
// ---------------------------------------------------------------------------

/// Build a `Φrest:<ClassName> [map=…]` marker line.
pub fn build_rest_controller_line(class_name: &str, mappings: &[RequestMappingMapping]) -> String {
    RestControllerLine {
        class_name,
        mappings,
    }
    .render()
}

/// Build a `Φctrl:<ClassName> [map=…]` marker line.
pub fn build_controller_line(class_name: &str, mappings: &[RequestMappingMapping]) -> String {
    ControllerLine {
        class_name,
        mappings,
    }
    .render()
}

/// Build a `Φsvc:<ClassName>` marker line.
pub fn build_service_line(class_name: &str) -> String {
    ServiceLine { class_name }.render()
}

/// Build a `Φrepo:<ClassName>` marker line.
pub fn build_repository_line(class_name: &str) -> String {
    RepositoryLine { class_name }.render()
}

/// Build a `Φconf:<ClassName>` marker line.
pub fn build_configuration_line(class_name: &str) -> String {
    ConfigurationLine { class_name }.render()
}

/// Build a `Φmap:<ClassName> map=[…]` marker line.
pub fn build_request_mapping_line(class_name: &str, mappings: &[RequestMappingMapping]) -> String {
    RequestMappingLine {
        class_name,
        mappings,
    }
    .render()
}

/// Build a `Φaut:<fieldName>` marker line.
pub fn build_autowired_line(field_name: &str) -> String {
    AutowiredLine { field_name }.render()
}

/// Build a `Φval:<fieldName>` marker line.
pub fn build_value_line(field_name: &str) -> String {
    ValueLine { field_name }.render()
}

/// Build a `Φbean:<methodName>` marker line.
pub fn build_bean_line(method_name: &str) -> String {
    BeanLine { method_name }.render()
}

/// Build a `Φprop:<ClassName>` marker line.
pub fn build_configuration_properties_line(class_name: &str) -> String {
    ConfigurationPropertiesLine { class_name }.render()
}

// ---------------------------------------------------------------------------
// Expansion — generic over PhiLineKind (no manual table updates needed)
// ---------------------------------------------------------------------------

/// Expand every recognised `Φ…` marker in a line back to its
/// annotation form. Used by the decompressor.
///
/// This is the round-trip counterpart to the `build_*` helpers
/// above. It is **deliberately conservative**: only the marker
/// prefix is rewritten (`Φrest:` → `@RestController`); the trailing
/// `key=value` attributes are left untouched because they are
/// already human-readable in the compressed form.
///
/// Unknown `Φ` tokens (e.g. a future Phase marker) are passed
/// through unchanged.
///
/// Adding a new marker to the vocabulary only requires updating
/// [`PhiLineKind`] — this function is generic and needs no edits.
/// Called by `decompression::markers::expand_phi_in_line` (retired in Phase C1);
/// kept for test-only round-trip validation.
#[allow(dead_code)]
pub fn expand_phi_in_line(line: &str) -> String {
    let mut s = line.to_string();
    for &kind in PhiLineKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        if s.contains(prefix) {
            // Expansion includes a trailing space so that
            // `Φrest:Controller` → `@RestController Controller` (space separates
            // annotation from name).
            s = s.replace(prefix, &format!("{} ", kind.expansion()));
        }
    }
    s
}

/// Expand a single `Φ` marker token to its annotation form. Returns
/// `None` for unknown markers so the caller can pass them through.
///
/// Adding a new marker to the vocabulary only requires updating
/// [`PhiLineKind`] — this function is generic and needs no edits.
/// Used by tests (`src/tests/spring_meta/markers_tests.rs`).
#[allow(dead_code)]
pub fn expand_phi(token: &str) -> Option<&'static str> {
    PhiLineKind::from_token(token).map(|k| k.expansion())
}
