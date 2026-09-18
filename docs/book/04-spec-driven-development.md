# Chapter 04 — Spec-Driven Development

> _"Code without a spec is a building without blueprints."_

---

## What Is SDD?

Spec-Driven Development (SDD) is a methodology where **specifications are
written before code**, treated as living contracts, and used to verify
implementation correctness.

### The Core Loop

```
Define Requirements → Write Spec → Implement Code → Verify Against Spec
       ↑                                                    │
       └────────────── Feedback Loop ──────────────────────┘
```

### Why SDD?

| Without SDD | With SDD |
|------------|----------|
| "What should this do?" | Spec answers it |
| "Is this feature complete?" | Check requirements table |
| "Did I break anything?" | Run spec-traceable tests |
| "What does this function do?" | Read the spec reference |
| Agent hallucinates behavior | Agent reads the spec |

---

## SDD in Practice

### Step 1: Create the Specification

Use the template at `specs/_templates/tool-spec.md`:

```bash
cp specs/_templates/tool-spec.md specs/catalog/my-tool.md
```

### Step 2: Write Requirements

Every requirement gets a unique ID and priority:

```markdown
| ID | Requirement | Priority | Status |
|----|------------|----------|--------|
| R-001 | Parse CSV files with configurable delimiter | Must | ⬜ |
| R-002 | Output results in JSON or plain text | Must | ⬜ |
| R-003 | Support streaming for files > 1GB | Should | ⬜ |
| R-004 | Display a progress bar for large files | Could | ⬜ |
```

**Priority levels:**
- **Must** — Required for the tool to be useful (MVP)
- **Should** — Important, but tool works without it
- **Could** — Nice to have, implement if time permits

### Step 3: Define the Interface Contract

Specify exactly what goes in and what comes out:

```markdown
### CLI Interface
\`\`\`
Usage: csv-analyzer [OPTIONS] <file>

Arguments:
  <file>    Path to CSV file

Options:
  -d, --delimiter <CHAR>   Field delimiter [default: ,]
  -f, --format <FORMAT>    Output format [default: text] [values: text, json]
  -h, --help               Show help
\`\`\`

### Programmatic Interface
\`\`\`python
def analyze(file_path: str, delimiter: str = ",") -> AnalysisResult:
    ...
\`\`\`
```

### Step 4: Write Acceptance Criteria

Acceptance criteria are **testable conditions** that prove a requirement
is satisfied:

```markdown
| AC ID | Criteria | Traces To |
|-------|----------|-----------|
| AC-001 | Given a valid CSV file, when parsed, then all rows are returned | R-001 |
| AC-002 | Given delimiter=';', when parsed, then semicolon-separated values are split | R-001 |
| AC-003 | Given --format=json, when run, then output is valid JSON | R-002 |
```

### Step 5: Implement with Traceability

Every function references its spec requirement:

```python
def parse_csv(file_path: str, delimiter: str = ",") -> list[dict]:
    """Parse a CSV file into a list of dictionaries.

    Implements: R-001 (Parse CSV with configurable delimiter)
    See: specs/catalog/csv-analyzer.md#R-001
    """
```

Every test references its acceptance criterion:

```python
def test_AC001_parse_csv_returns_all_rows():
    """AC-001: All rows are returned from valid CSV."""
```

---

## Maturity Levels

| Level | Name | Description | How to Reach |
|-------|------|-------------|-------------|
| L0 | Ad-hoc | No spec | ❌ Unacceptable |
| L1 | Documented | Informal spec exists | Write an overview |
| L2 | Structured | Numbered requirements | Fill all spec sections |
| **L3** | **Spec-as-Source** | **Code traced to spec** | **Add R-XXX refs in code** |
| L4 | Spec-to-Code | Auto-generated scaffolds | Aspirational |

**Target: L3 for all tools before release.**

---

## Common Pitfalls

| Pitfall | Consequence | Prevention |
|---------|-------------|------------|
| Vague requirements | Ambiguous implementation | Use specific, testable language |
| Missing edge cases | Bugs in production | List edge cases in spec |
| Spec drift | Code diverges from spec | Update spec with code changes |
| Over-specification | Paralysis by analysis | Spec the interface, not the implementation |

---

📎 **Related:**
- [Spec Template](../../specs/_templates/tool-spec.md)
- [SDD Workflow Rule](../../.agents/rules/04-sdd-workflow.md)
- [Testing Strategy](../../.agents/rules/05-testing-strategy.md)

---

_[← Tool Development Guide](03-tool-development-guide.md) | [Chapter 05: Agent Collaboration →](05-agent-collaboration.md)_
