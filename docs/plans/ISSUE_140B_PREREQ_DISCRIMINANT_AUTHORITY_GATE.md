# #140b prerequisite: legacy discriminant declaration authority

**#140b-PREREQ LEGACY NGRX DISCRIMINANT DECLARATION AUTHORITY BLOCKED**

Investigation base: `142b75c`, `feature/current-cbm-pinned-upgrade`.
The outcome is Model C for the standalone declaration: a generic TypeScript
constant. Additional structured NgRx qualification remains to be established.
No production implementation or invalid regression RED was manufactured.

## 33.1 Repository-supported syntax

| Form | Classification | Evidence and limits |
| --- | --- | --- |
| `export const SET_AUDIT = '[Audit] Set'` participating in `{type: SET_AUDIT}` dispatch and `switch(action.type) / case SET_AUDIT` | INTENDED_SUPPORTED as a combined historical capability | Original #140 acceptance; `Invoke-NativeIssueReconciliation.ps1` declaration, dispatch and switch fixtures. Does not define independent declaration qualification |
| The same standalone constant, including with a Store import | GENERIC_TYPESCRIPT_ONLY under current contract | `ngrx_legacy_declaration_negative_and_disabled_controls`; `Invoke-NgrxLegacyDeclarations.ps1` string-constant negative; #140a report |
| Directly imported NgRx `Action` implemented by a top-level class | INTENDED_SUPPORTED and implemented | Frozen `ngrx/legacy.rs`; class declaration authority is independent of field values |
| `readonly type = SET_AUDIT` or `static readonly type = SET_AUDIT` as discriminant qualification | NOT_ESTABLISHED | No such qualification contract found in inspected producers, NgRx tests/fixtures, reports or harnesses; handoff examples are illustrative |
| Enum/member/object literal discriminant declaration authority | NOT_ESTABLISHED | No authoritative legacy declaration producer identified |

The original #140 issue is historical acceptance evidence, not current-code
proof. Its expected combined capability cannot substitute for the missing rule
that distinguishes qualifying declarations from ordinary unused constants.
The #140a frozen negative is direct evidence against treating all constants in
an NgRx-importing file as Actions. No filename, name prefix or string format is
accepted as qualification.

## 33.2 First-loss trace

Primary standalone source:

```ts
export const SET_AUDIT = '[Audit] Set Audit';
```

| Stage | Classification | Evidence |
| --- | --- | --- |
| TypeScript declaration syntax | PRESENT | Repository operator fixture |
| Generic variable declaration/name query | PRESENT | `TS_CALL_QUERY` variable_declarator/declaration_name captures |
| Standalone persisted canonical constant identity | NOT ESTABLISHED by this investigation | Private invocation declaration context is not a general public constant catalog |
| NgRx-local declaration qualification | MISSING | Modern producer consumes `createAction`; frozen legacy producer consumes imported Action + class implements evidence |
| Authoritative discriminant identity | NOT ESTABLISHED | Generic constant/name/value does not prove the NgRx role |
| Independent discriminant semantic carrier | UNSUPPORTED currently | Existing self-Defines carrier can register known entities but cannot supply missing semantic qualification |
| Index/public NgRx discriminant declaration | MISSING upstream | No qualifying record is produced; no downstream publication fix is justified |

These are PROVEN BY CODE INSPECTION, except explicitly NOT ESTABLISHED rows.
Tree-sitter parse capability is not alleged to be broken. Generic source capture
and semantic authority are separate requirements.

## 33.3 Semantic authority

The primary declaration proves a constant binding to a string literal. It does
not alone prove NgRx affiliation. A Store/Action import elsewhere does not prove
that this particular constant is an Action discriminant. Existing object-type
dispatch extraction proves a use-site symbolic endpoint, not an independently
qualified declaration. Matching that endpoint name back to a constant would be
a new declaration/reference rule, not reuse of an established binding contract.

No constant folding, lexical classification or value comparison was added.
Switch linkage remains outside this prerequisite.

## 33.4 Identity contract

No new discriminant identity contract was implemented. Existing semantic entity
keys use domain/type/name with file-provenanced source occurrences; self-Defines
registration preserves source occurrences and is not a graph relationship.
That machinery can carry an already justified declaration, but cannot establish
qualification or same-file lexical reference resolution by itself.

If a future owner-approved form qualifies a discriminant, its source declaration
span and file must remain the authority. Equal string values cannot merge two
bindings, and a source occurrence must not be claimed as declaration-resolved
reference authority merely because its display name matches.

## 33.5 Class relationship

**NOT ESTABLISHED.** #140a `ClassActionEvidence` contains canonical class name,
class span, implements span and import span. `ClassActionDecl` contains name.
It neither extracts nor resolves an Action `type` initializer.

