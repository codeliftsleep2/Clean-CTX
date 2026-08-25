// src/dotnet_meta/markers.rs
//
// `Φ` (Phi) marker construction & expansion for .NET / C#.
//
// # Mirrors `angular_meta::markers` and `spring_meta::markers` shape
//
// Just like the Angular Meta-Layer emits `Φ` markers for decorators,
// the .NET Meta-Layer emits `Φ` markers for .NET framework attributes.
// They are:
//   - emitted by `run_meta_layer` after the compacted class entry
//   - collapsed by `expand_phi_in_line` on decompression
//
// # Marker vocabulary (single source of truth: [`PhiLineKind`])
//
// | Marker    | Expansion                          |
// |-----------|------------------------------------|
// | Φctrl:    | [Controller] / [ApiController]     |
// | Φapi:     | [ApiController]                    |
// | Φaction:  | HTTP action (verb, params, return) |
// | Φmodel:   | Input/output models                |
// | Φauth:    | [Authorize]                        |
// | Φef:      | DbContext                          |
// | Φdbset:   | DbSet<T>                           |
// | Φentity:  | Entity with key/relationships      |
// | Φrel:     | Navigation relationships           |
// | Φcfg:     | Configuration / Fluent API         |
// | Φmap:     | Mapper profile                     |
// | Φmapfrom: | CreateMap + mappings               |
// | Φignore:  | Ignored members                    |
// | Φproj:    | Projections                        |
// | Φhub:     | Hub class                          |
// | Φmethod:  | Hub method + client invocation     |
// | Φclient:  | Strongly-typed client interface    |
// | Φgroup:   | Group management                   |
// | Φuser:    | User targeting                     |
// | Φstream:  | Streaming endpoints                |
// | Φconn:    | Connection lifecycle               |
// | Φjson:    | JSON configuration/attributes      |
// | Φprop:    | Property-level JSON attributes     |
// | Φsvc:     | Services / repositories            |
// | Φdi:      | DI registration points             |
// | Φcommon:  | Cross-cutting validation/attributes|
// | Φvalid:   | Validator classes                  |
// | Φrule:    | RuleFor chains                     |
// | Φcustom:  | Custom validators                  |
// | Φidentity:| UserManager / SignInManager        |
// | Φjwt:     | JWT configuration                  |
// | Φcache:   | IMemoryCache / IDistributedCache   |
// | Φoutput:  | Output caching middleware          |
// | Φjob:     | BackgroundJob / RecurringJob       |
// | Φlog:     | ILogger structured logging         |
// | Φmetric:  | Application Insights / OpenTelemetry|
// | Φgraph:   | cross-file dependency graph        |
// | ΦBUNDLE   | layer bundle                       |
// | ΦMAP      | workspace meta-map footer          |

// ---------------------------------------------------------------------------
// PhiLineKind — single source of truth for the marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind. This enum is the **single source of truth**
/// for the marker vocabulary — adding a new marker means adding one variant
/// here plus its `marker_prefix` / `expansion` arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhiLineKind {
    // ASP.NET Core
    Controller,
    ApiController,
    Action,
    Model,
    Authorize,
    // EF Core
    DbContext,
    DbSet,
    Entity,
    Relationship,
    Config,
    // AutoMapper
    Mapper,
    MapFrom,
    Ignore,
    Projection,
    // SignalR
    Hub,
    HubMethod,
    Client,
    Group,
    User,
    Stream,
    Connection,
    // Serialization
    Json,
    Property,
    // General
    Service,
    Di,
    Common,
    // FluentValidation
    Validator,
    Rule,
    Custom,
    // Identity
    Identity,
    Jwt,
    // Caching
    Cache,
    Output,
    // Background Jobs
    Job,
    // Logging
    Log,
    Metric,
    // Graph
    Graph,
    Bundle,
    Map,
}

