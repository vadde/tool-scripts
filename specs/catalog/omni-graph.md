# Spec: Omni-Graph

> **Status**: In-Progress
> **Author**: @vadde
> **Created**: 2026-09-24
> **Last Updated**: 2026-09-24
> **Tool Path**: `tools/omni-graph/`

---

## 1. Overview

### 1.1 Problem Statement

AI coding agents — from Cursor to Claude Code to Antigravity — navigate codebases using primitive, token-wasteful operations: recursive `cat`, `grep`, and linear file-walking. These approaches:

- **Burn context windows**: A single "understand the codebase" cycle can consume 50K+ tokens of raw file content, leaving little room for reasoning.
- **Lack structural awareness**: Text search cannot distinguish a function _definition_ from a function _call_, or trace a 3-hop dependency chain from `handler → service → repository`.
- **Produce stale context**: Every time an agent restarts, it re-reads the same files from scratch with no persistent memory.
- **Cannot cluster semantics**: Agents cannot answer "which files are functionally related?" without exhaustively reading all of them.

The state-of-the-art (Graphify, CodeGraph, Serena) proves that deterministic AST parsing + vector embedding + graph storage eliminates these inefficiencies. But no existing tool delivers the full pipeline as a single, containerized, local-first system optimized for Apple Silicon.

### 1.2 Proposed Solution

**Omni-Graph** is a 100% local, multi-container semantic knowledge hub that models codebases as rich **Abstract Syntax Tree (AST) graphs** augmented with **384-dimensional vector embeddings**. It provides:

1. **Rust Orchestrator (Axum)**: High-performance ingestion engine using tree-sitter for deterministic, multi-language AST parsing. Pipes semantic blocks to a local embedding inference engine, stores structured nodes/edges in a multi-model database, and serves a graph analytics REST API.
2. **SurrealDB v2**: Multi-model database serving as both relational graph store (first-class `RELATE` edges) and vector similarity engine (HNSW index, 384-dim, cosine distance).
3. **HuggingFace TEI**: Runs `BAAI/bge-small-en-v1.5` locally for zero-cost, zero-latency embedding vectorization. ARM64-native with SIMD acceleration.
4. **React + TypeScript + Cosmograph UI**: Premium WebGL force-directed graph visualization. GPU-accelerated layout for 100K+ nodes. Semantic galaxies via community detection coloring.

All four services are orchestrated via Docker Compose on a dedicated network (`omni-net`), purpose-built for Apple Silicon ARM64 unified memory.

### 1.3 Target Audience

| Persona | Needs |
|---------|-------|
| **AI Coding Agent** | Query `/api/graph` for structural context instead of `cat`/`grep` — traces call chains, finds dependencies, discovers related code semantically |
| **Senior Engineer** | Visualize codebase topology, identify god-objects, trace cross-file dependency chains |
| **Tech Lead / Architect** | Understand system decomposition, community clustering, coupling metrics |
| **DevOps / Platform Engineer** | Monitor service health, manage the local container stack |

### 1.4 Success Criteria

- A single `docker compose up` launches the full stack on ARM64 macOS
- Tree-sitter parses a 1000-file polyglot codebase in < 60 seconds
- `/api/graph` returns a complete node/link JSON payload in < 500ms
- Cosmograph UI renders 10K+ nodes at 60fps+ with smooth force-directed animation
- Vector similarity search (`/api/search?q=...`) returns top-10 results in < 200ms
- Zero external network dependencies — fully air-gapped operation

---

## 2. Requirements

### 2.1 Functional Requirements

