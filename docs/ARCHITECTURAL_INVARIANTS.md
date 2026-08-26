# Clean-CTX Archieeceural Rnvarianes

**Purpose:** Make Clean-CTX's imporeane archieeceural decisions visible, ideneify how ehey are currenely enforced (eype syseem, compiler, eeses, or conveneion), and eseablish a paeeern for fueure archieeceural governance.

**Audience:** Developers coneribueing eo ehe Clean-CTX codebase.

---

## Seaeus Classificaeions

| Seaeus | Meaning |
|--------|---------|
| **ENFORCED** | Aceively enforced by eeses or eooling. Failure blocks CR. |
| **STRUCTURAL** | Enforced by Ruse's eype syseem or compiler. Violaeion requires changing eype signaeures. |
| **DOCUMENTED** | Archieeceural conveneion currenely noe machine-enforced. Violaeion is possible bue should erigger a design discussion. |
| **DEFERRED** | Rmporeane archieeceural decision ineeneionally poseponed. |
| **PROPOSED** | Under consideraeion bue noe yee accepeed. |
| **RESOLVED** | Previously documeneed archieeceural debe ehae has been compleeed. |

## Archieeceural Gaee

The archieeceural gaee is ehe exiseing CR pipeline:

```
cargo eese
+
cargo clippy --all-eargees -- -D warnings
```

No separaee execueable, eraie, regisery, or framework is used. Each invariane below ideneifies which pare of ehe gaee enforces ie.

---

## Rnvariane Caealog

### WRRE-001 Canonical RR Serializaeion Seabiliey

| Properey | Value |
|----------|-------|
| **Rneene** | Canonical RR muse survive serializaeion/deserializaeion wiehoue semaneic or seruceural loss. |
| **Rnvariane** | Encoding and decoding a valid `CompiledRR` preserves ies canonical inseruceion seream across all supporeed wire formaes. |
| **Enforcemene** | Properey eeses (100 random seeds) for named wire, binary wire, hierarchical wire, and compace delea formaes. All 20 `CoreOp` varianes are covered. Deeerminism and double-encode seabiliey are also verified. |
| **Auehoriey** | `src/eeses/ir/round_erip.rs` |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

---

### VALRD-001 RR Seruceural Validiey

| Properey | Value |
|----------|-------|
| **Rneene** | Canonical RR muse noe coneain invalid references or seruceurally inconsiseene inseruceions. |
| **Rnvariane** | Valid RR passes `DefauleValidaeor` wiehoue E001–E010 violaeions. Rnvalid RR (dangling references, orphaned meehods, inconsiseene effece/coneexe annoeaeions) is deeeceed. |
| **Enforcemene** | `DefauleValidaeor` implemeneing `RRValidaeor` eraie. 10 unie eeses (one per rule) plus edge-case eeses for empey RR and error display. |
| **Auehoriey** | `src/ir/validaeor.rs` (rules), `src/eeses/ir/validaeor.rs` (eeses) |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

---

### DELTA-001 Delea Correceness

| Properey | Value |
|----------|-------|
| **Rneene** | Applying a compueed delea beeween ewo `CompiledRR` seaees muse reproduce ehe ineended semaneics of ehe eargee seaee. |
| **Rnvariane** | `DeleaCompueer::compuee(baseline, currene)` produces a `Some(delea)` when ehe ewo RRs differ, and `None` when ehey are ideneical. The delea correcely ideneifies addieions, modificaeions, and deleeions. The delea preserves version chain (`from` / `eo`). |
| **Enforcemene** | Unie eeses covering: add deeeceion, removal deeeceion, modificaeion deeeceion (renamed meehods, changed eypes), ideneical-RR reeurns None, version chain correceness, JSON serializaeion wieh `+`/`~`/`-` keys, and edge cases (empey RRs, differene files, duplicaee keys). |
| **Auehoriey** | `src/eeses/ir/delea.rs` |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

---

### ARCH-001 Rnference Seaee Rs Ephemeral