impl PhiLineKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φctrl:"`).
    /// For prefix-less tokens (`ΦBUNDLE`, `ΦMAP`) the colon is omitted.
    pub fn marker_prefix(self) -> &'static str {
        match self {
            Self::Controller => "Φctrl:",
            Self::ApiController => "Φapi:",
            Self::Action => "Φaction:",
            Self::Model => "Φmodel:",
            Self::Authorize => "Φauth:",
            Self::DbContext => "Φef:",
            Self::DbSet => "Φdbset:",
            Self::Entity => "Φentity:",
            Self::Relationship => "Φrel:",
            Self::Config => "Φcfg:",
            Self::Mapper => "Φmap:",
            Self::MapFrom => "Φmapfrom:",
            Self::Ignore => "Φignore:",
            Self::Projection => "Φproj:",
            Self::Hub => "Φhub:",
            Self::HubMethod => "Φmethod:",
            Self::Client => "Φclient:",
            Self::Group => "Φgroup:",
            Self::User => "Φuser:",
            Self::Stream => "Φstream:",
            Self::Connection => "Φconn:",
            Self::Json => "Φjson:",
            Self::Property => "Φprop:",
            Self::Service => "Φsvc:",
            Self::Di => "Φdi:",
            Self::Common => "Φcommon:",
            Self::Validator => "Φvalid:",
            Self::Rule => "Φrule:",
            Self::Custom => "Φcustom:",
            Self::Identity => "Φidentity:",
            Self::Jwt => "Φjwt:",
            Self::Cache => "Φcache:",
            Self::Output => "Φoutput:",
            Self::Job => "Φjob:",
            Self::Log => "Φlog:",
            Self::Metric => "Φmetric:",
            Self::Graph => "Φgraph:",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
        }
    }

    /// The human-readable expansion (e.g. `"[Controller]"`). Does NOT
    /// include the trailing space that `expand_phi_in_line` adds.
    pub fn expansion(self) -> &'static str {
        match self {
            Self::Controller => "[Controller]",
            Self::ApiController => "[ApiController]",
            Self::Action => "[Action]",
            Self::Model => "[Model]",
            Self::Authorize => "[Authorize]",
            Self::DbContext => "[DbContext]",
            Self::DbSet => "[DbSet]",
            Self::Entity => "[Entity]",
            Self::Relationship => "[Relationship]",
            Self::Config => "[Config]",
            Self::Mapper => "[Mapper]",
            Self::MapFrom => "[MapFrom]",
            Self::Ignore => "[Ignore]",
            Self::Projection => "[Projection]",
            Self::Hub => "[Hub]",
            Self::HubMethod => "[HubMethod]",
            Self::Client => "[Client]",
            Self::Group => "[Group]",
            Self::User => "[User]",
            Self::Stream => "[Stream]",
            Self::Connection => "[Connection]",
            Self::Json => "[Json]",
            Self::Property => "[Property]",
            Self::Service => "[Service]",
            Self::Di => "[DI]",
            Self::Common => "[Common]",
            Self::Validator => "[Validator]",
            Self::Rule => "[Rule]",
            Self::Custom => "[Custom]",
            Self::Identity => "[Identity]",
            Self::Jwt => "[JWT]",
            Self::Cache => "[Cache]",
            Self::Output => "[Output]",
            Self::Job => "[Job]",
            Self::Log => "[Log]",
            Self::Metric => "[Metric]",
            Self::Graph => "[Graph]",
            Self::Bundle => "[Bundle]",
            Self::Map => "[Map]",
        }
    }

    /// All variants in a canonical order. Longer prefixes are placed
    /// before shorter ones to prevent partial-match issues in string
    /// replacement.
    pub fn all_in_expand_order() -> &'static [PhiLineKind] {
        &[
            Self::ApiController, // Φapi:     (5 chars)
            Self::Action,        // Φaction:  (8 chars)
            Self::Authorize,     // Φauth:    (6 chars)
            Self::Controller,    // Φctrl:    (6 chars)
            Self::Model,         // Φmodel:   (7 chars)
            Self::DbContext,     // Φef:      (4 chars)
            Self::DbSet,         // Φdbset:   (7 chars)
            Self::Entity,        // Φentity:  (8 chars)
            Self::Relationship,  // Φrel:     (5 chars)
            Self::Config,        // Φcfg:     (5 chars)
            Self::Mapper,        // Φmap:     (5 chars)
            Self::MapFrom,       // Φmapfrom: (9 chars)
            Self::Ignore,        // Φignore:  (8 chars)
            Self::Projection,    // Φproj:    (6 chars)
            Self::Hub,           // Φhub:     (5 chars)
            Self::HubMethod,     // Φmethod:  (8 chars)
            Self::Client,        // Φclient:  (8 chars)
            Self::Group,         // Φgroup:   (7 chars)
            Self::User,          // Φuser:    (6 chars)
            Self::Stream,        // Φstream:  (8 chars)
            Self::Connection,    // Φconn:    (6 chars)
            Self::Json,          // Φjson:    (6 chars)
            Self::Property,      // Φprop:    (6 chars)
            Self::Service,       // Φsvc:     (5 chars)
            Self::Di,            // Φdi:      (4 chars)
            Self::Common,        // Φcommon:  (8 chars)
            Self::Validator,     // Φvalid:   (7 chars)
            Self::Rule,          // Φrule:    (6 chars)
            Self::Custom,        // Φcustom:  (8 chars)
            Self::Identity,      // Φidentity:(10 chars)
            Self::Jwt,           // Φjwt:     (5 chars)
            Self::Cache,         // Φcache:   (7 chars)
            Self::Output,        // Φoutput:  (8 chars)
            Self::Job,           // Φjob:     (5 chars)
            Self::Log,           // Φlog:     (5 chars)
            Self::Metric,        // Φmetric:  (8 chars)
            Self::Graph,         // Φgraph:   (7 chars)
            Self::Bundle,        // ΦBUNDLE   (8 chars)
            Self::Map,           // ΦMAP      (5 chars)
        ]
    }

    /// Look up a [`PhiLineKind`] by its marker token string (without
    /// the trailing colon/binding). Returns `None` for unknown tokens.
    pub fn from_token(token: &str) -> Option<PhiLineKind> {
        match token {
            "Φctrl" => Some(Self::Controller),
            "Φapi" => Some(Self::ApiController),
            "Φaction" => Some(Self::Action),
            "Φmodel" => Some(Self::Model),
            "Φauth" => Some(Self::Authorize),
            "Φef" => Some(Self::DbContext),
            "Φdbset" => Some(Self::DbSet),
            "Φentity" => Some(Self::Entity),
            "Φrel" => Some(Self::Relationship),
            "Φcfg" => Some(Self::Config),
            "Φmap" => Some(Self::Mapper),
            "Φmapfrom" => Some(Self::MapFrom),
            "Φignore" => Some(Self::Ignore),
            "Φproj" => Some(Self::Projection),
            "Φhub" => Some(Self::Hub),
            "Φmethod" => Some(Self::HubMethod),
            "Φclient" => Some(Self::Client),
            "Φgroup" => Some(Self::Group),
            "Φuser" => Some(Self::User),
            "Φstream" => Some(Self::Stream),
            "Φconn" => Some(Self::Connection),
            "Φjson" => Some(Self::Json),
            "Φprop" => Some(Self::Property),
            "Φsvc" => Some(Self::Service),
            "Φdi" => Some(Self::Di),
            "Φcommon" => Some(Self::Common),
            "Φvalid" => Some(Self::Validator),
            "Φrule" => Some(Self::Rule),
            "Φcustom" => Some(Self::Custom),
            "Φidentity" => Some(Self::Identity),
            "Φjwt" => Some(Self::Jwt),
            "Φcache" => Some(Self::Cache),
            "Φoutput" => Some(Self::Output),
            "Φjob" => Some(Self::Job),
            "Φlog" => Some(Self::Log),
            "Φmetric" => Some(Self::Metric),
            "Φgraph" => Some(Self::Graph),
            "ΦBUNDLE" => Some(Self::Bundle),
            "ΦMAP" => Some(Self::Map),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    pub fn token(self) -> &'static str {
        match self {
            Self::Controller => "Φctrl",
            Self::ApiController => "Φapi",
            Self::Action => "Φaction",
            Self::Model => "Φmodel",
            Self::Authorize => "Φauth",
            Self::DbContext => "Φef",
            Self::DbSet => "Φdbset",
            Self::Entity => "Φentity",
            Self::Relationship => "Φrel",
            Self::Config => "Φcfg",
            Self::Mapper => "Φmap",
            Self::MapFrom => "Φmapfrom",
            Self::Ignore => "Φignore",
            Self::Projection => "Φproj",
            Self::Hub => "Φhub",
            Self::HubMethod => "Φmethod",
            Self::Client => "Φclient",
            Self::Group => "Φgroup",
            Self::User => "Φuser",
            Self::Stream => "Φstream",
            Self::Connection => "Φconn",
            Self::Json => "Φjson",
            Self::Property => "Φprop",
            Self::Service => "Φsvc",
            Self::Di => "Φdi",
            Self::Common => "Φcommon",
            Self::Validator => "Φvalid",
            Self::Rule => "Φrule",
            Self::Custom => "Φcustom",
            Self::Identity => "Φidentity",
            Self::Jwt => "Φjwt",
            Self::Cache => "Φcache",
            Self::Output => "Φoutput",
            Self::Job => "Φjob",
            Self::Log => "Φlog",
            Self::Metric => "Φmetric",
            Self::Graph => "Φgraph",
            Self::Bundle => "ΦBUNDLE",
            Self::Map => "ΦMAP",
        }
    }
}

