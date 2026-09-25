# 🧭 Rule 08 — Omni-Graph Architectural Enforcement & Navigation

> **Priority**: 🔴 Critical — Structure-First Navigation Over Brute-Force Scanning.
> This rule is mandatory for all coding agents operating in this repository.

---

## 1. Purpose & Core Philosophy

This monorepo indexes codebases into **Omni-Graph** — a deterministic AST knowledge hub backed by SurrealDB v2 and HuggingFace 384-d semantic embeddings. 

AI coding agents often suffer from **cognitive drift and context exhaustion** when blindly executing recursive `grep -rn`, whole-folder `find`, or dumping large multi-file `cat` outputs into the prompt. 

**Rule 08 strictly mandates structure-first, high-signal retrieval**:
- Never read 50,000 tokens of raw source code to answer what a 600-token condensed AST slice proves deterministically.
- Never guess function callers or line ranges with regex when directional AST edges (`CALLS`, `IMPORTS`, `IMPLEMENTS`) exist in the knowledge graph.

---

## 2. The 3-Tier Omniverse Reconnaissance Ladder

When investigating an unfamiliar codebase, feature request, or bug report, follow the **3-Tier Ladder** in order:

```
                      ┌───────────────────────────────────────┐
                      │  TIER 1: MACROSCOPIC RECONNAISSANCE   │
                      │  • make workspaces                    │
                      │  • make query-graph Q="..."           │
                      └───────────────────┬───────────────────┘
                                          │
                                          ▼
                      ┌───────────────────────────────────────┐
                      │   TIER 2: MESOSCOPIC SUBSYSTEM MAP    │
                      │  • make graph-galaxies PROJECT=...    │
                      │  • Filter by architectural cluster    │
                      └───────────────────┬───────────────────┘
                                          │
                                          ▼
                      ┌───────────────────────────────────────┐
                      │  TIER 3: MICROSCOPIC AST CONDENSATION │
                      │  • make graph-symbol SYM=...          │
                      │  • make graph-references SYM=...      │
                      │  • make graph-condense SYM=... HOPS=2 │
                      └───────────────────────────────────────┘
```

### Tier 1: Macroscopic Reconnaissance (Omniverse Discovery)
- **Goal**: Identify which partitioned codebase contains the relevant domain logic.
- **Action**: Run `make workspaces` to see all ingested projects and their language distributions.
- **Hybrid Synthesis**: Run `make query-graph Q="<high-level question>" [PROJECT=name]` to retrieve both a macroscopic architectural synthesis and an expanded call subgraph in a single prompt slice.

### Tier 2: Mesoscopic Subsystem Mapping (Galaxy Clusters)
- **Goal**: Understand the architectural boundaries, modules, and subsystems without reading files.
- **Action**: Run `make graph-galaxies [PROJECT=name]` (or `./scripts/omni.sh galaxies [ws]`).
- **Inspection**: Review detected Louvain/Leiden community clusters, dominant package paths, node counts, and key entrypoint symbols.

### Tier 3: Microscopic AST Condensation (<1500 Tokens)
- **Goal**: Extract exact function signatures, line ranges, and directional call graphs without noise.
- **Symbol Definition**: Run `make graph-symbol SYM=<name> [PROJECT=name]`.
- **Caller Lineage**: Run `make graph-references SYM=<name> [PROJECT=name]` to find every caller in the codebase.
- **Multi-Hop AST Slice**: Run `make graph-condense SYM=<name> [HOPS=2] [PROJECT=name]` to extract a dense, verified markdown slice ready to insert into reasoning.

---

## 3. Decision Matrix: Which Tool to Use When

| Agent Goal / Scenario | ❌ Prohibited Anti-Pattern | ✅ Mandatory Omni-Graph Action |
|---|---|---|
| Exploring a new repository | `ls -R`, recursive `find .` | `make workspaces` + `make graph-galaxies PROJECT=<name>` |
| Asking "How does X work?" | Reading 10 source files sequentially | `make query-graph Q="How does X work?" PROJECT=<name>` |
| Finding where a function is defined | `grep -rn "def init_pool" .` | `make graph-symbol SYM=init_pool [PROJECT=<name>]` |
| Finding all callers of an API | `grep -rn "init_pool" .` | `make graph-references SYM=init_pool [PROJECT=<name>]` |
| Tracing multi-file call chains | Reading files and following imports manually | `make graph-condense SYM=init_pool HOPS=2` |
| Fuzzy / Conceptual code search | Blind regex across directory | `make search-graph Q="database connection pool"` |

---

## 4. Guardrail & Lifecycle Hook Adherence

1. **PreToolUse Interception**:
   - The repo configuration in `.agents/hooks.json` intercepts whole-repo `grep` and `find` commands.
   - If a guardrail hook denies a tool call with guidance to use Omni-Graph, **never attempt to circumvent or rephrase the grep**. Immediately switch to the indicated Omni-Graph command.
2. **PreInvocation Awareness**:
   - When the ephemeral notification `🧭 [Omni-Graph Active]` is present in system context, Omni-Graph is online and ready for queries on port 8080.
3. **Fail-Safe Recovery**:
   - If the Omni-Graph stack is offline (`Connection error`), start it with `make omni-graph` (or `make -C tools/omni-graph up`) before proceeding.

---

## 5. Token Budgeting Protocols

- **Floor**: Aim for <1,500 tokens of high-fidelity code context per investigation step.
- **Density**: Use `make graph-condense` which strips file fluff and retains AST-typed relationships (`CALLS`, `IMPORTS`, `IMPLEMENTS`).
- **Precision**: Only open raw full source files via `view_file` when actively writing code edits or modifying lines after AST localization has pin-pointed the target slice.