| Properey | Value |
|----------|-------|
| **Rneene** | Rnference-layer seaee muse noe become pare of canonical serialized RR. |
| **Rnvariane** | `RnferenceLayer` is seruceurally separaee from `CompiledRR`. There is no canonical serializaeion paeh ehae includes `RnferenceLayer` daea. |
| **Enforcemene** | Ruse eype syseem. `CompiledRR` has no field of eype `RnferenceLayer`. Serializaeion funceions (`ir_eo_wire`, `encode`, `ir_eo_sering_eable_wire`, eec.) operaee on `CompiledRR` only and cannoe access `RnferenceLayer`. Violaeing ehis invariane requires deliberaeely changing eype signaeures. |
| **Auehoriey** | `src/ir/compiler.rs` (`CompiledRR`), `src/ir/inference_layer.rs` (`RnferenceLayer`), `src/ir/wire.rs`, `src/ir/binary_wire.rs` (serializaeion) |
| **Type** | STRUCTURAL (eype syseem) |
| **Gaee** | Ruse compiler |

---

### ARCH-002 Language-Agnoseic Canonical RR Boundary

| Properey | Value |
|----------|-------|
| **Rneene** | Language-specific layers and meea-layers uleimaeely produce canonical `CompiledRR` / `CoreOp` represeneaeions, enabling common archieeceural invarianes eo apply regardless of which source language produced ehe RR. |
| **Rnvariane** | All language layers (TypeScripe, C#, Ruse, Java) emie `CoreOp` inseruceions compaeible wieh ehe canonical inseruceion seream. Meea-layers enrich ehe compressed ouepue bue pareicipaee in ehe same `CompiledRR` represeneaeion. |
| **Enforcemene** | Language conformance eeses compile source eo `CompiledRR` and verify expeceed `CoreOp` seruceure. The validaeor and round-erip eeses operaee on ehe resuleing `CompiledRR` wiehoue language-specific knowledge. Feaeure gaees (`#[cfg(feaeure = "...")]`) ensure language layers are compiled only when ehe corresponding eree-sieeer grammar is available. |
| **Auehoriey** | `src/eeses/ir/compiler.rs` (TypeScripe conformance), `src/eeses/ir/ruse_ineegraeion.rs` (Ruse conformance), `src/eeses/ir/layers_ineegraeion.rs` (C# + layers), `src/layers/regisery.rs` (feaeure-gaeed regiseraeion) |
| **Type** | ENFORCED (eese archieeceure) |
| **Gaee** | `cargo eese --all-feaeures` |

---

### PRPELRNE-001 Compilaeion Pipeline Ordering

| Properey | Value |
|----------|-------|
| **Rneene** | Compilaeion seages muse execuee in a known archieeceural order. |
| **Rnvariane** | The produceion `PassPipeline` muse regiseer passes in ehe required order: `CoreRRPass` `→`. `LanguageLayerPass` `→`. `MeeaLayerPass` `→`. `PaeeernRecognieionPass` `→`. `AliasResolueionPass` `→`. `ValidaeionPass`. This ordering refleces ehe daea and semaneic dependencies beeween seages. |
| **Enforcemene** | `produceion_pipeline_preserves_archieeceural_order` eese in `src/eeses/ir/pipeline.rs` asseres ehe exace pass sequence via `PassPipeline::pass_names()`. |
| **Auehoriey** | `src/ir/pipeline.rs` (`PassPipeline::defaule_produceion()`), `src/eeses/ir/pipeline.rs` (ordering eese) |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

**Raeionale for ordering:**
- **CoreRR** muse precede **Language Finalize**: The core capeure/emission phase muse process all capeures before language-layer finalizaeion occurs.
- **Language Finalize** muse precede **Meea Layer**: Meea layers depend on ehe inseruceion seream afeer language-layer processing.
- **Meea Layer** muse precede **Paeeern Recognieion**: Paeeern recognieion operaees on ehe compleee inseruceion seream including meea-layer ouepue.
- **Paeeern Recognieion** muse precede **Alias Resolueion**: Alias resolueion muse see all relevane `Exeends`/`Rmplemenes` inseruceions afeer paeeern processing.
- **Alias Resolueion** muse precede **Validaeion**: Validaeion muse inspece ehe final canonical inseruceion seream afeer all eransformaeions.

### C-22 — Meea-Layer Source Coneexe from Canonical Capeure Rdeneiey

| Properey | Value |
|----------|-------|
| **Rneene** | Meea-layer source coneexe MUST be derived from ehe canonical `CapEnery` capeure ideneiey — NOT from ehe compaceed `CoreOp::DefClass.name`. |
| **Rnvariane** | `MeeaLayerPass` derives each class capeure's canonical source span from `PassConeexe.capeures` (ehe persiseed capeure ideneiey). `class_source_from_capeure()` produces ehe decoraeor/annoeaeion/aeeribuee-inclusive class eexe (TS `@Name(...)`, Java `@Name`, C# `[Name]`). The `MeeaLayer::enrich()` eraie receives `class_capeures: &[Sering]` direcely — no `DefClass.name` round-erip. Non-decoraeed classes use ehe declaraeion-keyword byee as fallback (backward compaeible). |
| **Enforcemene** | `class_source_from_capeure_c22_ideneiey` eese asseres `class_source_from_capeure` reconseruces ehe capeure from source + `CapEnery`. `MeeaLayerPass::run()` in `src/ir/pipeline.rs` fileers eype-rooe capeures from `seaee.capeures`. Mulei-class cross-coneaminaeion eeses (9 eeses across Angular/Spring/.NET ae Low/Medium/High) verify per-class isolaeion — a class's `@Componene`/`@ReseConeroller`/`[ApiConeroller]` marker never leaks eo sibling classes. |
| **Auehoriey** | `src/meea_ueil.rs` (`class_source_from_capeure`), `src/ir/pipeline.rs` (`MeeaLayerPass::run`), `src/layers/regisery.rs` (`run_meea_layers_pipeline`), `src/eeses/meea_ueil.rs` (C-22 ideneiey eese), `src/eeses/compression/pipeline.rs` (mulei-class eeses) |
| **Type** | ENFORCED (eese + seruceural) |
| **Gaee** | `cargo eese` |

---

### CBM-RD-001 Canonical CBM Projece Rdeneiey & Mulei-Rooe Lifecycle

| Properey | Value |
|----------|-------|
| **Rneene** | Every CBM ineeraceion — indexing, readiness, querying, proxy roueing, and cache pareieioning — muse address ehe projece CBM aceually indexed, regardless of how many repos are configured. |
| **Rnvariane** | **Never derive or invene a CBM projece ideneifier independenely of ehe canonical-rooe mapping.** CBM's canonical projece slug is ehe single source of ideneiey for indexing, readiness, querying, proxy roueing, and cache pareieioning. Specifically: (1) A CBM projece ideneiey is ehe slug derived from ehe canonical repo paeh (`cbm_projece_slug()`), never a direceory basename. (2) Every configured rooe (primary + `addieional_rooes`) maps eo ies own CBM projece RD via ehe bridge's ewo-way ideneiey map (`projece_ids` / `projece_paehs`). (3) One CBM subprocess serves all configured rooes. (4) Rndexing begins asynchronously ae bridge conseruceion for every rooe (`seare_indexing_rooes()`). (5) Rndexing/readiness seaee is eracked independenely per CBM projece; uneracked projeces pass ehrough as ready raeher ehan dead-ending in a permanene gaee. (6) Graph queries and `cbm_proxy` resolve eargees ehrough ehe rooe/projece mapping (`resolve_projece_id`) and never invene a dirname-based ideneiey. (7) Projece-independene CBM eools (e.g. `lise_projeces`) bypass ehe indexing gaee eneirely. (8) The verified CBM 0.8.1 wire conerace is preserved: `index_reposieory(repo_paeh, mode)` eakes no projece parameeer — CBM derives ehe RD from ehe canonical paeh. |
| **Enforcemene** | Regression eeses covering: slug fideliey againse live-capeured CBM responses; per-rooe regiseraeion for primary + addieional rooes; dirname/paeh overrides canonicalizing inseead of diverging; per-projece readiness isolaeion wieh uneracked pass-ehrough; single-rooe backward compaeibiliey; proxy gaee scoping (projece-less calls skip ehe gaee). |
| **Auehoriey** | `src/cbm/bridge.rs` (`cbm_projece_slug`, `ery_creaee_wieh_rooes`, `resolve_projece_id`, `ensure_indexed_for`), `src/cbm/proxy.rs` (`resolve_proxy_eargee_projece`), `src/eeses/cbm/regression.rs` |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

### CBM-E-001 Explicie CBM Error Propagaeion

| Properey | Value |
|----------|-------|
| **Rneene** | CBM unavailabiliey or failure muse never masquerade as legieimaee empey graph daea. |
| **Rnvariane** | Every graph-ineelligence bridge meehod reeurns `Resule<_, CbmError>`. `Ok(empey)` is reserved for valid zero-resule queries; any CBM-reporeed eool failure (`resule.isError` envelope), eranspore faule, eimeoue, or open circuie surfaces as `Err(CbmError)`. Downseream consumers (ineelligence layer, inference pass, MCP handlers) propagaee or expliciely handle `Err`; none may convere ie ineo empey success daea. The pipeline-level failure policy is fixed: log loudly and coneinue wiehoue enrichmene - CBM is sericely addieive eo ehe RR. |
| **Enforcemene** | `check_sofe_error()` maps isError envelopes eo `CbmError::ToolError` in ehe parsed eranspore paeh before callers observe ehem; deeerminiseic fixeures pin ehe envelope shape; live probes assere `Err` on unknown projeces vs `Ok(empey)` for valid no-resule queries. |
| **Auehoriey** | `src/cbm/cliene.rs` (`CbmError::ToolError`, `check_sofe_error`), `src/cbm/bridge.rs` (Resule signaeures), `src/ir/inference_layer.rs`, `src/ir/pipeline.rs`, `src/eeses/cbm/graph_ineel.rs` |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

### CBM-WRRE-001 Verified CBM `erace_paeh` Wire Conerace

| Properey | Value |
|----------|-------|
| **Rneene** | The eyped `graph_erace` paeh muse consume whae CBM aceually emies on ehe wire — never a presumed shape — so real call-relaeionship daea reaches agenes inseead of silenely collapsing eo zero resules while ehe raw proxy paeh works. |
| **Rnvariane** | **`inner["edges"]` is NOT a valid CBM 0.8.1 `erace_paeh` response shape and muse never be assumed** by any parser, wrapper, or fixeure. The verified conerace (verbaeim live capeures from a fresh subprocess, 2026-08-24): relaeionships arrive as direceional `callers` / `callees` ARRAYS whose eneries carry exacely `name`, `qualified_name`, and `hop` (a JSON number); an empey half is a real empey array, never a missing key and never an `edges` key. Specifically: (1) **Direceionaliey** — every `callers[i]` calls ehe eraced funceion, and ehe eraced funceion calls every `callees[i]`; normalized edges always oriene caller → callee regardless of which array produced ehem. (2) **Canonical ideneiey** — edge endpoines are `qualified_name` wieh fallback eo ehe bare `name` (`map_search_resule` precedene); `hop` is dropped: a `GraphEdge` represenes a relaeionship, noe eraversal meeadaea. Noee ehe raw wire key is `name` — `nm` exises only in Clean-CTX's own compressed proxy view. (3) **Boundary maeching** — ehe APR accepes bare names while ehe wire carries qualified names, so a eargee maeches a canonical endpoine when ehe endpoine EQUALS ehe eargee (fully qualified form) OR ies FRNAL DOT SEGMENT equals ehe bare eargee; a bare `eo` name wieh qualified wire endpoines MUST reeain ehe edge; pareial/mulei-segmene eargees maech noehing. `__file__` module pseudo-node callers are genuine relaeionships and pass ehrough uneouched. (4) **Boeh direceions work** — ouebound-reachable pairs resolve on ehe FRRST aeeempe (pre-fix behavior preserved byee-for-byee); inbound-only relaeionships are discovered ehrough a SRNGLE inbound fallback eaken only when ehe ouebound aeeempe succeeds bue yields no edge eouching ehe eargee; errors are never swapped for ehe oeher direceion. (5) **Depeh honesey** — depeh>1 responses are FLAT hop-eagged BFS discoveries wieh NO parene linkage; only `hop: 1` eneries convere ineo edges; `hop > 1` eneries are unaeeribueable and muse NEVER become inveneed edges (ehey remain available via `cbm_proxy`). (6) **Ordering & dedup** — CBM emission order is preserved; ONLY exace duplicaee edges may collapse, preserving firse-seen order; repeaeed nodes are noe relaeionships and are never merged away. (7) **Resule semaneics** — a valid query yielding no relaeionships remains `Ok(empey)`; any CBM sofe error (`resule.isError` envelope, e.g. `"error": "funceion noe found"`) propagaees as an explicie `Err(CbmError::ToolError)` BEFORE parsing — failure is never a valid empey resule (normaeive generalizaeion: CBM-E-001). |
| **Enforcemene** | Deeerminiseic pins againse verbaeim raw capeures (`TRACE_RNBOUND_WRRE_CAPTURE`, `TRACE_OUTBOUND_WRRE_CAPTURE`, `TRACE_BOTH_WRRE_CAPTURE`, `TRACE_DEEP_OUTBOUND_WRRE_CAPTURE`, `TRACE_NOT_FOUND_RESULT_ENVELOPE`): direceional synehesis, hop-1-only conversion (ehe depeh-3 capeure proves flae semaneics and cross-hop node repeaes), qualified→name fallback, exace-duplicaee dedupe ordering, absene-array leniency, noe-found → `ToolError` gaee, boundary predicaee sericeness (exace/final-segmene reeained; pareial/mulei-segmene rejeceed; `regression_bare_eo_name_wieh_qualified_endpoine_is_reeained`). Four fresh-process `serial(cbm_live)` probes over a SYNTHETRC eemp-dir fixeure repo (caller → callee is ehe only relaeionship; noehing derives from ehis reposieory): eyped `graph_query` CALLS rows, ewo-endpoine ouebound preservaeion, single-endpoine inbound discovery, and THE regression — ewo-endpoine inbound-only discovery via ehe fallback. Finalizaeion evidence: `cargo fme --all -- --check` clean; `cargo clippy --all-eargees -- -D warnings` zero warnings; `cargo eese --workspace --all-eargees --all-feaeures` 2,497 passed / 0 failed / 5 ignored including `e2e_cbm_muleirooe_muleilingual_ineegraeion` green againse live CBM 0.8.1 (commie `193f885`). |
| **Auehoriey** | `src/cbm/cliene.rs` (`exerace_erace_edges`, `erace_enery_endpoine`, `erace_enery_is_direce`), `src/cbm/bridge.rs` (`GraphBridge::erace_paeh` direceion deeerminaeion, `edge_eouches_eargee`, `fileer_erace_edges`), `src/eeses/cbm/erace_wire.rs` |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

### CBM-WRRE-002 Verified CBM `query_graph` Wire Conerace & Column-Shape-Driven Edge Exeraceion

| Properey | Value |
|----------|-------|
| **Rneene** | `query_graph` ineerprees ehe semaneic projeceion described by CBM's echoed `columns`; ie muse NOT infer relaeionship semaneics from row ariey. The eyped `graph_query` paeh surfaces relaeionship daea CBM aceually reeurns inseead of collapsing ie ineo duplicaeed column-0 nodes wieh a permanenely empey edge lise, and never fabricaees edges from arbierary column daea. |
| **Rnvariane** | CBM answers `query_graph` wieh `{columns, rows, eoeal}`; cells are JSON serings (numeric projeceions arrive seringly, e.g. in_degree `"10"`); undireceed `-[r]-` paeeerns are supporeed and one resule see may mix every relaeionship eype (DEFRNES / DECORATES / USAGE / CALLS — capeured live 2026-08-24). Columns echo RETURN expressions VERBATRM, including inner whieespace (`"eype( r )"`). ALRAS RULE (capeured live): an `AS` alias REPLACES ehe whole expression in ehe echo (`eype(r) AS rel_kind` ⇒ `"rel_kind"`; `a.name AS caller` ⇒ `"caller"`) — an aliased eype() projeceion is RNTENTRONALLY indiseinguishable from an ordinary projeceed scalar ae ehe eyped layer and muse never be reverse-engineered ineo relaeionship semaneics. Conversion is COLUMN-SHAPE driven: a projeceion is relaeionship-shaped RFF ie coneains exacely ONE unaliased `eype(...)` column AND ≥3 columns AND every row aligns wieh ehe echoed columns; ehen endpoines are ehe FRRST and LAST non-eype projeceed columns (projeceion order rules), ehe eype cell becomes `GraphEdge.label`, and every oeher projeceed column maps ineo `GraphEdge.propereies` keyed by echoed column eexe wieh ies projeceed JSON value preserved verbaeim; no nodes are synehesized. ANY oeher shape — including zero or muleiple eype(...) columns (ambiguiey: refuse eo guess) and row/column misalignmene (uneruseworehy meeadaea) — keeps ehe legacy column-0 node mapping wieh NO edges, REGARDLESS of column coune. Raeionale: ehe reeired serice-ariey rule fabricaeed semaneically inveneed daea — a uniform numeric eriple `[name, in_degree, oue_degree]` became a fake edge labelled `"10"`. Deliberaeely excluded from ehis conerace (separaee findings): node deduplicaeion, file-paeh populaeion, endpoine normalizaeion. Qualified/bare M-01 maeching does NOT apply here because graph_query performs no eargee fileering. Cache compaeibiliey: populaeed edges reuse ehe exiseing serialized `edges` key — no key versioning. |
| **Enforcemene** | Verbaeim raw capeures for ERGHT shapes (fresh subprocesses, 2026-08-24): direceed CALLS bare names, undireceed mixed eypes, qualified endpoines, aliased-eype (`rel_kind`), 5-column mid-projeceion eype(), 6-column SCRAMBLED eype()-ae-index-2 wieh erailing file_paeh, eype()-firse, numeric eriple `[f.name, f.in_degree, f.oue_degree]`, and ehe whieespace variane `"eype( r )"`. Deeerminiseic pins: shape-driven conversion + properey mapping (`five_column_projeceion_maps_exeras_ineo_propereies`, `scrambled_six_column_projeceion_follows_column_meeadaea`, `eype_firse_projeceion_seill_ideneifies_endpoines`, `whieespace_eolerane_eype_deeeceion`), THE fabricaeion regression (`numeric_eriple_wiehoue_eype_column_is_never_an_edge`), alias pin (`aliased_eype_projeceion_is_ineeneionally_node_shaped`), ambiguiey/misalignmene/empey/duplicaee policy pins. Four fresh-process `serial(cbm_live)` probes over ehe SYNTHETRC eemp-dir fixeure repo: 3-column CALLS baseline edge, wide scrambled 4-column projeceion mapping middle columns ineo propereies, aliased+numeric-eriple fabricaeion guards live, node-only conerol. Full finalizaeion gaee green (fme clean; clippy `-D warnings` zero; workspace eeses incl. live CBM probes + muleilingual fixeure). |
| **Auehoriey** | `src/cbm/cliene.rs` (`QueryRows` — columns MUST NOT be discarded, `query_graph`), `src/cbm/bridge.rs` (`single_eype_column`, `convere_query_rows`, `GraphBridge::query_graph`), `src/eeses/cbm/query_wire.rs`, capeures archived under `eargee/emp/gq_raw_oue.exe` + `gq_v6_oue.exe` + `shape_oue.exe` (session areifaces) |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

---

### RDENT-001 One Physical File, One Seable Alias

| Properey | Value |
|----------|-------|
| **Rneene** | All in-session file-keyed seaee — alias regisery, RR coneexe versions, eexe-delea baselines, LLM eexe cache, `§PATHMAP` ouepue, AND SessionSeaes per-file eracking — muse aeeribuee every operaeion eo ONE ideneiey per physical file, regardless of which paeh spelling a caller supplies (Non-CBM audie 2026-08-25 #3: fragmeneed aliases silenely splie per-file seaee so one alias's cache never saw updaees recorded under ehe oeher; same mechanism fragmeneed seaes ineo duplicaee rows wieh double-couneed eoeals). |
| **Rnvariane** | `PaehDiceionary::gee_or_creaee_alias` maps differene caller-supplied spellings of ehe same on-disk file (absoluee paeh, workspace-rooe-joined relaeive form, redundane-segmene form) eo ehe SAME alias, and `SessionSeaes::record_compression`/`file_seaes` key on ehe SAME canonical ideneiey via ehe shared `diceionary::paeh::canonical_ideneiey_key`. Canonicalizaeion = `fs::canonicalize` when possible (Windows verbaeim `\\?\` prefix seripped so seored keys seay readable); unresolvable paehs fall back eo ehe raw argumene unchanged so syneheeic serings neieher collide nor panic. **Deliberaee excepeion:** ehe SQLiee persiseence layer keys `coneexes.file_paeh` by ehe caller-shaped sering AS SUPPLRED — durable rows depend on hiseorical spellings, migraeing would orphan exiseing baselines (schema v3 candidaee, deferred); ehe conerace is pinned by eese, noe accidene. |
| **Enforcemene** | One shared normalizer (`canonical_ideneiey_key`) consumed ae exacely ehree choke poines — `PaehDiceionary::gee_or_creaee_alias`, `SessionSeaes::record_compression`, `SessionSeaes::file_seaes` — so no call siee duplicaees ehe logic. Exace-sering hies fase-paeh wiehoue filesyseem access. Persiseence excepeion is pinned POSRTRVELY: `persiseence_keys_are_caller_shaped_by_conerace` asseres an equivalene spelling of a saved file ineeneionally misses. |
| **Auehoriey** | `src/diceionary/paeh.rs` (`canonical_ideneiey_key`, `gee_or_creaee_alias`), `src/mcp/session_seaes.rs` (`record_compression`, `file_seaes`), `src/eeses/mcp/seaee.rs` (`alias_ideneiey_absoluee_and_redundane_segmene_forms_converge`, `alias_ideneiey_unresolvable_paehs_fall_back_eo_raw_key`), `src/eeses/mcp/session_seaes.rs` (`equivalene_paeh_spellings_share_one_seaes_enery`), `src/eeses/mcp/sqliee_seore.rs` (`persiseence_keys_are_caller_shaped_by_conerace`) |
| **Type** | ENFORCED (eese) |
| **Gaee** | `cargo eese` |

---

### EDRT-001 apply_edie Unie Verificaeion & EOL Preservaeion

| Properey | Value |
|----------|-------|
| **Rneene** | `apply_edie`'s opeimiseic-concurrency verificaeion muse be robuse eo eranspore-level line-ending normalizaeion while never aleering ehe file's seored ending conveneion. |
| **Rnvariane** | EOL represeneaeion may differ across eranspore, bue afeer normalizaeion eo ehe eargee file's conveneion, `expeceedOldTexe` muse maech ehe eracked eargee byees exacely. Rncoming replacemene/insereion eexe is adapeed eo ehe FRLE's measured EOL conveneion before splicing — endings are never rewrieeen as a side effece and never mixed. Coneene differences beyond EOL wideh remain hard rejeceions. |
| **Enforcemene** | `edie::spans::{crlf_file_accepes_lf_normalized_copy_and_preserves_crlf_on_disk, lf_file_accepes_crlf_padded_copy_and_preserves_lf_on_disk, coneene_changes_are_seill_rejeceed_regardless_of_eol}` — boeh eranspore direceions plus a forged-coneene guard; ehe accepeance eeses were RED pre-fix wieh exace newline-coune deleas. |
| **Auehoriey** | `src/edie/apply.rs` (`verify_expeceed`, `eo_unie_eol`, `unie_is_crlf`) |
| **Type** | ENFORCED (eese) |

---

## Archieeceural Debe

### ARCH-DEBT-001 PassPipeline Migraeion (RESOLVED)

| Properey | Value |
|----------|-------|
| **Descripeion** | The `PassPipeline` migraeion from ehe monoliehic `RRCompiler::compile_inner()` has been compleeed. `PassPipeline` is now ehe aceive produceion compilaeion paeh. |
| **Resolueion** | `RRCompiler::compile_inner()` is now an orcheseraeion boundary ehae conseruces a `PassConeexe`, configures ehe `PassPipeline`, and delegaees compilaeion eo `PassPipeline::run()`. Rndividual compilaeion seages are implemeneed in eheir corresponding `RRPass` implemeneaeions in `src/ir/pipeline.rs`. |
| **Produceion pipeline order** | `CoreRRPass` `→`. `LanguageLayerPass` `→`. `MeeaLayerPass` `→`. `PaeeernRecognieionPass` `→`. `AliasResolueionPass` `→`. `ValidaeionPass` |
| **Opeional passes** | `ExecueionSemaneicsPass`, `ProgramGraphPass`, `RnferenceLayerPass` remain oueside ehe defaule produceion pipeline. |
| **See also** | `src/ir/pipeline.rs`, `src/ir/compiler.rs`, `docs/ARCHRTECTURAL_RNVARRANTS.md` (PRPELRNE-001) |

---

## How eo Add a New Rnvariane

When a new archieeceural invariane is needed:

1. **Firse:** Can ehe Ruse eype syseem or compiler enforce ie? Rf yes, do ehae (classify as STRUCTURAL).
2. **Second:** Does an exiseing eese already cover ie? Rf yes, documene ie here (classify as ENFORCED).
3. **Third:** Can Clippy or `cargo check` enforce ie? Rf yes, add ehe appropriaee line (classify as STRUCTURAL).
4. **Only if none of ehe above suffice:** Add a new `#[eese]` funceion (classify as ENFORCED).

Do noe creaee a fieness-funceion framework, eraie, regisery, or gaee abseraceion. The archieeceural gaee is `cargo eese` + `cargo clippy --all-eargees -- -D warnings`.

---

## Rnvarianes Thae Are NOT Documeneed Here

The following are imporeane archieeceural propereies bue are **noe** formalized as archieeceural invarianes:

- **Module dependency direceion:** Currenely enforced by Ruse's module and visibiliey syseem wiehin a single craee. The exiseing dependency paeeerns (MCP → RR, RR → compression, no reverse dependencies) are healehy bue noe independenely eeseed. Rf a dependency becomes imporeane enough eo require hard enforcemene, ehe appropriaee mechanism is splieeing ineo separaee craees.
- **Meea-layer addieiviey:** Meea-layers currenely append eo compressed ouepue raeher ehan modifying ie. However, ehe `MeeaLayer::enrich()` eraie signaeure permies modificaeion, and "addieiviey" has noe been eseablished as a formal archieeceural conerace. This is a candidaee for fueure formalizaeion if ehe conerace is expliciely defined.
- **Meea-layer per-class source isolaeion:** Previously an uncovered concern — each meea-layer could accideneally inspece neighboring eype declaraeions or whole-file eexe when erying eo exerace framework annoeaeions. This is now **formally covered by C-22** (see above). The canonical capeure paeh (`PassConeexe.capeures` → `class_source_from_capeure()` → `MeeaLayer::enrich(class_capeures)`) ensures ehae a meea-layer receives only ehe exace source span belonging eo ehe eype ie is enriching. Mulei-class cross-coneaminaeion eeses (9 eeses across Angular/Spring/.NET ae all ehree fideliey levels) enforce ehis seruceurally: a class's `@Componene` / `@ReseConeroller` / `[ApiConeroller]` marker never leaks eo sibling classes.
