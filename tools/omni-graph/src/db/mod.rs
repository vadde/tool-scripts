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
    ) -> Result<(), String> {
        if nodes.is_empty() {
            return Ok(());
        }

        let mut query = String::new();
        for (node, emb) in nodes.iter().zip(embeddings.iter()) {
            let emb_str = serde_json::to_string(emb).unwrap_or_else(|_| "[]".to_string());
            let escaped_text = node
                .text
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
                .replace('\n', " ");
            let escaped_label = node.label.replace('\'', "\\'");
            let escaped_path = node.file_path.replace('\'', "\\'");
            let escaped_ws = node.workspace.replace('\'', "\\'");
            let escaped_id = node.id.replace('\'', "\\'");

            query.push_str(&format!(
                "UPSERT type::thing('node', '{}') CONTENT {{ workspace: '{}', label: '{}', kind: '{}', file_path: '{}', language: '{}', line_start: {}, line_end: {}, text: '{}', embedding: {} }};\n",
                escaped_id, escaped_ws, escaped_label, node.kind, escaped_path, node.language, node.line_start, node.line_end, escaped_text, emb_str
            ));
        }

        self.query_sql(&query).await?;
        Ok(())
    }

    /// Resolve and store graph relations with workspace isolation (R-006)
    pub async fn store_edges(&self, edges: &[ExtractedEdge]) -> Result<(), String> {
        if edges.is_empty() {
            return Ok(());
        }

        let mut query = String::new();
        for edge in edges {
            let escaped_ws = edge.workspace.replace('\'', "\\'");
            let escaped_label = edge.target_label.replace('\'', "\\'");
            let escaped_source = edge.source_id.replace('\'', "\\'");

            query.push_str(&format!(
                "LET $src = type::thing('node', '{}');\n\
                 LET $targets = (SELECT VALUE id FROM node WHERE workspace = '{}' AND label = '{}' LIMIT 1);\n\
                 RELATE $src->linked_to->$targets SET workspace = '{}', type = '{}', category = '{}';\n",
                escaped_source,
                escaped_ws,
                escaped_label,
                escaped_ws,
                edge.edge_type,
                edge.category
            ));
        }

        self.query_sql(&query).await?;
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
            Some(ws) if !ws.is_empty() => format!("WHERE workspace = '{}'", ws.replace('\'', "\\'")),
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
                let escaped = ws.replace('\'', "\\'");
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

    /// List all distinct workspaces with metadata
    pub async fn get_workspaces(&self) -> Result<Vec<serde_json::Value>, String> {
        let q = "SELECT workspace, count() AS total_nodes, array::distinct(language) AS languages, array::distinct(file_path) AS files FROM node GROUP BY workspace;";
        let resp = self.query_sql(q).await?;
        let mut list = Vec::new();
        if let Some(arr) = resp.as_array().and_then(|a| a.first()).and_then(|r| r.get("result")).and_then(|res| res.as_array()) {
            for item in arr {
                list.push(item.clone());
            }
        }
        Ok(list)
    }

    /// Symbolic lookup: Find symbol definitions by name (LSP textDocument/definition equivalent)
    pub async fn find_symbols(&self, name: &str, workspace: Option<&str>) -> Result<Vec<DbNode>, String> {
        let escaped = name.replace('\'', "\\'");
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("AND workspace = '{}'", ws.replace('\'', "\\'")),
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
        let escaped = symbol_name.replace('\'', "\\'");
        let ws_filter = match workspace {
            Some(ws) if !ws.is_empty() => format!("AND workspace = '{}'", ws.replace('\'', "\\'")),
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
                let escaped_id = clean_id.replace('\\', "\\\\").replace('\'', "\\'");
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
}

