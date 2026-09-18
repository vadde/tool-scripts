# Chapter 05 — Agent Collaboration

> _How AI agents and humans work together in this repository._

---

## The Agent-Native Repository

This repository is designed for **dual-audience development**: both human
engineers and AI agents can discover, understand, and modify any tool.

### How Agents Navigate

```
1. Read .agents/AGENTS.md          → Understand the repo
2. Read .agents/rules/*            → Learn the rules
3. Read tools/<name>/STATUS.md     → Know what phase the tool is in
4. Read specs/catalog/<name>.md    → Understand what to build
5. Read tools/<name>/src/          → Understand the code
6. Execute via Makefile             → Build, test, run
```

---

## Agent Behavioral Patterns

### ReAct (Reason + Act)

The foundational pattern for all agent work:

```
┌──────────────────────────────────────────┐
│  THINK: "What am I trying to achieve?"   │
│      ↓                                   │
│  PLAN: "Break it into steps"             │
│      ↓                                   │
│  ACT: "Execute one step"                 │
│      ↓                                   │
│  OBSERVE: "What happened?"               │
│      ↓                                   │
│  REFLECT: "Did it work? What next?"      │
│      ↓                                   │
│  (loop or conclude)                      │
└──────────────────────────────────────────┘
```

**Example:** Building a JSON validator tool:
1. **Think**: "I need to build a JSON validator per spec R-001 through R-005"
2. **Plan**: "Start with R-001 (basic validation), then R-002 (schema support)..."
3. **Act**: Write the `validate()` function
4. **Observe**: Run `make test-tool T=json-validator`
5. **Reflect**: "Tests pass for R-001. Moving to R-002."

### Reflexion (Self-Critique + Retry)

Applied after completing a unit of work:

```
┌──────────────────────────────────────────┐
│  ATTEMPT: Complete a task                │
│      ↓                                   │
│  EVALUATE: Does it meet the spec?        │
│      ↓                                   │
│  If NO → ANALYZE: What went wrong?       │
│      ↓                                   │
│  RETRY: Fix with new understanding       │
│      ↓                                   │
│  VERIFY: Run tests again                 │
└──────────────────────────────────────────┘
```

### Divide and Conquer

For complex tools, decompose into atomic sub-tasks:

```
BUILD: csv-analyzer tool
├── SUB-TASK 1: CSV parser (R-001)
│   └── Test: test_parse_csv_*
├── SUB-TASK 2: Output formatter (R-002)
│   └── Test: test_format_*
├── SUB-TASK 3: CLI interface (R-003)
│   └── Test: test_cli_*
└── INTEGRATE: Wire components together
    └── Test: test_end_to_end_*
```

---

## Agent Skills

Skills are pre-defined workflows that agents can invoke:

| Skill | File | When to Use |
|-------|------|-------------|
| **New Tool** | `.agents/skills/new-tool/SKILL.md` | Creating any new tool |
| **Debug Tool** | `.agents/skills/debug-tool/SKILL.md` | Investigating failures |
| **Release Tool** | `.agents/skills/release-tool/SKILL.md` | Preparing a release |

---

## Agent Rules

All rules live in `.agents/rules/` and are loaded in order:

| # | Rule | What It Governs |
|---|------|-----------------|
| 01 | Core Principles | ReAct, Reflexion, divide-and-conquer |
| 02 | Code Standards | How to write code (polyglot) |
| 03 | Commit Conventions | How to commit and branch |
| 04 | SDD Workflow | How to use specs |
| 05 | Testing Strategy | How to write and run tests |
| 06 | Documentation | How to document |

---

## SDLC-Aware Agent Behavior

Agents MUST check `STATUS.md` before acting on any tool:

| Status | Agent Can Do | Agent MUST NOT Do |
|--------|-------------|-------------------|
| `draft` | Write spec, research | Write production code |
| `spec-review` | Review spec, suggest improvements | Start implementing |
| `in-progress` | Write code, tests, docs | Skip tests |
| `testing` | Run tests, fix bugs | Add new features |
| `review` | Polish, final checks | Make major changes |
| `released` | Bug fixes only | Refactor or add features |
| `deprecated` | Nothing | Touch any code |

---

## Human-Agent Collaboration Model

```
HUMAN                              AGENT
  │                                  │
  ├── Sets priorities (ROADMAP)      │
  ├── Reviews specs                  │
  ├── Approves releases              │
  │                                  │
  │    ┌────────────────────────┐    │
  │    │  Shared: Specs, Tests, │    │
  │    │  Code, Documentation   │    │
  │    └────────────────────────┘    │
  │                                  │
  │                  Writes code ────┤
  │                  Runs tests ─────┤
  │                  Writes docs ────┤
  │                  Debugs issues ──┤
  │                                  │
```

**Principles:**
1. **Humans set direction** — Roadmap, priorities, spec approval
2. **Agents execute** — Code, tests, documentation, debugging
3. **Specs are the contract** — Both parties agree on what to build
4. **Tests are the proof** — Both parties can verify correctness
5. **Status tracks progress** — Both parties know where things stand

---

_[← Spec-Driven Development](04-spec-driven-development.md) | [Chapter 06: CI/CD and Releases →](06-ci-cd-and-releases.md)_
