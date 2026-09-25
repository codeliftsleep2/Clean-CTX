# Task-based SCHEMA-v5 evaluation — design

**Status:** edit tasks (1–3) implemented and green — 3/3 on both the v1
deterministic grader and the v2 `apply_edit` round-trip. Debug/Implement tasks
(4–7) remain proposed, not implemented. Supersedes the comprehension-quiz
oracles in `oracles.json`.

## Principle

Measure whether an LLM can **use** the SCHEMA-v5 context to do a unit of
production work — not whether it can answer a fact question about the notation.
Grading is **deterministic** (string/byte/structural checks against the tracked
fixture source), not another LLM's opinion.

This replaces the quiz oracles, which measured *comprehension* ("which type owns
`run`?") on small fixtures that return raw source anyway, with task oracles that
measure *task completion* on the large fixtures where SCHEMA-v5 is actually
emitted (the 56–65% files).

## Why it fixes the two known problems

1. **Wrong target.** The quiz measured notation-reading; production measures
   edit/debug/implement. Task oracles measure the latter.
2. **Noisy LLM judge.** The old scorer misread `run(+1)` arity as an "invented
   ID" and contradicted itself between runs. Deterministic grading removes the
   judge entirely.

## Task types

### 1. Edit (`apply_edit`) — the core test

The model receives the SCHEMA-v5 **Edit** presentation (the target method's body
byte-exact, everything else signature-only) plus an instruction, and must emit an
`apply_edit` operation.

Deterministic grading, three checks:

| Check | Against | Proves |
|---|---|---|
| `target` == `Class.method` | the declared target | correct ownership/navigation |
| `expectedOldText` byte-matches the body | the fixture source's exact body | it read the byte-exact body from the payload |
| `newText` == the reference edited body | an authored reference | it applied the intended change and nothing else |

`expectedOldText` byte-exactness is the signal: a correct value proves the model
read the presentation faithfully; a wrong one proves it misread or hallucinated.

### 2. Debug (closed-form)

The model receives the SCHEMA-v5 **High** presentation and a structured question
(which method / which condition / which outcome).

Deterministic grading: the answer must name the correct method + condition +
outcome (substring/structural checks) and must not contain a wrong fact. No
free-form "is this correct" judgment.

### 3. Implement (add a unit)

The model receives the SCHEMA-v5 **High** presentation and must produce a new
method.

Deterministic grading: signature matches (name, params, return) and the body uses
the expected repository/service calls (substring checks), or apply-and-typecheck.

## Initial task set (7)

All on the large fixtures (`src/test_files/LargeService.ts`,
`src/test_files/UserManagementService.ts`).

**Edit**

1. `UserService.createUser` — change the duplicate-email message
   `'User with this email already exists'` → `'Email already registered'`.
2. `UserService.getUserStats` — change `totalSpent?.total || 0` → `?? 0`.
3. `UserManagementService.isValidEmail` — change the length guard `> 254` →
   `>= 254`.

**Debug**

4. `LargeService.ts` — "Which method throws `BadRequestException`, and under
   what condition?" → `createUser`; when a user with the same email exists.
5. `UserManagementService.ts` — "Which method returns `{ success: false, ... }`
   on error?" → `handleError`; logs and returns a 500 `ApiResponse`.

**Implement**

6. `UserService` — add `getUserCount(): Promise<number>` returning
   `this.userRepository.count()`.
7. `UserManagementService` — add `getUserByEmail(email: string)` that delegates
   to the existing get-by-filter path and returns the first match or `null`.

## Capture requirements

Each **edit** task needs an Edit capture focused on its target method (so only
that body is byte-exact). These live in `schema-v5/task-scenarios.json` — kept
separate from the codec harness's `expected/scenarios.json` — captured by
`schema-v5/scripts/capture-tasks.ps1` into `captures/<scenario>/content.txt`
(the SCHEMA-v5 presentation).

## Integration

- `schema-v5/tasks.json` — task definitions (`target`, `find`/`replace`,
  `capture`).
- `schema-v5/task-scenarios.json` — task capture scenarios (separate from the
  codec harness's `expected/scenarios.json`).
- `schema-v5/scripts/capture-tasks.ps1` — captures the task scenarios (writes
  `content.txt`).
- `schema-v5/scripts/run-tasks.ps1` — feeds `content.txt` + instruction to the
  model and emits a structured `apply_edit` JSON.
- `schema-v5/scripts/grade-tasks.ps1` — the deterministic grader (no model).
- `schema-v5/scripts/grade-tasks-v2.ps1` — the `apply_edit` round-trip grader
  (feeds each answer's operation to the real tool; the byte-exact/EOL gate).
- Reuses `scripts/McpSession.ps1`; `run-tasks.ps1` carries the Codex/DeepSeek
  model invocation.