| ID | Requirement | Priority | Impl | Tested | Notes |
|----|------------|----------|------|--------|-------|
| R-001 | Docker Compose orchestrates 4 services (Rust app, SurrealDB, TEI, React UI) on a shared `omni-net` network | Must | ⬜ | ⬜ | ARM64-native images |
| R-002 | Rust orchestrator starts only after SurrealDB and TEI pass health checks | Must | ⬜ | ⬜ | `depends_on` with `condition: service_healthy` |
| R-003 | Tree-sitter parses source files into AST-derived semantic blocks (functions, structs, classes, imports, modules) | Must | ⬜ | ⬜ | Multi-language: Rust, Go, Python, TypeScript, JavaScript, Java, C/C++ |
| R-004 | Semantic blocks are sent to TEI (`BAAI/bge-small-en-v1.5`) for 384-dim vector embedding | Must | ⬜ | ⬜ | Batch API: `POST /embed` |
| R-005 | Nodes stored in SurrealDB with text, label, file_path, language, and 384-dim vector field | Must | ⬜ | ⬜ | Schema-full `node` table |
| R-006 | Directional graph edges (`linked_to`) track relationships: `CALLS`, `IMPORTS`, `CONTAINS`, `IMPLEMENTS`, `DEPENDS_ON` | Must | ⬜ | ⬜ | `RELATE node:a->linked_to->node:b SET type = 'CALLS'` |
| R-007 | HNSW vector similarity index on `node.embedding` with cosine distance metric | Must | ⬜ | ⬜ | `DEFINE INDEX ... HNSW DIMENSION 384 DIST COSINE TYPE F32` |
| R-008 | REST API: `GET /api/graph` returns full graph payload `{ nodes: [...], links: [...] }` | Must | ⬜ | ⬜ | JSON schema with id, text, label, community, source, target, type |
| R-009 | REST API: `GET /api/search?q=<query>&k=<n>` performs vector similarity search | Must | ⬜ | ⬜ | Embeds query via TEI, searches HNSW, returns top-k |
| R-010 | REST API: `POST /api/ingest` triggers ingestion of a target directory | Must | ⬜ | ⬜ | Accepts `{ "path": "/path/to/codebase" }` |
| R-011 | Ingestion tracks file staleness — skip files whose mtime hasn't changed since last parse | Should | ⬜ | ⬜ | Content-hash or mtime buffer |
| R-012 | Louvain/Leiden community detection assigns `community_id` to each node | Should | ⬜ | ⬜ | Used for galaxy clustering in UI |
| R-013 | React + TypeScript frontend with Cosmograph WebGL graph visualization | Must | ⬜ | ⬜ | `@cosmograph/react` |
| R-014 | UI: Dark-mode interface with GPU-accelerated force-directed layout | Must | ⬜ | ⬜ | WebGL canvas, 60fps+ |
| R-015 | UI: Community-based node coloring — functionally related nodes form visible "galaxies" | Should | ⬜ | ⬜ | Color palette mapped to `community_id` |
| R-016 | UI: Node hover reveals metadata panel (file path, language, text preview, edges) | Should | ⬜ | ⬜ | Tooltip or sidebar |
| R-017 | UI: Click node to highlight immediate neighbors and connected edges | Should | ⬜ | ⬜ | Path lineage trace |
| R-018 | UI: Search bar for semantic similarity search via `/api/search` | Should | ⬜ | ⬜ | Results highlight matching nodes |
| R-019 | SurrealDB schema initialization runs automatically on first startup | Must | ⬜ | ⬜ | Migration script or Rust init routine |
| R-020 | Graceful handling of orphaned graph entities (nodes with no edges) | Should | ⬜ | ⬜ | UI renders them; API doesn't crash |
| R-021 | File watcher daemon for live re-indexing on file changes (FSEvents on macOS) | Could | ⬜ | ⬜ | Inspired by CodeGraph's debounce loop |
| R-022 | REST API: `GET /api/health` returns aggregated health of all services | Must | ⬜ | ⬜ | `{ rust: "ok", surrealdb: "ok", tei: "ok" }` |
| R-023 | REST API: `GET /api/stats` returns ingestion statistics | Should | ⬜ | ⬜ | Total nodes, edges, files parsed, languages |
| R-024 | Cross-container endpoints use Docker DNS hostnames (`graph-db`, `embedding-engine`) — no `localhost` | Must | ⬜ | ⬜ | Decoupled networking |
| R-025 | Agent Enforcement Layer: Deterministic lifecycle hooks (`PreToolUse`, `PreInvocation`, `Stop`) in `hooks.json` to prevent blind `grep`/`cat` context burning and enforce graph-first exploration | Must | ⬜ | ⬜ | Hard runtime enforcement inspired by ECC & Antigravity hooks |
| R-026 | Omni-Graph Native Agent Interface: First-class MCP server & Antigravity Skill (`skills/omni-graph/`) providing structured tools (`search_symbols`, `get_call_chain`, `query_subgraph`, `find_dependencies`) | Must | ⬜ | ⬜ | Symbolic agent tooling inspired by Serena |
| R-027 | Automated Agent Setup & Onboarding CLI: Interactive bootstrap command (`make setup-agent` / `omni setup`) that configures workspace and global agent rules, skills, hooks, and MCP servers | Must | ⬜ | ⬜ | Validates container connectivity & sets up `.agents/` |
| R-028 | Subgraph Context Condenser: Condenses multi-hop call traces and dependency chains into token-efficient (<1500 tokens) AST subgraphs for agent prompts | Must | ⬜ | ⬜ | Replaces 50K-token raw file dumps with high-fidelity graph slices |

