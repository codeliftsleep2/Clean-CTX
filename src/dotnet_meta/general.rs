// src/dotnet_meta/general.rs
//
// General .NET patterns — Services, DI, validation, identity, caching, logging.
//
// Detects:
// - Service classes (interfaces, implementations)
// - DI registration patterns (AddScoped, AddSingleton, AddTransient, AddDbContext)
// - Validation attributes ([Required], [StringLength], etc.)
// - Identity/authentication (UserManager, SignInManager, [Authorize])
// - Caching (IMemoryCache, IDistributedCache, [ResponseCache])
// - Logging (ILogger<T>, ILoggerFactory)
// - Background jobs (BackgroundJob, RecurringJob)

use super::markers::{
    build_cache_line, build_common_line, build_custom_validator_line, build_di_line,
    build_identity_line, build_job_line, build_jwt_line, build_log_line, build_metric_line,
    build_output_line, build_rule_line, build_service_line, build_validator_line,
};
use crate::compression::Fidelity;
use crate::dotnet_meta::MetaBlock;

/// Extract general .NET markers from a single class capture.
///
/// This is the "catch-all" extractor for patterns that don't fit into
/// the specialized extractors (ASP.NET, EF Core, SignalR, AutoMapper, serialization).
#[allow(unused_variables)]
pub fn extract_general(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    let mut lines = Vec::new();

    // Extract services
    lines.extend(extract_services(class_source));

    // Extract DI registrations
    lines.extend(extract_di_registrations(class_source));

    // Extract validation attributes
    lines.extend(extract_validation(class_source));

    // Extract identity/authentication
    lines.extend(extract_identity(class_source));

    // Extract caching
    lines.extend(extract_caching(class_source));

    // Extract logging
    lines.extend(extract_logging(class_source));

    // Extract background jobs
    lines.extend(extract_background_jobs(class_source));

    if lines.is_empty() {
        None
    } else {
        Some(MetaBlock { lines })
    }
}

/// Known .NET framework interfaces that should NOT be flagged as service interfaces.
/// These are built-in .NET types that follow the "I" prefix convention but are
/// not application-level services or repositories.
const KNOWN_FRAMEWORK_INTERFACES: &[&str] = &[
    // System interfaces
    "IDisposable",
    "IAsyncDisposable",
    "IComparable",
    "IEquatable",
    "IFormattable",
    "ICloneable",
    "IConvertible",
    "IEnumerable",
    "IEnumerator",
    "IQueryable",
    "ICollection",
    "IList",
    "ISet",
    "IDictionary",
    "IComparer",
    "IEqualityComparer",
    "IHashCodeProvider",
    // System.IO
    "IAsyncResult",
    "IAsyncOperation",
    "IAsyncInfo",
    "INotifyCompletion",
    // System.ComponentModel
    "INotifyPropertyChanged",
    "INotifyPropertyChanging",
    "IDataErrorInfo",
    "INotifyDataErrorInfo",
    "ICustomTypeDescriptor",
    // System.Threading
    "IAsyncStateMachine",
    "IThreadPoolWorkItem",
    "IValueTaskSource",
    // System.Security
    "IPrincipal",
    "IIdentity",
    // ASP.NET Core / Http
    "IResult",
    "IActionResult",
    "IHeaderDictionary",
    "IFormFile",
    "IFormCollection",
    "IRequestCookieCollection",
    "IResponseCookies",
    "ISession",
    "IWebHostEnvironment",
    "IHostEnvironment",
    // Logging
    "ILogger",
    "ILoggerFactory",
    "ILoggerProvider",
    // DI
    "IServiceProvider",
    "IServiceCollection",
    "IServiceScope",
    "IServiceScopeFactory",
    // Caching
    "IMemoryCache",
    "IDistributedCache",
    "ICacheEntry",
    // Hosting
    "IHost",
    "IHostBuilder",
    "IApplicationBuilder",
    // Configuration
    "IConfiguration",
    "IConfigurationSection",
    "IConfigurationBuilder",
    "IConfigurationProvider",
    // HTTP client
    "IHttpClientFactory",
    "IHttpMessageHandlerFactory",
    "IHttpContextAccessor",
    // MVC / Routing
    "IRouter",
    "IUrlHelper",
    "IActionConstraint",
    "IFilterMetadata",
    "IExceptionFilter",
    "IActionFilter",
    "IResultFilter",
    "IAuthorizationFilter",
    "IResourceFilter",
    "IAsyncActionFilter",
    "IAsyncResultFilter",
    "IAsyncAuthorizationFilter",
    "IAsyncExceptionFilter",
    "IAsyncResourceFilter",
    "IAlwaysRunResultFilter",
    "IOrderedFilter",
    // JSON
    "IJsonTypeInfoResolver",
    // SignalR
    "IHubContext",
    "IHubCallerClients",
    "IClientProxy",
    "IHubCallerClients",
    // Entity Framework
    "IQueryable",
    "IAsyncEnumerable",
    "IAsyncEnumerator",
];

