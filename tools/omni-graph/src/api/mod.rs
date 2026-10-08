// Axum REST API handlers
// Implements: R-008, R-009, R-010, R-022, R-023, R-028

use crate::analysis::{CommunityDetector, GraphRagEngine};
use crate::analytics::AnalyticsEngine;
use crate::condenser::ContextCondenser;
use crate::db::{DbClient, RelationshipPayload};
use crate::embedder::EmbedderClient;
use crate::ingestion::IngestionPipeline;
use crate::watcher::WatchManager;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: DbClient,
    pub embedder: EmbedderClient,
    pub pipeline: Arc<IngestionPipeline>,
    pub watcher: Arc<WatchManager>,
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
    pub refresh: Option<bool>,
}

#[derive(Deserialize)]
pub struct IngestDismissPayload {
    pub workspace: String,
}

#[derive(Deserialize)]
pub struct WatchStartPayload {
    pub path: String,
    pub project: Option<String>,
    pub workspace: Option<String>,
    pub debounce_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct WatchStopPayload {
    pub workspace: String,
}

#[derive(Deserialize, Default)]
pub struct ClusterPayload {
    pub workspace: Option<String>,
    pub min_size: Option<usize>,
}

#[derive(Deserialize)]
pub struct GalaxyBoundaryParams {
    pub symbol: String,
    pub workspace: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct GalaxyTopologyParams {
    pub workspace: Option<String>,
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
        .route("/api/relationships", post(relationships_handler))
        .route("/api/relation", post(relationships_handler))
        .route("/api/condense", get(condense_handler))
        .route("/api/browse", get(browse_handler))
        .route("/api/query", post(query_handler))
        .route("/api/cluster", post(cluster_handler))
        .route("/api/galaxies", get(galaxies_handler))
        .route("/api/galaxy/boundary", get(galaxy_boundary_handler))
        .route("/api/galaxy/topology", get(galaxy_topology_handler))
        .route("/api/ingest", post(ingest_handler))
        .route("/api/ingest/status", get(ingest_status_handler))
        .route("/api/ingest/dismiss", post(ingest_dismiss_handler))
        .route("/api/analytics", get(analytics_handler))
        .route("/api/analytics/session/:id", get(session_detail_handler))
        .route("/api/watch/start", post(watch_start_handler))
        .route("/api/watch/stop", post(watch_stop_handler))
        .route("/api/watch/status", get(watch_status_handler))
        .route("/api/watch/status/:workspace", get(watch_workspace_status_handler))
        .route("/api/watch/events", get(watch_events_sse_handler))
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
    let start = std::time::Instant::now();
    let ws = normalize_workspace(params.workspace.as_deref());
    let min_size = params.min_size.unwrap_or(1);

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    tokio::spawn(async move {
        let duration_ms = start.elapsed().as_millis() as i64;
        let _ = db.record_agent_api_call(
            "/api/galaxies",
            &ws_log,
            "Omni-Graph: Architectural Galaxy Subsystems",
            None,
            None,
            duration_ms,
        ).await;
    });

    // 1. Try pre-computed galaxy table records first (R-042)
    if let Ok(records) = state.db.get_galaxies_records(ws.as_deref()).await {
        if !records.is_empty() {
            let mut galaxies: Vec<serde_json::Value> = records
                .into_iter()
                .filter(|g| g.node_count >= min_size)
                .map(|g| serde_json::json!({
                    "id": g.galaxy_id,
                    "name": g.name,
                    "dominant_path": g.dominant_path,
                    "node_count": g.node_count,
                    "role": g.role,
                    "instability": g.instability,
                    "afferent_coupling": g.afferent_coupling,
                    "efferent_coupling": g.efferent_coupling,
                    "languages": g.languages,
                    "sample_symbols": g.key_symbols
                }))
                .collect();

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
    let res = state.db.search_vector(&query_vec, k, ws.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let q_str = params.q.clone();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/search",
            &ws_log,
            "Omni-Graph: Vector Semantic Search",
            Some(&q_str),
            None,
            duration_ms,
        ).await;
    });

    match res {
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
    let start = std::time::Instant::now();
    let hops = params.hops.unwrap_or(2);
    let ws = normalize_workspace(params.workspace.as_deref());
    let res = state.db.get_symbol_subgraph(&params.symbol, ws.as_deref(), hops).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let sym_str = params.symbol.clone();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/condense",
            &ws_log,
            "Omni-Graph: Multi-Hop Subgraph Condenser",
            Some(&sym_str),
            None,
            duration_ms,
        ).await;
    });