**Priority levels**: Must (required for MVP), Should (important), Could (nice-to-have)

**Status legend**: ⬜ Not started · 🔨 In progress · ✅ Implemented · 🧪 Tested & verified · ❌ Blocked

### 2.2 Non-Functional Requirements

| ID | Requirement | Metric | Impl | Tested |
|----|------------|--------|------|--------|
| NF-001 | Performance — Ingestion | Parse 1000 files in < 60 seconds | ⬜ | ⬜ |
| NF-002 | Performance — Graph API | `/api/graph` response in < 500ms for 10K nodes | ⬜ | ⬜ |
| NF-003 | Performance — Vector Search | Similarity search returns in < 200ms | ⬜ | ⬜ |
| NF-004 | Performance — UI Render | Cosmograph renders 10K+ nodes at 60fps | ⬜ | ⬜ |
| NF-005 | Portability | Runs on macOS ARM64 (Apple Silicon) via Docker Compose | ⬜ | ⬜ |
| NF-006 | Isolation | 100% local, zero external network dependencies (air-gapped) | ⬜ | ⬜ |
| NF-007 | Reliability | Graceful degradation if TEI is slow — queue embeddings, don't block | ⬜ | ⬜ |
| NF-008 | Memory | Full stack runs within 8GB RAM (suitable for M-series base configs) | ⬜ | ⬜ |
| NF-009 | Startup | `docker compose up` → all services healthy in < 90 seconds | ⬜ | ⬜ |
| NF-010 | Data Safety | SurrealDB data persisted to Docker volume — survives container restarts | ⬜ | ⬜ |
| NF-011 | Security | All services bind to internal Docker network only; UI exposed on `localhost:3000` | ⬜ | ⬜ |

---

## 3. Interface Contract

### 3.1 CLI Interface

```
Usage: docker compose up [-d]          # Start the full stack
       docker compose down             # Stop and remove containers
       docker compose logs -f rust-app # Tail Rust orchestrator logs

# Trigger ingestion via API:
curl -X POST http://localhost:8080/api/ingest \
  -H "Content-Type: application/json" \
  -d '{"path": "/workspace/my-project"}'
```

### 3.2 REST API (Rust Orchestrator — port 8080)

#### `GET /api/health`

Aggregated health check for all services.

**Response:**
```json
{
  "status": "healthy",
  "services": {
    "rust_app": { "status": "ok", "version": "0.1.0" },
    "surrealdb": { "status": "ok", "latency_ms": 2 },
    "tei": { "status": "ok", "model": "BAAI/bge-small-en-v1.5", "latency_ms": 5 }
  }
}
```

#### `POST /api/ingest`

Triggers codebase ingestion.