/// Extract service classes (interfaces and implementations).
fn extract_services(class_source: &str) -> Vec<String> {
    let mut services = Vec::new();

    // Find all class/interface names
    let names = extract_all_class_names(class_source);

    for name in &names {
        // Service implementations (convention-based)
        if name.ends_with("Service") || name.ends_with("Repository") {
            services.push(build_service_line(name));
            continue;
        }

        // Interfaces starting with 'I' (convention) — but exclude known framework types
        if name.starts_with('I')
            && name.len() > 1
            && !KNOWN_FRAMEWORK_INTERFACES.contains(&name.as_str())
        {
            // Also require the name to appear in a DI registration context or
            // be referenced as a constructor parameter to avoid false positives
            // from generic usage of I-prefixed names in comments or strings.
            if appears_in_di_context(class_source, name)
                || appears_as_constructor_param(class_source, name)
            {
                services.push(build_service_line(name));
            }
        }
    }

    services
}

/// Check if an interface name appears in a DI registration context
/// (AddScoped, AddSingleton, AddTransient, AddDbContext).
fn appears_in_di_context(source: &str, name: &str) -> bool {
    let di_keywords = [
        "AddScoped<",
        "AddSingleton<",
        "AddTransient<",
        "AddDbContext<",
    ];
    for keyword in &di_keywords {
        if source.contains(&format!("{}{}", keyword, name))
            || source.contains(&format!("{}I{}", keyword, &name[1..]))
        {
            return true;
        }
    }
    false
}

/// Check if an interface name appears as a constructor parameter.
fn appears_as_constructor_param(source: &str, name: &str) -> bool {
    // Look for patterns like "ClassName(ITypeName paramName)" or "ClassName( ITypeName paramName )"
    let patterns = [
        format!("({} ", name),
        format!("( {} ", name),
        format!(",{} ", name),
        format!(", {} ", name),
    ];
    for pattern in &patterns {
        if source.contains(pattern) {
            return true;
        }
    }
    false
}

/// Extract all class/interface names from source.
fn extract_all_class_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let patterns = [
        "public class ",
        "internal class ",
        "private class ",
        "protected class ",
        "class ",
        "public interface ",
        "internal interface ",
        "interface ",
    ];

    for pattern in &patterns {
        let mut search_start = 0;
        while let Some(pos) = source[search_start..].find(pattern) {
            let actual_pos = search_start + pos;
            let start = actual_pos + pattern.len();
            let rest = &source[start..];
            let end = rest
                .find(|c: char| c == ':' || c == '<' || c.is_whitespace() || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..end].trim().to_string();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
            search_start = start + end;
        }
    }

    names
}

/// A structured DI registration fact extracted from source.
///
/// This is the authoritative structured representation of a DI registration
/// before it is projected into either:
/// - `Φdi` text markers (via `format_di_registration`)
/// - `Binds` semantic edges (via `extract_dotnet_di_bindings`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiRegistration {
    /// The abstraction/token type (first generic argument, e.g., `IService` in
    /// `AddScoped<IService, Service>()`).
    pub(crate) service: String,
    /// The implementation type (second generic argument, e.g., `Service` in
    /// `AddScoped<IService, Service>()`). `None` for single-type registrations
    /// like `AddDbContext<T>()`.
    pub(crate) impl_type: Option<String>,
    /// The lifetime pattern (e.g., "AddScoped", "AddSingleton").
    pub(crate) lifetime: String,
}

/// Format a `DiRegistration` into a `Φdi:<Service> → <Registration>` marker.
pub(crate) fn format_di_registration(reg: &DiRegistration) -> String {
    match &reg.impl_type {
        Some(impl_type) => build_di_line(&reg.service, &format!("{}<{}>", reg.lifetime, impl_type)),
        None => build_di_line(&reg.service, &format!("{}<{}>", reg.lifetime, reg.service)),
    }
}

