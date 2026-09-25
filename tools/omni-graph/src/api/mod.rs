// Axum REST API handlers
// Implements: R-008, R-009, R-010, R-022, R-023, R-028

use crate::analysis::{CommunityDetector, GraphRagEngine};
use crate::analytics::AnalyticsEngine;
use crate::condenser::ContextCondenser;
use crate::db::DbClient;
use crate::embedder::EmbedderClient;
use crate::ingestion::IngestionPipeline;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: DbClient,
    pub embedder: EmbedderClient,
    pub pipeline: Arc<IngestionPipeline>,
}

#[derive(Deserialize, Default)]
pub struct GraphParams {
    pub workspace: Option<String>,
    pub min_size: Option<usize>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub k: Option<usize>,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct SymbolParams {
    pub name: String,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct ReferenceParams {
    pub symbol: String,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct GraphRagPayload {
    pub prompt: String,
    pub top_k: Option<usize>,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct CondenseParams {
    pub symbol: String,
    pub hops: Option<usize>,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct IngestPayload {
    pub path: String,
    pub project: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct ClusterPayload {
    pub workspace: Option<String>,
    pub min_size: Option<usize>,
}

#[derive(Deserialize, Default)]
pub struct BrowseParams {
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_codebase: bool,
    pub languages: Vec<String>,
}

#[derive(Serialize)]
pub struct BrowseResponse {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<DirEntry>,
}

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub services: ServiceHealth,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub rust_app: ServiceComponent,
    pub surrealdb: ServiceComponent,
    pub tei: ServiceComponent,
}

#[derive(Serialize)]
pub struct ServiceComponent {
    pub status: String,
    pub details: Option<String>,
}

/// Normalizes workspace argument so that both paths (/Users/.../repo) and workspace names (repo) match correctly
pub fn normalize_workspace(ws: Option<&str>) -> Option<String> {
    ws.and_then(|raw| {
        let trimmed = raw.trim().trim_end_matches('/');
        if trimmed.is_empty() {
            None
        } else if trimmed.contains('/') {
            Some(trimmed.rsplit('/').next().unwrap_or(trimmed).to_string())
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/stats", get(stats_handler))
        .route("/api/workspaces", get(workspaces_handler))
        .route("/api/graph", get(graph_handler))
        .route("/api/search", get(search_handler))
        .route("/api/symbol", get(symbol_handler))
        .route("/api/references", get(references_handler))
        .route("/api/condense", get(condense_handler))
        .route("/api/browse", get(browse_handler))
        .route("/api/query", post(query_handler))
        .route("/api/cluster", post(cluster_handler))
        .route("/api/galaxies", get(galaxies_handler))
        .route("/api/ingest", post(ingest_handler))
        .route("/api/analytics", get(analytics_handler))
        .route("/api/analytics/session/:id", get(session_detail_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// GET /api/health (R-022, AC-012)
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = state.db.health().await;
    let tei_ok = state.embedder.health().await;

    let overall = if db_ok && tei_ok { "healthy" } else { "degraded" };

    let health = HealthStatus {
        status: overall.to_string(),
        services: ServiceHealth {
            rust_app: ServiceComponent {
                status: "ok".to_string(),
                details: Some("v0.1.0".to_string()),
            },
            surrealdb: ServiceComponent {
                status: if db_ok { "ok".to_string() } else { "unreachable".to_string() },
                details: None,
            },
            tei: ServiceComponent {
                status: if tei_ok { "ok".to_string() } else { "unreachable".to_string() },
                details: Some("BAAI/bge-small-en-v1.5".to_string()),
            },
        },
    };

    Json(health)
}

/// GET /api/stats (R-023)
async fn stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/workspaces
async fn workspaces_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_workspaces().await {
        Ok(workspaces) => (StatusCode::OK, Json(workspaces)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/graph (R-008, AC-006)
async fn graph_handler(
    State(state): State<AppState>,
    Query(params): Query<GraphParams>,
) -> impl IntoResponse {
    let ws = normalize_workspace(params.workspace.as_deref());
    match state.db.get_graph(ws.as_deref()).await {
        Ok((nodes, links)) => {
            let total_nodes = nodes.len();
            let total_links = links.len();
            Json(serde_json::json!({
                "nodes": nodes,
                "links": links,
                "stats": {
                    "total_nodes": total_nodes,
                    "total_links": total_links
                }
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/galaxies?workspace=<ws>&min_size=<n> — Query architectural subsystems without re-clustering
async fn galaxies_handler(
    State(state): State<AppState>,
    Query(params): Query<GraphParams>,
) -> impl IntoResponse {
    let ws = normalize_workspace(params.workspace.as_deref());
    let min_size = params.min_size.unwrap_or(1);

    // Try server-side aggregation first for instant response without loading full topology (Finding #7)
    if let Ok(agg_rows) = state.db.get_galaxy_aggregation(ws.as_deref()).await {
        if !agg_rows.is_empty() {
            let mut galaxies: Vec<serde_json::Value> = Vec::new();
            for row in agg_rows {
                let cid = row.get("community").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let count = row.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                if count < min_size {
                    continue; // Skip singletons/small clusters if min_size filter applied (Finding #6)
                }

                let files: Vec<String> = row.get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str| str.to_string())).collect())
                    .unwrap_or_default();

                let mut dir_counts: HashMap<String, usize> = HashMap::new();
                for f in &files {
                    if let Some(parent) = std::path::Path::new(f).parent() {
                        let p = parent.to_string_lossy().to_string();
                        if !p.is_empty() && p != "." {
                            *dir_counts.entry(p).or_insert(0) += 1;
                        }
                    }
                }
                let dominant = dir_counts
                    .into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(d, _)| d)
                    .unwrap_or_else(|| "root".to_string());

                let sample_symbols: Vec<String> = row.get("sample_symbols")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str| str.to_string())).collect())
                    .unwrap_or_default();

                let languages: Vec<String> = row.get("languages")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str| str.to_string())).collect())
                    .unwrap_or_default();

                galaxies.push(serde_json::json!({
                    "id": cid,
                    "name": dominant.clone(),
                    "dominant_path": dominant,
                    "node_count": count,
                    "languages": languages,
                    "sample_symbols": sample_symbols
                }));
            }

            galaxies.sort_by(|a, b| {
                b.get("node_count")
                    .and_then(|v| v.as_u64())
                    .cmp(&a.get("node_count").and_then(|v| v.as_u64()))
            });

            return Json(serde_json::json!({
                "total_galaxies": galaxies.len(),
                "workspace": ws,
                "galaxies": galaxies
            }))
            .into_response();
        }
    }

    // Fallback: load graph and aggregate in-memory
    match state.db.get_graph(ws.as_deref()).await {
        Ok((nodes, _links)) => {
            let mut comm_map: HashMap<i32, Vec<crate::db::DbNode>> = HashMap::new();
            for node in &nodes {
                if let Some(cid) = node.community {
                    comm_map.entry(cid).or_default().push(node.clone());
                }
            }

            let mut galaxies: Vec<serde_json::Value> = if !comm_map.is_empty() {
                comm_map
                    .into_iter()
                    .filter(|(_, cluster_nodes)| cluster_nodes.len() >= min_size)
                    .map(|(cid, cluster_nodes)| {
                        let mut dir_counts: HashMap<String, usize> = HashMap::new();
                        let mut languages: HashSet<String> = HashSet::new();
                        for n in &cluster_nodes {
                            if let Some(parent) = std::path::Path::new(&n.file_path).parent() {
                                let p = parent.to_string_lossy().to_string();
                                if !p.is_empty() && p != "." {
                                    *dir_counts.entry(p).or_insert(0) += 1;
                                }
                            }
                            if !n.language.is_empty() {
                                languages.insert(n.language.clone());
                            }
                        }
                        let dominant = dir_counts
                            .into_iter()
                            .max_by_key(|(_, c)| *c)
                            .map(|(d, _)| d)
                            .unwrap_or_else(|| "root".to_string());
                        let name = if cluster_nodes.len() == 1 {
                            format!("{}: {} • {}", dominant, cluster_nodes[0].kind, cluster_nodes[0].label)
                        } else {
                            dominant.clone()
                        };

                        let sample_symbols: Vec<String> =
                            cluster_nodes.iter().take(6).map(|n| n.label.clone()).collect();
                        let langs: Vec<String> = languages.into_iter().collect();

                        serde_json::json!({
                            "id": cid,
                            "name": name,
                            "dominant_path": dominant,
                            "node_count": cluster_nodes.len(),
                            "languages": langs,
                            "sample_symbols": sample_symbols
                        })
                    })
                    .collect()
            } else if !nodes.is_empty() {
                // Fallback to structural directory subsystems when Louvain/Leiden communities are not yet computed
                let mut dir_map: HashMap<String, Vec<&crate::db::DbNode>> = HashMap::new();
                for node in &nodes {
                    let parent = std::path::Path::new(&node.file_path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .filter(|p| !p.is_empty() && p != ".")
                        .unwrap_or_else(|| "root".to_string());
                    dir_map.entry(parent).or_default().push(node);
                }

                dir_map
                    .into_iter()
                    .enumerate()
                    .map(|(idx, (dir, dir_nodes))| {
                        let mut langs: HashSet<String> = HashSet::new();
                        let mut sample_syms: Vec<String> = Vec::new();
                        for n in &dir_nodes {
                            if !n.language.is_empty() { langs.insert(n.language.clone()); }
                            if sample_syms.len() < 6 { sample_syms.push(n.label.clone()); }
                        }
                        serde_json::json!({
                            "id": (idx + 1) as i32,
                            "name": format!("Subsystem: {}", dir),
                            "dominant_path": dir,
                            "node_count": dir_nodes.len(),
                            "languages": langs.into_iter().collect::<Vec<_>>(),
                            "sample_symbols": sample_syms
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };

            galaxies.sort_by(|a, b| {
                b.get("node_count")
                    .and_then(|v| v.as_u64())
                    .cmp(&a.get("node_count").and_then(|v| v.as_u64()))
            });

            Json(serde_json::json!({
                "total_galaxies": galaxies.len(),
                "workspace": ws,
                "galaxies": galaxies
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/search?q=<query>&k=<n>&workspace=<ws> (R-009, AC-005)
async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let k = params.k.unwrap_or(10);
    let start = std::time::Instant::now();

    // 1. Vectorize query via TEI
    let query_vec = match state.embedder.embed_single(&params.q).await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("TEI embedding failed: {}", e) })),
            )
                .into_response()
        }
    };

    // 2. Perform HNSW cosine similarity search
    let ws = normalize_workspace(params.workspace.as_deref());
    match state.db.search_vector(&query_vec, k, ws.as_deref()).await {
        Ok(results) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            Json(serde_json::json!({
                "query": params.q,
                "workspace": ws,
                "results": results,
                "total_results": results.len(),
                "search_latency_ms": latency_ms
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/condense?symbol=<sym>&hops=<n>&workspace=<ws> (R-028, AC-018)
async fn condense_handler(
    State(state): State<AppState>,
    Query(params): Query<CondenseParams>,
) -> impl IntoResponse {
    let hops = params.hops.unwrap_or(2);
    let ws = normalize_workspace(params.workspace.as_deref());
    match state.db.get_graph(ws.as_deref()).await {
        Ok((nodes, links)) => {
            let condensed = ContextCondenser::condense(&params.symbol, &nodes, &links, hops);
            Json(condensed).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// POST /api/ingest (R-010, AC-009)
async fn ingest_handler(
    State(state): State<AppState>,
    Json(payload): Json<IngestPayload>,
) -> impl IntoResponse {
    let path = payload.path;
    let project = payload.project;
    info!("Ingest request received for path: {}, project: {:?}", path, project);

    match state.pipeline.ingest_directory(&path, project.as_deref()).await {
        Ok(res) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "result": res
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": e
            })),
        )
            .into_response(),
    }
}

/// GET /api/symbol?name=<sym>&workspace=<ws> (Symbolic definition lookup — LSP textDocument/definition)
async fn symbol_handler(
    State(state): State<AppState>,
    Query(params): Query<SymbolParams>,
) -> impl IntoResponse {
    let ws = normalize_workspace(params.workspace.as_deref());
    match state.db.find_symbols(&params.name, ws.as_deref()).await {
        Ok(symbols) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "symbol": params.name,
                "workspace": ws,
                "matches": symbols,
                "count": symbols.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// GET /api/references?symbol=<sym>&workspace=<ws> (Symbolic reference lookup — LSP textDocument/references)
async fn references_handler(
    State(state): State<AppState>,
    Query(params): Query<ReferenceParams>,
) -> impl IntoResponse {
    let ws = normalize_workspace(params.workspace.as_deref());
    match state.db.find_references(&params.symbol, ws.as_deref()).await {
        Ok(callers) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "symbol": params.symbol,
                "workspace": ws,
                "references": callers,
                "count": callers.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// POST /api/query (Hybrid Graph-RAG Retrieval: Vector Seeds + AST Traversal + Community Summaries)
async fn query_handler(
    State(state): State<AppState>,
    Json(payload): Json<GraphRagPayload>,
) -> impl IntoResponse {
    let top_k = payload.top_k.unwrap_or(5);
    let ws = normalize_workspace(payload.workspace.as_deref());
    match GraphRagEngine::query(
        &state.db,
        &state.embedder,
        &payload.prompt,
        top_k,
        ws.as_deref(),
    )
    .await
    {
        Ok(rag_res) => (StatusCode::OK, Json(rag_res)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// POST /api/cluster (Run community detection on graph and store community IDs)
async fn cluster_handler(
    State(state): State<AppState>,
    payload_opt: Option<Json<ClusterPayload>>,
) -> impl IntoResponse {
    let raw_ws = payload_opt.as_ref().and_then(|p| p.workspace.clone());
    let min_size = payload_opt.as_ref().and_then(|p| p.min_size).unwrap_or(1);
    let workspace = normalize_workspace(raw_ws.as_deref());
    match state.db.get_graph(workspace.as_deref()).await {
        Ok((nodes, links)) => {
            let assignments = CommunityDetector::detect(&nodes, &links, 15);
            let summaries = CommunityDetector::summarize_filtered(&nodes, &assignments, min_size);
            let total_communities = summaries.len();

            if let Err(e) = state.db.update_communities(&assignments).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to update communities: {}", e) })),
                )
                    .into_response();
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "workspace": workspace,
                    "total_communities": total_communities,
                    "communities": summaries
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Allowlist of browseable root directories.
/// Set BROWSE_ROOTS env var to override (comma-separated), e.g. BROWSE_ROOTS=/workspace,/Users/aparv
fn allowed_browse_roots() -> Vec<std::path::PathBuf> {
    if let Ok(roots) = std::env::var("BROWSE_ROOTS") {
        return roots
            .split(',')
            .map(|s| std::path::PathBuf::from(s.trim()))
            .filter(|p| p.exists())
            .collect();
    }
    // Default: /workspace (Docker) and /Users (macOS host mount)
    let mut roots = Vec::new();
    if std::path::Path::new("/workspace").exists() {
        roots.push(std::path::PathBuf::from("/workspace"));
    }
    if std::path::Path::new("/Users").exists() {
        roots.push(std::path::PathBuf::from("/Users"));
    }
    if roots.is_empty() {
        // Fallback to cwd if nothing else exists
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd);
        }
    }
    roots
}

/// Check if a canonical path is under any allowed browse root
fn is_path_allowed(canonical: &std::path::Path, roots: &[std::path::PathBuf]) -> bool {
    roots.iter().any(|root| {
        if let Ok(canon_root) = root.canonicalize() {
            canonical.starts_with(&canon_root)
        } else {
            canonical.starts_with(root)
        }
    })
}

/// GET /api/browse?path=<dir> (Dynamic filesystem directory traversal for UI)
/// Security: paths are canonicalized and checked against an allowlist of browse roots.
async fn browse_handler(
    Query(params): Query<BrowseParams>,
) -> impl IntoResponse {
    let roots = allowed_browse_roots();

    let raw_path = params.path.unwrap_or_else(|| {
        roots.first()
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let current_dir = std::path::PathBuf::from(&raw_path);
    if !current_dir.exists() || !current_dir.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Path does not exist or is not a directory: {}", raw_path) })),
        )
            .into_response();
    }

    // Canonicalize to resolve symlinks and normalize the path
    let canonical = match current_dir.canonicalize() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Failed to resolve path" })),
            )
                .into_response();
        }
    };

    // Security: verify the resolved path is under an allowed root
    if !is_path_allowed(&canonical, &roots) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": format!("Access denied: path '{}' is outside allowed browse roots", raw_path),
                "allowed_roots": roots.iter().map(|r| r.to_string_lossy().to_string()).collect::<Vec<_>>()
            })),
        )
            .into_response();
    }

    // Only expose parent_path if it stays within an allowed root
    let parent_path = canonical.parent().and_then(|p| {
        if is_path_allowed(p, &roots) {
            Some(p.to_string_lossy().to_string())
        } else {
            None // At root boundary — don't navigate up
        }
    });

    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(&canonical) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip noisy hidden or build directories
            if name.starts_with('.') && name != ".github" {
                continue;
            }
            if name == "node_modules" || name == "target" || name == "dist" || name == "build" {
                continue;
            }

            let is_dir = path.is_dir();
            if !is_dir {
                continue; // Only browse directories for codebase selection
            }

            // Check if it's a codebase / project folder
            let mut is_codebase = false;
            let mut languages = Vec::new();

            if path.join("Cargo.toml").exists() {
                is_codebase = true;
                languages.push("Rust".to_string());
            }
            if path.join("package.json").exists() {
                is_codebase = true;
                languages.push("TypeScript/JS".to_string());
            }
            if path.join("go.mod").exists() {
                is_codebase = true;
                languages.push("Go".to_string());
            }
            if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
                is_codebase = true;
                languages.push("Python".to_string());
            }
            if path.join(".git").exists() || path.join("Makefile").exists() {
                is_codebase = true;
            }

            entries.push(DirEntry {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: true,
                is_codebase,
                languages,
            });
        }
    }

    // Sort codebases first, then alphabetically
    entries.sort_by(|a, b| {
        b.is_codebase
            .cmp(&a.is_codebase)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Json(BrowseResponse {
        current_path: canonical.to_string_lossy().to_string(),
        parent_path,
        entries,
    })
    .into_response()
}

/// GET /api/analytics
async fn analytics_handler() -> impl IntoResponse {
    let resp = AnalyticsEngine::scan_analytics();
    Json(resp).into_response()
}

/// GET /api/analytics/session/:id
async fn session_detail_handler(
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    match AnalyticsEngine::get_session_detail(&id) {
        Some(detail) => Json(detail).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Session {} not found", id)
            })),
        )
            .into_response(),
    }
}


