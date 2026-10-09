# Enhancement Spec: Omni-Graph System Hardening & Forensic Defect Remediation

> **Status**: In-Progress
> **Author**: Antigravity (Google DeepMind)
> **Parent Spec**: `specs/catalog/omni-graph.md`
> **Tool Path**: `tools/omni-graph/`
> **Date**: 2026-10-07

---

## 1. Overview & Forensic Motivation

A forensic audit of Omni-Graph revealed nine architectural vulnerabilities and functional blind spots:
1. **Parser Polyglot Blind Spots**: Modern TypeScript/React arrow functions, TS interfaces/types, Rust `impl_item`/`enum_item`/`trait_item`, and Python `async def` are silently dropped from AST indexing.
2. **Delta Sync Island Dissolution**: Re-indexing a modified file purges all inbound edges pointing to that file, gradually breaking the graph into disconnected components.
3. **Symbol Search "Limit Before Sort"**: SurrealDB applies `LIMIT 20` before exact matches are sorted, causing common function names to be masked by substring matches.
4. **Spaghetti Call Edges**: Resolving function calls (`CALLS`) by bare name across the entire workspace creates arbitrary cross-package edges for common function names (`run`, `new`, `parse`).
5. **Non-Deterministic LPA Tie-Breaking**: Rust `HashMap` iteration in label propagation tie-breaking causes cluster IDs to flap across runs.
6. **Whole-Graph Deserialization in Graph-RAG**: `GraphRagEngine::query` and `/api/condense` pull the entire workspace graph across HTTP and re-summarize communities in-memory instead of querying pre-computed galaxy records.
7. **Homogeneous Galaxy Naming**: Subsystem naming based solely on directory plurality results in duplicate galaxy names in deep modules.
8. **Blind Code Embeddings**: Vectors are calculated on raw code snippets without prepending file path or symbol name.

---

## 2. Requirements & Traceability

