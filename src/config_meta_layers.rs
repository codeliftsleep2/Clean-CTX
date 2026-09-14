use super::*;

/// Per-framework Meta-Layer configuration.
///
/// The shared fields support framework-level opt-out plus the established
/// Angular and .NET sub-layer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLayerConfig {
    /// Master switch for this framework's meta-layer. Defaults to
    /// `true` (the meta-layer runs whenever the framework is
    /// detected). Set to `false` in `.clean-ctx.json` to opt out
    /// of the meta-layer entirely (zero overhead).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// RxJS sub-layer config (Angular Ecosystem Deepening Phase 1).
    #[serde(default)]
    pub rxjs: RxJsConfig,
    /// NgRx sub-layer config (Angular Ecosystem Deepening Phase 2).
    #[serde(default)]
    pub ngrx: NgRxConfig,
    /// Signals sub-layer config (Angular Ecosystem Deepening Phase 3).
    #[serde(default)]
    pub signals: SignalsConfig,
    /// Angular Reactive Forms source-shape sub-layer config.
    #[serde(default)]
    pub reactive_forms: ReactiveFormsConfig,
    /// ngx-formly source-shape sub-layer config.
    #[serde(default)]
    pub formly: FormlyConfig,
    /// Routing sub-layer config (Angular Ecosystem Deepening Phase 4).
    #[serde(default)]
    pub routing: RoutingConfig,
    /// Framework-local testing sub-layer config (MSTest/Moq for .NET,
    /// Vitest/TestBed for Angular).
    #[serde(default)]
    pub testing: TestingConfig,
}

impl Default for MetaLayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rxjs: RxJsConfig::default(),
            ngrx: NgRxConfig::default(),
            signals: SignalsConfig::default(),
            reactive_forms: ReactiveFormsConfig::default(),
            formly: FormlyConfig::default(),
            routing: RoutingConfig::default(),
            testing: TestingConfig::default(),
        }
    }
}

/// RxJS sub-layer configuration (Angular Ecosystem Deepening Phase 1).
///
/// All fields are optional and `#[serde(default)]` so existing
/// `.clean-ctx.json` files remain backward-compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxJsConfig {
    /// Master switch for the RxJS meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Only emit `ΦpipeRx:` blocks when the pipe chain has at least
    /// this many operators (default 2). Prevents noise from trivial
    /// single-operator chains.
    #[serde(default = "default_min_pipe_operators")]
    pub min_pipe_operators: usize,
}

impl Default for RxJsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_pipe_operators: 2,
        }
    }
}

/// NgRx sub-layer configuration (Angular Ecosystem Deepening Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgRxConfig {
    /// Master switch for the NgRx meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Include `Φdispatch:` call sites in Medium/High output (default true).
    #[serde(default = "default_true")]
    pub include_dispatch_sites: bool,
    /// Include `Φselect:` call sites in Medium/High output (default true).
    #[serde(default = "default_true")]
    pub include_select_sites: bool,
    /// Include entity adapter default selectors in `Φentity:` block
    /// (default true).
    #[serde(default = "default_true")]
    pub entity_selectors: bool,
    /// Enable CBM cross-language effect→endpoint resolution
    /// (workspace + CBM only, default true).
    #[serde(default = "default_true")]
    pub cross_layer_cbm: bool,
}

impl Default for NgRxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_dispatch_sites: true,
            include_select_sites: true,
            entity_selectors: true,
            cross_layer_cbm: true,
        }
    }
}

/// Signals sub-layer configuration (Angular Ecosystem Deepening Phase 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalsConfig {
    /// Master switch for the Signals meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for SignalsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Angular Reactive Forms source-shape sub-layer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactiveFormsConfig {
    /// Master switch for the Reactive Forms meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// ngx-formly source-shape sub-layer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormlyConfig {
    /// Master switch for the Formly meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for FormlyConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for ReactiveFormsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Routing sub-layer configuration (Angular Ecosystem Deepening Phase 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Master switch for the Routing meta-layer. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn default_min_pipe_operators() -> usize {
    2
}

/// Framework-local testing sub-layer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    /// Master switch for testing markers and semantic edges. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
