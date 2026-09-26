// Incremental pipeline: single-file re-indexing and pruning
// Re-uses tree-sitter CodeParser and EmbedderClient for fine-grained AST updates

use crate::db::DbClient;
use crate::embedder::EmbedderClient;
use crate::ingestion::FileCache;
use crate::parser::CodeParser;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

pub struct IncrementalPipeline {
    db: DbClient,
    embedder: EmbedderClient,
    cache: FileCache,
}

impl IncrementalPipeline {
    pub fn new(db: DbClient, embedder: EmbedderClient, cache: FileCache) -> Self {
        Self {
            db,
            embedder,
            cache,
        }
    }

    /// Re-index a single created or modified file incrementally
    pub async fn reindex_file(
        &self,
        workspace: &str,
        root_dir: &str,
        rel_path: &str,
    ) -> Result<(), String> {
        let full_path = Path::new(root_dir).join(rel_path);

        if !full_path.exists() {
            // File might have been deleted right after a modify event
            return self.delete_file(workspace, rel_path).await;
        }

        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => {
                // Could be non-UTF-8 or lock file, skip silently
                warn!("Cannot read file '{}' for live sync: {}", rel_path, e);
                return Ok(());
            }
        };

        // Compute sha256 hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // Check staleness against cache
        if !self.cache.is_stale(rel_path, &hash).await {
            return Ok(());
        }

        // Parse AST or fallback chunker
        let parse_res = CodeParser::parse_file(workspace, rel_path, &content)
            .filter(|pr| !pr.nodes.is_empty())
            .or_else(|| CodeParser::parse_fallback(workspace, rel_path, &content));

        let parse_res = match parse_res {
            Some(pr) if !pr.nodes.is_empty() => pr,
            _ => {
                // If now empty, prune any old nodes
                return self.delete_file(workspace, rel_path).await;
            }
        };

        // 1. Prune old nodes and edges for this file before inserting fresh AST
        if let Err(e) = self.db.delete_file(workspace, rel_path).await {
            warn!("Failed to prune previous nodes for '{}': {}", rel_path, e);
        }

        // 2. Generate vector embeddings for newly parsed nodes
        let text_refs: Vec<&str> = parse_res.nodes.iter().map(|n| n.text.as_str()).collect();
        let embeddings = match self.embedder.embed_batch(&text_refs).await {
            Ok(embs) => embs,
            Err(e) => {
                warn!("Embedding generation failed for live sync on '{}': {}", rel_path, e);
                vec![vec![0.0f32; 384]; parse_res.nodes.len()]
            }
        };

        // 3. Store new nodes
        if let Err(e) = self.db.store_nodes(&parse_res.nodes, &embeddings, Some(&hash)).await {
            error!("Failed to store nodes during live sync for '{}': {}", rel_path, e);
            return Err(e);
        }

        // 4. Store new edges
        if let Err(e) = self.db.store_edges(&parse_res.edges).await {
            warn!("Failed to store edges during live sync for '{}': {}", rel_path, e);
        }

        // 5. Update staleness cache
        self.cache.update(rel_path, &hash).await;

        info!(
            "Live synced file '{}' in workspace '{}' ({} nodes, {} edges)",
            rel_path,
            workspace,
            parse_res.nodes.len(),
            parse_res.edges.len()
        );

        Ok(())
    }

    /// Prune nodes and relations for a deleted file
    pub async fn delete_file(&self, workspace: &str, rel_path: &str) -> Result<(), String> {
        info!("Pruning deleted file '{}' from workspace '{}'", rel_path, workspace);
        self.db.delete_file(workspace, rel_path).await?;
        self.cache.remove(rel_path).await;
        Ok(())
    }

    /// Rename file path references in the graph
    pub async fn rename_file(
        &self,
        workspace: &str,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), String> {
        info!(
            "Renaming file '{}' -> '{}' in workspace '{}'",
            old_path, new_path, workspace
        );
        self.db.rename_file(workspace, old_path, new_path).await?;
        self.cache.rename(old_path, new_path).await;
        Ok(())
    }

    /// Prune any files from SurrealDB that were cached but no longer exist on disk
    pub async fn prune_missing_files(&self, workspace: &str, root_dir: &str) -> Vec<String> {
        let cached_paths = self.cache.get_all_paths().await;
        let mut pruned = Vec::new();
        for rel_path in cached_paths {
            let full_p = Path::new(root_dir).join(&rel_path);
            if !full_p.exists() {
                if let Ok(()) = self.delete_file(workspace, &rel_path).await {
                    pruned.push(rel_path);
                }
            }
        }
        pruned
    }
}