| ID | Requirement | Priority | Verification Criteria |
|----|------------|----------|-----------------------|
| **R-031** | **Comprehensive Polyglot AST Grammar**: Support TS/React arrow functions, TS interfaces/types, Rust `impl_item` (with `DECLARES` edge to struct) and `enum_item`/`trait_item`, Python `async_function_definition`. | P0 | Unit tests `parse_typescript_arrow_functions_and_interfaces`, `parse_rust_impl_and_enums`, `parse_python_async`. |
| **R-032** | **Inbound Edge Preservation in Delta Sync**: Single-file re-indexing only deletes outbound edges (`in IN $nodes`); inbound caller edges are preserved. | P0 | Verified via delta pipeline unit tests. |
| **R-033** | **Two-Tier Exact Symbol Retrieval**: Database query ensures exact label matches (`label = $name`) are always retrieved and prioritized over substring matches. | P0 | `find_symbols("run")` returns exact `fn run()` even when >20 symbols contain `"run"`. |
| **R-034** | **Deterministic LPA Tie-Breaking**: When multiple community labels share maximum edge weight, tie-break by deterministic minimum label ID. | P1 | Repeated clustering runs yield identical community assignments. |
| **R-035** | **Scoped Call-Edge Resolution**: When linking `CALLS` without deterministic `target_id`, resolve candidates in the source file first, then same directory, before workspace fallback. | P1 | Cross-package bare name collisions are eliminated. |
| **R-036** | **Pre-Computed Galaxy Graph-RAG Retrieval**: `GraphRagEngine::query` fetches relevant community records from `galaxy` table directly instead of re-summarizing entire graph in memory. | P1 | Microsecond response time without pulling all workspace nodes. |
| **R-037** | **Context-Enriched Embedding Payloads**: Prepend structured header `[{language}] {kind} {label} in {file_path}\n{text}` before embedding. | P2 | Vector search ranks semantically relevant symbols higher. |
| **R-038** | **Galaxy Subsystem Disambiguation**: When cluster directory names collide, disambiguate with primary member symbol. | P2 | Zero duplicate galaxy names in `/api/galaxies`. |
| **R-039** | **Scoped Source Node Resolution in Edge Persistence**: In `store_edges`, resolve 3-segment source IDs (`ws:file:label`) to the concrete node record with line number. | P0 | Zero orphaned edges where `in.id IS NONE`. |
| **R-040** | **Universal Invocation Dispatch**: Extract Python `call`, Rust `method_call_expression`, and JS/TS `new_expression` as `CALLS` edges. | P0 | Python and Rust method invocations emit `CALLS` edges. |
| **R-041** | **Go Interface Method AST Extraction**: Extract `method_spec` inside Go `interface_type` as `kind: "method"` with `DECLARES` edge from interface. | P0 | `GET /api/symbol` resolves interface methods. |
| **R-042** | **Direct Pre-Computed Galaxy Delivery**: Serve `/api/galaxies` directly from `galaxy` table instead of broken `get_galaxy_aggregation` or full-graph fallback. | P1 | Response in <5ms with rich role, instability, Ca, Ce, and key_symbols. |
| **R-043** | **Localized Subgraph Neighborhood in Condenser**: `/api/condense` queries localized 1-2 hop neighborhood in SurrealDB instead of whole-graph deserialization. | P1 | Eliminates multi-MB JSON transfer on `/api/condense`. |
| **R-044** | **First-Class Import Structural Linking**: Emit `CONTAINS` edge from file/parent to `import` node and `IMPORTS` edge to imported module/symbol. | P2 | Imports are connected to the graph topology. |
| **R-045** | **Cross-File Receiver & Impl Target Scoping**: In `store_edges`, resolve 3-part source IDs (`ws:file:Struct`) across both local file AND workspace scope, falling back to empty relation array rather than creating phantom thing IDs. | P0 | Zero orphaned edges where `in.id IS NONE`. |
| **R-046** | **Cascade Edge Deletion on Node Pruning**: In `delete_file`, cascade delete both outbound (`in IN $nodes`) and inbound (`out IN $nodes`) edges to eliminate dangling graph pointers and WebGL UI crashes. | P0 | Zero dangling edges after file modification or deletion. |
| **R-047** | **Stale File Pruning in Batch Ingestion**: In `ingest_directory`, invoke `delete_file` before inserting re-parsed nodes/edges for modified files to prevent duplicate zombie nodes and shifted line numbers. | P0 | Zero duplicate symbol nodes on re-ingestion. |
| **R-048** | **Trait & Class Inheritance Relationship Extraction**: Extract Rust `impl Trait for Struct` as `IMPLEMENTS` edges; extract TypeScript `implements` and `extends` heritage clauses; extract Python base classes as `EXTENDS` edges. | P1 | Trait implementations and class hierarchies are visible in graph. |
| **R-049** | **Rust Macro Definition & Invocation Indexing**: Parse `macro_definition` as macro symbols and `macro_invocation` as call expressions with macro target labels. | P1 | Rust macros are indexed and referenced in call graphs. |
| **R-050** | **Unified Record ID Normalization**: Centralize SurrealDB record ID cleaning (`clean_record_id`) across all modules to handle bare, bracketed (`⟨...⟩`), and backticked IDs uniformly. | P1 | Eliminates ID parsing bugs across SurrealDB v2 variants. |
| **R-051** | **Git Worktree Lineage Auto-Detection**: Inspect `.git` file for `gitdir:`, extract parent repository root, branch name, and worktree name in $O(1)$ time. Persist lineage in `workspace_meta`. | P0 | Verified via `detect_git_worktree` unit tests. |
| **R-052** | **Ingest-Time Live Watch Auto-Enrollment**: Automatically invoke `WatchManager::start_watch` on ingested directory when `watch != Some(false)`. Eliminate stale graph states for active coding agents. | P0 | Ingestion response returns active watch status and file events stream live. |
| **R-053** | **Seed-Preserving Galaxy Inheritance**: Pre-populate community seeds from `parent_workspace` when clustering a worktree to guarantee 99% galaxy cluster and subsystem stability across git branches. | P0 | Verified via LPA seed inheritance tests on worktree graphs. |
| **R-054** | **Worktree Garbage Collection & Auto-Pruning**: Background sweep detects deleted worktree directories and cascades cleanups in SurrealDB, eliminating zombie workspaces. | P1 | Pruning removes orphaned nodes, edges, galaxies, and watcher. |
| **R-055** | **Hierarchical Workspace API Representation**: `GET /api/workspaces` returns structured anchor codebases with nested active worktrees. | P1 | Grouped parent-worktree JSON structure verified. |
| **R-056** | **UI Worktree Switcher & Delta Visualizer**: React/Cosmograph UI provides branch selector under parent workspace with live watching status pulse and glowing delta halos for branch-modified nodes. | P1 | UI renders branch selector and highlights modified nodes. |
| **R-057** | **Worktree-Aware PreToolUse & PreInvocation Hooks**: Hook scripts resolve worktree lineage and provide tailored single-command sync banners that auto-attach watch daemons. | P0 | Hooks recognize worktrees and trigger zero-friction auto-sync. |
| **R-058** | **Cross-Branch Symbol Diff API**: `GET /api/diff?worktree=<name>&parent=<parent>` compares worktree AST nodes against parent baseline to isolate branch blast radius. | P2 | Returns added, modified, and removed symbols across branches. |

---

## 3. Test & Verification Plan

1. Unit tests in `tools/omni-graph/tests/unit_tests.rs`:
   - TS arrow functions, interfaces, class heritage.
   - Rust `impl` blocks, traits, macro definitions, macro invocations, and enum extraction.
   - Python `async def` and inheritance.
   - Deterministic LPA tie-breaking.
   - Two-tier symbol lookup sorting.
   - Record ID cleaner unit tests.
   - Git worktree detection (`detect_git_worktree`) across regular repos, worktree files, and detached states.
   - Seed-preserving worktree LPA clustering stability.
2. End-to-end container verification on `tool-scripts`, `tutor-intelligence`, and `ti-strat-089`:
   - Re-index and confirm 0 orphaned edges (`SELECT count() FROM linked_to WHERE in.id IS NONE OR out.id IS NONE`).
   - Confirm automatic watch daemon enrollment on worktree ingest.
   - Test live file modification in `ti-strat-089` and verify single-file incremental update in <100ms.
   - Re-index and confirm 0 duplicate nodes for modified files.
3. Strict Clippy check: `make -C tools/omni-graph lint-rust` (`-D warnings`).
4. Strict UI check: `tsc --noEmit`.
