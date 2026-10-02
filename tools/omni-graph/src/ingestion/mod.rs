// Ingestion pipeline: Walk directory, parse AST, embed vectors, store in SurrealDB
// Implements: R-010, R-011

use crate::analysis::CommunityDetector;
use crate::db::DbClient;
use crate::embedder::EmbedderClient;
use crate::parser::CodeParser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Clone, Debug, Default)]
pub struct FileCache {
    hashes: Arc<Mutex<HashMap<String, String>>>,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            hashes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn populate(&self, hashes: HashMap<String, String>) {
        let mut map = self.hashes.lock().await;
        map.extend(hashes);
    }

    pub async fn is_stale(&self, file_path: &str, current_hash: &str) -> bool {
        let map = self.hashes.lock().await;
        if let Some(prev_hash) = map.get(file_path) {
            if prev_hash == current_hash {
                return false; // Unchanged
            }
        }
        true
    }

    pub async fn update(&self, file_path: &str, current_hash: &str) {
        let mut map = self.hashes.lock().await;
        map.insert(file_path.to_string(), current_hash.to_string());
    }

    pub async fn remove(&self, file_path: &str) {
        let mut map = self.hashes.lock().await;
        map.remove(file_path);
    }

    pub async fn rename(&self, old_path: &str, new_path: &str) {
        let mut map = self.hashes.lock().await;
        if let Some(h) = map.remove(old_path) {
            map.insert(new_path.to_string(), h);
        }
    }

    pub async fn get_all_paths(&self) -> Vec<String> {
        let map = self.hashes.lock().await;
        map.keys().cloned().collect()
    }

    /// Clear all cached hashes. Used by refresh-ingestion to force full re-scan.
    pub async fn clear(&self) {
        let mut map = self.hashes.lock().await;
        map.clear();
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IngestionJobStatus {
    pub workspace: String,
    pub path: String,
    pub is_refresh: bool,
    pub phase: String, // "purging", "scanning", "indexing", "clustering", "completed", "failed"
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub total_files: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub clusters_computed: usize,
    pub started_at: u64,
    pub elapsed_ms: u64,
    pub completed: bool,
    pub error: Option<String>,
}

pub struct IngestionPipeline {
    db: DbClient,
    embedder: EmbedderClient,
    cache: FileCache,
    jobs: Arc<RwLock<HashMap<String, IngestionJobStatus>>>,
}

#[derive(serde::Serialize, Debug)]
pub struct IngestResult {
    pub workspace: String,
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub clusters_computed: usize,
    pub duration_ms: u64,
}

impl IngestionPipeline {
    pub fn new(db: DbClient, embedder: EmbedderClient, cache: FileCache) -> Self {
        Self {
            db,
            embedder,
            cache,
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Retrieve real-time status of all active and recently completed ingestion jobs
    pub async fn get_status(&self) -> Vec<IngestionJobStatus> {
        let jobs = self.jobs.read().await;
        let mut list: Vec<IngestionJobStatus> = jobs.values().cloned().collect();
        list.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        list
    }

    /// Dismiss a completed or failed ingestion job from the tracker
    pub async fn dismiss_job(&self, workspace: &str) {
        let mut jobs = self.jobs.write().await;
        jobs.remove(workspace);
    }

    pub async fn ingest_directory(
        &self,
        root_dir: &str,
        project: Option<&str>,
        refresh: bool,
    ) -> Result<IngestResult, String> {
        let start_time = std::time::Instant::now();
        info!("Starting ingestion for directory: {}", root_dir);

        let root = Path::new(root_dir);
        if !root.exists() || !root.is_dir() {
            return Err(format!("Directory does not exist or is not a dir: {}", root_dir));
        }

        let workspace_name = match project {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => {
                let abs_path = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
                abs_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "default".to_string())
            }
        };

        info!("Ingestion assigned workspace namespace: '{}'", workspace_name);

        let abs_root = fs::canonicalize(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| root_dir.to_string());
        let _ = self.db.record_workspace_root(&workspace_name, &abs_root).await;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Register initial in-motion ingestion job in tracker
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(
                workspace_name.clone(),
                IngestionJobStatus {
                    workspace: workspace_name.clone(),
                    path: abs_root.clone(),
                    is_refresh: refresh,
                    phase: if refresh { "purging".to_string() } else { "scanning".to_string() },
                    files_scanned: 0,
                    files_indexed: 0,
                    files_skipped: 0,
                    total_files: 0,
                    nodes_created: 0,
                    edges_created: 0,
                    clusters_computed: 0,
                    started_at: now_ms,
                    elapsed_ms: 0,
                    completed: false,
                    error: None,
                },
            );
        }

        // REFRESH MODE: Purge all existing nodes/edges/galaxies before clean re-scan
        if refresh {
            info!("REFRESH mode: purging all existing data for workspace '{}'", workspace_name);
            match self.db.purge_workspace(&workspace_name).await {
                Ok(purged) => info!("Purged {} existing nodes from '{}' before fresh re-scan", purged, workspace_name),
                Err(e) => warn!("Purge warning for '{}': {} — proceeding with ingestion", workspace_name, e),
            }
            // Clear the in-memory staleness cache so ALL files get re-indexed
            self.cache.clear().await;
            {
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&workspace_name) {
                    job.phase = "scanning".to_string();
                    job.elapsed_ms = start_time.elapsed().as_millis() as u64;
                }
            }
        }

        // Restore staleness cache from persisted database file hashes (Finding #4)
        // (Skip if refresh mode — we just purged everything)
        if !refresh {
            if let Ok(persisted_hashes) = self.db.get_file_hashes(&workspace_name).await {
                if !persisted_hashes.is_empty() {
                    info!("Restored {} persisted file hashes for '{}' from SurrealDB", persisted_hashes.len(), workspace_name);
                    self.cache.populate(persisted_hashes).await;
                }
            }
        }

        let mut files_scanned = 0;
        let mut files_indexed = 0;
        let mut files_skipped = 0;
        let mut total_nodes = 0;
        let mut total_edges = 0;

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Exclude common noise directories
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                    && name != "dist"
                    && name != "build"
                    && name != "vendor"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            // Skip common binary artifacts and noise
            if matches!(
                ext,
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "svg" | "pdf"
                    | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z"
                    | "exe" | "bin" | "dll" | "dylib" | "so" | "a" | "o" | "obj"
                    | "wasm" | "pyc" | "pyo" | "pyd" | "class" | "jar"
                    | "lock" | "map" | "rlib" | "rmeta" | "timestamp"
            ) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => {
                    // Non-UTF-8 binary or unreadable file, skip cleanly
                    continue;
                }
            };

            files_scanned += 1;

            // Compute hash for staleness tracking (R-011)
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let hash = hex::encode(hasher.finalize());

            // Derive relative file path for clean and portable node IDs
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if !self.cache.is_stale(&rel_path, &hash).await {
                files_skipped += 1;
                continue;
            }

            // Parse AST or structured document, with universal fallback chunker (Guarantees zero dropped files)
            let parse_res = CodeParser::parse_file(&workspace_name, &rel_path, &content)
                .filter(|pr| !pr.nodes.is_empty())
                .or_else(|| CodeParser::parse_fallback(&workspace_name, &rel_path, &content));

            if let Some(parse_res) = parse_res {
                if parse_res.nodes.is_empty() {
                    continue;
                }

                // Batch vector embeddings via TEI (R-004)
                let text_refs: Vec<&str> = parse_res.nodes.iter().map(|n| n.text.as_str()).collect();
                let embeddings = match self.embedder.embed_batch(&text_refs).await {
                    Ok(embs) => embs,
                    Err(e) => {
                        warn!("Embedding generation failed for {}: {}", rel_path, e);
                        // Zero vector fallback if TEI is busy or degraded
                        vec![vec![0.0f32; 384]; parse_res.nodes.len()]
                    }
                };

                let mut stored_nodes_ok = false;
                // Store nodes with workspace namespace and file hash (R-005, Finding #4)
                if let Err(e) = self.db.store_nodes(&parse_res.nodes, &embeddings, Some(&hash)).await {
                    error!("Failed to store nodes for {}: {}", rel_path, e);
                } else {
                    total_nodes += parse_res.nodes.len();
                    stored_nodes_ok = true;
                }

                // Store edges with workspace namespace (R-006)
                if let Err(e) = self.db.store_edges(&parse_res.edges).await {
                    error!("Failed to store edges for {}: {}", rel_path, e);
                } else {
                    total_edges += parse_res.edges.len();
                }

                if stored_nodes_ok {
                    self.cache.update(&rel_path, &hash).await;
                }

                files_indexed += 1;

                if files_scanned % 15 == 0 || files_indexed % 10 == 0 {
                    let mut jobs = self.jobs.write().await;
                    if let Some(job) = jobs.get_mut(&workspace_name) {
                        job.phase = "indexing".to_string();
                        job.files_scanned = files_scanned;
                        job.files_indexed = files_indexed;
                        job.files_skipped = files_skipped;
                        job.nodes_created = total_nodes;
                        job.edges_created = total_edges;
                        job.elapsed_ms = start_time.elapsed().as_millis() as u64;
                    }
                }
            }
        }

