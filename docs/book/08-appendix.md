# Chapter 08 — Appendix

> _Glossary, references, and quick-lookup tables._

---

## Glossary

| Term | Definition |
|------|-----------|
| **ADR** | Architecture Decision Record — a document capturing a key design decision |
| **Agent** | An AI system that autonomously performs tasks in this repository |
| **Acceptance Criteria** | Testable conditions that prove a spec requirement is satisfied |
| **CLI** | Command Line Interface |
| **Conventional Commits** | A commit message format: `type(scope): description` |
| **Divide and Conquer** | Breaking complex problems into smaller, manageable sub-tasks |
| **GenAI** | Generative Artificial Intelligence |
| **HITL** | Human-In-The-Loop — requiring human approval for critical actions |
| **Keep-a-Changelog** | A standard format for changelog files |
| **LLM** | Large Language Model |
| **Monorepo** | A single repository containing multiple projects/tools |
| **MVP** | Minimum Viable Product |
| **Polyglot** | Supporting multiple programming languages |
| **RAG** | Retrieval-Augmented Generation |
| **ReAct** | Reason + Act — an agent pattern combining reasoning with action |
| **Reflexion** | Self-critique and retry — an agent pattern for improving accuracy |
| **SDD** | Spec-Driven Development |
| **SDLC** | Software Development Lifecycle |
| **SemVer** | Semantic Versioning (MAJOR.MINOR.PATCH) |
| **Spec** | Specification — a formal description of what a tool should do |

---

## Quick Reference: SDLC Statuses

| Status | Emoji | Meaning |
|--------|-------|---------|
| `draft` | 🟤 | Initial idea, spec being written |
| `spec-review` | 🟡 | Spec complete, awaiting review |
| `in-progress` | 🔵 | Active development |
| `testing` | 🟣 | Code complete, testing phase |
| `review` | 🟠 | Ready for human review |
| `released` | 🟢 | Published and stable |
| `deprecated` | ⚫ | No longer maintained |

---

## Quick Reference: Commit Types

| Type | Purpose | Version Bump |
|------|---------|-------------|
| `feat` | New feature | Minor |
| `fix` | Bug fix | Patch |
| `docs` | Documentation | None |
| `style` | Formatting | None |
| `refactor` | Restructure | None |
| `test` | Tests | None |
| `chore` | Maintenance | None |
| `ci` | CI/CD | None |
| `perf` | Performance | Patch |
| `build` | Build system | None |
| `revert` | Revert | Depends |

---

## Quick Reference: Make Targets

```bash
make help              # Show all available targets
make setup             # Install dependencies & hooks
make new-tool NAME=x   # Scaffold a new tool
make test              # Run ALL tool tests
make test-tool T=name  # Run tests for a specific tool
make lint              # Lint all tools
make lint-tool T=name  # Lint a specific tool
make validate-specs    # Validate all SDD specs
make catalog           # Regenerate tool catalog
make status            # Show SDLC status dashboard
make clean             # Clean all build artifacts
```

---

## Quick Reference: File Structure

```
tools/<name>/
├── README.md       → Documentation
├── spec.md         → Specification
├── STATUS.md       → SDLC status
├── CHANGELOG.md    → Version history
├── Makefile        → Build/test targets
├── src/            → Source code
├── tests/          → Test suite
└── examples/       → Usage examples
```

---

## References

### Standards

- [Conventional Commits v1.0.0](https://www.conventionalcommits.org/)
- [Semantic Versioning 2.0.0](https://semver.org/)
- [Keep a Changelog 1.1.0](https://keepachangelog.com/)
- [Contributor Covenant 2.1](https://www.contributor-covenant.org/)

### Patterns

- [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)
- [Reflexion: Language Agents with Verbal Reinforcement Learning](https://arxiv.org/abs/2303.11366)

### Style Guides

- [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html)
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- [Effective Go](https://go.dev/doc/effective_go)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

_[← Advanced Patterns](07-advanced-patterns.md) | [Back to Table of Contents](../README.md)_