/// Extract DI registration patterns as structured data.
///
/// Returns `Vec<DiRegistration>` for use by both Φdi marker generation and
/// Binds semantic-edge projection.
pub(crate) fn extract_di_registrations_structured(class_source: &str) -> Vec<DiRegistration> {
    let mut registrations = Vec::new();

    let di_patterns = [
        ("AddScoped<", "AddScoped"),
        ("AddSingleton<", "AddSingleton"),
        ("AddTransient<", "AddTransient"),
        ("AddDbContext<", "AddDbContext"),
    ];

    for (pattern, lifetime) in &di_patterns {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find(pattern) {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + pattern.len()..];

            if let Some(generic_end) = rest.find('>') {
                let types = rest[..generic_end].trim().to_string();

                if let Some(comma_pos) = types.find(',') {
                    let service = types[..comma_pos].trim().to_string();
                    let impl_type = types[comma_pos + 1..].trim().to_string();
                    registrations.push(DiRegistration {
                        service,
                        impl_type: Some(impl_type),
                        lifetime: lifetime.to_string(),
                    });
                } else {
                    // Single type (e.g., AddDbContext<AppDbContext>)
                    registrations.push(DiRegistration {
                        service: types.clone(),
                        impl_type: None,
                        lifetime: lifetime.to_string(),
                    });
                }
            }

            search_start = actual_pos + 1;
        }
    }

    registrations.dedup();
    registrations.truncate(10);
    registrations
}

/// Extract DI registration patterns (legacy Φdi string output).
///
/// Preserved for backward compatibility. New code should use
/// `extract_di_registrations_structured` for semantic-edge projection.
fn extract_di_registrations(class_source: &str) -> Vec<String> {
    extract_di_registrations_structured(class_source)
        .iter()
        .map(format_di_registration)
        .collect()
}

/// Extract validation attributes and FluentValidation validators.
fn extract_validation(class_source: &str) -> Vec<String> {
    let mut validations = Vec::new();

    // Look for AbstractValidator<T> (FluentValidation)
    if class_source.contains("AbstractValidator<") {
        if let Some(pos) = class_source.find("AbstractValidator<") {
            let rest = &class_source[pos + "AbstractValidator<".len()..];
            if let Some(generic_end) = rest.find('>') {
                let entity = rest[..generic_end].trim().to_string();
                validations.push(build_validator_line(&entity));
            }
        }
    }

    // Look for RuleFor chains
    if class_source.contains("RuleFor(") {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find("RuleFor(") {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + "RuleFor(".len()..];

            // Extract property name (x => x.PropertyName)
            if let Some(arrow_pos) = rest.find("=>") {
                let after_arrow = &rest[arrow_pos + 2..];
                if let Some(dot_pos) = after_arrow.find('.') {
                    let prop_rest = &after_arrow[dot_pos + 1..];
                    let prop_name = prop_rest
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(',')
                        .trim_end_matches(')')
                        .to_string();

                    if !prop_name.is_empty() {
                        // Extract validation rules
                        let rules = extract_validation_rules(rest);
                        validations.push(build_rule_line(&prop_name, &rules));
                    }
                }
            }

            search_start = actual_pos + 1;
        }
    }

    // Look for common validation attributes
    let attr_patterns = [
        ("[Required]", "Required"),
        ("[StringLength(", "StringLength"),
        ("[Range(", "Range"),
        ("[EmailAddress]", "EmailAddress"),
        ("[Phone]", "Phone"),
        ("[MinLength(", "MinLength"),
        ("[MaxLength(", "MaxLength"),
        ("[RegularExpression(", "RegularExpression"),
    ];

    for (attr, name) in &attr_patterns {
        if class_source.contains(attr) {
            validations.push(build_common_line(name));
        }
    }

    // Look for custom validators
    if class_source.contains("Custom(") {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find("Custom(") {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + "Custom(".len()..];

            if let Some(close_paren) = rest.find(')') {
                let custom_name = rest[..close_paren].trim().to_string();
                if !custom_name.is_empty() {
                    validations.push(build_custom_validator_line(&custom_name));
                }
            }

            search_start = actual_pos + 1;
        }
    }

    validations.dedup();
    validations.truncate(10);
    validations
}

