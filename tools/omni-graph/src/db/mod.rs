// SurrealDB v2 Client & Repository
// Implements: R-005, R-006, R-007, R-008, R-009, R-019, R-020, R-022, R-023

use crate::config::Config;
use crate::parser::{ExtractedEdge, ExtractedNode};
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::info;

/// Comprehensive SurrealQL string escaping to prevent injection attacks.
/// Handles: backslashes, single quotes, newlines, carriage returns, null bytes, and control chars.
pub fn surql_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push(' '),
            '\r' => {},
            '\0' => {},
            c if c.is_control() => {},
            c => out.push(c),
        }
    }
    out
}

/// Cleans SurrealDB record ID representations (e.g. node:⟨abc⟩, node:`abc`, `abc`, ⟨abc⟩)
/// down to bare identifier string for safe SurrealQL querying and hashing (R-050).
pub fn clean_record_id(raw: &str) -> &str {
    raw.strip_prefix("node:")
        .unwrap_or(raw)
        .trim_matches('`')
        .trim_matches('⟨')
        .trim_matches('⟩')
        .trim_matches('"')
        .trim_matches('\'')
}

/// Extracts all non-null result arrays from a multi-statement SurrealDB response.
/// Ignores LET statements (which return null results) and retrieves the result arrays
/// of successive SELECT statements safely regardless of statement index offsets.
pub fn extract_sql_arrays(resp: &serde_json::Value) -> Vec<&Vec<serde_json::Value>> {
    let mut arrays = Vec::new();
    if let Some(arr) = resp.as_array() {
        for stmt in arr {
            if let Some(res) = stmt.get("result").and_then(|r| r.as_array()) {
                arrays.push(res);
            }
        }
    }
    arrays
}