        // Auto-cluster upon ingestion: Compute Louvain/Leiden modularity communities
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(&workspace_name) {
                job.phase = "clustering".to_string();
                job.files_scanned = files_scanned;
                job.files_indexed = files_indexed;
                job.files_skipped = files_skipped;
                job.nodes_created = total_nodes;
                job.edges_created = total_edges;
                job.elapsed_ms = start_time.elapsed().as_millis() as u64;
            }
        }

        let mut clusters_computed = 0;
        if total_nodes > 0 {
            if let Ok((all_nodes, all_links)) = self.db.get_graph(Some(&workspace_name)).await {
                if !all_nodes.is_empty() {
                    let assignments = CommunityDetector::detect(&all_nodes, &all_links, 15);
                    let (summaries, galaxy_records) = CommunityDetector::compute_galaxy_metrics(&workspace_name, &all_nodes, &all_links, &assignments);
                    clusters_computed = summaries.len();
                    if let Err(e) = self.db.update_communities(&assignments).await {
                        warn!("Auto-clustering failed after ingestion for '{}': {}", workspace_name, e);
                    } else {
                        let _ = self.db.store_galaxies(&workspace_name, &galaxy_records).await;
                        info!("Auto-clustered '{}' into {} modular communities", workspace_name, clusters_computed);
                    }
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        info!(
            "Ingestion completed for '{}': {} files scanned, {} indexed, {} skipped, {} clusters in {}ms",
            workspace_name, files_scanned, files_indexed, files_skipped, clusters_computed, duration_ms
        );

        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(&workspace_name) {
                job.phase = "completed".to_string();
                job.completed = true;
                job.files_scanned = files_scanned;
                job.files_indexed = files_indexed;
                job.files_skipped = files_skipped;
                job.nodes_created = total_nodes;
                job.edges_created = total_edges;
                job.clusters_computed = clusters_computed;
                job.elapsed_ms = duration_ms;
            }
        }

        Ok(IngestResult {
            workspace: workspace_name,
            files_scanned,
            files_indexed,
            files_skipped,
            nodes_created: total_nodes,
            edges_created: total_edges,
            clusters_computed,
            duration_ms,
        })
    }
}