**Request:**
```json
{
  "path": "/workspace/my-project",
  "languages": ["rust", "go", "python", "typescript"],
  "exclude_patterns": ["**/node_modules/**", "**/target/**", "**/.git/**"]
}
```

**Response:**
```json
{
  "status": "accepted",
  "job_id": "ing-20260924-001",
  "files_discovered": 342,
  "estimated_duration_ms": 15000
}
```

#### `GET /api/graph`

Returns the full graph topology for visualization.

**Response:**
```json
{
  "nodes": [
    {
      "id": "node:fn_main_rs_42",
      "text": "fn main() { ... }",
      "label": "main",
      "kind": "function",
      "file_path": "src/main.rs",
      "language": "rust",
      "community": 3
    }
  ],
  "links": [
    {
      "id": "linked_to:edge_001",
      "source": "node:fn_main_rs_42",
      "target": "node:fn_init_db_rs_10",
      "type": "CALLS"
    }
  ],
  "stats": {
    "total_nodes": 1842,
    "total_links": 5210,
    "communities": 12,
    "languages": ["rust", "typescript"]
  }
}
```

#### `GET /api/search?q=<query>&k=<n>`

Semantic vector similarity search.

**Request Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `q` | string | Yes | Natural language query |
| `k` | int | No | Number of results (default: 10) |

**Response:**
```json
{
  "query": "database connection pool",
  "results": [
    {
      "id": "node:fn_init_pool_rs_15",
      "text": "pub async fn init_pool(config: &DbConfig) -> Pool { ... }",
      "label": "init_pool",
      "kind": "function",
      "file_path": "src/db.rs",
      "language": "rust",
      "similarity": 0.9234,
      "community": 2
    }
  ],
  "total_results": 10,
  "search_latency_ms": 45
}
```

#### `GET /api/stats`

Ingestion and graph statistics.

**Response:**
```json
{
  "files_indexed": 342,
  "total_nodes": 1842,
  "total_edges": 5210,
  "languages": { "rust": 120, "typescript": 180, "python": 42 },
  "last_ingestion": "2026-09-24T14:00:00Z",
  "communities_detected": 12
}
```

### 3.3 Internal Service Endpoints

| Service | Internal Hostname | Port (Internal) | Port (Host) | Purpose |
|---------|-------------------|-----------------|-------------|---------|
| Rust Orchestrator | `rust-app` | 8080 | 8080 | Graph API, ingestion engine |
| SurrealDB | `graph-db` | 8000 | 8000 | Multi-model database + Surrealist console |
| HuggingFace TEI | `embedding-engine` | 80 | 8081 | Embedding inference |
| React UI | `graph-ui` | 3000 | 3000 | WebGL visualization dashboard |

### 3.4 Error Codes (HTTP)

| Code | Meaning |
|------|---------|
| 200 | Success |
| 202 | Accepted (async ingestion started) |
| 400 | Invalid request (bad path, missing params) |
| 404 | Resource not found |
| 500 | Internal error (DB connection, TEI timeout) |
| 503 | Service unavailable (dependency not ready) |

---

## 4. Constraints

### 4.1 Technical Constraints

- Must run entirely on ARM64 (Apple Silicon) via Docker Desktop
- Rust orchestrator compiled for `aarch64-unknown-linux-musl` inside multi-stage Docker build
- SurrealDB v2.x with native HNSW vector indexing (NOT MTREE — MTREE is deprecated/experimental)
- TEI container: `ghcr.io/huggingface/text-embeddings-inference:cpu-arm64-1.9` (CPU-only in Docker; Metal not available in containers)
- Embedding model: `BAAI/bge-small-en-v1.5` (384 dimensions, BERT architecture)
- React UI built with Vite, bundled as static assets served by Nginx in production container
- Tree-sitter grammars bundled in the Rust binary — no runtime grammar downloads

### 4.2 Security Constraints

