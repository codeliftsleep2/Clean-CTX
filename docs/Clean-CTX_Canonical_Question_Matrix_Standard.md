# Clean-CTX Canonical Question Matrix Standard

**Status:** HARD ARCHITECTURAL REQUIREMENT\
**Applies to:** Every meta-layer, producer, consumer, feature,
functionality, semantic relationship, and material capability in
Clean-CTX.

------------------------------------------------------------------------

## 1. Governing Rule

A semantic capability is **not complete merely because its internal
representation is correct**.

It is complete only when Clean-CTX can use that capability to answer the
real architectural questions for which the capability exists, through an
authoritative, current, reachable production query path.

``` text
EVERY SEMANTIC CAPABILITY
    ⇒ AT LEAST ONE REAL USER / LLM QUESTION

EVERY REQUIRED QUESTION
    ⇒ PROVABLE SEMANTIC FACTS
     + REQUIRED AUTHORITY
     + REQUIRED ACQUISITION / DEPENDENCY CLOSURE
     + QUERYABLE REPRESENTATION
     + COMPLETENESS CONTRACT

IMPLEMENTED BUT NOT QUERYABLE
    = INCOMPLETE

QUERYABLE BUT NOT AUTHORITATIVE
    = INCORRECT

AUTHORITATIVE BUT NOT CURRENT
    = STALE TRUTH

A QUERY THAT CANNOT EXPRESS THE PROVED SEMANTICS
    = AN INCOMPLETE FEATURE
```

Clean-CTX does not implement semantic facts for their own sake.

It implements authoritative semantic knowledge so an AI can ask
architectural questions and receive answers it does **not** need to
reconstruct from raw source.

------------------------------------------------------------------------

## 2. Scope

This requirement applies to:

-   every meta-layer;
-   every language/framework semantic producer;
-   every cross-file relationship;
-   every source-local relationship that exists to answer a material
    architectural question;
-   every identity/resolution mechanism;
-   every derived semantic fact;
-   every completeness certificate;
-   every query evaluator;
-   every MCP/tool exposure;
-   every new feature;
-   every upgrade to existing functionality.

This applies equally to C#, TypeScript, Angular, EF, NgRx, routing, DI,
templates, inheritance, testing, and every future semantic domain.

A feature is not exempt because it predates this standard.

------------------------------------------------------------------------

## 3. Mandatory Canonical Question Matrix

Every implementation campaign MUST create or update a **Canonical
Question Matrix**.

Every material semantic capability must map to one or more real
questions Clean-CTX is expected to answer.

  ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
  ID           Canonical Question                    User / LLM      Semantic Facts Required   Authority / Identity  Acquisition / Dependency Closure                                    Public Query / Tool Path    Completeness Contract    Expected Answer   Adversarial / Lifecycle Status /
                                                     Intent                                    Required                                                                                                                                       Shape             Cases                   Evidence
  ------------ ------------------------------------- --------------- ------------------------- --------------------- ------------------------------------------------------------------- --------------------------- ------------------------ ----------------- ----------------------- ----------
  EXAMPLE-01   Why can `<user-card>` be used in      Explain actual  template owner; selector; canonical TS          owner + standalone/NgModule composition + barrels/re-exports +      relevant workspace/template Exact only when visible  owner →           export removal; barrel  Example
               `DashboardComponent`'s template?      architectural   declaration identity;     declaration; Angular  current contributors                                                semantic query              candidate universe and   composition path  retarget; duplicate     
                                                     visibility      imports/exports; visible  Component/Directive                                                                                                   required composition     → declaration →   selector outside scope; 
                                                     rather than     scope                     authority; exact                                                                                                      closure are proved;      selector, with    foreign decorator;      
                                                     find matching                             composition identity                                                                                                  otherwise                provenance        tsconfig retarget;      
                                                     text.                                                                                                                                                           lower-bound/unresolved                     cold/warm/order         

  EXAMPLE-02   Where is this injection token         Trace DI        consumer; injector        canonical token       component/module/route/ApplicationConfig/bootstrap/provider-array   provider/DI semantic query  Exact only when          consumer →        `useExisting`; `multi`; Example
               provided, and what satisfies it here? registration    boundary; token;          identity; Angular     closure                                                                                         applicable injector      injector scope →  same-name token; alias; 
                                                     applicable to a registration; provider    provider authority;                                                                                                   neighborhood is closed   provider →        provider removal; route 
                                                     consumer.       strategy; endpoint        injector-scope                                                                                                        and current              strategy/target   scope; lookalike        
                                                                                               authority                                                                                                                                                        package                 

  EXAMPLE-03   What component does this lazy route   Resolve runtime route owner; lazy         canonical TS          route + import/export/package/config witnesses + target source      routing semantic query      Exact only when lazy     route → lazy      barrel retarget;        Example
               actually load?                        architecture    expression; export        export/declaration                                                                                                    target resolution and    reference →       package export change;  
                                                     from route      target; declaration role  identity; routing API                                                                                                 target role are proved   canonical target  ambiguous star;         
                                                     declaration.                              authority                                                                                                                                      declaration       deletion/restoration;   
                                                                                                                                                                                                                                                                same terminal name      

  EXAMPLE-04   What breaks if I remove this          Impact analysis incoming semantic         canonical identities  closed affected dependency neighborhood sufficient for requested    reverse/dependency/impact   Must state exact vs      changed fact →    unindexed dependent;    Example
               import/declaration/provider/export?   before change.  dependencies;             for affected          scope                                                               query                       lower-bound; never imply dependent         warm/cold; alias;       
                                                                     visibility/provider/use   endpoints                                                                                                             closed-world impact from facts/consumers → barrel; negative        
                                                                     relationships                                                                                                                                   partial acquisition      reason            candidate appears;      
                                                                                                                                                                                                                                                                project boundary        
  ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

