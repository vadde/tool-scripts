# 🧭 Rule 00 — Omni-Graph Mandatory Retrieval & Guardrails

> **Priority**: 🔴 CRITICAL — NON-NEGOTIABLE GLOBAL INVARIANT.
> **Scope**: Machine-wide across all 100+ repositories and polyglot codebases (Go, Rust, Python, TypeScript/JavaScript, C/C++, Java, Zig, Elixir, Scala, Swift, Kotlin, Shell, etc.).
> **Order**: Prefixed with `00-` to guarantee top-tier precedence and prevent rule shadowing by downstream repository-specific rules.

---

## 1. Purpose & Core Philosophy

This entire engineering ecosystem indexes codebases into **Omni-Graph** — a high-performance AST knowledge hub backed by SurrealDB v2 and HuggingFace 384-dimensional vector embeddings.

Language models frequently suffer from **context exhaustion, cognitive drift, and hallucinated references** when attempting to understand code through brute-force text search (`grep -r`), blind directory walking (`find`), or raw whole-file dumping (`cat` / `view_file` on large files).

**Rule 00 enforces three mandatory structural invariants**:
1. **Never read whole source code files (>150 lines)** without prior AST symbol localization or condensation.
2. **Never refactor or delete symbols** without verifying caller blast radius via `/api/references`.
3. **Never guess multi-file call hierarchies** when compiler AST edges (`CALLS`, `IMPORTS`, `IMPLEMENTS`) exist in the knowledge graph.

---

## 2. Active Guardrails & Gated Tool Policies

### A. The 150-Line Source Code Gate & 450-Line Documentation Ceiling (`view_file`)
- **Active PreToolUse Interception**: Any call to `view_file` on a source code file without `EndLine` specified or spanning **more than 150 lines** is **automatically BLOCKED** by the guardrail.
- **Calibrated Documentation Ceiling**: For Markdown files (`.md`, `.markdown`), reading without `EndLine` on files $>450$ lines or requesting line spans $>450$ lines is intercepted to prevent context exhaustion, guiding agents to `/api/search`. Small documentation files ($\le 450$ lines) or targeted chapter/section slices ($\le 450$ lines) are permitted.
- **Allowed Actions**:
  - ✅ Query symbol definition: `curl -s "http://localhost:8080/api/symbol?name=<symbol>&workspace=<ws>"`
  - ✅ Extract topological subgraph slice: `curl -s "http://localhost:8080/api/condense?symbol=<symbol>&workspace=<ws>&hops=2"`
  - ✅ View focused line slice: `view_file` with `StartLine: X, EndLine: Y` where `(Y - X + 1) <= 150` lines for code or `<= 450` lines for documentation.
  - ✅ Reading non-code configs (`.txt`, `.json`, `.yaml`, `.toml`, `.csv`) is unbounded and permitted.

### B. Prohibited Brute-Force Shell & Grep Scans
- **Recursive Grep Blocked**: `grep -r`, `grep -rn`, `find . -name`, `ag`, `rg` with blanket patterns are actively blocked by PreToolUse.
- **Required Alternative**:
  - Semantic vector search: `curl -s "http://localhost:8080/api/search?q=<natural_language_query>&workspace=<ws>&k=5"`
  - High-level architectural Q&A: `curl -s "http://localhost:8080/api/query?q=<question>&workspace=<ws>"`

### C. Caller Blast-Radius Verification (Before Edits)
- Before renaming, modifying a public signature, or deleting any function, method, struct, or type:
  - **MANDATORY**: Run `curl -s "http://localhost:8080/api/references?symbol=<name>&workspace=<ws>"`
  - Review all callers across all files in the repository before making edits.

---

## 3. The 3-Tier Omniverse Reconnaissance Ladder

When tasked with any bug fix, optimization, feature implementation, or codebase exploration:

```
                      ┌───────────────────────────────────────┐
                      │  TIER 1: MACROSCOPIC RECONNAISSANCE   │
                      │  • GET http://localhost:8080/api/workspaces
                      │  • GET http://localhost:8080/api/query?q=...
                      └───────────────────┬───────────────────┘
                                          │
                                          ▼
                      ┌───────────────────────────────────────┐
                      │   TIER 2: MESOSCOPIC SUBSYSTEM MAP    │
                      │  • GET http://localhost:8080/api/galaxies?workspace=...
                      │  • Inspect Louvain/Leiden modular clusters
                      └───────────────────┬───────────────────┘
                                          │
                                          ▼
                      ┌───────────────────────────────────────┐
                      │  TIER 3: MICROSCOPIC AST CONDENSATION │
                      │  • GET http://localhost:8080/api/symbol?name=...
                      │  • GET http://localhost:8080/api/references?symbol=...
                      │  • GET http://localhost:8080/api/condense?symbol=...&hops=2
                      └───────────────────────────────────────┘
```

### Tier 1: Macroscopic Reconnaissance (Omniverse Discovery)
- **Goal**: Identify which workspace and core files contain the relevant domain logic.
- **Action**: Check workspace status in prompt header or `GET /api/workspaces`. Run hybrid Graph-RAG queries (`/api/query?q=...`) for high-level architectural understanding.

### Tier 2: Mesoscopic Subsystem Mapping (Galaxy Clusters)
- **Goal**: Map architectural boundaries, component clusters, and dependencies without reading files.
- **Action**: Query `/api/galaxies?workspace=<ws>` to view detected community partitions and entrypoint symbols.

### Tier 3: Microscopic AST Condensation (<1500 Tokens)
- **Goal**: Extract pinpoint function signatures, line numbers, and directional call graphs.
- **Action**: Use `/api/symbol`, `/api/references`, and `/api/condense` to get high-density prompt slices (<1,500 tokens).

---

## 4. Decision Matrix: Polyglot Action Guide

| Scenario / Intent | ❌ Prohibited Anti-Pattern | ✅ Mandatory Omni-Graph Action |
|---|---|---|
| Exploring a codebase | `view_file` on entire files (>150 lines) | `/api/galaxies` or `/api/workspaces` |
| Understanding algorithm / logic | Blind reading of 5–10 files | `/api/condense?symbol=<root>&hops=2` |
| Finding a symbol definition | `grep -rn "func Solve" .` | `/api/symbol?name=Solve&workspace=<ws>` |
| Finding all callers of a function | `grep -rn "Solve(" .` | `/api/references?symbol=Solve&workspace=<ws>` |
| Conceptual / fuzzy search | Raw text regex across repository | `/api/search?q=<concepts>&workspace=<ws>` |
| Inspecting implementation | `view_file` without line numbers | `view_file` with `StartLine` & `EndLine` (<= 150 lines) |

---

## 5. Fail-Safe Liveness Policy

- If the Omni-Graph server (`http://localhost:8080/api/health`) is temporarily unreachable:
  - Guardrail hooks **fail-open** to ensure agent continuity.
  - However, agents should attempt to start the stack via `make -C tools/omni-graph up` (or `docker compose -f tools/omni-graph/docker-compose.yml up -d`) to restore high-density AST capabilities.