- All services on internal `omni-net` Docker bridge — not exposed to host network
- Only `localhost` port bindings for host-accessible services (8080, 8000, 3000, 8081)
- SurrealDB credentials set via Docker secrets or environment variables (not hardcoded)
- Ingestion API accepts only absolute paths within bind-mounted volumes
- No telemetry, no analytics, no phone-home

### 4.3 Performance Constraints

- Ingestion: < 60 seconds for 1000 files
- Graph API: < 500ms for 10K-node payload
- Vector search: < 200ms for top-10 similarity results
- UI: 60fps+ at 10K nodes (WebGL/GPU-accelerated)
- Full stack memory: < 8GB RAM
- Cold start: < 90 seconds from `docker compose up` to all healthy

---

## 5. Dependencies

### 5.1 External Dependencies — Rust Orchestrator

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `axum` | 0.7.x | HTTP framework |
| `serde` / `serde_json` | 1.x | Serialization |
| `reqwest` | 0.12.x | HTTP client (TEI calls) |
| `surrealdb` | 2.x | SurrealDB native driver |
| `tree-sitter` | 0.24.x | AST parsing runtime |
| `tree-sitter-rust` | latest | Rust grammar |
| `tree-sitter-go` | latest | Go grammar |
| `tree-sitter-python` | latest | Python grammar |
| `tree-sitter-typescript` | latest | TypeScript grammar |
| `tree-sitter-javascript` | latest | JavaScript grammar |
| `tree-sitter-java` | latest | Java grammar |
| `tree-sitter-c` | latest | C grammar |
| `tree-sitter-cpp` | latest | C++ grammar |
| `tower-http` | 0.5.x | CORS middleware |
| `tracing` / `tracing-subscriber` | latest | Structured logging |
| `walkdir` | 2.x | Recursive directory traversal |

### 5.2 External Dependencies — React UI

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | 18.x | UI framework |
| `react-dom` | 18.x | DOM rendering |
| `typescript` | 5.x | Type safety |
| `@cosmograph/react` | latest | WebGL graph visualization |
| `@cosmograph/cosmograph` | latest | Core graph engine |
| `vite` | 5.x | Build tool |

### 5.3 Infrastructure Dependencies

| Service | Image | Purpose |
|---------|-------|---------|
| SurrealDB | `surrealdb/surrealdb:v2` | Multi-model database |
| HuggingFace TEI | `ghcr.io/huggingface/text-embeddings-inference:cpu-arm64-1.9` | Local embedding inference |
| Docker Desktop | 4.x+ | Container runtime (ARM64 native) |

### 5.4 Internal Dependencies

| Tool | Purpose |
|------|---------|
| None | Standalone tool |

---

## 6. Acceptance Criteria

Tests that MUST pass for this spec to be considered satisfied:

