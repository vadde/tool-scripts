# Spec: [Tool Name]

> **Status**: Draft | Review | Approved | Implemented | Verified
> **Author**: @vadde
> **Created**: YYYY-MM-DD
> **Last Updated**: YYYY-MM-DD
> **Tool Path**: `tools/<tool-name>/`

---

## 1. Overview

### 1.1 Problem Statement

_What problem does this tool solve? Why does it need to exist?_

### 1.2 Proposed Solution

_How does this tool solve the problem? High-level approach._

### 1.3 Target Audience

_Who will use this tool? What is their technical level?_

### 1.4 Success Criteria

_How do we know this tool is successful?_

---

## 2. Requirements

### 2.1 Functional Requirements

| ID | Requirement | Priority | Status |
|----|------------|----------|--------|
| R-001 | _Description_ | Must | ⬜ |
| R-002 | _Description_ | Must | ⬜ |
| R-003 | _Description_ | Should | ⬜ |
| R-004 | _Description_ | Could | ⬜ |

**Priority levels**: Must (required for MVP), Should (important), Could (nice-to-have)

### 2.2 Non-Functional Requirements

| ID | Requirement | Metric |
|----|------------|--------|
| NF-001 | Performance | _e.g., Process 10K records in < 5s_ |
| NF-002 | Reliability | _e.g., Handle malformed input gracefully_ |
| NF-003 | Portability | _e.g., Run on macOS, Linux_ |

---

## 3. Interface Contract

### 3.1 CLI Interface

```
Usage: tool-name [OPTIONS] <input>

Arguments:
  <input>    Description of the input argument

Options:
  -o, --output <FORMAT>    Output format [default: text] [values: text, json, csv]
  -v, --verbose            Enable verbose output
  -h, --help               Show help message
  --version                Show version
```

### 3.2 Programmatic Interface (API)

```python
# Primary function signature
def process(input: InputType, config: Config | None = None) -> OutputType:
    """Brief description."""
    ...
```

### 3.3 Input Specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| _field_ | _type_ | Yes/No | _description_ |

### 3.4 Output Specification

| Field | Type | Description |
|-------|------|-------------|
| _field_ | _type_ | _description_ |

### 3.5 Error Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage/argument error |
| 3 | Configuration error |

---

## 4. Constraints

### 4.1 Technical Constraints

- _e.g., Must work with Python 3.10+_
- _e.g., No external network calls_

### 4.2 Security Constraints

- _e.g., Must not log sensitive data_
- _e.g., Input must be sanitized_

### 4.3 Performance Constraints

- _e.g., Memory usage < 512MB_
- _e.g., Startup time < 2s_

---

## 5. Dependencies

### 5.1 External Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| _package_ | _>=x.y_ | _why needed_ |

### 5.2 Internal Dependencies

| Tool | Purpose |
|------|---------|
| _tool-name_ | _why needed_ |

---

## 6. Acceptance Criteria

Tests that MUST pass for this spec to be considered satisfied:

| AC ID | Criteria | Traces To |
|-------|----------|-----------|
| AC-001 | _Given X, when Y, then Z_ | R-001 |
| AC-002 | _Given A, when B, then C_ | R-002 |
| AC-003 | _Given invalid input, then error with code 2_ | R-001, NF-002 |

---

## 7. Design Notes

_Optional: Architecture decisions, algorithm choices, diagrams._

---

## 8. Open Questions

_List any unresolved questions that need answers before implementation._

- [ ] _Question 1?_
- [ ] _Question 2?_

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| YYYY-MM-DD | @vadde | Initial draft |
