# 🗺️ Roadmap

> **What we're building, when, and why.**

---

## Vision

Build the most comprehensive, well-organized collection of engineering tools —
from simple math scripts to complex GenAI-powered workflows — all governed by
spec-driven development and navigable by both humans and AI agents.

---

## Milestones

### 🏁 M0: Foundation (Current)

**Goal**: Establish the repository architecture and governance framework.

- [x] Repository structure designed and implemented
- [x] Agent rules and skills defined
- [x] SDD templates and workflow established
- [x] CI/CD pipelines configured
- [x] Documentation eBook started
- [x] Root Makefile and automation scripts

### 🎯 M1: First Tools

**Goal**: Deliver the first batch of useful tools to validate the architecture.

#### Active Tools

| Tool | Spec | Status | Description |
|------|------|--------|-------------|
| [`session-explorer`](../tools/session-explorer/) | [spec](../specs/catalog/session-explorer.md) | `spec-review` | Agent session library & explorer with glassmorphic web UI |

#### Milestones

- [/] First tool scaffolded and spec complete (session-explorer)
- [ ] 3-5 utility tools (different languages)
- [ ] End-to-end workflow tested (scaffold → spec → build → test → release)
- [ ] CI/CD pipelines validated in production
- [ ] First tagged release

### 🚀 M2: Growth

**Goal**: Expand the tool catalog across categories.

- [ ] 10+ tools across ≥3 categories
- [ ] Plugin architecture validated
- [ ] Cross-tool composition demonstrated
- [ ] Documentation eBook complete
- [ ] Community contribution workflow tested

### 🌟 M3: GenAI & Advanced

**Goal**: Add GenAI-powered tools and advanced patterns.

- [ ] LLM-powered tools (prompt engineering, evaluation)
- [ ] RAG components (chunkers, retrievers)
- [ ] Agent workflow tools
- [ ] Multi-language hybrid tools
- [ ] Performance benchmarking suite

---

## Tool Ideas (Backlog)

### 🧮 Math & Computation
- [ ] `matrix-ops` — Matrix operations library (Python/Rust)
- [ ] `stats-toolkit` — Statistical analysis utilities (Python)
- [ ] `number-cruncher` — Numerical computation helpers (Go)

### 🤖 GenAI & ML
- [ ] `prompt-validator` — Validate and lint LLM prompts (Python)
- [ ] `token-counter` — Count tokens for various LLM models (Python/Rust)
- [ ] `eval-harness` — LLM evaluation framework (Python)
- [ ] `rag-chunker` — Document chunking for RAG pipelines (Python)

### 🛠️ DevOps & Infrastructure
- [ ] `env-validator` — Validate environment configurations (Shell/Go)
- [ ] `port-scanner` — Local port scanner and manager (Go)
- [ ] `log-parser` — Parse and analyze log files (Rust)

### 📊 Data & Analytics
- [ ] `csv-merger` — Merge and transform CSV files (Python)
- [ ] `json-validator` — Validate JSON against schemas (Go)
- [ ] `data-profiler` — Profile datasets for quality (Python)

### ⚡ Utilities
- [ ] `file-hasher` — Hash files with multiple algorithms (Rust)
- [ ] `git-stats` — Git repository statistics (Shell)
- [ ] `url-checker` — Validate URLs and check availability (Go)

---

## Contributing to the Roadmap

1. Open an issue with the `tool-proposal` template
2. If approved, the tool is added to the backlog above
3. When work begins, a spec is created and status set to `draft`