#[derive(Clone, Debug)]
pub struct DbClient {
    base_url: String,
    ns: String,
    db: String,
    client: Client,
    auth_header: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbNode {
    pub id: String,
    pub workspace: Option<String>,
    pub label: String,
    pub kind: String,
    pub file_path: String,
    pub language: String,
    pub line_start: usize,
    pub line_end: usize,
    pub text: String,
    pub community: Option<i32>,
    #[serde(default)]
    pub file_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbLink {
    pub id: String,
    pub workspace: Option<String>,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceResult {
    #[serde(flatten)]
    pub caller: DbNode,
    #[serde(rename = "type", default = "default_calls_type")]
    pub edge_type: String,
    #[serde(default = "default_inferred_category")]
    pub category: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl std::ops::Deref for ReferenceResult {
    type Target = DbNode;
    fn deref(&self) -> &Self::Target {
        &self.caller
    }
}

fn default_calls_type() -> String {
    "CALLS".to_string()
}

fn default_inferred_category() -> String {
    "INFERRED".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipPayload {
    pub workspace: String,
    pub source_symbol: String,
    pub target_symbol: String,
    #[serde(rename = "type", default = "default_calls_type")]
    pub rel_type: String,
    #[serde(default = "default_inferred_category")]
    pub category: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub workspace: Option<String>,
    pub label: String,
    pub kind: String,
    pub file_path: String,
    pub language: String,
    pub text: String,
    pub similarity: f32,
    pub community: Option<i32>,
    #[serde(default)]
    pub line_start: Option<usize>,
    #[serde(default)]
    pub line_end: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GalaxyRecord {
    pub workspace: String,
    pub galaxy_id: i32,
    pub name: String,
    pub dominant_path: String,
    pub node_count: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub afferent_coupling: usize, // Ca
    pub efferent_coupling: usize, // Ce
    pub instability: f64,         // Ce / (Ca + Ce)
    pub role: String,             // Core Foundation, Domain Service, Orchestrator
    pub key_symbols: Vec<String>,
    pub languages: Vec<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GalaxySubsystemRef {
    pub id: i32,
    pub name: String,
    pub dominant_path: String,
    pub instability: f64,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallerRef {
    pub symbol: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub galaxy_id: Option<i32>,
    pub galaxy_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainmentStatus {
    pub is_exported: bool,
    pub internal_callers_count: usize,
    pub cross_galaxy_callers_count: usize,
    pub architectural_status: String, // "INTERNAL_ONLY", "BOUNDARY_CROSSING", "ISOLATED"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentBoundaryAdvice {
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH"
    pub summary: String,
    pub rule_of_thumb: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolBoundaryInfo {
    pub symbol: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub workspace: String,
    pub home_galaxy: Option<GalaxySubsystemRef>,
    pub containment: ContainmentStatus,
    pub internal_callers: Vec<CallerRef>,
    pub cross_galaxy_callers: Vec<CallerRef>,
    pub agent_actionable_advice: AgentBoundaryAdvice,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct SurrealResponse<T> {
    result: Option<Vec<T>>,
    status: Option<String>,
}

impl DbClient {
    pub fn new(config: &Config) -> Self {
        let auth = format!("{}:{}", config.surreal_user, config.surreal_pass);
        // Base64 encoding
        let encoded_auth = format!("Basic {}", Self::base64_encode(auth.as_bytes()));

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Self {
            base_url: config.surreal_url.clone(),
            ns: config.surreal_ns.clone(),
            db: config.surreal_db.clone(),
            client,
            auth_header: encoded_auth,
        }
    }

    fn base64_encode(input: &[u8]) -> String {
        const CHARSET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        let mut i = 0;
        while i < input.len() {
            let b0 = input[i] as usize;
            let b1 = if i + 1 < input.len() {
                input[i + 1] as usize
            } else {
                0
            };
            let b2 = if i + 2 < input.len() {
                input[i + 2] as usize
            } else {
                0
            };

            let triple = (b0 << 16) | (b1 << 8) | b2;

            out.push(CHARSET[(triple >> 18) & 0x3F] as char);
            out.push(CHARSET[(triple >> 12) & 0x3F] as char);

            if i + 1 < input.len() {
                out.push(CHARSET[(triple >> 6) & 0x3F] as char);
            } else {
                out.push('=');
            }

            if i + 2 < input.len() {
                out.push(CHARSET[triple & 0x3F] as char);
            } else {
                out.push('=');
            }

            i += 3;
        }
        out
    }

    /// Execute raw SurrealQL query string
    pub async fn query_sql(&self, surql: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/sql", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("surreal-ns", &self.ns)
            .header("surreal-db", &self.db)
            .header("NS", &self.ns)
            .header("DB", &self.db)
            .header(AUTHORIZATION, &self.auth_header)
            .header(ACCEPT, "application/json")
            .body(surql.to_string())
            .send()
            .await
            .map_err(|e| format!("SurrealDB request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("SurrealDB error {}: {}", status, body));
        }

        let json = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse SurrealDB response: {}", e))?;

        if let Some(arr) = json.as_array() {
            for item in arr {
                if item.get("status").and_then(|s| s.as_str()) == Some("ERR") {
                    let err_msg = item
                        .get("result")
                        .and_then(|r| r.as_str())
                        .unwrap_or("Unknown SurrealDB query error");
                    return Err(format!("SurrealDB query error: {}", err_msg));
                }
            }
        }

        Ok(json)
    }

    /// Health check for SurrealDB (R-022)
    pub async fn health(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        if let Ok(resp) = self.client.get(&url).send().await {
            if resp.status().is_success() {
                return true;
            }
        }
        // Fallback to query
        self.query_sql("INFO FOR DB;").await.is_ok()
    }

    /// Initialize schema on startup (R-019)
    pub async fn init_schema(&self, schema_content: &str) -> Result<(), String> {
        info!("Initializing SurrealDB schema and HNSW index...");
        self.query_sql(schema_content).await?;
        info!("SurrealDB schema initialized successfully.");
        Ok(())
    }

    /// Batch insert nodes and embeddings with workspace isolation (R-005)
    pub async fn store_nodes(
        &self,
        nodes: &[ExtractedNode],
        embeddings: &[Vec<f32>],
        file_hash: Option<&str>,
    ) -> Result<(), String> {
        self.store_nodes_with_community(nodes, embeddings, file_hash, None).await
    }

    /// Batch insert/merge nodes with optional community assignment (preserves existing community if None)
    pub async fn store_nodes_with_community(
        &self,
        nodes: &[ExtractedNode],
        embeddings: &[Vec<f32>],
        file_hash: Option<&str>,
        default_community: Option<i32>,
    ) -> Result<(), String> {
        if nodes.is_empty() {
            return Ok(());
        }

        let hash_field = match file_hash {
            Some(h) => format!(", file_hash: '{}'", surql_escape(h)),
            None => String::new(),
        };

        let comm_field = match default_community {
            Some(c) => format!(", community: {}", c),
            None => String::new(),
        };

        // Chunk in batches of 50 nodes to avoid HTTP payload buffer limits on large files
        for (chunk_nodes, chunk_embs) in nodes.chunks(50).zip(embeddings.chunks(50)) {
            let mut query = String::new();
            for (node, emb) in chunk_nodes.iter().zip(chunk_embs.iter()) {
                let emb_str = serde_json::to_string(emb).unwrap_or_else(|_| "[]".to_string());
                let escaped_text = surql_escape(&node.text);
                let escaped_label = surql_escape(&node.label);
                let escaped_path = surql_escape(&node.file_path);
                let escaped_ws = surql_escape(&node.workspace);
                let clean_id = clean_record_id(&node.id);
                let escaped_id = surql_escape(clean_id);
                let escaped_kind = surql_escape(&node.kind);
                let escaped_lang = surql_escape(&node.language);

                // Use MERGE instead of CONTENT so that existing fields like community are preserved when not explicitly overridden
                query.push_str(&format!(
                    "UPSERT type::thing('node', '{}') MERGE {{ workspace: '{}', label: '{}', kind: '{}', file_path: '{}', language: '{}', line_start: {}, line_end: {}, text: '{}', embedding: {}{}{} }};\n",
                    escaped_id, escaped_ws, escaped_label, escaped_kind, escaped_path, escaped_lang, node.line_start, node.line_end, escaped_text, emb_str, hash_field, comm_field
                ));
            }
            self.query_sql(&query).await?;
        }
        Ok(())
    }

    /// Resolve and store graph relations with workspace isolation (R-006)
    pub async fn store_edges(&self, edges: &[ExtractedEdge]) -> Result<(), String> {
        if edges.is_empty() {
            return Ok(());
        }

        // Chunk in batches of 30 edges (each edge produces 3 SurrealDB statements)
        for chunk in edges.chunks(30) {
            let mut query = String::new();
            for edge in chunk {
                let escaped_ws = surql_escape(&edge.workspace);
                let escaped_label = surql_escape(&edge.target_label);
                let escaped_type = surql_escape(&edge.edge_type);
                let escaped_cat = surql_escape(&edge.category);

                let src_parts: Vec<&str> = edge.source_id.split(':').collect();
                let src_resolution_surql = if src_parts.len() == 3 {
                    // 3-part ID: ws:file:label (e.g. emitted by impl_item or receiver type without line number - R-039, R-045)
                    let s_ws = surql_escape(src_parts[0]);
                    let s_file = surql_escape(src_parts[1]);
                    let s_label = surql_escape(src_parts[2]);
                    format!(
                        "LET $local_src = (SELECT VALUE id FROM node WHERE workspace = '{}' AND file_path = '{}' AND label = '{}' LIMIT 1);\n\
                         LET $src = IF array::len($local_src) > 0 THEN $local_src ELSE (SELECT VALUE id FROM node WHERE workspace = '{}' AND label = '{}' LIMIT 1) END;\n",
                        s_ws, s_file, s_label, s_ws, s_label
                    )
                } else {
                    let clean_src = clean_record_id(&edge.source_id);
                    format!("LET $src = [type::thing('node', '{}')];\n", surql_escape(clean_src))
                };

                query.push_str(&src_resolution_surql);

                if let Some(target_id) = &edge.target_id {
                    let clean_tgt = clean_record_id(target_id);
                    let escaped_target = surql_escape(clean_tgt);
                    query.push_str(&format!(
                        "LET $tgt = [type::thing('node', '{}')];\n\
                         RELATE $src->linked_to->$tgt SET workspace = '{}', type = '{}', category = '{}';\n",
                        escaped_target,
                        escaped_ws,
                        escaped_type,
                        escaped_cat
                    ));
                } else {
                    // Scoped edge resolution: check local source file first to avoid cross-package collisions (R-035)
                    let source_file = edge.source_id.split(':').nth(1).unwrap_or("");
                    let escaped_file = surql_escape(source_file);
                    query.push_str(&format!(
                        "LET $local_targets = (SELECT VALUE id FROM node WHERE workspace = '{}' AND file_path = '{}' AND label = '{}' LIMIT 1);\n\
                         LET $targets = IF array::len($local_targets) > 0 THEN $local_targets ELSE (SELECT VALUE id FROM node WHERE workspace = '{}' AND label = '{}' LIMIT 1) END;\n\
                         RELATE $src->linked_to->$targets SET workspace = '{}', type = '{}', category = '{}';\n",
                        escaped_ws,
                        escaped_file,
                        escaped_label,
                        escaped_ws,
                        escaped_label,
                        escaped_ws,
                        escaped_type,
                        escaped_cat
                    ));
                }
            }
            self.query_sql(&query).await?;
        }
        Ok(())
    }

    /// Semantic search using vector cosine similarity on HNSW index (R-007, R-009)
    pub async fn search_vector(
        &self,
        query_vec: &[f32],
        k: usize,
        workspace: Option<&str>,
    ) -> Result<Vec<SearchResult>, String> {
        let vec_json = serde_json::to_string(query_vec).map_err(|e| e.to_string())?;
        let where_clause = match workspace {
            Some(ws) if !ws.is_empty() => format!("WHERE workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let q = format!(
            "SELECT id, workspace, label, kind, file_path, language, text, community, line_start, line_end, \
             vector::similarity::cosine(embedding, {}) AS similarity \
             FROM node {} ORDER BY similarity DESC LIMIT {};",
            vec_json, where_clause, k
        );

        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();

        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let Ok(res) = serde_json::from_value::<SearchResult>(item.clone()) {
                    results.push(res);
                }
            }
        }

        Ok(results)
    }

    /// Fetch entire graph topology for visualization (R-008, R-020)
    pub async fn get_graph(&self, workspace: Option<&str>) -> Result<(Vec<DbNode>, Vec<DbLink>), String> {
        let (node_q, link_q) = match workspace {
            Some(ws) if !ws.is_empty() => {
                let escaped = surql_escape(ws);
                (
                    format!("SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community FROM node WHERE workspace = '{}';", escaped),
                    format!("SELECT id, workspace, in AS source, out AS target, type, category FROM linked_to WHERE workspace = '{}';", escaped),
                )
            }
            _ => (
                "SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community FROM node;".to_string(),
                "SELECT id, workspace, in AS source, out AS target, type, category FROM linked_to;".to_string(),
            ),
        };

        let q = format!("{}\n{}", node_q, link_q);
        let resp = self.query_sql(&q).await?;
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        if let Some(results) = resp.as_array() {
            if let Some(node_arr) = results.first().and_then(|r| r.get("result")).and_then(|v| v.as_array()) {
                for n in node_arr {
                    if let Ok(node) = serde_json::from_value::<DbNode>(n.clone()) {
                        nodes.push(node);
                    }
                }
            }
            if let Some(link_arr) = results.get(1).and_then(|r| r.get("result")).and_then(|v| v.as_array()) {
                for l in link_arr {
                    if let Ok(link) = serde_json::from_value::<DbLink>(l.clone()) {
                        links.push(link);
                    }
                }
            }
        }

        Ok((nodes, links))
    }

    /// Efficient localized subgraph retrieval around a target symbol (R-043)
    /// Performs localized 1-2 hop BFS expansion in SurrealDB rather than loading full workspace topology.
    pub async fn get_symbol_subgraph(
        &self,
        symbol: &str,
        workspace: Option<&str>,
        _hops: usize,
    ) -> Result<(Vec<DbNode>, Vec<DbLink>), String> {
        let roots = self.find_symbols(symbol, workspace).await?;
        if roots.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let esc_ws = workspace.map(surql_escape).unwrap_or_default();
        let ws_clause = if !esc_ws.is_empty() {
            format!("AND workspace = '{}'", esc_ws)
        } else {
            String::new()
        };

        // Collect root ID representations for SurrealQL
        let root_id_exprs: Vec<String> = roots
            .iter()
            .map(|r| {
                let clean = clean_record_id(&r.id);
                format!("type::thing('node', '{}')", surql_escape(clean))
            })
            .collect();
        let roots_surql_array = format!("[{}]", root_id_exprs.join(", "));

        // Query 1-hop and 2-hop edges + distinct member nodes directly in SurrealDB
        let q = format!(
            "LET $roots = {};\n\
             LET $hop1_links = (SELECT id, workspace, in AS source, out AS target, type, category FROM linked_to WHERE (in IN $roots OR out IN $roots) {});\n\
             LET $hop1_nodes = array::distinct(array::concat($hop1_links.source, $hop1_links.target));\n\
             LET $hop2_links = (SELECT id, workspace, in AS source, out AS target, type, category FROM linked_to WHERE (in IN $hop1_nodes OR out IN $hop1_nodes) {});\n\
             LET $all_links = array::distinct(array::concat($hop1_links, $hop2_links));\n\
             LET $all_node_ids = array::slice(array::distinct(array::concat($all_links.source, $all_links.target, $roots)), 0, 60);\n\
             SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community FROM node WHERE id IN $all_node_ids;\n\
             SELECT id, workspace, source, target, type, category FROM $all_links;\n",
            roots_surql_array, ws_clause, ws_clause
        );

        let resp = self.query_sql(&q).await?;
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        let arrays = extract_sql_arrays(&resp);
        if arrays.len() >= 2 {
            for item in arrays[0] {
                if let Ok(n) = serde_json::from_value::<DbNode>(item.clone()) {
                    nodes.push(n);
                }
            }
            for item in arrays[1] {
                if let Ok(l) = serde_json::from_value::<DbLink>(item.clone()) {
                    links.push(l);
                }
            }
        }

        // If localized query returned nodes, return immediately; else fall back to full graph
        if !nodes.is_empty() {
            Ok((nodes, links))
        } else {
            self.get_graph(workspace).await
        }
    }

    /// Localized 1-hop neighborhood subgraph around a set of seed nodes (R-043)
    /// Used by Graph-RAG to retrieve targeted AST context without pulling full-graph topology.
    pub async fn get_neighborhood_subgraph(
        &self,
        seed_ids: &[String],
        workspace: Option<&str>,
    ) -> Result<(Vec<DbNode>, Vec<DbLink>), String> {
        if seed_ids.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let esc_ws = workspace.map(surql_escape).unwrap_or_default();
        let ws_clause = if !esc_ws.is_empty() {
            format!("AND workspace = '{}'", esc_ws)
        } else {
            String::new()
        };

        // Collect seed ID representations for SurrealQL
        let seed_id_exprs: Vec<String> = seed_ids
            .iter()
            .map(|id| {
                let clean = clean_record_id(id);
                format!("type::thing('node', '{}')", surql_escape(clean))
            })
            .collect();
        let seeds_surql_array = format!("[{}]", seed_id_exprs.join(", "));

        // Query 1-hop edges + distinct member nodes directly in SurrealDB
        let q = format!(
            "LET $seeds = {};\n\
             LET $hop1_links = (SELECT id, workspace, in AS source, out AS target, type, category FROM linked_to WHERE (in IN $seeds OR out IN $seeds) {});\n\
             LET $all_links = array::distinct($hop1_links);\n\
             LET $all_node_ids = array::slice(array::distinct(array::concat($all_links.source, $all_links.target, $seeds)), 0, 40);\n\
             SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community FROM node WHERE id IN $all_node_ids;\n\
             SELECT id, workspace, source, target, type, category FROM $all_links;\n",
            seeds_surql_array, ws_clause
        );

        let resp = self.query_sql(&q).await?;
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        let arrays = extract_sql_arrays(&resp);
        if arrays.len() >= 2 {
            for item in arrays[0] {
                if let Ok(n) = serde_json::from_value::<DbNode>(item.clone()) {
                    nodes.push(n);
                }
            }
            for item in arrays[1] {
                if let Ok(l) = serde_json::from_value::<DbLink>(item.clone()) {
                    links.push(l);
                }
            }
        }

        // If localized query returned nodes, return immediately; else fall back to full graph
        if !nodes.is_empty() {
            Ok((nodes, links))
        } else {
            self.get_graph(workspace).await
        }
    }

    /// Aggregated stats (R-023)
    pub async fn get_stats(&self) -> Result<serde_json::Value, String> {
        let q = "SELECT count() FROM node GROUP ALL;\n\
                 SELECT count() FROM linked_to GROUP ALL;\n\
                 SELECT language, count() FROM node GROUP BY language;\n\
                 SELECT workspace, count() FROM node GROUP BY workspace;";

        let resp = self.query_sql(q).await?;
        let mut total_nodes = 0;
        let mut total_edges = 0;
        let mut languages = HashMap::new();
        let mut workspaces = HashMap::new();

        if let Some(arr) = resp.as_array() {
            if let Some(n) = arr.first().and_then(|r| r.get("result")).and_then(|res| res.as_array()).and_then(|a| a.first()).and_then(|o| o.get("count")).and_then(|c| c.as_i64()) {
                total_nodes = n;
            }
            if let Some(e) = arr.get(1).and_then(|r| r.get("result")).and_then(|res| res.as_array()).and_then(|a| a.first()).and_then(|o| o.get("count")).and_then(|c| c.as_i64()) {
                total_edges = e;
            }
            if let Some(langs) = arr.get(2).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
                for l in langs {
                    if let (Some(lang), Some(cnt)) = (l.get("language").and_then(|v| v.as_str()), l.get("count").and_then(|v| v.as_i64())) {
                        languages.insert(lang.to_string(), cnt);
                    }
                }
            }
            if let Some(ws_arr) = arr.get(3).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
                for w in ws_arr {
                    if let (Some(ws), Some(cnt)) = (w.get("workspace").and_then(|v| v.as_str()), w.get("count").and_then(|v| v.as_i64())) {
                        workspaces.insert(ws.to_string(), cnt);
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "total_nodes": total_nodes,
            "total_edges": total_edges,
            "languages": languages,
            "workspaces": workspaces
        }))
    }

    /// Record full workspace metadata including worktree lineage (R-051)
    pub async fn record_workspace_meta(
        &self,
        workspace: &str,
        root_path: &str,
        is_worktree: bool,
        parent_workspace: Option<&str>,
        branch: Option<&str>,
        worktree_name: Option<&str>,
    ) -> Result<(), String> {
        let esc_ws = surql_escape(workspace);
        let esc_path = surql_escape(root_path);
        let parent_field = match parent_workspace {
            Some(p) => format!(", parent_workspace: '{}'", surql_escape(p)),
            None => String::new(),
        };
        let branch_field = match branch {
            Some(b) => format!(", branch: '{}'", surql_escape(b)),
            None => String::new(),
        };
        let wt_field = match worktree_name {
            Some(w) => format!(", worktree_name: '{}'", surql_escape(w)),
            None => String::new(),
        };
        let q = format!(
            "UPSERT type::thing('workspace_meta', '{}') CONTENT {{ workspace: '{}', root_path: '{}', is_worktree: {}, updated_at: time::now(){}{}{} }};",
            esc_ws, esc_ws, esc_path, is_worktree, parent_field, branch_field, wt_field
        );
        self.query_sql(&q).await?;
        Ok(())
    }

    /// Record the root filesystem path for a workspace (defaulting to anchor repo)
    pub async fn record_workspace_root(&self, workspace: &str, root_path: &str) -> Result<(), String> {
        self.record_workspace_meta(workspace, root_path, false, None, None, None).await
    }

    /// Retrieve community assignments from parent workspace translated to target workspace for seed inheritance (R-053)
    pub async fn get_community_seeds_for_parent(
        &self,
        parent_workspace: &str,
        target_workspace: &str,
    ) -> Result<HashMap<String, i32>, String> {
        let esc_parent = surql_escape(parent_workspace);
        let q = format!(
            "SELECT id, community FROM node WHERE workspace = '{}' AND community != NONE;",
            esc_parent
        );
        let resp = self.query_sql(&q).await?;
        let mut seeds = HashMap::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let (Some(id_str), Some(comm)) = (
                    item.get("id").and_then(|v| v.as_str()),
                    item.get("community").and_then(|v| v.as_i64()),
                ) {
                    let clean_id = clean_record_id(id_str);
                    let target_id = clean_id.replacen(parent_workspace, target_workspace, 1);
                    seeds.insert(format!("node:`{}`", target_id), comm as i32);
                    seeds.insert(target_id, comm as i32);
                }
            }
        }
        Ok(seeds)
    }

    /// Purge dead worktrees whose root directory no longer exists on disk (R-054)
    pub async fn purge_dead_worktrees(&self) -> Result<Vec<String>, String> {
        let q = "SELECT workspace, root_path FROM workspace_meta WHERE is_worktree = true;";
        let resp = self.query_sql(q).await?;
        let mut purged = Vec::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let (Some(ws), Some(path_str)) = (
                    item.get("workspace").and_then(|v| v.as_str()),
                    item.get("root_path").and_then(|v| v.as_str()),
                ) {
                    let p = std::path::Path::new(path_str);
                    if !p.exists() {
                        info!("Dead worktree detected on disk at '{}' ({}); auto-purging...", path_str, ws);
                        let _ = self.purge_workspace(ws).await;
                        let del_meta = format!("DELETE FROM type::thing('workspace_meta', '{}');", surql_escape(ws));
                        let _ = self.query_sql(&del_meta).await;
                        purged.push(ws.to_string());
                    }
                }
            }
        }
        Ok(purged)
    }

    /// Retrieve the root filesystem path for a workspace
    pub async fn get_workspace_root(&self, workspace: &str) -> Result<Option<String>, String> {
        let esc_ws = surql_escape(workspace);
        let q = format!(
            "SELECT VALUE root_path FROM type::thing('workspace_meta', '{}') LIMIT 1;",
            esc_ws
        );
        let resp = self.query_sql(&q).await?;
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            if let Some(val) = arr.first().and_then(|v| v.as_str()) {
                return Ok(Some(val.to_string()));
            }
        }
        Ok(None)
    }

    /// List all distinct workspaces with metadata (including root_path and worktree lineage) (R-055)
    pub async fn get_workspaces(&self) -> Result<Vec<serde_json::Value>, String> {
        self.get_workspaces_internal(false).await
    }

    /// List all distinct workspaces flat without nesting worktrees (R-055)
    pub async fn get_workspaces_flat(&self) -> Result<Vec<serde_json::Value>, String> {
        self.get_workspaces_internal(true).await
    }

    /// Internal workspace query supporting both nested and flat modes
    pub async fn get_workspaces_internal(&self, flat: bool) -> Result<Vec<serde_json::Value>, String> {
        // First run automatic garbage collection on dead worktrees (R-054)
        let _ = self.purge_dead_worktrees().await;

        // Fetch all recorded workspace metadata
        let meta_q = "SELECT workspace, root_path, is_worktree, parent_workspace, branch, worktree_name FROM workspace_meta;";
        let meta_resp = self.query_sql(meta_q).await.unwrap_or_default();
        let mut meta_map: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(arr) = meta_resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for m in arr {
                if let Some(ws) = m.get("workspace").and_then(|v| v.as_str()) {
                    meta_map.insert(ws.to_string(), m.clone());
                }
            }
        }

        let q = "SELECT workspace, count() AS total_nodes, array::distinct(language) AS languages, array::distinct(file_path) AS files FROM node GROUP BY workspace;";
        let resp = self.query_sql(q).await?;
        let mut map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let Some(raw_ws) = item.get("workspace").and_then(|v| v.as_str()) {
                    let ws_str = raw_ws.trim();
                    if ws_str.is_empty() || ws_str == "default" || ws_str == "global" {
                        continue;
                    }
                    let canonical_ws = if ws_str == "workspace" {
                        "tool-scripts".to_string()
                    } else {
                        ws_str.to_string()
                    };

                    let nodes = item.get("total_nodes").and_then(|n| n.as_u64()).unwrap_or(0);
                    let langs = item.get("languages").and_then(|l| l.as_array()).cloned().unwrap_or_default();
                    let files = item.get("files").and_then(|f| f.as_array()).cloned().unwrap_or_default();

                    if let Some(existing) = map.get_mut(&canonical_ws) {
                        if let Some(prev_nodes) = existing.get("total_nodes").and_then(|n| n.as_u64()) {
                            existing["total_nodes"] = serde_json::Value::Number((prev_nodes + nodes).into());
                        }
                        if let Some(prev_langs) = existing.get_mut("languages").and_then(|l| l.as_array_mut()) {
                            for l in langs {
                                if !prev_langs.contains(&l) {
                                    prev_langs.push(l);
                                }
                            }
                        }
                        if let Some(prev_files) = existing.get_mut("files").and_then(|f| f.as_array_mut()) {
                            for f in files {
                                if !prev_files.contains(&f) {
                                    prev_files.push(f);
                                }
                            }
                        }
                    } else {
                        let mut obj = item.clone();
                        obj["workspace"] = serde_json::Value::String(canonical_ws.clone());
                        let mut resolved_root: Option<String> = None;

                        if let Some(m) = meta_map.get(&canonical_ws) {
                            if let Some(rp) = m.get("root_path").and_then(|v| v.as_str()) {
                                resolved_root = Some(rp.to_string());
                            }
                            if let Some(is_wt) = m.get("is_worktree").and_then(|v| v.as_bool()) {
                                obj["is_worktree"] = serde_json::Value::Bool(is_wt);
                            }
                            if let Some(pw) = m.get("parent_workspace").and_then(|v| v.as_str()) {
                                obj["parent_workspace"] = serde_json::Value::String(pw.to_string());
                            }
                            if let Some(br) = m.get("branch").and_then(|v| v.as_str()) {
                                obj["branch"] = serde_json::Value::String(br.to_string());
                            }
                            if let Some(wn) = m.get("worktree_name").and_then(|v| v.as_str()) {
                                obj["worktree_name"] = serde_json::Value::String(wn.to_string());
                            }
                        }

                        if resolved_root.is_none() {
                            resolved_root = self.get_workspace_root(&canonical_ws).await.unwrap_or(None);
                        }
                        
                        if resolved_root.is_none() {
                            let sample_files: Vec<String> = files.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .take(5)
                                .collect();
                            if let Some(auto_p) = resolve_disk_path(&canonical_ws, &sample_files) {
                                let _ = self.record_workspace_root(&canonical_ws, &auto_p).await;
                                resolved_root = Some(auto_p);
                            }
                        }

                        if let Some(rp) = resolved_root {
                            obj["root_path"] = serde_json::Value::String(rp);
                        }
                        obj["worktrees"] = serde_json::Value::Array(Vec::new());
                        map.insert(canonical_ws, obj);
                    }
                }
            }
        }

        // Second pass: nest worktrees under their parent workspaces (R-055)
        if !flat {
            let worktree_keys: Vec<(String, String)> = map.iter()
                .filter_map(|(k, v)| {
                    let is_wt = v.get("is_worktree").and_then(|b| b.as_bool()).unwrap_or(false);
                    let parent = v.get("parent_workspace").and_then(|s| s.as_str()).map(|s| s.to_string());
                    if is_wt {
                        parent.map(|p| (k.clone(), p))
                    } else {
                        None
                    }
                })
                .collect();

            for (wt_key, parent_key) in worktree_keys {
                if let Some(wt_entry) = map.get(&wt_key).cloned() {
                    if let Some(parent_entry) = map.get_mut(&parent_key) {
                        if let Some(arr) = parent_entry.get_mut("worktrees").and_then(|a| a.as_array_mut()) {
                            arr.push(serde_json::json!({
                                "workspace": wt_key,
                                "branch": wt_entry.get("branch").and_then(|v| v.as_str()).unwrap_or(""),
                                "root_path": wt_entry.get("root_path").and_then(|v| v.as_str()).unwrap_or(""),
                                "total_nodes": wt_entry.get("total_nodes").and_then(|v| v.as_u64()).unwrap_or(0),
                            }));
                        }
                        // Prune adopted worktree from top-level map to eliminate UI duplicate display
                        map.remove(&wt_key);
                    }
                }
            }
        }

        let mut list: Vec<serde_json::Value> = map.into_values().collect();
        list.sort_by(|a, b| {
            let name_a = a.get("workspace").and_then(|v| v.as_str()).unwrap_or("");
            let name_b = b.get("workspace").and_then(|v| v.as_str()).unwrap_or("");
            name_a.cmp(name_b)
        });
        Ok(list)
    }

    /// Symbolic lookup: Find symbol definitions by name (LSP textDocument/definition equivalent)
    pub async fn find_symbols(&self, name: &str, workspace: Option<&str>) -> Result<Vec<DbNode>, String> {
        let escaped = surql_escape(name);
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("AND workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let q = format!(
            "SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community, \
             (label = '{}') AS is_exact \
             FROM node WHERE (label = '{}' OR label CONTAINS '{}') {} \
             ORDER BY is_exact DESC, label ASC LIMIT 20;",
            escaped, escaped, escaped, ws_filter
        );

        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();

        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let Ok(node) = serde_json::from_value::<DbNode>(item.clone()) {
                    results.push(node);
                }
            }
        }
        results.sort_by(|a, b| {
            let exact_a = a.label == name;
            let exact_b = b.label == name;
            if exact_a != exact_b {
                return exact_b.cmp(&exact_a);
            }
            let is_import_a = a.kind == "import";
            let is_import_b = b.kind == "import";
            if is_import_a != is_import_b {
                return is_import_a.cmp(&is_import_b);
            }
            a.id.cmp(&b.id)
        });

        Ok(results)
    }