/// Extract validation rules from a RuleFor chain.
fn extract_validation_rules(rule_chain: &str) -> Vec<String> {
    let mut rules = Vec::new();

    // Common validation methods
    let rule_patterns = [
        ".NotEmpty()",
        ".NotNull()",
        ".Length(",
        ".MinimumLength(",
        ".MaximumLength(",
        ".LessThan(",
        ".LessThanOrEqualTo(",
        ".GreaterThan(",
        ".GreaterThanOrEqualTo(",
        ".EmailAddress()",
        ".Phone()",
        ".Matches(",
        ".In(",
        ".IsInEnum()",
    ];

    for pattern in &rule_patterns {
        if rule_chain.contains(pattern) {
            let rule_name = pattern.trim_start_matches('.').trim_end_matches("()");
            if !rule_name.is_empty() {
                rules.push(rule_name.to_string());
            }
        }
    }

    rules.truncate(5);
    rules
}

/// Extract identity/authentication patterns.
fn extract_identity(class_source: &str) -> Vec<String> {
    let mut identity = Vec::new();

    // Look for UserManager<T>, SignInManager<T>
    if class_source.contains("UserManager<") {
        if let Some(pos) = class_source.find("UserManager<") {
            let rest = &class_source[pos + "UserManager<".len()..];
            if let Some(generic_end) = rest.find('>') {
                let user_type = rest[..generic_end].trim().to_string();
                identity.push(build_identity_line(&format!("UserManager<{}>", user_type)));
            }
        }
    }

    if class_source.contains("SignInManager<") {
        if let Some(pos) = class_source.find("SignInManager<") {
            let rest = &class_source[pos + "SignInManager<".len()..];
            if let Some(generic_end) = rest.find('>') {
                let user_type = rest[..generic_end].trim().to_string();
                identity.push(build_identity_line(&format!(
                    "SignInManager<{}>",
                    user_type
                )));
            }
        }
    }

    // Look for JWT configuration
    if class_source.contains("AddJwtBearer") || class_source.contains("JwtBearer") {
        identity.push(build_jwt_line("JwtBearer"));
    }

    identity
}

/// Extract caching patterns.
fn extract_caching(class_source: &str) -> Vec<String> {
    let mut caching = Vec::new();

    // Look for IMemoryCache, IDistributedCache
    if class_source.contains("IMemoryCache") {
        caching.push(build_cache_line("IMemoryCache"));
    }

    if class_source.contains("IDistributedCache") {
        caching.push(build_cache_line("IDistributedCache"));
    }

    // Look for [ResponseCache]
    if class_source.contains("[ResponseCache") {
        caching.push(build_output_line("ResponseCache"));
    }

    // Look for IMemoryCache injection
    if class_source.contains("IMemoryCache") && class_source.contains("constructor") {
        caching.push(build_cache_line("injected"));
    }

    caching
}

/// Extract logging patterns.
fn extract_logging(class_source: &str) -> Vec<String> {
    let mut logging = Vec::new();

    // Look for ILogger<T>
    if class_source.contains("ILogger<") {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find("ILogger<") {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + "ILogger<".len()..];

            if let Some(generic_end) = rest.find('>') {
                let logger_type = rest[..generic_end].trim().to_string();
                logging.push(build_log_line(&format!("ILogger<{}>", logger_type)));
            }

            search_start = actual_pos + 1;
        }
    }

    // Look for ILoggerFactory
    if class_source.contains("ILoggerFactory") {
        logging.push(build_log_line("ILoggerFactory"));
    }

    // Look for Application Insights / OpenTelemetry
    if class_source.contains("AddApplicationInsights")
        || class_source.contains("ApplicationInsights")
    {
        logging.push(build_metric_line("ApplicationInsights"));
    }

    if class_source.contains("AddOpenTelemetry") || class_source.contains("OpenTelemetry") {
        logging.push(build_metric_line("OpenTelemetry"));
    }

    logging.dedup();
    logging.truncate(5);
    logging
}

/// Extract background job patterns.
fn extract_background_jobs(class_source: &str) -> Vec<String> {
    let mut jobs = Vec::new();

    // Look for BackgroundJob, RecurringJob
    if class_source.contains("BackgroundJob") {
        jobs.push(build_job_line("BackgroundJob"));
    }

    if class_source.contains("RecurringJob") {
        jobs.push(build_job_line("RecurringJob"));
    }

    jobs
}

/// Extract the class name from a class declaration.
#[allow(dead_code)]
fn extract_class_name(source: &str) -> Option<String> {
    let patterns = [
        "public class ",
        "internal class ",
        "private class ",
        "protected class ",
        "class ",
        "public interface ",
        "internal interface ",
        "interface ",
    ];

    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            let start = pos + pattern.len();
            let rest = &source[start..];
            let end = rest
                .find(|c: char| c == ':' || c == '<' || c.is_whitespace() || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/general.rs"]
mod tests;
