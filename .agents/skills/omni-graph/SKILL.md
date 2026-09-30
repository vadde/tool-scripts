---
name: omni-graph
description: >-
  Use this skill to navigate, semantically search, trace call chains, and condense AST subgraphs for codebases using the local Omni-Graph knowledge hub. Always use this instead of running brute-force grep or multi-file cat searches.
---

# 🧭 Omni-Graph Codebase Intelligence Skill

This skill gives coding agents **deterministic AST analysis, multi-workspace exploration, and vector similarity search** across indexed codebases, condensing multi-file call chains into high-signal prompt slices (<1,500 tokens).

Always use this skill instead of running blind recursive `grep -rn` or reading whole files sequentially.

---

## 🎯 When to Use

1. **Exploring an Unfamiliar Codebase**: When you need to understand the architectural subsystems, packages, or directory relationships without reading hundreds of files.
2. **Locating Exact Symbol Definitions**: Finding functions, structs, classes, or interfaces, their exact line ranges, and signatures without regex text false positives.
3. **Tracing Callers & References**: Finding every function that calls a specific method or imports a package.
4. **Extracting Call Subgraphs**: Pulling a 1-to-3 hop directional call graph slice around a target function directly into your context.
5. **High-Level Conceptual Questions**: Asking "How does X feature work?" and getting a hybrid Graph-RAG answer combining macroscopic summaries with microscopic code slices.

---

## ⚡ Quick Make Commands (From Repository Root)

The easiest way to interact with Omni-Graph is via the root `Makefile`:

```bash
# 1. Discover all indexed codebases / workspaces
make workspaces

# 2. Inspect architectural subsystems (Galaxies) for a workspace
make graph-galaxies PROJECT=session-explorer
make graph-galaxies PROJECT=DSA

# 3. Locate exact symbol definition (LSP textDocument/definition)
make graph-symbol SYM=extractWorkspace PROJECT=session-explorer

# 4. Find all callers of a symbol (LSP textDocument/references)
make graph-references SYM=extractWorkspace PROJECT=session-explorer

# 5. Extract a condensed AST call graph slice (<1500 tokens)
make graph-condense SYM=extractWorkspace PROJECT=session-explorer HOPS=2

# 6. Semantic vector similarity code search
make search-graph Q="session scanner" PROJECT=session-explorer K=5

# 7. Hybrid Graph-RAG architectural query
make query-graph Q="How does session extraction and date parsing work?" PROJECT=session-explorer

# 8. Augment knowledge graph with dynamic runtime relationships
make relate-graph SRC="ChatUI" TGT="FastAPI" TYPE="ROUTES_TO" PROJECT="QuarkDock"
```

---

## 🛠️ CLI Script Commands (`omni.sh`)

You can also use `./tools/omni-graph/scripts/omni.sh` directly:

### 1. Architectural Subsystem Mapping (`galaxies`)
```bash
./tools/omni-graph/scripts/omni.sh galaxies [workspace]
```
Displays an ASCII table of detected Louvain/Leiden modular clusters, dominant file paths, node counts, and key symbol entrypoints.

### 2. Symbol Definition Lookup (`symbol`)
```bash
./tools/omni-graph/scripts/omni.sh symbol "init_pool" [workspace]
```
Returns JSON containing exact file path, start line, end line, AST node kind, and full definition snippet.

### 3. Symbol Caller Lineage (`references`)
```bash
./tools/omni-graph/scripts/omni.sh references "init_pool" [workspace]
```
Finds all functions and tests in the graph with directional `CALLS` or `IMPORTS` edges pointing to the target symbol.

### 4. Agent Relationship Augmentation (`relate`)
```bash
./tools/omni-graph/scripts/omni.sh relate <source_sym> <target_sym> [relation_type] [workspace]
```
Tree-Sitter parsers only extract static in-file AST edges (`category: "EXTRACTED"`). When you discover dynamic cross-process or runtime relationships (e.g. React UI connecting via SSE to FastAPI, or WebSocket event dispatchers), use `relate` to augment SurrealDB with inferred edges (`category: "INFERRED"`). If either node is external or virtual, Omni-Graph synthesizes auxiliary virtual components automatically.

### 5. Condensed Multi-Hop Subgraph (`condense`)
```bash
./tools/omni-graph/scripts/omni.sh condense "init_pool" [workspace] 2
```
Outputs a dense, beautifully formatted Markdown slice with definitions, signatures, and ASCII call graph traces ready to paste into your reasoning context.

### 6. Semantic Vector Search (`search`)
```bash
./tools/omni-graph/scripts/omni.sh search "websocket message dispatcher" [workspace] 5
```
Performs 384-dimensional cosine ANN vector search across SurrealDB HNSW index.

### 7. Hybrid Graph-RAG Retrieval (`query`)
```bash
./tools/omni-graph/scripts/omni.sh query "How does authentication flow work?" [workspace] 5
```
Combines microscopic vector seeds with macroscopic Louvain community cluster summaries and expanded AST subgraphs.

---

## 🌐 Direct HTTP REST API (`http://localhost:8080`)

For automated subagents, direct REST queries are available:

- `GET /api/workspaces` — List all indexed codebases
- `GET /api/galaxies?workspace=<name>` — List architectural clusters
- `GET /api/symbol?name=<sym>&workspace=<ws>` — Symbol definition
- `GET /api/references?symbol=<sym>&workspace=<ws>` — Symbol callers (includes inferred edges and metadata)
- `GET /api/condense?symbol=<sym>&workspace=<ws>&hops=2` — Condensed markdown slice
- `GET /api/search?q=<query>&workspace=<ws>&k=10` — Vector search
- `POST /api/query` (`{"prompt": "...", "workspace": "..."}`) — Graph-RAG query
- `POST /api/relationships` (`{"source_symbol": "...", "target_symbol": "...", "type": "CALLS|ROUTES_TO|...", "category": "INFERRED", "metadata": {...}}`) — Augment graph with runtime dependency

---

## 📋 Recommended Agent Workflow Recipes

### Recipe A: Investigating a Bug in an Unfamiliar Module
1. Run `make workspaces` to confirm the workspace name (e.g. `session-explorer`).
2. Run `make graph-galaxies PROJECT=session-explorer` to find which subsystem owns the feature.
3. Run `make search-graph Q="<error term or function>" PROJECT=session-explorer` to get the entrypoint symbol.
4. Run `make graph-condense SYM=<entrypoint> PROJECT=session-explorer HOPS=2` to understand the full call trace.
5. Only now open the specific target file via `view_file` at the exact line range to write your fix.

### Recipe B: Refactoring or Renaming a Function
1. Run `make graph-symbol SYM=<target>` to inspect its current implementation and line range.
2. Run `make graph-references SYM=<target>` to get a complete, guaranteed list of all caller locations and test files that depend on it.
3. Edit the definition and all callers with targeted changes, ensuring zero broken dependencies.

### Recipe C: Augmenting Dynamic Cross-Service Dependencies
1. When inspecting system configuration, Dockerfiles, or client-server communications (e.g. frontend fetching from backend endpoint), identify runtime connections that static AST parsers cannot detect.
2. Run `make relate-graph SRC="ChatUI" TGT="FastAPI" TYPE="ROUTES_TO" PROJECT="QuarkDock"` (or call `POST /api/relationships`).
3. Future agent sessions and blast-radius queries (`make graph-references SYM=FastAPI`) will immediately reveal these callers across system boundaries.
