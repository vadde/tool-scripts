# Chapter 02 — Architecture

> _Understanding the bones of the machine._

---

## Repository Architecture

```
tool-scripts/
│
├── .agents/               🧠 Agent Intelligence Layer
│   ├── AGENTS.md          Master instructions for AI agents
│   ├── rules/             Behavioral rules (ReAct, code standards, etc.)
│   ├── skills/            Composable agent capabilities
│   └── plugins/           Agent plugins (future)
│
├── specs/                 📋 Specification Layer
│   ├── _templates/        Spec templates
│   └── catalog/           Active specifications (one per tool)
│
├── tools/                 🔧 Tool Layer (the products)
│   ├── _template/         Canonical tool template
│   └── <tool-name>/       Each tool is self-contained
│
├── docs/                  📚 Knowledge Layer
│   ├── book/              This eBook
│   ├── architecture/      ADRs (Architecture Decision Records)
│   ├── guides/            How-to guides
│   └── assets/            Images, diagrams
│
├── sdlc/                  🔄 Governance Layer
│   ├── ROADMAP.md         Project roadmap
│   ├── CHANGELOG.md       Global changelog
│   └── status-board.md    Aggregated tool status
│
├── scripts/               🛠️ Automation Layer
│   ├── scaffold-tool.sh   Tool scaffolding
│   ├── run-all-tests.sh   Cross-tool test runner
│   ├── update-catalog.sh  Catalog regeneration
│   └── validate-specs.sh  Spec validation
│
├── .github/               ⚙️ CI/CD Layer
│   ├── workflows/         GitHub Actions pipelines
│   └── ISSUE_TEMPLATE/    Issue & PR templates
│
└── Root Files
    ├── Makefile            Unified command interface
    ├── README.md           Grand Central README
    ├── CONTRIBUTING.md     Contribution guidelines
    └── ...                 License, security, etc.
```

---

## Design Principles

### Principle 1: Self-Contained Tools

Each tool in `tools/<name>/` is a **complete, independent unit**:

```
tools/<name>/
├── README.md       Documentation
├── spec.md         Specification
├── STATUS.md       SDLC lifecycle status
├── CHANGELOG.md    Version history
├── Makefile        Build/test/run commands
├── src/            Source code (any language)
├── tests/          Test suite
└── examples/       Runnable demonstrations
```

**Why?** A tool should be understandable without reading any other tool.
Copy a tool directory to another repo and it should still make sense.

### Principle 2: Separation of Concerns

| Layer | Responsibility | Changes Independently? |
|-------|---------------|----------------------|
| Agent Layer | How agents behave | Yes |
| Spec Layer | What tools should do | Yes |
| Tool Layer | How tools work | Yes (per tool) |
| Docs Layer | Institutional knowledge | Yes |
| Governance Layer | Status & planning | Yes |
| Automation Layer | Repo scripts | Yes |
| CI/CD Layer | Pipelines | Yes |

### Principle 3: Polyglot First

The architecture makes zero assumptions about programming language. The
Makefile provides a unified interface (`make test`, `make lint`) while each
tool's local Makefile translates to language-specific commands.

```
make test-tool T=json-validator  →  tools/json-validator/Makefile test  →  pytest
make test-tool T=file-hasher     →  tools/file-hasher/Makefile test     →  go test
make test-tool T=csv-merger      →  tools/csv-merger/Makefile test      →  vitest
```

### Principle 4: Convention Over Configuration

| Convention | Standard |
|-----------|----------|
| Tool naming | `kebab-case` |
| Source directory | `src/` |
| Test directory | `tests/` |
| Entry point | `main.<ext>` |
| Spec location | `specs/catalog/<tool-name>.md` |
| Status tracking | `STATUS.md` with YAML frontmatter |
| Version history | `CHANGELOG.md` in Keep-a-Changelog format |

---

## Data Flow

```
                    ┌─────────────────┐
                    │   ROADMAP.md    │  What to build next
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   specs/        │  Define what to build
                    │   catalog/      │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   tools/        │  Build it
                    │   <name>/src/   │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   tools/        │  Verify it
                    │   <name>/tests/ │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   STATUS.md     │  Track it
                    │   CHANGELOG.md  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   Release       │  Ship it
                    │   (CI/CD)       │
                    └─────────────────┘
```

---

## Key Architectural Decisions

All major decisions are documented as ADRs in
[`docs/architecture/`](../architecture/README.md).

| ADR | Decision | Rationale |
|-----|----------|-----------|
| [ADR-001](../architecture/adr-001-monorepo-structure.md) | Monorepo over multi-repo | Discoverability, shared standards, atomic changes |

---

_[← Getting Started](01-getting-started.md) | [Chapter 03: Tool Development Guide →](03-tool-development-guide.md)_