// ---------------------------------------------------------------------------
// PhiLine trait — each marker type renders itself
// ---------------------------------------------------------------------------

/// A marker line that can render itself to its `Φ…` string form.
/// Used by tests (`src/tests/dotnet_meta/markers.rs`) to verify
/// marker round-trips. Kept under `#[allow(dead_code)]` because the
/// trait is only exercised in test code today.
#[allow(dead_code)]
pub trait PhiLine {
    /// The kind of this marker.
    fn kind(&self) -> PhiLineKind;

    /// Render the full marker line (e.g. `"Φctrl:UserController [api/users]"`).
    fn render(&self) -> String;
}

// ---------------------------------------------------------------------------
// build_* free functions
// ---------------------------------------------------------------------------

/// Build a `Φctrl:<ClassName> [route]` marker line.
pub fn build_controller_line(class_name: &str, route: Option<&str>) -> String {
    match route {
        Some(r) => format!("Φctrl:{} [{}]", class_name, r),
        None => format!("Φctrl:{}", class_name),
    }
}

/// Build a `Φapi:<ClassName>` marker line.
pub fn build_api_controller_line(class_name: &str) -> String {
    format!("Φapi:{}", class_name)
}

/// Build a `Φaction:<Verb> <Name>(<params>) → <return>` marker line.
pub fn build_action_line(
    verb: &str,
    name: &str,
    params: &str,
    return_type: Option<&str>,
) -> String {
    match return_type {
        Some(rt) => format!("Φaction:{} {}({}) → {}", verb, name, params, rt),
        None => format!("Φaction:{} {}({})", verb, name, params),
    }
}

