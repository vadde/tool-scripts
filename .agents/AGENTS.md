# 🤖 AGENTS.md — Master Agent Instructions

> **This file is the single source of truth for any AI agent operating in this repository.**
> Read this document in its entirety before performing any action.

---

## Repository Identity

| Field | Value |
|-------|-------|
| **Name** | `vadde/tool-scripts` |
| **Purpose** | Enterprise monorepo for engineering tools, scripts, plugins, and workflows |
| **Architecture** | Polyglot monorepo with self-contained tool folders |
| **Governance** | Spec-Driven Development (SDD) with SDLC status tracking |

---

## Repository Map

```
tool-scripts/
├── .agents/          → YOU ARE HERE — Agent rules, skills, plugins
├── specs/            → Specifications (read BEFORE writing code)
├── tools/            → Self-contained tools (each is independent)
├── docs/             → Documentation eBook, ADRs, guides
├── sdlc/             → SDLC status tracking, roadmap, changelog
├── scripts/          → Repo-level automation scripts
├── .github/          → CI/CD workflows, issue/PR templates
├── Makefile          → Unified command interface (start here)
├── README.md         → Grand Central README with tool catalog
└── CONTRIBUTING.md   → Contribution guidelines
```

---

## Agent Operating Protocol

### 1. Orientation Phase (ALWAYS do this first)

Before ANY action, complete this checklist:

- [ ] Read this file (`AGENTS.md`) completely
- [ ] Identify the target tool in `tools/<name>/`
- [ ] **Read the tool's `CONTEXT.md` FIRST** — this is the single orientation file
- [ ] Read the tool's `DEVLOG.md` (latest entry) for recent session context
- [ ] Verify the tool's `STATUS.md` to confirm SDLC phase
- [ ] Read the tool's spec in `specs/catalog/<name>.md` (if implementing)
- [ ] Read relevant rules in `.agents/rules/`
- [ ] Check `sdlc/ROADMAP.md` for context on priorities

> **⚠️ CRITICAL**: After EVERY session, you MUST update `CONTEXT.md` and append
> to `DEVLOG.md`. See Rule 07 for the mandatory session end protocol.

### 2. Core Behavioral Rules

All rules are in `.agents/rules/`. Read them in order:

| Rule File | Domain | Priority |
|-----------|--------|----------|
| [`01-core-principles.md`](rules/01-core-principles.md) | ReAct, Reflexion, divide-and-conquer | 🔴 Critical |
| [`02-code-standards.md`](rules/02-code-standards.md) | Coding conventions (polyglot) | 🔴 Critical |
| [`03-commit-conventions.md`](rules/03-commit-conventions.md) | Git workflow, branches, commits | 🔴 Critical |
| [`04-sdd-workflow.md`](rules/04-sdd-workflow.md) | Spec-Driven Development protocol | 🔴 Critical |
| [`05-testing-strategy.md`](rules/05-testing-strategy.md) | Testing pyramid, coverage | 🟡 Important |
| [`06-documentation.md`](rules/06-documentation.md) | Documentation standards | 🟡 Important |
| [`07-iterative-development.md`](rules/07-iterative-development.md) | Session continuity, CONTEXT.md, DEVLOG.md | 🔴 Critical |

### 3. Available Skills

Skills are composable capabilities for common workflows:

| Skill | Purpose | When to Use |
|-------|---------|-------------|
| [`new-tool`](skills/new-tool/SKILL.md) | Scaffold a new tool | Creating any new tool |
| [`debug-tool`](skills/debug-tool/SKILL.md) | Systematic debugging | Investigating failures |
| [`release-tool`](skills/release-tool/SKILL.md) | Prepare for release | Finalizing a tool version |

### 4. SDLC Status Guide

Every tool has a `STATUS.md` with its current lifecycle phase. Your actions
MUST align with the tool's current status:

| Status | Meaning | Agent Actions Allowed |
|--------|---------|----------------------|
| `draft` | Initial idea, no spec yet | Write spec, research, plan |
| `spec-review` | Spec written, awaiting review | Review spec, suggest improvements |
| `in-progress` | Active development | Write code, create tests, update docs |
| `testing` | Code complete, testing phase | Run tests, fix bugs, improve coverage |
| `review` | Ready for human review | Polish docs, run final checks |
| `released` | Published and stable | Bug fixes only (patch versions) |
| `deprecated` | No longer maintained | Do not modify; redirect users |

> **⚠️ CRITICAL**: Never advance a tool's status without completing ALL
> requirements for the current phase. See `04-sdd-workflow.md` for details.

### 5. Tool Discovery

Each tool lives in `tools/<name>/` and contains:

```
tools/<name>/
├── CONTEXT.md      → 🧭 START HERE — Agent orientation & current state
├── DEVLOG.md       → 📓 Development journal (append-only, session history)
├── README.md       → Documentation (usage, installation, examples)
├── spec.md         → Specification (links to specs/catalog/<name>.md)
├── STATUS.md       → SDLC status (phase, traceability matrix, progress)
├── CHANGELOG.md    → Version history (Keep-a-Changelog format)
├── Makefile        → Tool-local targets (test, lint, run, build)
├── src/            → Source code (any language)
├── tests/          → Test suite
└── examples/       → Runnable usage examples
```

The **tool catalog** is in [`tools/README.md`](../tools/README.md).

### 6. Language Auto-Detection

Tools can be written in ANY language. Detect the language by checking for:

| Language | Indicator Files |
|----------|----------------|
| Python | `requirements.txt`, `pyproject.toml`, `setup.py`, `*.py` in `src/` |
| Node.js | `package.json`, `*.js`/`*.ts` in `src/` |
| Go | `go.mod`, `*.go` in `src/` |
| Rust | `Cargo.toml`, `*.rs` in `src/` |
| Shell/Bash | `*.sh` in `src/` |
| C/C++ | `CMakeLists.txt`, `*.c`/`*.cpp` in `src/` |
| Java | `pom.xml`, `build.gradle`, `*.java` in `src/` |
| Ruby | `Gemfile`, `*.rb` in `src/` |

### 7. Key Commands

Use the root `Makefile` as your primary interface:

```bash
make help              # Show all available targets
make new-tool NAME=x   # Scaffold a new tool
make test              # Run ALL tool tests
make test-tool T=name  # Run tests for a specific tool
make lint              # Lint all tools
make validate-specs    # Validate all SDD specs
make catalog           # Regenerate tool catalog
make status            # Show SDLC status dashboard
```

---

## Emergency Protocols

### If You're Uncertain
1. **STOP** — Do not guess or assume
2. **Re-read** the tool's `spec.md` and `STATUS.md`
3. **Check** related tools for patterns to follow
4. **Ask** the human for clarification if ambiguity persists

### If Tests Fail
1. **Read** the full error output carefully
2. **Correlate** with the spec requirements
3. **Apply Reflexion** — analyze what went wrong and why
4. **Fix** with a targeted change, not a broad rewrite
5. **Verify** the fix resolves the issue without regressions

### If You Break Something
1. **Revert** your changes immediately
2. **Document** what happened and why
3. **Re-approach** with a smaller, safer change
