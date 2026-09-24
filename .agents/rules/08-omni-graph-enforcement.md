# Rule 08 — Omni-Graph Architectural Enforcement & Navigation

> **Priority**: 🔴 Critical — Avoid primitive text scanning when Omni-Graph is active.

---

## Purpose

This rule enforces structural and semantic code discovery over brute-force string searches (`grep`, `find`, `cat`), preserving token budgets and providing high-fidelity call graph comprehension.

---

## Mandatory Protocols

### 1. Structure-First Navigation
When investigating architecture, call chains, or cross-file dependencies:
- **DO NOT** run recursive `grep -rn` or multi-file `cat` across whole directories.
- **DO** query the Omni-Graph AST semantic endpoints:
  - Vector search: `GET http://localhost:8080/api/search?q=<query>&k=10`
  - Subgraph condensation: `GET http://localhost:8080/api/condense?symbol=<name>&hops=2`
  - Topology query: `GET http://localhost:8080/api/graph`

### 2. Guardrail Integration
- When `PreToolUse` lifecycle hooks fire, adhere to the guidance without attempting bypasses.
- If a symbol definition or caller lineage is needed, use the `omni-graph` skill rather than manual line scanning.

### 3. Verification
- Validate that all reported call relationships and imports match AST-verified edges (`CALLS`, `IMPORTS`, `IMPLEMENTS`) in the knowledge graph.
