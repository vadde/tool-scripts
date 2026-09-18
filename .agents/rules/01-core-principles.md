# Rule 01 — Core Principles

> **Priority**: 🔴 Critical — These principles govern ALL agent behavior in this repository.

---

## 1. The ReAct Protocol (Reason + Act)

Every task MUST follow the ReAct loop. Never act without reasoning first.

```
┌─────────────────────────────────────────────────────┐
│                   ReAct Loop                        │
│                                                     │
│   THINK ──→ PLAN ──→ ACT ──→ OBSERVE ──→ REFLECT  │
│     ↑                                        │      │
│     └────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────┘
```

### THINK
- What is the goal?
- What is the current state?
- What constraints exist (spec, status, dependencies)?

### PLAN
- Break the problem into discrete steps
- Identify risks and dependencies
- Determine verification criteria for each step

### ACT
- Execute ONE step at a time
- Make the smallest change that achieves the step's goal
- Never combine unrelated changes

### OBSERVE
- Check the result of the action
- Compare actual vs. expected outcome
- Read error messages completely — every word matters

### REFLECT
- Did the action achieve its goal?
- Were there unexpected side effects?
- What did I learn that changes my plan?
- Should I continue, adjust, or stop?

---

## 2. Reflexion Protocol (Self-Critique + Retry)

After completing a significant unit of work (e.g., implementing a function,
writing a test suite), apply the Reflexion gate:

### Reflexion Gate Checklist

```
┌─ REFLEXION GATE ────────────────────────────────────┐
│                                                     │
│  □ Does the code satisfy ALL spec requirements?     │
│  □ Are ALL edge cases handled?                      │
│  □ Do ALL tests pass?                               │
│  □ Is the code readable and well-documented?        │
│  □ Does it follow language-specific conventions?    │
│  □ Are there any assumptions I haven't verified?    │
│  □ Would a reviewer find obvious issues?            │
│                                                     │
│  If ANY box is unchecked → FIX before proceeding    │
└─────────────────────────────────────────────────────┘
```

### Reflexion Memory

When a Reflexion gate reveals an issue:
1. **Document** the issue and root cause
2. **Fix** the issue
3. **Update** your mental model to prevent recurrence
4. **Re-verify** from the beginning of the affected scope

---

## 3. Divide and Conquer Strategy

Complex problems MUST be decomposed before implementation.

### Decomposition Rules

1. **Maximum sub-task size**: Each sub-task should be completable in a single
   focused action (one function, one test, one config change)
2. **Maximum parallelism**: Identify sub-tasks that can be done independently
3. **Dependency ordering**: Execute dependent sub-tasks in topological order
4. **Atomic commits**: Each sub-task gets its own commit

### Decomposition Template

```
PROBLEM: [describe the complex problem]
├── SUB-TASK 1: [atomic, testable unit]
│   ├── Dependencies: [none | sub-task X]
│   ├── Verification: [how to know it's done]
│   └── Risk: [low | medium | high]
├── SUB-TASK 2: [atomic, testable unit]
│   ├── Dependencies: [sub-task 1]
│   ├── Verification: [how to know it's done]
│   └── Risk: [low | medium | high]
└── INTEGRATION: [how sub-tasks combine]
    └── Verification: [end-to-end test]
```

---

## 4. Semantic Correlation

Every change must be analyzed in context of the entire system.

### Before Making a Change

1. **Identify all related files** — specs, tests, docs, dependent tools
2. **Trace the impact** — What else references or depends on this?
3. **Check for patterns** — How do similar tools/functions handle this?
4. **Verify consistency** — Does this change maintain naming, style, and
   architectural consistency?

### Cross-Reference Matrix

For every significant change, mentally construct:

```
CHANGE: [what you're changing]
├── SPEC REFERENCE: [which spec requirement(s) this addresses]
├── AFFECTED TESTS: [which tests need updating]
├── AFFECTED DOCS: [which docs need updating]
├── DEPENDENT TOOLS: [tools that import/use this]
└── CHANGELOG ENTRY: [what to add to CHANGELOG.md]
```

---

## 5. Zero-Assumption Policy

> **NEVER assume. ALWAYS verify.**

### Rules

1. **Don't assume a function exists** — Search for it first
2. **Don't assume a dependency is installed** — Check the manifest
3. **Don't assume a test passes** — Run it
4. **Don't assume the spec is current** — Cross-reference with code
5. **Don't assume a file path** — Verify it exists
6. **Don't assume error handling** — Check what happens on failure

### Verification Hierarchy

```
STRONGEST  → Run the code and observe the output
           → Read the source code directly
           → Read the test suite
           → Read the specification
           → Read the documentation
WEAKEST    → Recall from memory (NEVER rely solely on this)
```

---

## 6. Hallucination Prevention

### Grounding Rules

1. **Every claim must be verifiable** — Point to a specific file, line, or test
2. **Every API call must be confirmed** — Check the actual function signature
3. **Every import must exist** — Verify the module/package is available
4. **Every assumption must be stated** — If you must assume, say so explicitly
5. **When uncertain, say so** — "I'm not sure about X, let me verify" is
   always better than a confident wrong answer

### Anti-Hallucination Checklist

Before committing ANY code:
- [ ] All imports resolve to real modules
- [ ] All function calls use correct signatures
- [ ] All file paths are verified to exist
- [ ] All configuration keys are documented
- [ ] No placeholder values remain (e.g., `TODO`, `FIXME`, `xxx`)

---

## 7. Resilience Patterns

### Retry with Backoff

If an action fails:
1. **First retry**: Immediately, with careful re-reading of the error
2. **Second retry**: With a modified approach based on error analysis
3. **Third attempt**: With a fundamentally different strategy
4. **Escalate**: If three attempts fail, document the issue and ask for help

### Graceful Degradation

When encountering blockers:
1. **Document** what's blocked and why
2. **Continue** with non-blocked work
3. **Mark** blocked items in the task list
4. **Provide** a clear summary of what needs human intervention

### Rollback Strategy

Always maintain the ability to undo:
1. **Commit early** — Save progress before risky changes
2. **Branch** — Use feature branches for experimental work
3. **Test first** — Verify the existing state before modifying it
