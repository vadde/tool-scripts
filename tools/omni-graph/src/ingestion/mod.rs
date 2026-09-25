// Ingestion pipeline: Walk directory, parse AST, embed vectors, store in SurrealDB
// Implements: R-010, R-011

use crate::db::DbClient;
use crate::embedder::EmbedderClient;
use crate::parser::CodeParser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
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
}

pub struct IngestionPipeline {
    db: DbClient,
    embedder: EmbedderClient,
    cache: FileCache,
}

#[derive(serde::Serialize, Debug)]
pub struct IngestResult {
    pub workspace: String,
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub duration_ms: u64,
}

impl IngestionPipeline {
    pub fn new(db: DbClient, embedder: EmbedderClient, cache: FileCache) -> Self {
        Self {
            db,
            embedder,
            cache,
        }
    }

    pub async fn ingest_directory(
        &self,
        root_dir: &str,
        project: Option<&str>,
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

        // Restore staleness cache from persisted database file hashes (Finding #4)
        if let Ok(persisted_hashes) = self.db.get_file_hashes(&workspace_name).await {
            if !persisted_hashes.is_empty() {
                info!("Restored {} persisted file hashes for '{}' from SurrealDB", persisted_hashes.len(), workspace_name);
                self.cache.populate(persisted_hashes).await;
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

            let path_str = path.to_string_lossy().to_string();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext, "rs" | "py" | "go" | "js" | "jsx" | "ts" | "tsx") {
                continue;
            }

            files_scanned += 1;

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read file {}: {}", path_str, e);
                    continue;
                }
            };

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

            // Parse AST (R-003)
            if let Some(parse_res) = CodeParser::parse_file(&workspace_name, &rel_path, &content) {
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
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        info!(
            "Ingestion completed for '{}': {} files scanned, {} indexed, {} skipped in {}ms",
            workspace_name, files_scanned, files_indexed, files_skipped, duration_ms
        );

        Ok(IngestResult {
            workspace: workspace_name,
            files_scanned,
            files_indexed,
            files_skipped,
            nodes_created: total_nodes,
            edges_created: total_edges,
            duration_ms,
        })
    }
}