    match res {
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
    let start = std::time::Instant::now();
    let path = payload.path;
    let project = payload.project;
    let refresh = payload.refresh.unwrap_or(false);
    info!("Ingest request received for path: {}, project: {:?}, refresh: {}", path, project, refresh);

    let pipeline = state.pipeline.clone();
    let path_clone = path.clone();
    let project_clone = project.clone();

    // Run ingestion in a spawned task so client aborts/timeouts do not cancel mid-flight
    let join_handle = tokio::spawn(async move {
        pipeline.ingest_directory(&path_clone, project_clone.as_deref(), refresh).await
    });

    let res = match join_handle.await {
        Ok(inner_res) => inner_res,
        Err(e) => Err(format!("Ingestion task terminated unexpectedly: {}", e)),
    };
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = project.clone().unwrap_or_else(|| "default".to_string());
    let path_str = path.clone();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/ingest",
            &ws_log,
            "Omni-Graph: Codebase AST Ingestion",
            Some(&path_str),
            None,
            duration_ms,
        ).await;
    });

    match res {
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

/// GET /api/ingest/status
async fn ingest_status_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let statuses = state.pipeline.get_status().await;
    Json(statuses).into_response()
}

/// POST /api/ingest/dismiss
async fn ingest_dismiss_handler(
    State(state): State<AppState>,
    Json(payload): Json<IngestDismissPayload>,
) -> impl IntoResponse {
    let ws = normalize_workspace(Some(&payload.workspace)).unwrap_or(payload.workspace);
    state.pipeline.dismiss_job(&ws).await;
    StatusCode::OK.into_response()
}

