# 🔄 SDLC — Software Development Lifecycle

> **Governance, tracking, and lifecycle management for all tools.**

---

## Lifecycle Phases

```
  🟤 draft ──→ 🟡 spec-review ──→ 🔵 in-progress ──→ 🟣 testing
                                                           │
                                                           ▼
                  ⚫ deprecated ←── 🟢 released ←── 🟠 review
```

---

## Phase Definitions

### 🟤 Draft
- **Entry**: Tool idea conceived
- **Activities**: Research, write spec draft, scaffold tool directory
- **Exit Criteria**: Spec has overview and numbered requirements
- **Owner**: Author

### 🟡 Spec-Review
- **Entry**: Spec draft complete
- **Activities**: Review spec for completeness, clarity, feasibility
- **Exit Criteria**: All spec sections filled, requirements numbered
- **Owner**: Reviewer

### 🔵 In-Progress
- **Entry**: Spec approved
- **Activities**: Write code, write tests, create examples
- **Exit Criteria**: All requirements implemented, all tests pass
- **Owner**: Developer/Agent

### 🟣 Testing
- **Entry**: Implementation complete
- **Activities**: Run full test suite, fix bugs, improve coverage
- **Exit Criteria**: ≥80% coverage, all tests pass, no known bugs
- **Owner**: Developer/Agent

### 🟠 Review
- **Entry**: Testing complete
- **Activities**: Polish docs, final code review, verify examples
- **Exit Criteria**: Human approval (or self-review for agent)
- **Owner**: Reviewer

### 🟢 Released
- **Entry**: Review approved
- **Activities**: Tag release, update changelog, publish
- **Post-Release**: Bug fixes only (patch versions)
- **Owner**: Maintainer

### ⚫ Deprecated
- **Entry**: Tool superseded or no longer needed
- **Activities**: Mark as deprecated, redirect users
- **Rules**: No modifications, no new features
- **Owner**: Maintainer

---

## Tracking Tools

| Resource | Location | Purpose |
|----------|----------|---------|
| Tool STATUS.md | `tools/<name>/STATUS.md` | Per-tool lifecycle status |
| Status Board | `sdlc/status-board.md` | Aggregated dashboard |
| Roadmap | `sdlc/ROADMAP.md` | Future plans and priorities |
| Changelog | `sdlc/CHANGELOG.md` | Global change history |

---

## Commands

```bash
make status           # Show aggregated status dashboard
make validate-specs   # Validate all specs are complete
```