| AC ID | Criteria | Traces To |
|-------|----------|-----------|
| AC-001 | Given `docker compose up`, when all containers start, then all 4 services reach healthy status within 90 seconds | R-001, R-002, NF-009 |
| AC-002 | Given a Rust source file, when ingested, then tree-sitter extracts functions, structs, and impls as separate nodes | R-003 |
| AC-003 | Given extracted text blocks, when sent to TEI, then 384-dim float32 vectors are returned and stored | R-004, R-005 |
| AC-004 | Given parsed AST with function calls, when edges are created, then `linked_to` relations with `type: CALLS` exist in SurrealDB | R-006 |
| AC-005 | Given 10K nodes with embeddings, when `/api/search?q=database pool` is called, then top-10 semantically similar results return in < 200ms | R-007, R-009, NF-003 |
| AC-006 | Given `/api/graph` is called, then response matches `{ nodes: [...], links: [...] }` schema with all required fields | R-008 |
| AC-007 | Given the React UI loads, then Cosmograph renders the graph with dark theme, force-directed layout, and community coloring | R-013, R-014, R-015 |
| AC-008 | Given a node is hovered in the UI, then its metadata (file, language, text) is displayed | R-016 |
| AC-009 | Given `POST /api/ingest` with a valid path, then files are parsed, embedded, and stored — response returns job status | R-010 |
| AC-010 | Given a re-ingestion of the same directory, then unchanged files are skipped | R-011 |
| AC-011 | Given Rust container references SurrealDB, then it uses `http://graph-db:8000` not `localhost` | R-024 |
| AC-012 | Given `/api/health`, then all three downstream services are probed and status reported | R-022 |
| AC-013 | Given orphaned nodes (no edges), then UI renders them without crash and API returns them normally | R-020 |
| AC-014 | Given the SurrealDB schema definition, then `node` table has HNSW index on `embedding` field with COSINE distance, 384 dimensions | R-005, R-007 |
| AC-015 | Given an agent executing arbitrary `grep` or multi-file `cat`, when `PreToolUse` hook fires, then the command is intercepted or augmented to enforce Omni-Graph query endpoints | R-025 |
| AC-016 | Given Omni-Graph MCP server or Antigravity Skill, when an agent invokes `search_symbols` or `get_call_chain`, then structured AST results return in <100ms | R-026 |
| AC-017 | Given `make setup-agent` execution, then `.agents/rules/`, `.agents/skills/omni-graph/`, and `.agents/hooks.json` are installed and validated against local services | R-027 |
| AC-018 | Given a cross-file call chain query, when condensed, then subgraph returns <=1500 tokens of high-density semantic context rather than full raw files | R-028 |

---

## 7. Design Notes

### 7.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Docker Network: omni-net                        │
│                                                                     │
│  ┌──────────────────────┐     ┌──────────────────────────────┐     │
│  │   REACT UI           │     │   RUST ORCHESTRATOR          │     │
│  │   (graph-ui:3000)    │────▶│   (rust-app:8080)            │     │
│  │   Cosmograph WebGL   │     │   Axum + tree-sitter         │     │
│  │   Vite + TypeScript  │     │   Ingestion + Graph API      │     │
│  └──────────────────────┘     └──────────┬───────────────────┘     │
│                                          │                          │
│                         ┌────────────────┼────────────────┐        │
│                         │                │                │        │
│                         ▼                ▼                │        │
│           ┌─────────────────┐  ┌─────────────────────┐    │        │
│           │   SURREALDB     │  │   HUGGINGFACE TEI   │    │        │
│           │  (graph-db:8000)│  │ (embedding-engine:80)│    │        │
│           │   v2 + HNSW     │  │  bge-small-en-v1.5  │    │        │
│           │   Graph + Vector│  │  384-dim ARM64/SIMD  │    │        │
│           └─────────────────┘  └─────────────────────┘    │        │
│                                                           │        │
└─────────────────────────────────────────────────────────────────────┘

Host Ports: 3000 (UI) | 8080 (API) | 8000 (SurrealDB/Surrealist) | 8081 (TEI)
```

### 7.2 Ingestion Pipeline

```
Source Files → tree-sitter AST Parse → Semantic Blocks → TEI Embed (384-dim)
     │                                       │                    │
     │                                       ▼                    ▼
     │                              SurrealDB: CREATE node    HNSW Index
     │                                       │
     ▼                                       ▼
 AST Edge Extraction ──────────▶ SurrealDB: RELATE node->linked_to->node
 (calls, imports, contains)       SET type = 'CALLS' | 'IMPORTS' | ...
