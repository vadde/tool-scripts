# Changelog

All notable changes to this tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-10-09

### Added
- **Polyglot Struct Field & Interface Property Extraction (R-057)**: Extracted `field_declaration` (Rust, Go), `property_signature` (TypeScript), and `public_field_definition`/`field_definition` (TS/JS) into `kind: "field"` nodes linked via `CONTAINS` edges from parent structs/interfaces and `REFERENCES` edges to concrete types.
- **Targeted Neighborhood Subgraph Retrieval (`DbClient::get_neighborhood_subgraph`)**: Localized 1-hop BFS in SurrealDB replaces full-graph HTTP downloads in `GraphRagEngine::query`.
- **Strict Context Budgeting on Graph-RAG Synthesis**: Two-tier budgeting capping symbol definitions at 3,800 chars and reserving $\ge 1,800$ chars for call/import relationships, enforcing the <1500 token SLA.
- **Relational Graph Edge Indexes (`schema.surql`)**: Defined `idx_edge_in`, `idx_edge_out`, `idx_edge_out_ws`, and `idx_edge_in_ws` on `linked_to` table for sub-millisecond edge traversals.
- **Worktree Hierarchy Deduplication**: Pruned adopted worktrees from top-level listing in `/api/workspaces` while adding `?flat=true` parameter for flat backwards compatibility.
- **Robust Multi-Statement SurrealQL Parsing (`extract_sql_arrays`)**: Neutralized `LET` statement index offset fragility by dynamically filtering non-null result arrays.

## [0.2.1] - 2026-10-09

### Added
- **Ambient Live-Watch Auto-Enrollment**: Pre-invocation hooks dynamically detect active indexed workspaces and enroll them in background live-watch without user intervention.
- **Multi-Symbol Polyglot Imports (R-044)**: Discrete `IMPORTS` edges extracted across TS/JS named imports (`import { A, B }`), Python multi-imports (`from m import A, B`), and Rust grouped use statements (`use m::{A, B}`).
- **Two-Tier Subgraph Condenser Token Budgeting (R-028)**: Bounded symbol rendering guaranteeing $\ge 35\%$ of character budget is preserved for structural call/import traces (<1500 token SLA).
- **Ingestion Security Path Containment**: Added canonical path resolution and `allowed_browse_roots()` verification on `POST /api/ingest`, rejecting unauthorized paths with `403 Forbidden`.
- **Zero-Blackout Live Re-indexing**: Reordered live sync pipeline to generate vector embeddings before pruning old nodes, eliminating the 404 deletion window.

## [0.2.0] - 2026-10-09

### Added
- **Ephemeral Branch Fabric (EBF) for Git Worktrees (R-051 - R-058)**:
  - **O(1) Worktree Pointer Peeking (R-051)**: Detects `.git` worktree pointers in $O(1)$ without spawning subshells or executing `git` CLI, resolving parent repository path, branch name, and lineage.
  - **Enforced Auto-Watch Enrollment (R-052)**: `/api/ingest` automatically starts the live watch daemon (`watch: true` default), ensuring worktrees edited by coding agents are immediately kept fresh without manual watcher setup.
  - **Seed-Preserving Prior Inheritance (R-053)**: Worktree LPA galaxy clustering inherits community seeds from the parent workspace, preserving 99% cluster stability across branches.
  - **Dead Worktree Garbage Collection (R-054)**: `/api/workspaces` automatically sweeps and purges deleted worktrees from SurrealDB, preventing phantom zombie workspaces.
  - **Worktree Hierarchy & Lineage (R-055)**: `/api/workspaces` nests active worktrees under parent repositories (`worktrees: [...]`) while preserving backward-compatible flat workspace listing.
  - **Worktree Visualizer & Branch Navigation (R-056)**: Cosmograph React UI displays worktree indicators, branch tags, and nested worktree switchers in the workspace selector.
  - **Proactive Agent Hook & Remediation (R-057)**: Pre-invocation and pre-tool hooks detect unindexed worktrees and emit one-command self-healing ingest banners.
  - **Rebase Storm Surge Suppression (R-058)**: Idempotent watch registration and event debounce windows prevent CPU thrashing during multi-file git rebases.

## [0.1.1] - 2026-10-07

### Added
- **Full Polyglot AST Grammar (R-031)**:
  - Go: method declarations (`method_declaration`) with receiver `DECLARES` edges, structs and interfaces (`type_spec`), constants (`const_spec`), and imports (`import_spec`).
  - TypeScript/JavaScript: lexical arrow functions (`const x = () => ...`) and function expressions under `variable_declarator`.
  - TypeScript: interfaces (`interface_declaration`), type aliases (`type_alias_declaration`), and enums (`enum_declaration`).
  - Rust: `impl_item` blocks emitting `DECLARES` edges from target structs to methods, `enum_item`, and `trait_item`.
  - Python: coroutines (`async_function_definition`).
