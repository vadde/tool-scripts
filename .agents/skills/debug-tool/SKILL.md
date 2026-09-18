---
name: debug-tool
description: Systematic debugging skill using the ReAct + Reflexion pattern. Use when investigating failures, bugs, or unexpected behavior in any tool.
---

# Skill: Debug a Tool

## When to Use

Use this skill when:
- A tool's tests are failing
- A tool produces incorrect output
- A tool crashes or hangs
- A user reports unexpected behavior

---

## The Debug Protocol (ReAct + Reflexion)

### Phase 1: OBSERVE — Gather Evidence

Before changing ANY code:

1. **Read the error** — Every word of the error message, stack trace, or log
2. **Read the spec** — What is the EXPECTED behavior? (`tools/<name>/spec.md`)
3. **Read the test** — What is the test asserting?
4. **Reproduce** — Can you reproduce the issue reliably?

```bash
# Run the specific failing test
make test-tool T=<name>

# Run with verbose output
cd tools/<name> && python -m pytest tests/ -v --tb=long
```

5. **Check STATUS.md** — Is this tool in a valid state for debugging?
6. **Check CHANGELOG.md** — What changed recently?

### Phase 2: THINK — Form Hypotheses

Based on evidence, form ranked hypotheses:

```
SYMPTOM: [describe what's happening]

HYPOTHESIS 1 (most likely):
  EVIDENCE FOR: [what supports this]
  EVIDENCE AGAINST: [what contradicts this]
  TEST: [how to verify/disprove]

HYPOTHESIS 2:
  EVIDENCE FOR: ...
  EVIDENCE AGAINST: ...
  TEST: ...
```

### Phase 3: ACT — Test Hypotheses

Test hypotheses ONE AT A TIME, starting with the most likely:

1. **Isolate** — Reduce to the smallest reproducing case
2. **Instrument** — Add targeted logging/print statements
3. **Test** — Run the specific test case
4. **Compare** — Actual vs. expected output

### Phase 4: FIX — Apply Targeted Fix

Once root cause is identified:

1. **Make the smallest fix** that addresses the root cause
2. **Don't fix symptoms** — Fix the underlying problem
3. **Add a regression test** — Ensure this bug can't recur
4. **Update spec** if the bug reveals a spec gap

### Phase 5: REFLECT — Verify and Learn

1. **Run ALL tests** — Not just the failing one
   ```bash
   make test-tool T=<name>
   ```
2. **Check for side effects** — Did the fix break anything else?
3. **Update CHANGELOG** — Document the fix
4. **Update STATUS.md** — If the fix changes the tool's state

---

## Debugging Checklist

- [ ] Error message read completely
- [ ] Spec reviewed for expected behavior
- [ ] Issue reproduced locally
- [ ] Root cause identified (not just symptoms)
- [ ] Fix is minimal and targeted
- [ ] Regression test added
- [ ] All tests pass after fix
- [ ] CHANGELOG updated
- [ ] Commit follows conventions: `fix(<tool>): description`

---

## Common Root Causes

| Symptom | Common Cause | Investigation |
|---------|-------------|---------------|
| ImportError | Missing dependency | Check manifest file |
| TypeError | Wrong argument type | Check function signature vs. call site |
| FileNotFoundError | Hardcoded path | Check for path assumptions |
| Timeout | Infinite loop or deadlock | Add logging, check loop conditions |
| Wrong output | Logic error | Trace through with debug prints |
| Flaky test | Race condition or shared state | Check for mutable globals |

---

## Anti-Patterns in Debugging

1. **Shotgun debugging** — Changing random things hoping it works → STOP, think first
2. **Fixing the test** — Making the test pass without fixing the code → Fix the code
3. **Broad rewrites** — Rewriting large sections to fix a small bug → Minimize changes
4. **Ignoring the spec** — Fixing to what you think is right → Check the spec
5. **No regression test** — Fixing without adding a test → Always add a test