The example rows above do **not** define the complete matrix for any
meta-layer.

------------------------------------------------------------------------

## 4. Required Question Classes

Every meta-layer must explicitly consider whether it needs canonical
questions from each of these classes.

### Identity

-   What is this thing really?
-   Which declaration does this reference denote?
-   Which framework/library API does this symbol resolve to?
-   Are two same-named things actually the same semantic entity?

### Forward Relationships

-   What does this entity depend on?
-   What does it import, call, expose, provide, load, inherit from,
    compose, or reference?
-   What semantic effects does it produce?

### Reverse Relationships

-   What depends on this entity?
-   What imports, calls, consumes, renders, injects, inherits from, or
    is affected by it?
-   Where is this entity used semantically?

### Why / Provenance

-   Why is this relationship true?
-   Which declarations, configuration, composition boundaries, and
    source snapshots prove it?
-   What authoritative path connects the two endpoints?

### Scope / Visibility

-   Where is this entity visible or applicable?
-   Why is it available here?
-   Why is it *not* available somewhere else?
-   What scope determines the answer?

### Composition

-   How do multiple declarations/files/modules/configurations combine
    into the effective architecture?
-   What is the resulting visible/provider/routing/framework universe?

### Impact / Change

-   What changes if this declaration, import, export, provider, route,
    configuration, base class, selector, or dependency changes?
-   What semantic facts must retract?

### Completeness

-   Is this answer exact?
-   Is it lower-bound?
-   Is it unresolved?
-   Is it ambiguous?
-   Is the requested capability unsupported?
-   What proof justifies that classification?

### Lifecycle

-   Does the answer remain correct after mutation?
-   deletion?
-   restoration?
-   rename?
-   alias change?
-   barrel/re-export retarget?
-   project/config/package change?
-   persistence/reopen?
-   cold/warm execution?
-   reversed query or batch order?

### Cross-Layer Questions

-   Can higher-level questions compose facts from multiple meta-layers?
-   Can the answer be produced without falling back to terminal names,
    proximity, or raw-source reconstruction?

------------------------------------------------------------------------

## 5. Question Design Rules

Canonical questions MUST:

1.  Represent realistic developer or LLM architectural intent.
2.  Ask something Clean-CTX exists to know, not merely restate an
    internal data structure.
3.  Prefer semantic questions over questions answerable by grep.
4.  Include multi-hop questions where architecture requires multi-hop
    reasoning.
5.  Include `why` questions for material derived relationships.
6.  Consider reverse questions for every meaningful forward
    relationship.
7.  Include provenance and lifecycle questions for cross-file claims.
8.  Define the completeness proof required for closed-world answers.
9.  Allow `ambiguous`, `unresolved`, or `lower-bound` to be correct
    answers when authority does not justify a single exact result.
10. Never require the LLM to reconstruct a relationship from raw source
    when Clean-CTX claims semantic support for that relationship.

Do **not** create a query merely because an internal edge exists.

The direction is:

``` text
REAL ARCHITECTURAL QUESTION
    ↓
REQUIRED SEMANTIC KNOWLEDGE
    ↓
REQUIRED AUTHORITY
    ↓
REQUIRED REPRESENTATION
    ↓
REQUIRED QUERY CONTRACT
```

not:

``` text
WE HAVE AN EDGE
    ↓
FIND SOMETHING TO QUERY ABOUT IT
```

------------------------------------------------------------------------

## 6. Query Reachability Gate

