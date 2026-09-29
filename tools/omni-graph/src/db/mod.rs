// SurrealDB v2 Client & Repository
// Implements: R-005, R-006, R-007, R-008, R-009, R-019, R-020, R-022, R-023

use crate::config::Config;
use crate::parser::{ExtractedEdge, ExtractedNode};
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
            .timeout(Duration::from_secs(30))
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
        if nodes.is_empty() {
            return Ok(());
        }

        let hash_field = match file_hash {
            Some(h) => format!(", file_hash: '{}'", surql_escape(h)),
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
                let escaped_id = surql_escape(&node.id);
                let escaped_kind = surql_escape(&node.kind);
                let escaped_lang = surql_escape(&node.language);

                query.push_str(&format!(
                    "UPSERT type::thing('node', '{}') CONTENT {{ workspace: '{}', label: '{}', kind: '{}', file_path: '{}', language: '{}', line_start: {}, line_end: {}, text: '{}', embedding: {}{} }};\n",
                    escaped_id, escaped_ws, escaped_label, escaped_kind, escaped_path, escaped_lang, node.line_start, node.line_end, escaped_text, emb_str, hash_field
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
                let escaped_source = surql_escape(&edge.source_id);
                let escaped_type = surql_escape(&edge.edge_type);
                let escaped_cat = surql_escape(&edge.category);

                query.push_str(&format!(
                    "LET $src = type::thing('node', '{}');\n\
                     LET $targets = (SELECT VALUE id FROM node WHERE workspace = '{}' AND label = '{}' LIMIT 1);\n\
                     RELATE $src->linked_to->$targets SET workspace = '{}', type = '{}', category = '{}';\n",
                    escaped_source,
                    escaped_ws,
                    escaped_label,
                    escaped_ws,
                    escaped_type,
                    escaped_cat
                ));
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
            "SELECT id, workspace, label, kind, file_path, language, text, community, \
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
            if let Some(node_arr) = results.get(0).and_then(|r| r.get("result")).and_then(|v| v.as_array()) {
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
            if let Some(n) = arr.get(0).and_then(|r| r.get("result")).and_then(|res| res.as_array()).and_then(|a| a.first()).and_then(|o| o.get("count")).and_then(|c| c.as_i64()) {
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

    /// Record the root filesystem path for a workspace
    pub async fn record_workspace_root(&self, workspace: &str, root_path: &str) -> Result<(), String> {
        let esc_ws = surql_escape(workspace);
        let esc_path = surql_escape(root_path);
        let q = format!(
            "UPSERT type::thing('workspace_meta', '{}') CONTENT {{ workspace: '{}', root_path: '{}', updated_at: time::now() }};",
            esc_ws, esc_ws, esc_path
        );
        self.query_sql(&q).await?;
        Ok(())
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

    /// List all distinct workspaces with metadata (including root_path)
    pub async fn get_workspaces(&self) -> Result<Vec<serde_json::Value>, String> {
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
                        let mut resolved_root: Option<String> = self.get_workspace_root(&canonical_ws).await.unwrap_or(None);
                        
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
                        map.insert(canonical_ws, obj);
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
            "SELECT id, workspace, label, kind, file_path, language, line_start, line_end, text, community \
             FROM node WHERE (label = '{}' OR label CONTAINS '{}') {} LIMIT 20;",
            escaped, escaped, ws_filter
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

        Ok(results)
    }

    /// Symbolic references: Find all callers / references of a symbol (LSP textDocument/references equivalent)
    pub async fn find_references(&self, symbol_name: &str, workspace: Option<&str>) -> Result<Vec<DbNode>, String> {
        let escaped = surql_escape(symbol_name);
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("AND workspace = '{}'", surql_escape(ws)),
            _ => String::new(),
        };
        let q = format!(
            "LET $targets = (SELECT id FROM node WHERE label = '{}' {});\n\
             SELECT in.* AS caller FROM linked_to WHERE out IN $targets.id;\n",
            escaped, ws_filter
        );

        let resp = self.query_sql(&q).await?;
        let mut results = Vec::new();

        if let Some(arr) = resp.as_array() {
            if let Some(res_arr) = arr.get(1).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
                for item in res_arr {
                    if let Some(caller) = item.get("caller") {
                        if let Ok(node) = serde_json::from_value::<DbNode>(caller.clone()) {
                            results.push(node);
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
                let clean_id = node_id
                    .strip_prefix("node:")
                    .unwrap_or(node_id)
                    .trim_matches('`')
                    .trim_matches('⟨')
                    .trim_matches('⟩')
                    .trim_matches('"')
                    .trim_matches('\'');
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

    /// Atomic prune of file nodes and connected edges (R-011 / Delta Sync)
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