/// Build a `Φmodel:<ModelName>` marker line.
pub fn build_model_line(model_name: &str) -> String {
    format!("Φmodel:{}", model_name)
}

/// Build a `Φauth:<Policy>` marker line.
pub fn build_auth_line(policy: Option<&str>) -> String {
    match policy {
        Some(p) => format!("Φauth:{}", p),
        None => "Φauth:true".to_string(),
    }
}

/// Build a `Φef:<ClassName>` marker line.
pub fn build_ef_line(class_name: &str) -> String {
    format!("Φef:{}", class_name)
}

/// Build a `Φdbset:<Name>` marker line.
pub fn build_dbset_line(name: &str) -> String {
    format!("Φdbset:{}", name)
}

/// Build a `Φentity:<Name> { <fields> }` marker line.
pub fn build_entity_line(name: &str, fields: &[String]) -> String {
    if fields.is_empty() {
        format!("Φentity:{}", name)
    } else {
        format!("Φentity:{} {{ {} }}", name, fields.join(", "))
    }
}

/// Build a `Φrel:<Name> → <Target>` marker line.
/// Used by tests (`src/tests/dotnet_meta/markers.rs`).
#[allow(dead_code)]
pub fn build_relationship_line(name: &str, target: &str) -> String {
    format!("Φrel:{} → {}", name, target)
}

/// Build a `Φcfg:<ClassName>` marker line.
pub fn build_config_line(class_name: &str) -> String {
    format!("Φcfg:{}", class_name)
}

/// Build a `Φmap:<ClassName>` marker line.
pub fn build_mapper_line(class_name: &str) -> String {
    format!("Φmap:{}", class_name)
}

/// Build a `Φmapfrom:<Source> → <Destination>` marker line.
pub fn build_mapfrom_line(source: &str, dest: &str) -> String {
    format!("Φmapfrom:{} → {}", source, dest)
}

/// Build a `Φignore:<Member>` marker line.
pub fn build_ignore_line(member: &str) -> String {
    format!("Φignore:{}", member)
}

/// Build a `Φproj:<Target>` marker line.
pub fn build_projection_line(target: &str) -> String {
    format!("Φproj:{}", target)
}

/// Build a `Φhub:<ClassName>` marker line.
pub fn build_hub_line(class_name: &str, client_interface: Option<&str>) -> String {
    match client_interface {
        Some(ci) => format!("Φhub:{} [{}]", class_name, ci),
        None => format!("Φhub:{}", class_name),
    }
}

/// Build a `Φmethod:<Name>(<params>) → <target>` marker line.
pub fn build_hub_method_line(name: &str, params: &str, target: &str) -> String {
    format!("Φmethod:{}({}) → {}", name, params, target)
}

