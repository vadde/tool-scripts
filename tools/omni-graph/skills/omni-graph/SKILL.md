---
name: omni-graph
description: >-
  Use this skill to navigate, semantically search, trace call chains, and condense AST subgraphs for codebases using the local Omni-Graph knowledge hub. Always use this instead of running brute-force grep or multi-file cat searches.
---

# 🧭 Omni-Graph Codebase Intelligence Skill

This skill provides deterministic AST analysis and vector similarity search across indexed codebases, condensing multi-file call chains into compact prompt slices (<1500 tokens).

---

## When to Use

- When locating symbol definitions, functions, structs, or methods across a codebase
- When tracing who calls a function or which dependencies a module imports
- When looking up code semantically by intent rather than exact text matching
- When needing architectural context without burning 50K tokens in raw files

---

## Available Commands

Use the helper script `omni.sh` located in `./scripts/omni.sh` or directly query the REST endpoints:

### 1. Semantic Vector Search
Find functions and classes related to a concept or feature:
```bash
./scripts/omni.sh search "database pool handler" 5
```
Or via HTTP:
```bash
curl -s "http://localhost:8080/api/search?q=database+pool+handler&k=5"
```

### 2. Symbolic Definition Lookup (LSP textDocument/definition)
Locate symbol definitions, signatures, and file line ranges without text regex:
```bash
./scripts/omni.sh symbol "init_pool"
```
Or via HTTP: `GET http://localhost:8080/api/symbol?name=init_pool`

### 3. Symbolic Reference Tracing (LSP textDocument/references)
Find all functions and callers that invoke or reference a symbol:
```bash
./scripts/omni.sh references "init_pool"
```
Or via HTTP: `GET http://localhost:8080/api/references?symbol=init_pool`

### 4. Condense Multi-Hop Subgraph (<1500 tokens)
Extract the immediate call trace and connected AST nodes around a function or struct:
```bash
./scripts/omni.sh condense "init_pool" 2
```
This outputs a rich Markdown block ready to insert into your reasoning context.

### 5. Hybrid Graph-RAG Retrieval (Micro Seeds + Macro Communities)
Execute a hybrid retrieval combining vector similarity, community summaries, and AST subgraphs:
```bash
./scripts/omni.sh query "How does authentication and session storage work?" 5
```
Or via HTTP: `POST http://localhost:8080/api/query` with `{"prompt": "...", "top_k": 5}`

### 6. Run Community Clustering
Trigger Louvain/Leiden community detection across all AST nodes:
```bash
./scripts/omni.sh cluster
```

### 7. Check Hub Health
Verify that the Rust orchestrator, SurrealDB, and HuggingFace TEI are active:
```bash
./scripts/omni.sh health
```

### 8. Ingest Target Directory
Index a new directory or trigger a re-scan:
```bash
./scripts/omni.sh ingest "/workspace/my-project"
```
