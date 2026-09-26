-- ==============================================================================
-- Omni-Graph SurrealDB Schema (SurrealDB v2.x)
-- Defines schema-full node table, HNSW vector similarity index (384-dim COSINE),
-- and first-class typed relationship edges.
-- ==============================================================================

-- Namespace & Database setup
DEFINE NAMESPACE IF NOT EXISTS omni;
USE NS omni;
DEFINE DATABASE IF NOT EXISTS graph;
USE DB graph;

-- ─── 1. Schema-full Node Table ────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS node SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS workspace  ON TABLE node TYPE string DEFAULT 'default';
DEFINE FIELD IF NOT EXISTS text       ON TABLE node TYPE string;
DEFINE FIELD IF NOT EXISTS label      ON TABLE node TYPE string;
DEFINE FIELD IF NOT EXISTS kind       ON TABLE node TYPE string;  -- function, struct, class, trait, interface, import, module
DEFINE FIELD IF NOT EXISTS file_path  ON TABLE node TYPE string;
DEFINE FIELD IF NOT EXISTS language   ON TABLE node TYPE string;
DEFINE FIELD IF NOT EXISTS line_start ON TABLE node TYPE int;
DEFINE FIELD IF NOT EXISTS line_end   ON TABLE node TYPE int;
DEFINE FIELD IF NOT EXISTS embedding  ON TABLE node TYPE array<float> ASSERT array::len($value) = 384;
DEFINE FIELD IF NOT EXISTS community  ON TABLE node TYPE option<int>;
DEFINE FIELD IF NOT EXISTS file_hash  ON TABLE node TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE node TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE node TYPE datetime DEFAULT time::now();

-- ─── 2. HNSW Vector Similarity Index ──────────────────────────────────────────
-- Production-grade Approximate Nearest Neighbor (ANN) index for 384-dim embeddings
DEFINE INDEX IF NOT EXISTS idx_node_embedding ON TABLE node FIELDS embedding
  HNSW DIMENSION 384 DIST COSINE TYPE F32;

-- ─── 3. Auxiliary Secondary Indexes ───────────────────────────────────────────
DEFINE INDEX IF NOT EXISTS idx_node_workspace ON TABLE node FIELDS workspace;
DEFINE INDEX IF NOT EXISTS idx_node_file      ON TABLE node FIELDS file_path;
DEFINE INDEX IF NOT EXISTS idx_node_language  ON TABLE node FIELDS language;
DEFINE INDEX IF NOT EXISTS idx_node_kind      ON TABLE node FIELDS kind;
DEFINE INDEX IF NOT EXISTS idx_node_label     ON TABLE node FIELDS label;

-- ─── 4. Typed Graph Relation Edges ────────────────────────────────────────────
-- First-class graph relations: in (source node) -> linked_to -> out (target node)
DEFINE TABLE IF NOT EXISTS linked_to TYPE RELATION;

DEFINE FIELD IF NOT EXISTS workspace  ON TABLE linked_to TYPE string DEFAULT 'default';
DEFINE FIELD IF NOT EXISTS type       ON TABLE linked_to TYPE string;    -- CALLS, IMPORTS, CONTAINS, IMPLEMENTS, DEPENDS_ON, TYPE_REF
DEFINE FIELD IF NOT EXISTS category   ON TABLE linked_to TYPE string;    -- EXTRACTED (direct AST) or INFERRED (cross-file analysis)
DEFINE FIELD IF NOT EXISTS metadata   ON TABLE linked_to TYPE option<object>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE linked_to TYPE datetime DEFAULT time::now();

-- Indexes on relation endpoints for rapid traversal
DEFINE INDEX IF NOT EXISTS idx_edge_workspace ON TABLE linked_to FIELDS workspace;
DEFINE INDEX IF NOT EXISTS idx_edge_type      ON TABLE linked_to FIELDS type;

-- ─── 5. Workspace Root Path Metadata ──────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS workspace_meta SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS workspace  ON TABLE workspace_meta TYPE string;
DEFINE FIELD IF NOT EXISTS root_path  ON TABLE workspace_meta TYPE string;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE workspace_meta TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_workspace_meta_name ON TABLE workspace_meta FIELDS workspace;