    /// Symbolic references: Find all callers / references of a symbol (LSP textDocument/references equivalent)
    pub async fn find_references(&self, symbol_name: &str, workspace: Option<&str>) -> Result<Vec<ReferenceResult>, String> {
        let escaped = surql_escape(symbol_name);
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("AND workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let q = format!(
            "LET $targets = (SELECT id FROM node WHERE (label = '{escaped}' OR string::ends_with(label, '::{escaped}') OR string::ends_with(label, '.{escaped}')) {ws_filter});\n\
             SELECT in.* AS caller, type, category, metadata FROM linked_to WHERE out IN $targets.id;\n"
        );

        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();

        if let Some(arr) = resp.as_array() {
            if let Some(res_arr) = arr.get(1).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
                for item in res_arr {
                    if let Ok(ref_res) = serde_json::from_value::<ReferenceResult>(item.clone()) {
                        results.push(ref_res);
                    } else if let Some(caller) = item.get("caller") {
                        if let Ok(node) = serde_json::from_value::<DbNode>(caller.clone()) {
                            results.push(ReferenceResult {
                                caller: node,
                                edge_type: item.get("type").and_then(|t| t.as_str()).unwrap_or("CALLS").to_string(),
                                category: item.get("category").and_then(|c| c.as_str()).unwrap_or("EXTRACTED").to_string(),
                                metadata: item.get("metadata").cloned(),
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Batch update community IDs for nodes (R-012 Graph RAG clustering)
    pub async fn update_communities(&self, assignments: &HashMap<String, i32>) -> Result<(), String> {
        if assignments.is_empty() {
            return Ok(());
        }

        let entries: Vec<(&String, &i32)> = assignments.iter().collect();
        info!("Updating communities for {} nodes in batches...", entries.len());

        for chunk in entries.chunks(150) {
            let mut query = String::new();
            for (node_id, comm_id) in chunk {
                let clean_id = clean_record_id(node_id);
                let escaped_id = surql_escape(clean_id);
                query.push_str(&format!(
                    "UPDATE type::thing('node', '{}') SET community = {};\n",
                    escaped_id, comm_id
                ));
            }

            self.query_sql(&query).await?;
        }

        info!("Successfully updated communities for {} nodes.", entries.len());
        Ok(())
    }

    /// Server-side galaxy aggregation query — avoids fetching all nodes (Finding #7)
    /// Returns community-level aggregates directly from SurrealDB.
    #[allow(dead_code)]
    pub async fn get_galaxy_aggregation(&self, workspace: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("WHERE workspace = '{}' AND community IS NOT NONE", surql_escape(ws)),
            _ => "WHERE community IS NOT NONE".to_string(),
        };
        let q = format!(
            "SELECT community, count() AS node_count, array::distinct(language) AS languages, \
             array::distinct(file_path) AS files, array::slice(array::distinct(label), 0, 6) AS sample_symbols \
             FROM node {} GROUP BY community ORDER BY node_count DESC;",
            ws_filter
        );

        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                results.push(item.clone());
            }
        }
        Ok(results)
    }

    /// Retrieve persisted file hashes for staleness cache restoration (Finding #4)
    pub async fn get_file_hashes(&self, workspace: &str) -> Result<HashMap<String, String>, String> {
        let escaped = surql_escape(workspace);
        let q = format!(
            "SELECT file_path, file_hash FROM node WHERE workspace = '{}' AND file_hash IS NOT NONE GROUP BY file_path, file_hash;",
            escaped
        );
        let resp = self.query_sql(&q).await?;
        let mut map = HashMap::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let (Some(path), Some(hash)) = (
                    item.get("file_path").and_then(|v| v.as_str()),
                    item.get("file_hash").and_then(|v| v.as_str()),
                ) {
                    map.insert(path.to_string(), hash.to_string());
                }
            }
        }
        Ok(map)
    }

    /// Atomic prune of file nodes and connected edges (R-011, R-046 / Delta Sync)
    pub async fn delete_file(&self, workspace: &str, file_path: &str) -> Result<(), String> {
        let esc_ws = surql_escape(workspace);
        let esc_path = surql_escape(file_path);
        let q = format!(
            "LET $nodes = (SELECT VALUE id FROM node WHERE workspace = '{}' AND file_path = '{}');\n\
             DELETE linked_to WHERE in IN $nodes OR out IN $nodes;\n\
             DELETE node WHERE workspace = '{}' AND file_path = '{}';",
            esc_ws, esc_path, esc_ws, esc_path
        );
        self.query_sql(&q).await?;
        Ok(())
    }

    /// Atomic purge of ALL nodes, edges, and galaxy records for a workspace.
    /// Used by refresh-ingestion to clear zombie nodes before a clean re-scan.
    pub async fn purge_workspace(&self, workspace: &str) -> Result<u64, String> {
        let esc_ws = surql_escape(workspace);

        // Count existing nodes for audit trail
        let count_q = format!(
            "SELECT count() AS total FROM node WHERE workspace = '{}' GROUP ALL;",
            esc_ws
        );
        let count_result = self.query_sql(&count_q).await.unwrap_or_default();
        let purged_count: u64 = count_result
            .as_array()
            .and_then(|a| a.first())
            .and_then(|r| r.get("result"))
            .and_then(|res| res.as_array())
            .and_then(|a| a.first())
            .and_then(|row| row.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // 1. Delete linked_to edges
        let _ = self.query_sql(&format!(
            "DELETE FROM linked_to WHERE workspace = '{}';",
            esc_ws
        )).await;

        // 2. Delete galaxies
        let _ = self.query_sql(&format!(
            "DELETE FROM galaxy WHERE workspace = '{}';",
            esc_ws
        )).await;

        // 3. Delete nodes:
        // For large workspaces (> 5,000 nodes), detaching the HNSW vector index first makes bulk delete 100x faster,
        // preventing transaction timeouts and memory exhaustion, then re-attaches the index.
        if purged_count > 5000 {
            info!("Large workspace detected ({} nodes) — optimizing bulk purge via index detachment...", purged_count);
            let _ = self.query_sql("REMOVE INDEX IF EXISTS idx_node_embedding ON TABLE node;").await;
            let _ = self.query_sql(&format!("DELETE FROM node WHERE workspace = '{}';", esc_ws)).await;
            let _ = self.query_sql("DEFINE INDEX IF NOT EXISTS idx_node_embedding ON TABLE node FIELDS embedding HNSW DIMENSION 384 DIST COSINE TYPE F32;").await;
        } else {
            let _ = self.query_sql(&format!("DELETE FROM node WHERE workspace = '{}';", esc_ws)).await;
        }

        // 4. Delete workspace metadata
        let _ = self.query_sql(&format!("DELETE FROM workspace_meta WHERE workspace = '{}';", esc_ws)).await;
        let _ = self.query_sql(&format!("DELETE FROM type::thing('workspace_meta', '{}');", esc_ws)).await;

        info!("Purged {} nodes + edges + galaxies for workspace '{}'", purged_count, workspace);
        Ok(purged_count)
    }

    /// Rename file path on nodes across workspace
    pub async fn rename_file(&self, workspace: &str, old_path: &str, new_path: &str) -> Result<(), String> {
        let esc_ws = surql_escape(workspace);
        let esc_old = surql_escape(old_path);
        let esc_new = surql_escape(new_path);
        let q = format!(
            "UPDATE node SET file_path = '{}' WHERE workspace = '{}' AND file_path = '{}';",
            esc_new, esc_ws, esc_old
        );
        self.query_sql(&q).await?;
        Ok(())
    }

    /// Record a live agent API call for real-time telemetry
    pub async fn record_agent_api_call(
        &self,
        endpoint: &str,
        workspace: &str,
        capability: &str,
        query_param: Option<&str>,
        caller: Option<&str>,
        duration_ms: i64,
    ) -> Result<(), String> {
        let esc_ep = surql_escape(endpoint);
        let esc_ws = surql_escape(workspace);
        let esc_cap = surql_escape(capability);
        let q_part = match query_param {
            Some(q) => format!("'{}'", surql_escape(q)),
            None => "NONE".to_string(),
        };
        let caller_part = match caller {
            Some(c) => format!("'{}'", surql_escape(c)),
            None => "NONE".to_string(),
        };
        let q = format!(
            "CREATE agent_api_call CONTENT {{ endpoint: '{}', workspace: '{}', capability: '{}', query_param: {}, caller: {}, duration_ms: {}, created_at: time::now() }};",
            esc_ep, esc_ws, esc_cap, q_part, caller_part, duration_ms
        );
        match self.query_sql(&q).await {
            Ok(_) => tracing::info!("Recorded agent_api_call for endpoint {}", endpoint),
            Err(e) => tracing::error!("Failed to record agent_api_call: {}", e),
        }
        Ok(())
    }

    /// Retrieve live agent API telemetry summary
    pub async fn get_api_calls_summary(&self) -> Result<Vec<serde_json::Value>, String> {
        let q = "SELECT capability, workspace, count() as total_calls, math::mean(duration_ms) as avg_duration_ms FROM agent_api_call GROUP BY capability, workspace;";
        let resp = self.query_sql(q).await?;
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            return Ok(arr.clone());
        }
        Ok(Vec::new())
    }

    /// Retrieve recent live agent API telemetry records
    #[allow(dead_code)]
    pub async fn get_api_recent_calls(&self, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let q = format!(
            "SELECT endpoint, workspace, capability, query_param, caller, duration_ms, created_at FROM agent_api_call ORDER BY created_at DESC LIMIT {};",
            limit
        );
        let resp = self.query_sql(&q).await?;
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            return Ok(arr.clone());
        }
        Ok(Vec::new())
    }

    /// Store computed architectural galaxy subsystems with coupling metrics
    pub async fn store_galaxies(&self, workspace: &str, galaxies: &[GalaxyRecord]) -> Result<(), String> {
        if galaxies.is_empty() {
            return Ok(());
        }

        let esc_ws = surql_escape(workspace);
        let delete_q = format!("DELETE galaxy WHERE workspace = '{}';", esc_ws);
        let _ = self.query_sql(&delete_q).await;

        let mut query = String::new();
        for g in galaxies {
            let key_syms_json = serde_json::to_string(&g.key_symbols).unwrap_or_else(|_| "[]".to_string());
            let langs_json = serde_json::to_string(&g.languages).unwrap_or_else(|_| "[]".to_string());
            let id_str = format!("{}:{}", g.workspace, g.galaxy_id);
            let esc_id = surql_escape(&id_str);
            let esc_name = surql_escape(&g.name);
            let esc_dom = surql_escape(&g.dominant_path);
            let esc_role = surql_escape(&g.role);

            query.push_str(&format!(
                "UPSERT type::thing('galaxy', '{}') CONTENT {{ \
                    workspace: '{}', galaxy_id: {}, name: '{}', dominant_path: '{}', \
                    node_count: {}, internal_edges: {}, external_edges: {}, \
                    afferent_coupling: {}, efferent_coupling: {}, instability: {}, \
                    role: '{}', key_symbols: {}, languages: {}, updated_at: time::now() \
                }};\n",
                esc_id, esc_ws, g.galaxy_id, esc_name, esc_dom,
                g.node_count, g.internal_edges, g.external_edges,
                g.afferent_coupling, g.efferent_coupling, g.instability,
                esc_role, key_syms_json, langs_json
            ));
        }

        self.query_sql(&query).await?;
        info!("Persisted {} architectural galaxy records for '{}'", galaxies.len(), workspace);
        Ok(())
    }

    /// Retrieve stored architectural galaxy subsystem records
    pub async fn get_galaxies_records(&self, workspace: Option<&str>) -> Result<Vec<GalaxyRecord>, String> {
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("WHERE workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let q = format!("SELECT * FROM galaxy {} ORDER BY node_count DESC;", ws_filter);
        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                if let Ok(rec) = serde_json::from_value::<GalaxyRecord>(item.clone()) {
                    results.push(rec);
                }
            }
        }
        Ok(results)
    }