Every canonical question must trace through the actual production path.

``` text
authoritative source
    ↓
acquisition / dependency closure
    ↓
canonical identity
    ↓
semantic producer
    ↓
retained fact + proof
    ↓
publication / persistence
    ↓
query evaluator
    ↓
MCP / public tool response
    ↓
canonical question answer
```

A custom-pipeline unit test that constructs semantic objects directly
does **not** prove query reachability.

A feature is not complete until the relevant production query/tool can
expose the answer.

------------------------------------------------------------------------

## 7. Required Evidence Per Material Matrix Row

For each material canonical question, acceptance should include where
applicable:

-   a positive fixture producing a nonempty authoritative answer;
-   a negative control proving same spelling/name/proximity cannot
    manufacture the answer;
-   cold/warm equivalence;
-   repeat-query equivalence;
-   relevant query-order equivalence;
-   relevant batch-order equivalence;
-   relevant admission/indexing-order equivalence;
-   mutation that changes or retracts the answer;
-   deletion and restoration;
-   rename/alias/re-export/retarget behavior;
-   config/package/project changes;
-   ambiguity behavior;
-   same-terminal-name adversarial cases;
-   persistence/reopen behavior for durable semantics;
-   exact/lower-bound/unresolved/ambiguous classification assertions;
-   public-path verification proving the actual query/tool exposes the
    intended answer.

Not every row requires every case.

Every omitted case must be irrelevant by architecture, not merely
inconvenient to test.

------------------------------------------------------------------------

## 8. Meta-Layer Completion Gate

A meta-layer or feature may not be marked COMPLETE until its Canonical
Question Matrix has been reviewed.

### Eligible for completion

``` text
ANSWERED
+ AUTHORITATIVE
+ QUERYABLE
+ CURRENT
+ CORRECT COMPLETENESS CLASSIFICATION
```

### Incomplete

``` text
IMPLEMENTED INTERNALLY
BUT NOT QUERYABLE
```

### Incorrect

``` text
QUERYABLE
BUT BASED ON NAME / SPELLING / PROXIMITY / HEURISTIC
WHEN STRONGER AUTHORITY IS REQUIRED
```

### Lower-bound only

``` text
ANSWER EXISTS
BUT REQUIRED AUTHORITY UNIVERSE / CLOSURE IS NOT PROVED
```

### Query-contract gap

``` text
MATERIAL QUESTION
HAS PROVABLE SEMANTICS
BUT NO QUERY REPRESENTATION
```

That gap must be implemented or the supported capability must be
explicitly narrowed.

### Not an acceptable completion argument

``` text
THE QUESTION IS MATERIAL
AND INSIDE THE INTENDED CAPABILITY
BUT WE LABEL IT UNSUPPORTED
BECAUSE IMPLEMENTING IT REQUIRES MORE ARCHITECTURE
```

Truthful degradation is a safety boundary.

It is not a substitute for required capability.

------------------------------------------------------------------------

## 9. Shared-Foundation Consumer Requirement

When a shared authority foundation is upgraded, every applicable
meta-layer and consumer MUST be audited.

No consumer may retain a weaker competing mechanism for a fact the
shared foundation now authoritatively provides.

For each affected consumer record:

  ---------------------------------------------------------------------------------------------------
  Consumer /   Canonical     Old         Shared       Specialized   Old         Query    Regression
  Meta-Layer   Question(s)   Authority   Foundation   Semantics     Mechanics   Path     Evidence
                             Path        Capability   Retained      Removed /            
                                                                    Delegated            
  ------------ ------------- ----------- ------------ ------------- ----------- -------- ------------

  ---------------------------------------------------------------------------------------------------

Apply:

``` text
SHARE MECHANICS
SPECIALIZE SEMANTICS
COMPOSE AUTHORITATIVE FACTS
```

and:

``` text
ZERO COMPETING AUTHORITY PATHS
FOR THE SAME SEMANTIC QUESTION
```

------------------------------------------------------------------------

## 10. Cross-Meta-Layer Questions

Some of the most valuable Clean-CTX questions will require multiple
semantic domains.

Example:

``` text
Why can <user-card> be used here?

template occurrence
    ↓
template owner
    ↓
Angular composition scope
    ↓
TypeScript import/export identity
    ↓
canonical declaration
    ↓
Angular Component authority
    ↓
selector
```

Another example:

``` text
What happens if I remove this service registration?

provider registration
    ↓
injector scope
    ↓
canonical token identity
    ↓
constructor / inject() consumers
    ↓
route/component/application composition
    ↓
affected consumers
```

No cross-layer query may solve missing authority by falling back to raw
names or asking the LLM to infer the missing architecture.