```

### 7.3 Relationship Types (Edge Taxonomy)

| Type | Description | Detection Method |
|------|-------------|------------------|
| `CALLS` | Function/method invocation | AST: `call_expression` nodes |
| `IMPORTS` | Module/package import | AST: `import_statement`, `use_declaration` |
| `CONTAINS` | Parent scope contains child | AST: `function_item` inside `impl_item` |
| `IMPLEMENTS` | Implements trait/interface | AST: `impl_item` with trait path |
| `DEPENDS_ON` | File-level dependency | Inferred from imports |
| `TYPE_REF` | Type reference/annotation | AST: type annotations, generics |

Categorization follows Graphify's `[EXTRACTED]` vs `[INFERRED]` taxonomy:
- **EXTRACTED**: Directly observed in AST (CALLS, IMPORTS, CONTAINS, IMPLEMENTS)
- **INFERRED**: Resolved by cross-file analysis (DEPENDS_ON, TYPE_REF)

### 7.4 SurrealQL Schema

```sql
-- Schema-full node table
DEFINE TABLE node SCHEMAFULL;
DEFINE FIELD text       ON TABLE node TYPE string;
DEFINE FIELD label      ON TABLE node TYPE string;
DEFINE FIELD kind       ON TABLE node TYPE string;  -- function, struct, class, import, module
DEFINE FIELD file_path  ON TABLE node TYPE string;
DEFINE FIELD language   ON TABLE node TYPE string;
DEFINE FIELD line_start ON TABLE node TYPE int;
DEFINE FIELD line_end   ON TABLE node TYPE int;
DEFINE FIELD embedding  ON TABLE node TYPE array<float> ASSERT array::len($value) = 384;
DEFINE FIELD community  ON TABLE node TYPE option<int>;
DEFINE FIELD file_hash  ON TABLE node TYPE option<string>;
DEFINE FIELD created_at ON TABLE node TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE node TYPE datetime DEFAULT time::now();

-- HNSW vector similarity index
DEFINE INDEX idx_node_embedding ON TABLE node FIELDS embedding
  HNSW DIMENSION 384 DIST COSINE TYPE F32;

-- Auxiliary indexes
DEFINE INDEX idx_node_file     ON TABLE node FIELDS file_path;
DEFINE INDEX idx_node_language ON TABLE node FIELDS language;
DEFINE INDEX idx_node_kind     ON TABLE node FIELDS kind;

-- Relation table
DEFINE TABLE linked_to TYPE RELATION;
DEFINE FIELD type       ON TABLE linked_to TYPE string;
DEFINE FIELD category   ON TABLE linked_to TYPE string;  -- EXTRACTED or INFERRED
DEFINE FIELD created_at ON TABLE linked_to TYPE datetime DEFAULT time::now();
```

### 7.5 UI Design Philosophy

- **Dark theme default**: Deep slate/navy backgrounds (#0a0e17, #111827)
- **WebGL canvas**: Cosmograph GPU-accelerated force-directed layout
- **Community galaxies**: Distinct color clusters using a 12-color curated palette
- **Glassmorphic panels**: Semi-transparent metadata sidebar with `backdrop-filter: blur()`
- **Typography**: Inter or JetBrains Mono (code blocks)
- **Micro-animations**: Node hover glow, edge path highlights, zoom transitions
- **Responsive**: Works on large monitors (2560px+) and standard displays (1440px)

### 7.6 Inspirations & Prior Art

| Project | Key Technique Borrowed |
|---------|----------------------|
| **Graphify** | `[EXTRACTED]` vs `[INFERRED]` edge categorization; Leiden community detection; zero-LLM AST parsing |
| **CodeGraph** | File watcher daemon with FSEvents + debounce; content-hash staleness detection |
| **Serena** | Agent-first tool design; symbolic query interface over raw text; LSP semantic operations |
| **ECC Agent Harness** | Content-hash caching; PreToolUse guardrails for token throughput optimization |
| **Microsoft GraphRAG** | Hierarchical community detection & macroscopic summary aggregation |
| **RAGFlow** | Deep document parsing & structural chunking over raw lexical chunking |

### 7.7 Multi-Layered Agent Enforcement Architecture

The central flaw of standard agent architectures is the **Cognitive Drift Gap**: even when instructed in `AGENTS.md` to avoid recursive `grep`/`cat`, an LLM agent under complex task load defaults to primitive text searching because `run_command("grep ...")` is deeply embedded in foundational pre-training.

To ensure deterministic compliance, Omni-Graph implements a **4-tier enforcement shield**:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        AGENT EXECUTION LOOP                            │
│                                                                        │
│  [Tier 1: Cognitive Steering]                                          │
│  .agents/rules/08-omni-graph-enforcement.md                            │
│  Mandates semantic AST lookup before any raw file inspection           │
│                                                                        │
│  [Tier 2: Pre-Invocation Context Injection]                            │
│  hooks.json: PreInvocation Hook                                        │
│  Injects dynamic ephemeral hints with active graph endpoints & stats   │
│                                                                        │
│  [Tier 3: Pre-Tool Deterministic Guardrail]                            │
│  hooks.json: PreToolUse Hook (matcher: run_command|grep_search)        │
│  Detects brute-force grep/cat; rewrites or hard-blocks with guidance   │
│                                                                        │
│  [Tier 4: Native Symbolic Tooling]                                     │
│  Antigravity Skill: skills/omni-graph/ + MCP Server                    │
│  Provides native structured operations:                                │
│  - trace_call_chain(symbol)                                            │
│  - semantic_search(query, k)                                           │
│  - get_subgraph(node_id, depth)                                        │
│  - find_dependencies(file_path)                                        │
└────────────────────────────────────────────────────────────────────────┘
```

