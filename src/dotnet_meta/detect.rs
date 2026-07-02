// src/dotnet_meta/detect.rs
//
// .NET / C# detection heuristic.
//
// The Meta-Layer must run **only** on .NET framework files. Plain C#
// files (utility classes, POCOs, etc.) should pay **zero** overhead —
// no Φ markers, no extra parse, no newlines.
//
// # Strategy
//
// We use string-based detection (same strategy as Angular/Spring Boot
// Phase 1). A file is considered ".NET framework" if it contains any
// of the following strong signals:
//
// - ASP.NET Core attributes: [ApiController], [Route], [HttpGet], etc.
// - ASP.NET Core base classes: : ControllerBase, : Controller, : Hub<T>
// - EF Core: : DbContext, DbSet<T>, [Key], [ForeignKey]
// - AutoMapper: : Profile, CreateMap<, >
// - SignalR: : Hub<T>, IHubContext, HubCallerContext
// - FluentValidation: AbstractValidator<T>
// - Identity: UserManager<T>, SignInManager<T>, IdentityUser
// - Caching: IMemoryCache, IDistributedCache, [ResponseCache]
// - Background jobs: BackgroundJob, RecurringJob
// - Logging: ILogger<T>, ILoggerFactory
// - DI: AddScoped, AddSingleton, AddTransient, AddDbContext
//
// A file with none of these signals is treated as plain C# and
// produces zero Φ markers.

/// Strong .NET framework signals. A single match anywhere in the source
/// is enough to consider the file a .NET framework file.
const STRONG_SIGNALS: &[&str] = &[
    // ASP.NET Core
    "[ApiController]",
    "[Route(",
    "[HttpGet(",
    "[HttpPost(",
    "[HttpPut(",
    "[HttpDelete(",
    "[HttpPatch(",
    "[Authorize]",
    "[AllowAnonymous]",
    ": ControllerBase",
    ": Controller",
    // EF Core
    ": DbContext",
    "DbSet<",
    "[Key]",
    "[ForeignKey(",
    "[Table(",
    "[Column(",
    "[Required]",
    "[StringLength(",
    // AutoMapper
    ": Profile",
    "CreateMap<",
    // SignalR
    ": Hub",
    "IHubContext<",
    "HubCallerContext",
    // FluentValidation
    "AbstractValidator<",
    // Identity
    "UserManager<",
    "SignInManager<",
    "IdentityUser",
    "IdentityRole",
    // Caching
    "IMemoryCache",
    "IDistributedCache",
    "[ResponseCache",
    // Background Jobs
    "BackgroundJob",
    "RecurringJob",
    // Logging
    "ILogger<",
    "ILoggerFactory",
    // DI
    "AddScoped<",
    "AddSingleton<",
    "AddTransient<",
    "AddDbContext<",
    // JWT
    "AddAuthentication",
    "AddJwtBearer",
    // OpenTelemetry / Metrics
    "AddOpenTelemetry",
    "AddApplicationInsights",
];

/// Decide whether the given source code is from a .NET framework file.
///
/// A file is ".NET framework" iff it contains at least one **strong**
/// signal from the list above.
///
/// Plain C# files (utility classes, POCOs, enums, etc.) return `false`
/// — they should not get any Φ markers.
pub fn is_dotnet_file(source: &str) -> bool {
    for signal in STRONG_SIGNALS {
        if source.contains(signal) {
            return true;
        }
    }
    false
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/detect.rs"]
mod tests;