/// GET /api/symbol?name=<sym>&workspace=<ws> (Symbolic definition lookup — LSP textDocument/definition)
async fn symbol_handler(
    State(state): State<AppState>,
    Query(params): Query<SymbolParams>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let ws = normalize_workspace(params.workspace.as_deref());
    let res = state.db.find_symbols(&params.name, ws.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let sym_name = params.name.clone();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/symbol",
            &ws_log,
            "Omni-Graph: AST Definition (LSP)",
            Some(&sym_name),
            None,
            duration_ms,
        ).await;
    });

    match res {
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
    let start = std::time::Instant::now();
    let ws = normalize_workspace(params.workspace.as_deref());
    let res = state.db.find_references(&params.symbol, ws.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let sym_name = params.symbol.clone();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/references",
            &ws_log,
            "Omni-Graph: Call Graph References (LSP)",
            Some(&sym_name),
            None,
            duration_ms,
        ).await;
    });

    match res {
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

/// POST /api/relationships or POST /api/relation (Agent Relationship Augmentation)
async fn relationships_handler(
    State(state): State<AppState>,
    Json(payload): Json<RelationshipPayload>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let ws = normalize_workspace(Some(&payload.workspace)).unwrap_or_else(|| payload.workspace.clone());

    if payload.source_symbol.trim().is_empty() || payload.target_symbol.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Both source_symbol and target_symbol must be non-empty"
            })),
        )
            .into_response();
    }

    let res = state
        .db
        .add_relationship(
            &ws,
            &payload.source_symbol,
            &payload.target_symbol,
            &payload.rel_type,
            &payload.category,
            payload.metadata.as_ref(),
        )
        .await;

    let duration_ms = start.elapsed().as_millis() as i64;
    let db = state.db.clone();
    let ws_log = ws.clone();
    let query_summary = format!("{} -> {}", payload.source_symbol, payload.target_symbol);

    tokio::spawn(async move {
        let _ = db
            .record_agent_api_call(
                "/api/relationships",
                &ws_log,
                "Omni-Graph: Dynamic Relationship Augmentation",
                Some(&query_summary),
                None,
                duration_ms,
            )
            .await;
    });

    match res {
        Ok(rel_id) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "relationship_id": rel_id,
                "source": payload.source_symbol,
                "target": payload.target_symbol,
                "type": payload.rel_type,
                "category": payload.category
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
    let start = std::time::Instant::now();
    let top_k = payload.top_k.unwrap_or(5);
    let ws = normalize_workspace(payload.workspace.as_deref());
    let res = GraphRagEngine::query(
        &state.db,
        &state.embedder,
        &payload.prompt,
        top_k,
        ws.as_deref(),
    )
    .await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let prompt_preview = payload.prompt.chars().take(80).collect::<String>();
    tokio::spawn(async move {
        let _ = db.record_agent_api_call(
            "/api/query",
            &ws_log,
            "Omni-Graph: Hybrid Graph-RAG",
            Some(&prompt_preview),
            None,
            duration_ms,
        ).await;
    });

    match res {
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
    let start = std::time::Instant::now();
    let raw_ws = payload_opt.as_ref().and_then(|p| p.workspace.clone());
    let min_size = payload_opt.as_ref().and_then(|p| p.min_size).unwrap_or(1);
    let workspace = normalize_workspace(raw_ws.as_deref());

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = workspace.clone().unwrap_or_else(|| "default".to_string());
    tokio::spawn(async move {
        let duration_ms = start.elapsed().as_millis() as i64;
        let _ = db.record_agent_api_call(
            "/api/cluster",
            &ws_log,
            "Omni-Graph: Modularity Community Clustering",
            None,
            None,
            duration_ms,
        ).await;
    });

    match state.db.get_graph(workspace.as_deref()).await {
        Ok((nodes, links)) => {
            let ws_name = workspace.clone().unwrap_or_else(|| "default".to_string());
            let assignments = CommunityDetector::detect(&nodes, &links, 15);
            let (summaries, galaxy_records) = CommunityDetector::compute_galaxy_metrics(&ws_name, &nodes, &links, &assignments);
            let filtered_summaries: Vec<_> = summaries.into_iter().filter(|s| s.node_count >= min_size).collect();
            let total_communities = filtered_summaries.len();

            if let Err(e) = state.db.update_communities(&assignments).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to update communities: {}", e) })),
                )
                    .into_response();
            }

            let _ = state.db.store_galaxies(&ws_name, &galaxy_records).await;

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "workspace": workspace,
                    "total_communities": total_communities,
                    "communities": filtered_summaries,
                    "galaxies": galaxy_records
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

/// GET /api/galaxy/boundary?symbol=<sym>&workspace=<ws>
/// Returns architectural subsystem containment, caller contracts, and cross-boundary blast radius risk
async fn galaxy_boundary_handler(
    State(state): State<AppState>,
    Query(params): Query<GalaxyBoundaryParams>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let ws = normalize_workspace(params.workspace.as_deref());
    let sym = params.symbol.trim();

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    let sym_str = sym.to_string();
    tokio::spawn(async move {
        let duration_ms = start.elapsed().as_millis() as i64;
        let _ = db.record_agent_api_call(
            "/api/galaxy/boundary",
            &ws_log,
            "Omni-Graph: Architectural Subsystem Boundary Contract",
            Some(&sym_str),
            None,
            duration_ms,
        ).await;
    });

    match state.db.get_symbol_boundary(sym, ws.as_deref()).await {
        Ok(Some(info)) => (StatusCode::OK, Json(info)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Symbol '{}' not found in workspace '{:?}'", sym, ws),
                "symbol": sym,
                "workspace": ws
            })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ).into_response(),
    }
}