### 7.8 Antigravity Hooks, Skills & Rules Specification

#### 1. Lifecycle Hooks (`hooks.json`)

Located in `.agents/hooks.json` (workspace) or configured globally:

```json
{
  "omni-graph-guardrail": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command|grep_search",
        "hooks": [
          {
            "type": "command",
            "command": "./tools/omni-graph/scripts/hook_pre_tool.sh",
            "timeout": 5
          }
        ]
      }
    ],
    "PreInvocation": [
      {
        "type": "command",
        "command": "./tools/omni-graph/scripts/hook_pre_invocation.sh",
        "timeout": 5
      }
    ],
    "Stop": [
      {
        "type": "command",
        "command": "./tools/omni-graph/scripts/hook_stop.sh",
        "timeout": 5
      }
    ]
  }
}
```

- **`hook_pre_tool.sh`**: Inspects tool args. If `grep -rn` or recursive `find`/`cat` is detected across >10 files, outputs `{"decision": "deny", "reason": "Blind whole-repo grep blocked. Use Omni-Graph semantic search (/api/search) or symbol lookup (/api/graph) instead."}` or rewrites command with `overwrite`.
- **`hook_pre_invocation.sh`**: Injects `{"injectSteps": [{"ephemeralMessage": "Omni-Graph is ACTIVE. Query http://localhost:8080/api/graph or run `omni search <q>` for codebase structure."}]}`.
- **`hook_stop.sh`**: Verifies that any reported structural architecture matches indexed nodes.

#### 2. Antigravity Skill (`skills/omni-graph/SKILL.md`)

Provides concise step-by-step workflows and scripts for:
- Querying symbol definitions, callers, and callees without token waste.
- Fetching condensed AST subgraphs (<1500 tokens).
- Checking Docker cluster health (`omni status`).

#### 3. Automated Setup Command (`make setup-agent`)

Provides an automated setup utility:
```bash
make setup-agent TARGET=workspace   # Installs to .agents/ (repo-level)
make setup-agent TARGET=global      # Installs to ~/.gemini/config (machine-wide)
```
- Validates Docker daemon and Omni-Graph container health.
- Configures `rules/`, `skills/omni-graph/`, and `hooks.json`.
- Tests roundtrip connectivity from agent tool calling to SurrealDB and TEI.

---

## 8. Open Questions

- [ ] Should community detection (Leiden) run in Rust or as a SurrealQL function?
- [ ] Should the file watcher (R-021) be a separate sidecar container or embedded in the Rust app?
- [ ] Should we support incremental graph updates (patch) or full re-ingestion only?
- [ ] What is the maximum supported codebase size (files/LOC) before memory pressure?
- [ ] Should the UI support 3D mode (`spaceDimensions: 3`) via Cosmograph?

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-09-24 | @vadde | Initial draft — full specification with research from Graphify, CodeGraph, Serena, ECC |