/// Build a `Φclient:<Interface>.<Method>(<params>)` marker line.
/// Used by tests (`src/tests/dotnet_meta/markers.rs`).
#[allow(dead_code)]
pub fn build_client_line(interface: &str, method: &str, params: &str) -> String {
    format!("Φclient:{}.{}({})", interface, method, params)
}

/// Build a `Φgroup:<GroupName>` marker line.
pub fn build_group_line(group_name: &str) -> String {
    format!("Φgroup:{}", group_name)
}

/// Build a `Φuser:<UserId>` marker line.
pub fn build_user_line(user_id: &str) -> String {
    format!("Φuser:{}", user_id)
}

/// Build a `Φstream:<MethodName> → <Type>` marker line.
pub fn build_stream_line(method_name: &str, stream_type: &str) -> String {
    format!("Φstream:{} → {}", method_name, stream_type)
}

/// Build a `Φconn:<Event>` marker line.
pub fn build_connection_line(event: &str) -> String {
    format!("Φconn:{}", event)
}

/// Build a `Φjson:<Config>` marker line.
pub fn build_json_line(config: &str) -> String {
    format!("Φjson:{}", config)
}

/// Build a `Φprop:<Name>` marker line.
pub fn build_property_line(name: &str) -> String {
    format!("Φprop:{}", name)
}

/// Build a `Φsvc:<ClassName>` marker line.
pub fn build_service_line(class_name: &str) -> String {
    format!("Φsvc:{}", class_name)
}

/// Build a `Φdi:<Service> → <Registration>` marker line.
pub fn build_di_line(service: &str, registration: &str) -> String {
    format!("Φdi:{} → {}", service, registration)
}

/// Build a `Φcommon:<Attribute>` marker line.
pub fn build_common_line(attribute: &str) -> String {
    format!("Φcommon:{}", attribute)
}

/// Build a `Φvalid:<ClassName>` marker line.
pub fn build_validator_line(class_name: &str) -> String {
    format!("Φvalid:{}", class_name)
}

/// Build a `Φrule:<Property> → <Rules>` marker line.
pub fn build_rule_line(property: &str, rules: &[String]) -> String {
    if rules.is_empty() {
        format!("Φrule:{}", property)
    } else {
        format!("Φrule:{} → {}", property, rules.join(", "))
    }
}

/// Build a `Φcustom:<Name>` marker line.
pub fn build_custom_validator_line(name: &str) -> String {
    format!("Φcustom:{}", name)
}

/// Build a `Φidentity:<ClassName>` marker line.
pub fn build_identity_line(class_name: &str) -> String {
    format!("Φidentity:{}", class_name)
}

/// Build a `Φjwt:<Config>` marker line.
pub fn build_jwt_line(config: &str) -> String {
    format!("Φjwt:{}", config)
}

/// Build a `Φcache:<Type>` marker line.
pub fn build_cache_line(cache_type: &str) -> String {
    format!("Φcache:{}", cache_type)
}

/// Build a `Φoutput:<Config>` marker line.
pub fn build_output_line(config: &str) -> String {
    format!("Φoutput:{}", config)
}

/// Build a `Φjob:<Name>` marker line.
pub fn build_job_line(name: &str) -> String {
    format!("Φjob:{}", name)
}

/// Build a `Φlog:<Pattern>` marker line.
pub fn build_log_line(pattern: &str) -> String {
    format!("Φlog:{}", pattern)
}

/// Build a `Φmetric:<Provider>` marker line.
pub fn build_metric_line(provider: &str) -> String {
    format!("Φmetric:{}", provider)
}

// ---------------------------------------------------------------------------
// Expansion — generic over PhiLineKind (no manual table updates needed)
// ---------------------------------------------------------------------------

/// Expand every recognised `Φ…` marker in a line back to its
/// attribute form. Used by the decompressor.
/// Called by `decompression::markers::expand_phi_in_line` when the
/// `dotnet` feature is enabled; always used by tests.
#[cfg_attr(not(feature = "dotnet"), allow(dead_code))]
pub fn expand_phi_in_line(line: &str) -> String {
    let mut s = line.to_string();
    for &kind in PhiLineKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        if s.contains(prefix) {
            s = s.replace(prefix, &format!("{} ", kind.expansion()));
        }
    }
    s
}

/// Expand a single `Φ` marker token to its attribute form. Returns
/// `None` for unknown markers so the caller can pass them through.
/// Used by tests (`src/tests/dotnet_meta/markers.rs`).
#[allow(dead_code)]
pub fn expand_phi(token: &str) -> Option<&'static str> {
    PhiLineKind::from_token(token).map(|k| k.expansion())
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/markers.rs"]
mod tests;
