---

## [0.1.2] — 2026-06-07

### Added

#### Angular Meta-Layer Phase 3 (Cross-File Dependency Graph)
- `src/angular_meta/graph.rs` — `AngularGraph` struct with `ClassKind` (Component/Service/Directive/Pipe/Module), `ClassEntry` metadata, DI injection resolution (`resolve_inject_type` → `"UserService@α12"`), selector linkage (`resolve_selector` → `"UserCardComponent@α9"`), `Φgraph:` marker lines with injects/injected-by edges, `§ΦGRAPH` footer formatter
- `src/angular_meta/graph_state.rs` — `AngularGraphHandle` (Arc<Mutex<Option<AngularGraph>>>) for thread-safe McpState integration
- `src/mcp/state.rs` — New `angular_graph: AngularGraphHandle` field for cross-file graph lifecycle
- `src/mcp/workspace.rs` — Phase 3 post-compression graph building pass: text-based class block extraction (`extract_class_blocks`), `GraphCollector` batch collection, angular graph build + store + manifest emission
- `src/angular_meta/decorators.rs` — New `extract_graph_entries()` public function returning `(class_name, kind, selector, injects, pipe_name)` for graph registration
- Test fixtures: `src/test_files/angular/graph/` directory with 4 files (`user-card.component.ts`, `user.service.ts`, `logger.service.ts`, `user-page.component.ts`) for cross-file DI + selector testing
- 36 new unit tests: graph construction/selectors/DI/footers (14), DI resolution including transitive + reverse edges + resolution failure (11), selector linkage including multi-class + directive + pipe (11)
- Total test count: 279 (up from 244)
- `cargo clippy --all-targets -- -D warnings`: 0 warnings (clean)
- Documentation updated: `docs/ARCHITECTURE_OVERVIEW.md` (new Phase 3 section + module tree), `docs/ROADMAP.md` (R-22 Phase 3 ✅)

### Fixed
- Collapsible `if` in `src/angular_meta/template.rs` (pre-existing clippy warning suppressed by collapsing nested check into single condition)

---