    /// Look up existing community ID for a given file
    pub async fn get_file_community(&self, workspace: &str, file_path: &str) -> Result<Option<i32>, String> {
        let esc_ws = surql_escape(workspace);
        let esc_path = surql_escape(file_path);
        let q = format!(
            "SELECT VALUE community FROM node WHERE workspace = '{}' AND file_path = '{}' AND community IS NOT NONE LIMIT 1;",
            esc_ws, esc_path
        );
        let resp = self.query_sql(&q).await?;
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            if let Some(val) = arr.first().and_then(|v| v.as_i64()) {
                return Ok(Some(val as i32));
            }
        }
        Ok(None)
    }

    /// Retrieve symbol architectural boundary contract and cross-galaxy blast radius
    pub async fn get_symbol_boundary(&self, symbol: &str, workspace: Option<&str>) -> Result<Option<SymbolBoundaryInfo>, String> {
        let symbols = self.find_symbols(symbol, workspace).await?;
        let target = match symbols.into_iter().next() {
            Some(s) => s,
            None => return Ok(None),
        };

        let target_ws = target.workspace.clone().unwrap_or_else(|| workspace.unwrap_or("default").to_string());
        let target_community = target.community;

        // Fetch callers of the target symbol
        let callers = self.find_references(symbol, Some(&target_ws)).await.unwrap_or_default();

        // Fetch galaxy records for context
        let galaxies = self.get_galaxies_records(Some(&target_ws)).await.unwrap_or_default();
        let galaxy_map: HashMap<i32, &GalaxyRecord> = galaxies.iter().map(|g| (g.galaxy_id, g)).collect();

        let home_galaxy = target_community.and_then(|cid| {
            galaxy_map.get(&cid).map(|g| GalaxySubsystemRef {
                id: g.galaxy_id,
                name: g.name.clone(),
                dominant_path: g.dominant_path.clone(),
                instability: g.instability,
                role: g.role.clone(),
            })
        });

        let mut internal_callers = Vec::new();
        let mut cross_galaxy_callers = Vec::new();

        for c in callers {
            let caller_cid = c.community;
            let caller_gname = caller_cid.and_then(|id| galaxy_map.get(&id).map(|g| g.name.clone()));
            let caller_ref = CallerRef {
                symbol: c.label.clone(),
                kind: c.kind.clone(),
                file_path: c.file_path.clone(),
                line_start: c.line_start,
                galaxy_id: caller_cid,
                galaxy_name: caller_gname,
            };

            if caller_cid == target_community && target_community.is_some() {
                internal_callers.push(caller_ref);
            } else {
                cross_galaxy_callers.push(caller_ref);
            }
        }

        let is_exported = true; // AST level symbol
        let internal_cnt = internal_callers.len();
        let cross_cnt = cross_galaxy_callers.len();

        let (arch_status, risk_level, summary, rule_of_thumb) = if cross_cnt > 0 {
            let foreign_galaxies: HashSet<String> = cross_galaxy_callers
                .iter()
                .filter_map(|c| c.galaxy_name.clone().or_else(|| c.galaxy_id.map(|id| format!("Galaxy #{}", id))))
                .collect();
            let foreign_list = foreign_galaxies.into_iter().collect::<Vec<_>>().join(", ");
            let risk = if cross_cnt >= 3 || foreign_list.contains(',') { "HIGH" } else { "MEDIUM" };
            (
                "BOUNDARY_CROSSING".to_string(),
                risk.to_string(),
                format!("Symbol '{}' is called by {} foreign components across boundary subsystems: {}.", symbol, cross_cnt, foreign_list),
                format!("⚠️ ARCHITECTURAL CONTRACT: Modifying this symbol signature requires coordinated updates across external subsystems ({}). Internal callers ({}) within the home galaxy are safe.", foreign_list, internal_cnt),
            )
        } else if internal_cnt > 0 {
            (
                "INTERNAL_ONLY".to_string(),
                "LOW".to_string(),
                format!("Symbol '{}' has {} internal callers strictly contained within its home galaxy.", symbol, internal_cnt),
                "✅ INTERNAL SUBSYSTEM DETAIL: Change is 100% contained within the home galaxy. Safe to refactor without breaking foreign architectural boundaries.".to_string(),
            )
        } else {
            (
                "ISOLATED".to_string(),
                "LOW".to_string(),
                format!("Symbol '{}' has zero recorded callers in the AST graph.", symbol),
                "ℹ️ ISOLATED COMPONENT: No callers found in graph. Check if this is an external API entry point or unreferenced code.".to_string(),
            )
        };

        Ok(Some(SymbolBoundaryInfo {
            symbol: target.label,
            kind: target.kind,
            file_path: target.file_path,
            line_start: target.line_start,
            workspace: target_ws,
            home_galaxy,
            containment: ContainmentStatus {
                is_exported,
                internal_callers_count: internal_cnt,
                cross_galaxy_callers_count: cross_cnt,
                architectural_status: arch_status,
            },
            internal_callers,
            cross_galaxy_callers,
            agent_actionable_advice: AgentBoundaryAdvice {
                risk_level,
                summary,
                rule_of_thumb,
            },
        }))
    }

    /// Retrieve cross-galaxy architectural dependency edges
    pub async fn get_inter_galaxy_dependencies(
        &self,
        workspace: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("WHERE workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let clause = if ws_filter.is_empty() {
            "WHERE".to_string()
        } else {
            format!("{} AND", ws_filter)
        };
        let q = format!(
            "SELECT in.community AS from_galaxy, out.community AS to_galaxy, in.label AS source_symbol, out.label AS target_symbol, type AS edge_type \
             FROM linked_to {} in.community IS NOT NONE AND out.community IS NOT NONE AND in.community != out.community LIMIT 200;",
            clause
        );
        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                results.push(item.clone());
            }
        }
        Ok(results)
    }

    /// Agent Relationship Augmentation: Dynamically link symbols with inferred runtime relationships
    pub async fn add_relationship(
        &self,
        workspace: &str,
        source_symbol: &str,
        target_symbol: &str,
        rel_type: &str,
        category: &str,
        metadata: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        let ws_esc = surql_escape(workspace);
        let src_esc = surql_escape(source_symbol);
        let tgt_esc = surql_escape(target_symbol);
        let type_esc = surql_escape(rel_type);
        let cat_esc = surql_escape(category);
        let meta_str = match metadata {
            Some(v) if !v.is_null() => serde_json::to_string(v).unwrap_or_else(|_| "NONE".to_string()),
            _ => "NONE".to_string(),
        };
        let zero_emb = serde_json::to_string(&vec![0.0f32; 384]).unwrap();

        let q = format!(
            "LET $src_list = (SELECT VALUE id FROM node WHERE workspace = '{ws}' AND (label = '{src}' OR string::ends_with(label, '::{src}') OR string::ends_with(label, '.{src}')) LIMIT 1);\n\
             LET $src = IF array::len($src_list) > 0 THEN array::first($src_list) ELSE (UPSERT type::thing('node', '{ws}:virtual:{src}') MERGE {{ workspace: '{ws}', label: '{src}', kind: 'virtual_service', file_path: 'virtual', language: 'virtual', line_start: 0, line_end: 0, text: 'Virtual component: {src}', embedding: {zero_emb} }}).id END;\n\
             LET $tgt_list = (SELECT VALUE id FROM node WHERE workspace = '{ws}' AND (label = '{tgt}' OR string::ends_with(label, '::{tgt}') OR string::ends_with(label, '.{tgt}')) LIMIT 1);\n\
             LET $tgt = IF array::len($tgt_list) > 0 THEN array::first($tgt_list) ELSE (UPSERT type::thing('node', '{ws}:virtual:{tgt}') MERGE {{ workspace: '{ws}', label: '{tgt}', kind: 'endpoint', file_path: 'virtual', language: 'virtual', line_start: 0, line_end: 0, text: 'Virtual component: {tgt}', embedding: {zero_emb} }}).id END;\n\
             RELATE $src->linked_to->$tgt CONTENT {{ workspace: '{ws}', type: '{rel_type}', category: '{cat}', metadata: {meta}, created_at: time::now() }};\n",
            ws = ws_esc,
            src = src_esc,
            tgt = tgt_esc,
            rel_type = type_esc,
            cat = cat_esc,
            meta = meta_str,
            zero_emb = zero_emb
        );

        let resp = self.query_sql(&q).await?;
        if let Some(arr) = resp.as_array() {
            for item in arr.iter().rev() {
                if let Some(res_arr) = item.get("result").and_then(|r| r.as_array()) {
                    if let Some(first_edge) = res_arr.first() {
                        if let Some(id_str) = first_edge.get("id").and_then(|v| v.as_str()) {
                            return Ok(id_str.to_string());
                        }
                    }
                }
            }
        }

        Ok("linked_to:created".to_string())
    }
}

/// Smart heuristic to find where a workspace root directory lives on disk
pub fn resolve_disk_path(workspace: &str, sample_files: &[String]) -> Option<String> {
    if workspace == "tool-scripts" || workspace == "workspace" || workspace == "default" {
        if std::path::Path::new("/workspace").exists() {
            return Some("/workspace".to_string());
        }
        let host_p = "/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/tool-scripts";
        if std::path::Path::new(host_p).exists() {
            return Some(host_p.to_string());
        }
    }

    let candidates = vec![
        format!("/workspace/tools/{}", workspace),
        format!("/workspace/{}", workspace),
        format!("/workspace/../{}", workspace),
        format!("/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/{}", workspace),
        format!("/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/tool-scripts/tools/{}", workspace),
    ];

    for candidate in &candidates {
        let p = std::path::Path::new(candidate);
        if p.exists() && p.is_dir() {
            if sample_files.is_empty() {
                return Some(candidate.clone());
            }
            for sf in sample_files {
                if p.join(sf).exists() {
                    return Some(candidate.clone());
                }
            }
        }
    }

    for candidate in &candidates {
        let p = std::path::Path::new(candidate);
        if p.exists() && p.is_dir() {
            return Some(candidate.clone());
        }
    }

    None
}

