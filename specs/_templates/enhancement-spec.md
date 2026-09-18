# Enhancement Spec: [Enhancement Title]

> **Status**: Draft | Review | Approved | Implemented | Verified
> **Author**: @vadde
> **Created**: YYYY-MM-DD
> **Target Tool**: `tools/<tool-name>/`
> **Spec Reference**: `specs/catalog/<tool-name>.md`

---

## 1. Summary

_One-paragraph description of the enhancement._

## 2. Motivation

_Why is this enhancement needed? What problem does it solve?_

## 3. Proposed Changes

### 3.1 New Requirements

| ID | Requirement | Priority |
|----|------------|----------|
| R-0XX | _Description_ | Must/Should/Could |

### 3.2 Modified Requirements

| Original ID | Change | Reason |
|-------------|--------|--------|
| R-00X | _What changes_ | _Why_ |

### 3.3 Interface Changes

_Describe any changes to CLI flags, API signatures, or data formats._

```diff
- def process(input: str) -> str:
+ def process(input: str, format: str = "text") -> str:
```

## 4. Breaking Changes

_List any backward-incompatible changes. If none, state "None."_

## 5. Acceptance Criteria

| AC ID | Criteria | Traces To |
|-------|----------|-----------|
| AC-0XX | _Given X, when Y, then Z_ | R-0XX |

## 6. Migration Guide

_If there are breaking changes, provide a migration guide._

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| YYYY-MM-DD | @vadde | Initial draft |