- **Relational Delta Sync Integrity (R-032)**: Single-file delta re-indexing only deletes outbound edges (`in IN $nodes`), preserving inbound caller edges to eliminate graph island dissolution.
- **Two-Tier Exact Symbol Retrieval (R-033)**: SurrealQL `ORDER BY (label = $name) DESC, label ASC LIMIT 20` prioritizes exact symbol matches over substrings.
- **Deterministic LPA Tie-Breaking (R-034)**: Tie-break equal community edge weights using minimum label ID for 100% deterministic galaxy assignments across runs.
- **Scoped Call-Edge Resolution (R-035)**: AST call edges resolve against local source file and directory before falling back to workspace, eliminating cross-package collisions.
- **Pre-Computed Galaxy Graph-RAG Retrieval (R-036)**: `GraphRagEngine::query` reads pre-computed `galaxy` table records directly instead of re-summarizing entire workspace in-memory.
- **Context-Enriched Embedding Payloads (R-037)**: Prepend structured header `[{language}] {kind} {label} in {file_path}\n{text}` before generating TEI vector embeddings.
- **Unified Record ID Normalization (R-050)**: Standardized SurrealDB ID sanitization via `clean_record_id`, stripping `node:`, `⟨...⟩`, and backticks across all storage/query paths, eliminating all 114 orphaned edges in SurrealDB.
- **Cross-File Receiver & Impl Target Scoping (R-045)**: Resolved 3-part source IDs (`ws:file:Struct`) locally in file, then workspace scope; falls back to empty relation array `[]` instead of creating phantom node records.
- **Cascade Edge Deletion on Node Pruning (R-046)**: Cascaded edge deletion across both inbound (`out IN $nodes`) and outbound (`in IN $nodes`) directions during file re-indexing/deletion, ensuring zero dangling graph pointers.
- **Stale File Pruning in Batch Ingestion (R-047)**: Pre-prune file records before batch insertion in `ingest_directory` to prevent duplicate zombie nodes when file contents shift.
- **Trait & Class Inheritance Relationship Extraction (R-048)**:
  - Rust: `impl Trait for Struct` emits `IMPLEMENTS` edges.
  - Python: class inheritance from `superclasses` emits `EXTENDS` edges.
  - TypeScript/JavaScript: `class_heritage` clauses emit `EXTENDS` and `IMPLEMENTS` edges.
- **Rust Macro Definition & Invocation Indexing (R-049)**: Tree-sitter `macro_definition` emits `kind: "macro"`; `macro_invocation` emits `CALLS` edge.
- **Universal Invocation Dispatch (R-040)**: Extracted Python `call`, Rust `method_call_expression`, and TS/JS `new_expression` as `CALLS` edges.
- **Go Interface Method AST Extraction (R-041)**: Extracted `method_spec` inside Go `interface_type` as `kind: "method"` with `DECLARES` edge from parent interface.
- **Direct Pre-Computed Galaxy Delivery (R-042)**: Served `/api/galaxies` directly from `galaxy` table with zero runtime graph deserialization overhead.
- **Localized Subgraph Neighborhood in Condenser (R-043)**: `/api/condense` queries localized 1-2 hop neighborhood in SurrealDB instead of whole-graph deserialization.
- **First-Class Import Structural Linking (R-044)**: Emitted `CONTAINS` edge from parent file to `import` node and `IMPORTS` edge to imported module/symbol.
- **Galaxy Subsystem Disambiguation (R-038)**: Automatically append primary cohesive symbol on directory name collisions to eliminate duplicate galaxy titles.

## [0.1.0] - 2026-09-24

### Fixed
- Fixed ingestion bottleneck on large datasets/dumps by pruning non-code directories (`solutions`, `data`, `logs`), applying 512KB file limit, and capping fallback blocks.
- Fixed SurrealDB v2 HNSW delete timeout during workspace re-indexing by temporarily detaching the vector index during bulk record purges.
- Fixed Axum synchronous ingestion cancellation by running ingestion pipelines in detached Tokio tasks.
- Fixed duplicate and uncanonical workspace listings in `/api/workspaces` by coalescing legacy `workspace` into `tool-scripts` and merging AST node/file counts.
- Filtered out empty, `default`, and `global` pseudo-workspaces from API output.
- Hardened React UI `loadWorkspaces` with client-side Set deduplication and filter checks.

### Added
- Initial tool scaffolding with full SDD specification
- Project genesis: 24 functional requirements, 14 acceptance criteria, 11 NFRs
- Research integration from Graphify, CodeGraph, Serena, ECC reference architectures