------------------------------------------------------------------------

## 11. Canonical Question Review Record

Use this record for each material question.

``` text
Meta-layer / Feature:

Canonical Question ID:

Question:

Why does a developer / LLM need this answer?

Required semantic facts:

Required identity / authority:

Required acquisition / dependency closure:

Producer(s):

Specialized semantic owner:

Consumer / query evaluator:

Public MCP / tool / query path:

Expected answer shape:

Completeness classification and proof:

Negative / adversarial controls:

Mutation / lifecycle controls:

Cold / warm / order controls:

Persistence requirements:

Cross-meta-layer dependencies:

Current status:
    NOT IMPLEMENTED
    PARTIAL
    PROVED

Evidence / tests:

Known gaps:

Supported-envelope decision:
```

------------------------------------------------------------------------

## 12. Required Implementation Deliverables

Every material implementation campaign must return:

1.  The Canonical Question Matrix for every touched meta-layer/feature.
2.  Mapping from each question to required semantic facts.
3.  Mapping from each question to required authority and identity.
4.  Mapping from each question to required acquisition/dependency
    closure.
5.  Mapping from each question to the public query/tool path.
6.  Expected answer shape.
7.  Completeness contract.
8.  Adversarial/lifecycle regression plan.
9.  Shared-foundation consumer audit when applicable.
10. List of unanswered material questions.
11. Explicit supported-envelope decisions for remaining gaps.
12. Final public-path verification demonstrating that the model receives
    the intended semantic answer.

------------------------------------------------------------------------

## 13. Feature / Functionality Definition of Done

A feature is DONE only when:

``` text
SEMANTIC IMPLEMENTATION
    +
AUTHORITATIVE IDENTITY
    +
REQUIRED DEPENDENCY CLOSURE
    +
CURRENTNESS / PUBLICATION PROOF
    +
CANONICAL QUESTION COVERAGE
    +
PUBLIC QUERY REACHABILITY
    +
CORRECT COMPLETENESS SEMANTICS
    +
ADVERSARIAL / LIFECYCLE VERIFICATION
```

all hold for the intended supported envelope.

Passing internal semantic tests alone is insufficient.

Passing public query tests against unproven semantics is insufficient.

Producing correct answers only from warm/pre-indexed state is
insufficient.

Producing an answer whose provenance cannot be explained is
insufficient.

------------------------------------------------------------------------

## 14. Architectural Review Questions

Before approving any semantic implementation, reviewers MUST ask:

1.  **What real question does this capability let Clean-CTX answer?**
2.  **What exact facts are required to answer it?**
3.  **What authority proves those facts?**
4.  **What must be acquired before the answer is complete?**
5.  **How does the query reach those facts?**
6.  **Can the public answer preserve the semantics we proved?**
7.  **What makes the answer exact versus lower-bound?**
8.  **What mutation would make the answer cease to be true?**
9.  **Will that mutation actually invalidate/retract the answer?**
10. **Does another meta-layer still answer the same question through
    weaker mechanics?**
    Apply [MECHANISM-REUSE-001](ARCHITECTURAL_INVARIANTS.md#mechanism-reuse-001-equivalent-mechanics-are-shared-semantics-keep-their-owners): compare actual authority, lifecycle and correctness contracts, reuse equivalent mechanics and document justified specialization.
11. **Can the answer survive cold/warm, ordering, batching, persistence,
    and restoration?**
12. **Would the LLM still need raw source to reconstruct something
    Clean-CTX claims to understand?**

If the answer to #12 is **yes**, the capability is not finished.

------------------------------------------------------------------------

## 15. Final Invariants

``` text
CLEAN-CTX DOES NOT EXIST
TO STORE SEMANTIC FACTS.

IT EXISTS
TO ANSWER ARCHITECTURAL QUESTIONS
FROM AUTHORITATIVE SEMANTIC KNOWLEDGE.
```

``` text
IF CLEAN-CTX KNOWS A FACT
BUT CANNOT USE IT TO ANSWER
THE QUESTION THAT JUSTIFIED BUILDING THAT FACT,
THE FEATURE IS NOT FINISHED.
```

``` text
IF CLEAN-CTX CAN ANSWER THE QUESTION
BUT CANNOT PROVE WHY THE ANSWER IS TRUE,
CURRENT, AND COMPLETE TO THE CLAIMED DEGREE,
THE FEATURE IS NOT CORRECT.
```

``` text
REAL QUESTION
⇒ AUTHORITATIVE KNOWLEDGE
⇒ QUERYABLE ANSWER
⇒ PROVABLE TRUTH
```

This matrix standard is a **hard completion requirement**, not
documentation guidance.
