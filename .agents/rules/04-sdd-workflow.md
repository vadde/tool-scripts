# Rule 04 — Spec-Driven Development (SDD) Workflow

> **Priority**: 🔴 Critical — No code shall be written without a specification.

---

## The SDD Manifesto

1. **Spec First, Code Second** — The spec is the contract; code is the implementation
2. **Specs Are Living Documents** — They evolve with the tool, never abandoned
3. **Traceability Is Non-Negotiable** — Every function traces to a requirement
4. **Verification Closes the Loop** — Tests prove the spec is satisfied

---

## SDD Maturity Levels

| Level | Name | Description | Target |
|-------|------|-------------|--------|
| L0 | Ad-hoc | No spec, vibe coding | ❌ Unacceptable |
| L1 | Documented | Spec exists but is informal | Minimum for `draft` |
| L2 | Structured | Spec uses template with numbered requirements | Minimum for `in-progress` |
| L3 | Spec-as-Source | Code is validated against spec requirements | Target for all tools |
| L4 | Spec-to-Code | Spec generates code scaffolds/tests automatically | Aspirational |

**All tools MUST reach L2 before implementation begins and L3 before release.**

---

## The SDD Lifecycle

```
┌──────────────────────────────────────────────────────────────────────┐
│                        SDD Lifecycle                                 │
│                                                                      │
│  ┌─────────┐   ┌──────────┐   ┌───────────┐   ┌──────────────────┐ │
│  │ IDEATE  │──▶│ SPECIFY  │──▶│ IMPLEMENT │──▶│ VERIFY & RELEASE │ │
│  └─────────┘   └──────────┘   └───────────┘   └──────────────────┘ │
│       │              │              │                    │           │
│    Draft         Spec-Review    In-Progress        Testing/Review   │
│    STATUS.md     STATUS.md      STATUS.md          STATUS.md        │
│                                                                      │
│  ◄─── Feedback loops at every stage ───►                            │
└──────────────────────────────────────────────────────────────────────┘
```

### Phase 1: IDEATE (`draft`)

**Input**: An idea, problem statement, or user request
**Output**: A draft spec in `specs/catalog/<tool-name>.md`

**Actions**:
1. Create `specs/catalog/<tool-name>.md` using the template
2. Fill in: Overview, Problem Statement, High-level Requirements
3. Create `tools/<tool-name>/STATUS.md` with `status: draft`
4. Add entry to `tools/README.md` catalog

**Exit Criteria**:
- [ ] Spec file exists with Overview and Problem Statement
- [ ] STATUS.md created with `status: draft`
- [ ] Tool appears in catalog

### Phase 2: SPECIFY (`spec-review`)

**Input**: Draft spec
**Output**: Complete, reviewed spec with numbered requirements

**Actions**:
1. Complete ALL spec sections (use the template checklist)
2. Number all requirements: `R-001`, `R-002`, ...
3. Define the Interface Contract (inputs, outputs, CLI flags)
4. Define Acceptance Criteria (testable conditions)
5. Update `STATUS.md` to `status: spec-review`

**Exit Criteria**:
- [ ] All spec template sections completed
- [ ] All requirements numbered (R-XXX format)
- [ ] Interface contract defined
- [ ] Acceptance criteria are testable
- [ ] Human review requested (if available)

### Phase 3: IMPLEMENT (`in-progress`)

**Input**: Reviewed spec
**Output**: Working code with tests

**Actions**:
1. Update `STATUS.md` to `status: in-progress`
2. Implement code in `tools/<name>/src/`
3. Write tests in `tools/<name>/tests/`
4. Create examples in `tools/<name>/examples/`
5. Write tool `README.md`
6. **Traceability**: Add spec requirement references in code comments:
   ```python
   def validate(schema: dict, data: dict) -> bool:
       """Validate data against a JSON schema.

       Implements: R-001 (Schema validation)
       See: specs/catalog/json-validator.md#R-001
       """
   ```

**Exit Criteria**:
- [ ] All spec requirements have corresponding code
- [ ] All spec requirements have corresponding tests
- [ ] Tests pass locally (`make test-tool T=<name>`)
- [ ] Linting passes (`make lint-tool T=<name>`)
- [ ] README documents usage and installation
- [ ] At least one runnable example exists

### Phase 4: VERIFY & RELEASE (`testing` → `review` → `released`)

**Input**: Implemented tool
**Output**: Released, documented, tested tool

**Actions (testing)**:
1. Update `STATUS.md` to `status: testing`
2. Run full test suite with coverage
3. Run edge case and integration tests
4. Verify all examples work
5. Cross-reference tests against ALL spec requirements

**Actions (review)**:
1. Update `STATUS.md` to `status: review`
2. Finalize CHANGELOG.md
3. Polish documentation
4. Self-review against Code Standards (Rule 02)

**Actions (released)**:
1. Update `STATUS.md` to `status: released`
2. Set version in STATUS.md
3. Tag the release
4. Update global `sdlc/CHANGELOG.md`

---

## Spec Template Location

- **Tool spec template**: `specs/_templates/tool-spec.md`
- **Enhancement spec template**: `specs/_templates/enhancement-spec.md`
- **Active specs**: `specs/catalog/<tool-name>.md`

---

## Spec-Code Traceability Matrix

For every tool, maintain a mental (or documented) traceability matrix:

```
┌──────────┬──────────────────────┬────────────────────┬───────────┐
│ Req ID   │ Spec Requirement     │ Implementation     │ Test      │
├──────────┼──────────────────────┼────────────────────┼───────────┤
│ R-001    │ Validate JSON schema │ src/validator.py:42│ test_001  │
│ R-002    │ CLI --format flag    │ src/cli.py:15      │ test_002  │
│ R-003    │ Error reporting      │ src/errors.py:8    │ test_003  │
└──────────┴──────────────────────┴────────────────────┴───────────┘
```

---

## SDD Anti-Patterns (NEVER Do These)

| Anti-Pattern | Why It's Wrong | Do This Instead |
|-------------|----------------|-----------------|
| Code without spec | No contract = no quality guarantee | Write spec first |
| Spec without requirements | Unstructured prose is not testable | Use numbered R-XXX requirements |
| Spec abandoned after coding | Spec drift = confusion | Keep spec updated |
| Tests without spec reference | Can't verify coverage | Reference R-XXX in test names |
| Skipping spec-review | Unreviewed specs = wrong direction | Always review before coding |
