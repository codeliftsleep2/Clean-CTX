# Spring Boot Meta-Layer

> **Owner:** Current Spring/Java framework extraction reference · **Status:**
> Living per-layer reference (shipped, feature-gated)

The Spring Boot meta-layer is compiled by the `spring_boot` Cargo feature,
which depends on `java` and is not enabled by the default feature set. Runtime
configuration may disable an already compiled layer with
`meta_layers.spring_boot.enabled`; runtime configuration cannot add a layer
that was omitted at compile time.

## Production boundary

`src/spring_meta/` owns Spring detection, annotations, properties, presentation
markers, and typed semantic-edge extraction. It does not own cross-file graph
lifecycle. Produced edges travel with canonical compilation results into
`InferenceLayer.semantic_edges`; `WorkspaceIndex` owns occurrence replacement,
scope, hydration visibility, persistence restoration, and `workspace_query`
consumption.

Current typed relationships produced by `src/spring_meta/semantic.rs` are:

- `EndpointMapsTo` for controller endpoint mappings;
- `Autowired` for injection sites;
- `BeanProduces` for `@Bean` producers; and
- `ConfigurationProperties` for configuration-property ownership.

These relationship names are production facts, not a promise that every
annotation or marker becomes a graph edge. Dependency-cycle queries use their
separately approved relation policy and do not automatically treat every Spring
relationship as a dependency edge.

## Extraction scope

The layer recognizes the shipped Spring vocabulary implemented under
`src/spring_meta/`, including controller/service/repository/configuration
annotations, request mappings, injection, bean producers, configuration
properties, and Spring property files. Fidelity controls presentation detail;
semantic query completeness is governed by the compilation and WorkspaceIndex
boundaries above.

## Configuration

```json
{
  "meta_layers": {
    "spring_boot": {
      "enabled": true
    }
  }
}
```

See [`CONFIGURATION.md`](CONFIGURATION.md) for feature/runtime interaction,
[`ARCHITECTURAL_INVARIANTS.md`](ARCHITECTURAL_INVARIANTS.md) for durable
semantic ownership, and [`agent/tooling.md`](agent/tooling.md) for querying the
resulting workspace evidence.