A class and constant with equal literals are distinct declarations. Their equal
values do not establish a class-to-discriminant relation. An explicit identifier
initializer would be stronger syntactic evidence than equal literals, but its
supported qualification, shadowing/binding rules and public semantic meaning
are not established by the current repository contract. No such relationship
was invented or implemented.

## 33.6 Negative controls

The current tracked #140a regression explicitly treats a standalone exported
string constant with a Store import as a negative. The corresponding live
harness has the same negative oracle. They were inspected, not executed here.

No new ordinary/unused/same-value/same-name collision tests were run. A proposed
qualifier must explain why `SET_THEME = 'dark'` and a nonparticipating constant
remain generic before any discriminant-positive RED is valid.

## 33.7 RED/GREEN matrix

No case has a valid positive discriminant semantic oracle under the currently
established contract. Therefore no RED was registered or run, and there is no
GREEN claim. The handoff explicitly requires stopping when qualification or
identity cannot be established; a manufactured positive test would violate it.

## 33.8 Public MCP evidence

No MCP runtime was run for this prerequisite. Prior #140a evidence is historical
preservation evidence for its own slice, not new discriminant acceptance.
Raw constant text, a marker or a use-site Action endpoint would not establish
independent discriminant declaration authority.

## 33.9 Reference authority

The existing private invocation occurrence channel captures argument/callee and
declaration-context spans. Its documentation explicitly excludes persistence,
semantic edges and public MCP schemas. It is not a general TypeScript identifier
binding engine. Current switch/case roles are absent from TS queries.

Exact same-file binding for a discriminant reference is NOT ESTABLISHED here.
Cross-file direct imports, aliases and namespaces remain SEPARATE DEFECTS for
this proposed authority path; no resolver or suffix matching was introduced.
This prerequisite makes no reducer-linkage claim.

## 33.10 Lifecycle

No runtime lifecycle was executed. Rename, qualification removal and deletion
acceptance must follow a valid declaration contract and implementation; none is
claimed prematurely. No #142 behavior was modified or inferred.

## 33.11 Compatibility/generation

No durable product changed. Angular marker generation remains **4**, semantic
producer generation **6**, canonical pipeline generation unchanged. No reflexive
version bump was made.

## 33.12 Preservation

All frozen production and test code is unchanged. No fresh preservation commands
were run; inspected historical tests are not reported as current executed checks.

## 33.13 Remaining #140 work and owner decision

The smallest next decision is an explicit qualification/model contract, rather
than downstream publication. Reasonable choices:

1. Authorize a same-file, source-bound identifier initializer on an already
   admitted #140a class (`readonly type = SET_AUDIT`) as qualifying the referenced
   constant as a distinct discriminant. This is a PROPOSAL, not an observed
   repository contract. It requires a truthful public kind/carrier decision and
   collision/shadowing controls; no constant/class merging is permissible.
2. Authorize declaration qualification from an explicitly typed NgRx Action
   object carrier if that is the actual repository's legacy declaration form.
   Supply representative source and an independent role oracle first.
3. Keep standalone constants generic and pursue use-site reference facts only.
   This cannot satisfy independent discriminant publication or the current
   prerequisite objective and must be stated as a different scope.

Recommendation: establish the actual application's explicit declaration carrier
and choose the public meaning before implementation. A mere exported literal
cannot distinguish an Action discriminant from ordinary configuration.

The existing Action kind already accommodates creators and class Actions, but
that does not prove that a type-value binding should silently become another
Action. Public discriminant kind/meaning remains an owner-reviewed modeling
boundary under the handoff; no new kind or relation is authorized here.

Switch/case captures, legacy reducer extraction, resumed TriggersReducer linkage,
unsupported discriminants, constructor dispatch, aliases/namespaces/barrels and
module binding, service ownership, Selectors/Effects, representative recall and
cold/warm behavior remain separate. Existing Action → TriggersReducer → Reducer
and Effect → HandlesAction → Action semantics remain unchanged.

## 33.14 Changed files

Added only this report. The preceding
`ISSUE_140B_LEGACY_HANDLER_AUTHORITY_GATE.md` remains the earlier investigation
artifact; it has not been rewritten. No production, test, harness, cache,
discovery, hydration, template, CBM or LSP changes were made.

## 33.15 Commands and artifacts

Executed bounded handoff download/read, focused `rg` searches, `sed` reads and
`git status`; final UTF-8/BOM and `git diff --check` checks. One search glob for
nonexistent NgRx plan names returned a diagnostic; subsequent searches used
actual docs paths. No finding relies on that absent glob.

No Cargo/test/build/Clippy, binary/server, PowerShell harness, live CBM or model
host was started. No runtime authorization was requested because the required
valid semantic oracle was not established. The full suite remains CI-only.
This report is investigation evidence, not coverage or runtime acceptance.

STOP BLOCKED at NgRx qualification/public discriminant meaning. Do not resume
#140b reducer linkage automatically.
