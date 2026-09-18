<div align="center">

# 🔧 tool-scripts

### Enterprise Engineering Tools, Scripts, Plugins & Workflows

[![CI](https://github.com/vadde/tool-scripts/actions/workflows/ci.yml/badge.svg)](https://github.com/vadde/tool-scripts/actions/workflows/ci.yml)
[![Security](https://github.com/vadde/tool-scripts/actions/workflows/security-audit.yml/badge.svg)](https://github.com/vadde/tool-scripts/actions/workflows/security-audit.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![SDD](https://img.shields.io/badge/methodology-Spec%20Driven-purple)](specs/README.md)

*A polyglot monorepo housing self-contained engineering tools — from simple
math scripts to complex GenAI-powered workflows — governed by spec-driven
development and navigable by both humans and AI agents.*

[**Getting Started**](docs/book/01-getting-started.md) · [**Tool Catalog**](tools/README.md) · [**Documentation**](docs/README.md) · [**Roadmap**](sdlc/ROADMAP.md) · [**Contributing**](CONTRIBUTING.md)

</div>

---

## ✨ What Is This?

**tool-scripts** is a monorepo for engineering tools built with a first-principles
approach to code quality, documentation, and lifecycle management:

- 🔧 **Self-contained tools** — Each tool lives in its own folder with everything
  it needs: source, tests, docs, spec, and lifecycle status
- 📋 **Spec-driven development** — No code without a specification. Every
  requirement is numbered, traced, and tested
- 🤖 **Agent-native** — Built for AI agents to autonomously navigate, build,
  test, and maintain tools using ReAct and Reflexion patterns
- 🌍 **Polyglot** — Python, Go, Rust, Node.js, Shell — whatever fits the problem
- 🔄 **Full lifecycle** — Every tool has an SDLC status: draft → spec-review →
  in-progress → testing → review → released

---

## 🗺️ Repository Map

```
tool-scripts/
├── .agents/          🧠 Agent rules, skills, and behavioral guidelines
├── .github/          ⚙️ CI/CD workflows, issue templates, CODEOWNERS
├── specs/            📋 Specifications (SDD templates and active specs)
├── tools/            🔧 The tools themselves (each is self-contained)
├── docs/             📚 Documentation eBook, ADRs, how-to guides
├── sdlc/             🔄 SDLC governance (roadmap, changelog, status board)
├── scripts/          🛠️ Repo automation (scaffold, test, validate)
├── Makefile          🎯 Unified command interface
├── CONTRIBUTING.md   🤝 How to contribute
└── README.md         📍 You are here
```

---

## 🚀 Quick Start

```bash
# Clone
git clone https://github.com/vadde/tool-scripts.git
cd tool-scripts

# See what's available
make help

# Create your first tool
make new-tool NAME=my-awesome-tool

# Run all tests
make test

# View the SDLC dashboard
make status
```

---

## 🔧 Tool Catalog

> See the [full catalog](tools/README.md) for details.

| Tool | Category | Language | Status | Description |
|------|----------|----------|--------|-------------|
| _Coming soon_ | — | — | — | _First tools are in development_ |

### Categories

| 🧮 Math & Computation | 🤖 GenAI & ML | 🛠️ DevOps | 📊 Data & Analytics |
|---|---|---|---|
| Numerical tools | LLM utilities | CI/CD helpers | ETL pipelines |
| Statistical analysis | Prompt engineering | Infrastructure | Format conversion |
| Algorithms | Model evaluation | Deployment | Visualization |

| 🔌 Plugins | ⚡ Utilities | 🔬 Scientific | 🌐 Web & API |
|---|---|---|---|
| Editor extensions | Validators | Simulations | API clients |
| CLI tools | Converters | Research | Web scrapers |

---

## 📚 Documentation

The **[Tool-Scripts Book](docs/README.md)** is a 9-chapter guide covering
everything from getting started to advanced patterns:

| Chapter | Topic |
|---------|-------|
| [00 — Preface](docs/book/00-preface.md) | Vision and design philosophy |
| [01 — Getting Started](docs/book/01-getting-started.md) | Setup and first tool |
| [02 — Architecture](docs/book/02-architecture.md) | Repository structure |
| [03 — Tool Development](docs/book/03-tool-development-guide.md) | Building tools end-to-end |
| [04 — Spec-Driven Dev](docs/book/04-spec-driven-development.md) | SDD methodology |
| [05 — Agent Collaboration](docs/book/05-agent-collaboration.md) | AI agent workflows |
| [06 — CI/CD & Releases](docs/book/06-ci-cd-and-releases.md) | Pipelines and releases |
| [07 — Advanced Patterns](docs/book/07-advanced-patterns.md) | GenAI, plugins, composition |
| [08 — Appendix](docs/book/08-appendix.md) | Glossary and references |

---

## 🤖 Agent Collaboration

This repository is designed for autonomous AI agents. The
[Agent Intelligence Layer](.agents/AGENTS.md) includes:

- **6 behavioral rules** — ReAct protocol, code standards, commit conventions,
  SDD workflow, testing strategy, documentation standards
- **3 skills** — New tool scaffolding, debugging, release preparation
- **SDLC awareness** — Agents check tool status before acting
- **Divide & conquer** — Complex problems decomposed into atomic sub-tasks
- **Reflexion gates** — Self-critique after every significant action

---

## 🎯 Make Targets

```bash
make help              # Show all targets
make setup             # Install pre-commit hooks
make new-tool NAME=x   # Scaffold a new tool
make test              # Test ALL tools
make test-tool T=name  # Test a specific tool
make lint              # Lint ALL tools
make lint-tool T=name  # Lint a specific tool
make validate-specs    # Validate all specs
make catalog           # Regenerate tool catalog
make status            # SDLC status dashboard
make clean             # Clean artifacts
```

---

## 🔄 Development Workflow

```
1. Spec First     →  Write the spec before code
2. Scaffold       →  make new-tool NAME=my-tool
3. Implement      →  Write code in tools/my-tool/src/
4. Test           →  make test-tool T=my-tool
5. Document       →  Update README, examples, CHANGELOG
6. Review         →  Submit PR with SDD checklist
7. Release        →  Tag and publish
```

---

## 🏗️ Architecture Decisions

Key decisions are documented as [Architecture Decision Records](docs/architecture/README.md):

| ADR | Decision |
|-----|----------|
| [ADR-001](docs/architecture/adr-001-monorepo-structure.md) | Monorepo over multi-repo |

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for the full
guide. Quick version:

1. Fork and clone
2. Create a feature branch: `feat/tool-name/description`
3. Write a spec in `specs/catalog/`
4. Implement with tests
5. Submit a PR with the SDD checklist

---

## 📄 License

[MIT](LICENSE) — Use freely, build boldly.

---

<div align="center">

**Built with ❤️ and rigorous engineering discipline.**

[Report Bug](.github/ISSUE_TEMPLATE/bug_report.yml) · [Request Feature](.github/ISSUE_TEMPLATE/feature_request.yml) · [Propose Tool](.github/ISSUE_TEMPLATE/tool_proposal.yml)

</div>
