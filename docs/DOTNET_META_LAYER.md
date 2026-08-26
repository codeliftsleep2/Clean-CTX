# .NET / C# Meta-Layer

> **Owner:** .NET/C# Meta-Layer design (R-35  R-41/R-42) · **Status:** Living per-layer reference (shipped)
>
> **Implementation:** Phase 1 (C# Core) complete with full ASP.NET Core  EF Core  SignalR  AutoMapper  and DI support. The .NET meta-layer is now integrated and available via the dotnet Cargo feature flag (enabled by default).
> **Ship status:** see `docs/ROADMAP.md`. **Test counts / audit rounds:** see `docs/CHANGELOG.md`. This document does not duplicate them.

---

## Decisions Locked

| Question | Decision |
|----------|----------|
| Compiler approach | String-based extraction on tree-sitter C# captures (same strategy as Angular/Spring Boot). No re-parse of AST. |
| Marker approach | `Φ`-prefixed markers  no new opcodes. Opcodes stay language-agnostic primitives. |
| Phasing | Three independently shippable phases. Stop after any phase for a useful Meta-Layer. |
| Default state | On  opt-out via `.clean-ctx.json`. Non-.NET files pay zero overhead. |
| Workspace scope | Tier 1 (per-file markers) works in both modes. Tiers 2 & 3 are workspace-only. |
| New dependencies | None. Uses existing `tree-sitter-c-sharp` grammar (already in `Cargo.toml`). |
| Feature flag | `dotnet` — depends on `csharp` feature. Registered in `LayerRegistry`. |

---

## Notation Map

| Prefix | Job | Examples |
|--------|-----|---------|
| `$xx` | Opcodes — language primitives | `$c` = class  `$ctor` = constructor  `$a` = async |
| `⊕` | Behavior markers — control-flow annotations | `⊕guard`  `⊕loop`  `⊕⇒`  `⊕!` |
| `α / β / γ` | Path aliases — file references | `α7` = `/path/to/file.cs` |
| `Φ` (new) | Framework-annotation markers | `Φctrl:`  `Φef:`  `Φhub:`  `Φmap:`  `Φsvc:`  `Φdi:` |

> **Notation scope:** `$xx` opcodes and `⊕` markers are emitted by the LEGACY text compressor (`compress_workspace` manifests; `⊕` at Medium/High, `§` micro-codes at Low) and decoded by `decompress_code_context`. Interactive responses use SCHEMA v2 notation instead. The `Φ` framework vocabulary remains current.

---

## Proposed .NET Φ Markers

### ASP.NET Core

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φctrl:` | `[Controller]` / `[ApiController]` | Controller / route summary |
| `Φapi:` | `[ApiController]` | ApiController details |
| `Φaction:` | HTTP action | Verb  parameters  return type |
| `Φmodel:` | Input/output models | Request/response DTOs |
| `Φauth:` | `[Authorize]` | Authorization rules |

### Entity Framework Core

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φef:` | `DbContext` | DbContext class |
| `Φdbset:` | `DbSet<T>` | DbSet properties |
| `Φentity:` | Entity model | Entity with key/relationships |
| `Φrel:` | Navigation | Navigation relationships |
| `Φcfg:` | Fluent API | Configuration / `OnModelCreating` |

### AutoMapper

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φmap:` | `Profile` | Mapper profile |
| `Φmapfrom:` | `CreateMap<TSrc  TDst>()` | CreateMap + mappings |
| `Φignore:` | `ForMember().Ignore()` | Ignored members |
| `Φproj:` | `ProjectTo<T>()` | Projections |

### SignalR

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φhub:` | `Hub<T>` | Hub class |
| `Φmethod:` | Hub method | Hub method + client invocation |
| `Φclient:` | `IClientProxy` | Strongly-typed client interface |
| `Φgroup:` | `Groups.Add/Remove` | Group management |
| `Φuser:` | `Clients.User` | User targeting |
| `Φstream:` | `ChannelReader<T>` | Streaming endpoints |
| `Φconn:` | `OnConnectedAsync` | Connection lifecycle |

### Serialization

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φjson:` | `[JsonPropertyName]` / `[JsonConverter]` | JSON configuration/attributes |
| `Φprop:` | `[DataMember]` / `[IgnoreDataMember]` | Property-level attributes |

### General

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φsvc:` | `[Service]` | Services / repositories |
| `Φdi:` | `AddScoped` / `AddSingleton` / `AddTransient` | DI registration points |
| `Φcommon:` | `[Required]` / `[StringLength]` | Cross-cutting validation/attributes |

### FluentValidation

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φvalid:` | `AbstractValidator<T>` | Validator classes |
| `Φrule:` | `RuleFor()` | RuleFor chains (`RuleFor(x => x.Email).NotEmpty().EmailAddress()`) |
| `Φcustom:` | `Custom()` | Custom validators |

### Identity / Authentication

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φidentity:` | `UserManager` / `SignInManager` | UserManager  SignInManager  IdentityUser |
| `Φauth:` | `[Authorize]` | Authorization — policy-based  claims  roles |
| `Φjwt:` | JWT | JWT configuration / token generation |

### Caching

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φcache:` | `IMemoryCache` / `IDistributedCache` | Memory cache  distributed cache  `[ResponseCache]` |
| `Φoutput:` | Output caching | Output caching middleware |

### Background Jobs

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φjob:` | `BackgroundJob` / `RecurringJob` | Hangfire background jobs  recurring jobs  attributes |

### Logging / Telemetry

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φlog:` | `ILogger` | Structured logging patterns |
| `Φmetric:` | `Application Insights` / `OpenTelemetry` | Application Insights / OpenTelemetry usage |

---

## Example Output Style

```
Φctrl:UserController [api/users]
  Φaction:GET GetById(id) → UserDto
  Φaction:POST Create([FromBody] request)

Φhub:NotificationHub
  Φmethod:SendToUser(userId  message) → Clients.User
  Φclient:INotificationClient.Receive(...)

Φef:AppDbContext
  Φdbset:Users  Orders
  Φentity:Order { Id  UserId (FK) }

Φdi:UserService → AddScoped<IUserService  UserService>
Φdi:AppDbContext → AddDbContext<AppDbContext>(connectionString)
```

---

## Phase 1: C# Core Meta-Layer (5-7 days)

### Goal

Deliver high-value compression for the primary .NET tech stack: ASP.NET Core  EF Core  SignalR  AutoMapper  and serialization. This phase alone gives the LLM controller routes  hub methods  DbContext shape  and DI registration context.

### Scope

**New Module: `src/dotnet_meta/`**

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/dotnet_meta/mod.rs` | Public surface  `MetaBlock` struct  `run_meta_layer` entry point  `GraphCollector` |
| Create | `src/dotnet_meta/detect.rs` | AST-based .NET file detection (ASP.NET attributes  `: Controller`  `: DbContext`  `: Hub<T>`  etc.) |
| Create | `src/dotnet_meta/aspnet.rs` | Controller/Minimal API detection — `[ApiController]`  `[Route]`  `[HttpGet]`  method params  return types |
| Create | `src/dotnet_meta/efcore.rs` | Entity Framework — `DbContext`  `DbSet<T>`  `[Key]`  `[ForeignKey]`  `[Table]`  Fluent API via `OnModelCreating` |
| Create | `src/dotnet_meta/automapper.rs` | AutoMapper profiles — `CreateMap<TSrc  TDst>()`  `ForMember()`  `Ignore()`  `ProjectTo()` |
| Create | `src/dotnet_meta/signalr.rs` | SignalR — `Hub<T>`  `[HubMethodName]`  strongly-typed `IClientProxy`  streaming `ChannelReader<T>`  groups  `IHubContext` injection |
| Create | `src/dotnet_meta/serialization.rs` | `[JsonPropertyName]`  `[JsonIgnore]`  `[DataMember]`  `[IgnoreDataMember]`  `[JsonConverter]` |
| Create | `src/dotnet_meta/general.rs` | `[Service]`  DI registration (`AddScoped`/`AddSingleton`/`AddTransient`/`AddDbContext`)  validation attributes (`[Required]`  `[StringLength]`)  exception filters |
| Create | `src/dotnet_meta/markers.rs` | All Φ marker types — `PhiLineKind` enum  marker structs  `build_*` functions  `expand_phi_in_line` |
| Create | `src/dotnet_meta/graph.rs` | DI graph — service → controller → EF context resolution  hub → client interface links |
| Create | `src/dotnet_meta/graph_state.rs` | `DotnetGraphHandle` — McpState integration (mirrors `AngularGraphHandle`) |
| Create | `src/dotnet_meta/footer.rs` | `§ΦMAP` workspace footer for .NET bundles |

**Modifications to existing files:**

| Action | File | Purpose |
|--------|------|---------|
| Modify | `Cargo.toml` | Add `dotnet = ["csharp"]` feature |
| Modify | `src/layers/meta/mod.rs` | Add `DotNetMetaLayer` struct + `MetaLayer` impl (feature-gated) |
| Modify | `src/layers/registry.rs` | Register `DotNetMetaLayer` when `dotnet` feature is enabled |
| Modify | `src/mcp/state.rs` | Add `dotnet_graph: DotnetGraphHandle` field |
| Modify | `src/mcp/workspace.rs` | Post-compression graph build + `§ΦMAP` footer emission |
| Modify | `src/decompression/markers.rs` | Add `expand_phi_in_line` for .NET markers |
| Modify | `src/mcp/prompts.rs` | New ".NET Framework Meta Markers" section in `SYSTEM_PROMPT` |
| Modify | `src/config.rs` | Add `dotnet` meta-layer config |
| Modify | `docs/ARCHITECTURE_OVERVIEW.md` | Add `dotnet_meta/` module tree |
| Modify | `docs/ROADMAP.md` | Add .NET Meta-Layer items |

**Test files:**

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/tests/dotnet_meta/mod.rs` | Integration tests |
| Create | `src/tests/dotnet_meta/detect.rs` | Detector unit tests (positive + negative) |
| Create | `src/tests/dotnet_meta/aspnet.rs` | ASP.NET extraction tests |
| Create | `src/tests/dotnet_meta/efcore.rs` | EF Core extraction tests |
| Create | `src/tests/dotnet_meta/signalr.rs` | SignalR extraction tests |
| Create | `src/tests/dotnet_meta/automapper.rs` | AutoMapper extraction tests |
| Create | `src/tests/dotnet_meta/serialization.rs` | Serialization extraction tests |
| Create | `src/tests/dotnet_meta/general.rs` | General DI + validation tests |
| Create | `src/tests/dotnet_meta/markers.rs` | Φ marker round-trip tests |
| Create | `src/tests/dotnet_meta/graph.rs` | Graph build + resolution tests |
| Create | `src/tests/dotnet_meta/footer.rs` | Footer formatting tests |
| Create | `src/test_files/dotnet/` | Test fixtures (Controllers  Hubs  DbContexts  Profiles) |

### Completion Criteria — Phase 1

You will know Phase 1 is complete when **all** of the following are true:

**Functional**
- A `.cs` file with `[ApiController]` + `[Route("api/users")]` produces a `// --- Φ .NET Meta ---` block below the existing compacted class.
- The block contains `Φctrl:`  `Φaction:`  `Φmodel:`  `Φauth:` lines as appropriate.
- A `Hub<T>` file produces `Φhub:`  `Φmethod:`  `Φclient:`  `Φgroup:`  `Φstream:` lines.
- A `DbContext` file produces `Φef:`  `Φdbset:`  `Φentity:`  `Φrel:`  `Φcfg:` lines.
- An AutoMapper `Profile` produces `Φmap:`  `Φmapfrom:`  `Φignore:`  `Φproj:` lines.
- DI registrations (`AddScoped`  `AddSingleton`  `AddTransient`  `AddDbContext`) produce `Φdi:` lines.
- Validation attributes (`[Required]`  `[StringLength]`) produce `Φcommon:` lines.

**Non-regression**
- A non-.NET `.cs` file produces **zero** Φ markers and **zero** newlines of overhead.
- All existing tests still pass.
- `cargo clippy --all-targets -- -D warnings` is clean.

**Round-trip**
- `decompress_code_context` expands `Φctrl:` → `[Controller]`  `Φhub:` → `[Hub]`  etc.
- The expanded output is human-readable and preserves all original class names.

**Tests**
- New unit tests: detector (positive + negative)  extraction (each .NET subsystem)  marker round-trip  graph build.
- At least 12 new test files.
- All tests pass.

**Effort:** 5-7 days. **Risk:** Low-Medium (additive  no existing API changes  zero new deps).

---

## Phase 2: Angular Ecosystem Deepening (4-6 days)

> **Status:** shipped as R-23/R-24/R-25 — see `docs/ANGULAR_ECOSYSTEM_DEEPENING.md` (the living owner of the Angular RxJS/NgRx/Signals/Routing marker vocabulary) and `docs/ANGULAR_META_LAYER.md`. This phase's planning content is superseded by those documents.

---

## Phase 3: Integration & Pilot Readiness (3-5 days)

### Goal

Polish the meta-layers for real-world pilot usage: cross-layer CBM integration  per-domain stats  pilot config preset  comprehensive test fixtures  and documentation.

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `.clean-ctx-pilot.json` | Pilot config preset with optimal fidelity settings |
| Modify | `src/mcp/tool_handlers/stats/mod.rs` | Per-domain `context_stats` breakdown (EF  SignalR  NgRx  etc.) |
| Modify | `src/cbm/bridge.rs` | Cross-layer CBM edges — backend C# controller ↔ frontend Angular service |
| Create | `src/test_files/dotnet/pilot/` | Realistic multi-file .NET project fixture |
| Create | `src/test_files/angular/pilot/` | Realistic multi-file Angular project fixture |
| Create | `docs/DOTNET_META_LAYER.md` | This document — complete with before/after examples |
| Create | `docs/ANGULAR_ECOSYSTEM_DEEPENING.md` | Angular ecosystem plan document |
| Modify | `README.md` | Before/after examples for .NET + Angular |
| Modify | `docs/ROADMAP.md` | Mark all items complete |

**Pilot Config Preset (`.clean-ctx-pilot.json`):**
```json
{
  "fidelity": "medium" 
  "meta_layers": {
    "dotnet": { "enabled": true } 
    "angular": { "enabled": true } 
    "spring_boot": { "enabled": false }
  } 
  "resource_limits": {
    "max_file_size": 10485760 
    "max_workspace_files": 10000
  }
}
```

**Effort:** 3-5 days. **Risk:** Low (configuration + documentation + test fixtures).

---

## Red Flags & Gotchas

| Risk | Mitigation |
|------|------------|
| **Generic-heavy code** (AutoMapper  EF) | tree-sitter C# grammar handles generics `<T>` string scanners look for `CreateMap<`  `DbSet<`  `Hub<` patterns |
| **Partial classes & source generators** | tree-sitter can't resolve partial class merging detect `partial class` and emit a `⊕partial` marker |
| **SignalR streaming** | `ChannelReader<T>` return types on hub methods signal streaming `IAsyncEnumerable<T>` is another pattern |
| **Strongly-typed SignalR clients** | Detect `Hub<T>` where `T : class` is the client interface extract method signatures from the interface |
| **Version differences (.NET 6-9)** | Target the common subset (attributes stable since .NET 6) version-specific markers added later |
| **Attribute vs Fluent patterns in EF** | Detect both `[Key]` attributes and `HasKey()` Fluent API calls in `OnModelCreating` |
| **AutoMapper complex lambdas** | `ForMember(dest => dest.Name  opt => opt.MapFrom(src => src.FullName))` — string-based fallback scanning |
| **Performance on large DbContexts/Hubs** | Limit extraction to top-N DbSets/methods per class (configurable via fidelity) |

---

## Must-Haves for Pilot Success

- [ ] High-fidelity support for Controllers  DbContexts  Hubs  and NgRx stores
- [ ] Clear  LLM-friendly Φ markers
- [ ] Cross-file / cross-language graph edges (via CBM)
- [ ] Strong fallback behavior (string-based when AST fails)
- [ ] Detailed `context_stats` per domain (EF  SignalR  NgRx  etc.)
- [ ] Pilot-specific configuration & documentation with before/after examples

---

## Tracking

Each phase ends with:
1. A passing test suite (`cargo test`)
2. A clean linter (`cargo clippy --all-targets -- -D warnings`)
3. A ROADMAP status update (`📋 proposed` → `🚧 in-progress` → `✅ done`)
4. An entry in `CHANGELOG.md`

A phase is **not** complete until the user signs off on its completion criteria. We do not start the next phase until the current one is signed off.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.