/// GET /api/galaxy/topology?workspace=<ws>
/// Returns macro architectural subsystem map, coupling metrics, and cross-galaxy dependencies
async fn galaxy_topology_handler(
    State(state): State<AppState>,
    Query(params): Query<GalaxyTopologyParams>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let ws = normalize_workspace(params.workspace.as_deref());

    // Record telemetry asynchronously
    let db = state.db.clone();
    let ws_log = ws.clone().unwrap_or_else(|| "default".to_string());
    tokio::spawn(async move {
        let duration_ms = start.elapsed().as_millis() as i64;
        let _ = db.record_agent_api_call(
            "/api/galaxy/topology",
            &ws_log,
            "Omni-Graph: Macro Subsystem Topology",
            None,
            None,
            duration_ms,
        ).await;
    });

    let mut records = state.db.get_galaxies_records(ws.as_deref()).await.unwrap_or_default();
    if records.is_empty() {
        // If galaxy table empty, compute on the fly
        if let Ok((nodes, links)) = state.db.get_graph(ws.as_deref()).await {
            if !nodes.is_empty() {
                let ws_name = ws.clone().unwrap_or_else(|| "default".to_string());
                let assignments = CommunityDetector::detect(&nodes, &links, 15);
                let (_, computed_records) = CommunityDetector::compute_galaxy_metrics(&ws_name, &nodes, &links, &assignments);
                let _ = state.db.update_communities(&assignments).await;
                let _ = state.db.store_galaxies(&ws_name, &computed_records).await;
                records = computed_records;
            }
        }
    }

    let inter_deps = state.db.get_inter_galaxy_dependencies(ws.as_deref()).await.unwrap_or_default();

    Json(serde_json::json!({
        "workspace": ws,
        "total_galaxies": records.len(),
        "galaxies": records,
        "inter_galaxy_dependencies": inter_deps,
        "duration_ms": start.elapsed().as_millis() as u64
    })).into_response()
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
async fn analytics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let resp = AnalyticsEngine::scan_analytics_with_db(&state.db).await;
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

/// POST /api/watch/start
async fn watch_start_handler(
    State(state): State<AppState>,
    Json(payload): Json<WatchStartPayload>,
) -> impl IntoResponse {
    let ws_name = match payload.workspace.or(payload.project) {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => {
            let p = std::path::Path::new(&payload.path);
            let abs_path = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
            abs_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".to_string())
        }
    };

    // Auto-resolve path if it doesn't exist on disk
    let mut target_path = payload.path;
    let trimmed = target_path.trim_end_matches('/');
    if trimmed.ends_with("/tools/tool-scripts") || trimmed.ends_with("/tools/workspace") {
        if let Some(prefix) = trimmed.strip_suffix("/tools/tool-scripts").or_else(|| trimmed.strip_suffix("/tools/workspace")) {
            let candidate = if prefix.is_empty() { "/workspace".to_string() } else { prefix.to_string() };
            if std::path::Path::new(&candidate).exists() {
                target_path = candidate;
            }
        }
    }

    let path_obj = std::path::Path::new(&target_path);
    if !path_obj.exists() || !path_obj.is_dir() {
        if let Ok(Some(resolved)) = state.db.get_workspace_root(&ws_name).await {
            target_path = resolved;
        } else if let Some(resolved) = crate::db::resolve_disk_path(&ws_name, &[]) {
            target_path = resolved;
        }
    }

    match state.watcher.start_watch(&target_path, &ws_name, payload.debounce_ms).await {
        Ok(status) => {
            let _ = state.db.record_workspace_root(&ws_name, &target_path).await;
            let db = state.db.clone();
            let ws_log = ws_name.clone();
            let target_log = target_path.clone();
            tokio::spawn(async move {
                let _ = db.record_agent_api_call(
                    "/api/watch/start",
                    &ws_log,
                    "Omni-Graph: Live Delta Watch Daemon",
                    Some(&target_log),
                    None,
                    1,
                ).await;
            });
            (StatusCode::OK, Json(serde_json::to_value(status).unwrap_or_default())).into_response()
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": e
            })),
        )
            .into_response(),
    }
}

/// POST /api/watch/stop
async fn watch_stop_handler(
    State(state): State<AppState>,
    Json(payload): Json<WatchStopPayload>,
) -> impl IntoResponse {
    let ws = normalize_workspace(Some(&payload.workspace)).unwrap_or(payload.workspace);
    match state.watcher.stop_watch(&ws).await {
        Ok(status) => (StatusCode::OK, Json(serde_json::to_value(status).unwrap_or_default())).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": e
            })),
        )
            .into_response(),
    }
}

/// GET /api/watch/status
async fn watch_status_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let statuses = state.watcher.get_status().await;
    Json(statuses).into_response()
}

/// GET /api/watch/status/:workspace
async fn watch_workspace_status_handler(
    State(state): State<AppState>,
    AxumPath(workspace): AxumPath<String>,
) -> impl IntoResponse {
    let ws = normalize_workspace(Some(&workspace)).unwrap_or(workspace);
    match state.watcher.get_workspace_status(&ws).await {
        Some(status) => Json(status).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("No watcher found for workspace '{}'", ws)
            })),
        )
            .into_response(),
    }
}

/// GET /api/watch/events (Server-Sent Events)
async fn watch_events_sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = state.watcher.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| {
        match res {
            Ok(evt) => {
                let json = serde_json::to_string(&evt).unwrap_or_default();
                Some(Ok(SseEvent::default().event("delta").data(json)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}


