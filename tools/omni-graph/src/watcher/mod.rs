// Delta/Watch: File system watcher with incremental re-indexing
// Implements dynamic live sync for active workspaces
//
// Architecture:
//   WatchManager → manages N concurrent workspace watchers
//   Each watcher uses notify (FSEvents on macOS) with 500ms debounce
//   DeltaChangeset → classified events → IncrementalPipeline

pub mod delta;
pub mod pipeline;

use crate::analysis::CommunityDetector;
use crate::db::DbClient;
use crate::embedder::EmbedderClient;
use crate::ingestion::FileCache;
use delta::DeltaKind;
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use pipeline::IncrementalPipeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::{error, info, warn};

/// Status of a single workspace watcher
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub workspace: String,
    pub path: String,
    pub status: String, // "watching", "syncing", "stopped", "error"
    pub started_at: String,
    pub files_tracked: usize,
    pub last_sync: Option<String>,
    pub events_processed: u64,
    pub files_reindexed: u64,
    pub files_deleted: u64,
    pub avg_sync_ms: u64,
    pub cluster_status: String, // "current", "stale"
    pub last_cluster_at: Option<String>,
    pub debounce_ms: u64,
}

/// A live event pushed via SSE
#[derive(Clone, Debug, Serialize)]
pub struct WatchEvent {
    pub workspace: String,
    pub kind: String,
    pub file_path: String,
    pub timestamp: String,
    pub sync_ms: Option<u64>,
}

/// Internal state for a single active watcher
struct ActiveWatcher {
    workspace: String,
    path: String,
    status: Arc<RwLock<WatcherStatus>>,
    cancel_tx: mpsc::Sender<()>,
}

/// Central manager for all file watchers
pub struct WatchManager {
    watchers: Arc<Mutex<HashMap<String, ActiveWatcher>>>,
    db: DbClient,
    embedder: EmbedderClient,
    event_tx: broadcast::Sender<WatchEvent>,
}

// Directories to exclude from watching
const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "__pycache__",
    ".next",
    ".cache",
    ".idea",
    ".vscode",
    "venv",
    ".venv",
    "env",
    ".DS_Store",
];

// File extensions to exclude (binary artifacts)
const EXCLUDED_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "ico", "svg", "pdf", "zip", "tar", "gz", "bz2", "xz",
    "7z", "exe", "bin", "dll", "dylib", "so", "a", "o", "obj", "wasm", "pyc", "pyo", "pyd",
    "class", "jar", "lock", "map", "rlib", "rmeta", "timestamp",
];

impl WatchManager {
    pub fn new(db: DbClient, embedder: EmbedderClient) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            db,
            embedder,
            event_tx,
        }
    }

    /// Subscribe to the SSE event stream
    pub fn subscribe(&self) -> broadcast::Receiver<WatchEvent> {
        self.event_tx.subscribe()
    }

    /// Start watching a workspace directory
    pub async fn start_watch(
        &self,
        path: &str,
        workspace: &str,
        debounce_ms: Option<u64>,
    ) -> Result<WatcherStatus, String> {
        let mut watchers = self.watchers.lock().await;

        // Check if already watching
        if watchers.contains_key(workspace) {
            return Err(format!(
                "Already watching workspace '{}'. Stop it first.",
                workspace
            ));
        }

        let root = PathBuf::from(path);
        if !root.exists() || !root.is_dir() {
            return Err(format!("Path does not exist or is not a directory: {}", path));
        }

        let debounce = debounce_ms.unwrap_or(500);
        let now = chrono::Utc::now().to_rfc3339();

        let status = Arc::new(RwLock::new(WatcherStatus {
            workspace: workspace.to_string(),
            path: path.to_string(),
            status: "watching".to_string(),
            started_at: now.clone(),
            files_tracked: 0,
            last_sync: None,
            events_processed: 0,
            files_reindexed: 0,
            files_deleted: 0,
            avg_sync_ms: 0,
            cluster_status: "current".to_string(),
            last_cluster_at: None,
            debounce_ms: debounce,
        }));

        // Count initial files
        let initial_count = count_tracked_files(path);
        {
            let mut s = status.write().await;
            s.files_tracked = initial_count;
        }

        let (cancel_tx, cancel_rx) = mpsc::channel::<()>(1);

        // Spawn the watcher task
        let watcher_status = status.clone();
        let watcher_db = self.db.clone();
        let watcher_embedder = self.embedder.clone();
        let watcher_event_tx = self.event_tx.clone();
        let watcher_path = path.to_string();
        let watcher_workspace = workspace.to_string();

        tokio::spawn(async move {
            run_watcher(
                watcher_path,
                watcher_workspace,
                debounce,
                watcher_db,
                watcher_embedder,
                watcher_status,
                watcher_event_tx,
                cancel_rx,
            )
            .await;
        });

        let current_status = status.read().await.clone();

        watchers.insert(
            workspace.to_string(),
            ActiveWatcher {
                workspace: workspace.to_string(),
                path: path.to_string(),
                status,
                cancel_tx,
            },
        );

        info!(
            "Started watching workspace '{}' at path '{}' (debounce: {}ms, {} files)",
            workspace, path, debounce, initial_count
        );

        Ok(current_status)
    }

    /// Stop watching a workspace
    pub async fn stop_watch(&self, workspace: &str) -> Result<WatcherStatus, String> {
        let mut watchers = self.watchers.lock().await;

        if let Some(watcher) = watchers.remove(workspace) {
            // Signal the watcher task to stop
            let _ = watcher.cancel_tx.send(()).await;
            let mut status = watcher.status.write().await;
            status.status = "stopped".to_string();
            info!("Stopped watching workspace '{}'", workspace);
            Ok(status.clone())
        } else {
            Err(format!("No active watcher for workspace '{}'", workspace))
        }
    }

    /// Get status of all active watchers
    pub async fn get_status(&self) -> Vec<WatcherStatus> {
        let watchers = self.watchers.lock().await;
        let mut statuses = Vec::new();
        for (_, watcher) in watchers.iter() {
            let status = watcher.status.read().await;
            statuses.push(status.clone());
        }
        statuses
    }

    /// Get status of a specific workspace watcher
    pub async fn get_workspace_status(&self, workspace: &str) -> Option<WatcherStatus> {
        let watchers = self.watchers.lock().await;
        if let Some(watcher) = watchers.get(workspace) {
            Some(watcher.status.read().await.clone())
        } else {
            None
        }
    }
}

/// Count files in a directory (excluding noise)
fn count_tracked_files(path: &str) -> usize {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|d| name.as_ref() == *d)
        })
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if !EXCLUDED_EXTS.contains(&ext) {
                count += 1;
            }
        }
    }
    count
}

/// Core watcher loop running in a background tokio task
async fn run_watcher(
    path: String,
    workspace: String,
    debounce_ms: u64,
    db: DbClient,
    embedder: EmbedderClient,
    status: Arc<RwLock<WatcherStatus>>,
    event_tx: broadcast::Sender<WatchEvent>,
    mut cancel_rx: mpsc::Receiver<()>,
) {
    let (tx, mut rx) = mpsc::channel::<Event>(512);

    // Create the native FS watcher
    let mut watcher = match RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        NotifyConfig::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create file watcher for '{}': {}", workspace, e);
            let mut s = status.write().await;
            s.status = "error".to_string();
            return;
        }
    };

    // Start watching the directory tree
    if let Err(e) = watcher.watch(std::path::Path::new(&path), RecursiveMode::Recursive) {
        error!("Failed to watch path '{}': {}", path, e);
        let mut s = status.write().await;
        s.status = "error".to_string();
        return;
    }

    info!(
        "Watcher active for workspace '{}' at '{}'",
        workspace, path
    );

    let cache = FileCache::new();
    if let Ok(hashes) = db.get_file_hashes(&workspace).await {
        cache.populate(hashes).await;
    }
    let pipeline = IncrementalPipeline::new(db.clone(), embedder, cache);
    let debounce_duration = Duration::from_millis(debounce_ms);
    let mut pending_events: Vec<Event> = Vec::new();
    let mut last_batch_time = Instant::now();
    let mut sweep_interval = tokio::time::interval(Duration::from_secs(3));
    sweep_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let root_path = PathBuf::from(&path);
    let mut total_sync_ms: u64 = 0;
    let mut sync_count: u64 = 0;
    let mut files_changed_since_cluster: u64 = 0;
    let mut last_event_processed_time = Instant::now();

    loop {
        tokio::select! {
            // Cancellation signal
            _ = cancel_rx.recv() => {
                info!("Watcher for '{}' received cancellation signal", workspace);
                break;
            }

            // File system event
            Some(event) = rx.recv() => {
                // Filter out noise events
                if should_process_event(&event, &root_path) {
                    pending_events.push(event);
                }

                // Process batch if debounce window elapsed
                if last_batch_time.elapsed() >= debounce_duration && !pending_events.is_empty() {
                    let changeset = delta::classify_events(
                        &pending_events,
                        &workspace,
                        &root_path,
                    );
                    pending_events.clear();
                    last_batch_time = Instant::now();

                    if !changeset.events.is_empty() {
                        // Update status to syncing
                        {
                            let mut s = status.write().await;
                            s.status = "syncing".to_string();
                        }

                        let sync_start = Instant::now();

                        // Process each delta event
                        for delta in &changeset.events {
                            let result = match delta.kind {
                                DeltaKind::Created | DeltaKind::Modified => {
                                    pipeline.reindex_file(&workspace, &path, &delta.file_path).await
                                }
                                DeltaKind::Deleted => {
                                    pipeline.delete_file(&workspace, &delta.file_path).await
                                }
                                DeltaKind::Renamed => {
                                    if let Some(old) = &delta.old_path {
                                        pipeline.rename_file(&workspace, old, &delta.file_path).await
                                    } else {
                                        pipeline.reindex_file(&workspace, &path, &delta.file_path).await
                                    }
                                }
                            };

                            // Emit SSE event
                            let watch_event = WatchEvent {
                                workspace: workspace.clone(),
                                kind: format!("{:?}", delta.kind),
                                file_path: delta.file_path.clone(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                sync_ms: Some(sync_start.elapsed().as_millis() as u64),
                            };
                            let _ = event_tx.send(watch_event);

                            if let Err(e) = result {
                                warn!("Delta sync error for {}: {}", delta.file_path, e);
                            }
                        }

                        let elapsed_ms = sync_start.elapsed().as_millis() as u64;
                        total_sync_ms += elapsed_ms;
                        sync_count += 1;
                        files_changed_since_cluster += changeset.events.len() as u64;
                        last_event_processed_time = Instant::now();

                        // Update status
                        {
                            let mut s = status.write().await;
                            s.status = "watching".to_string();
                            s.last_sync = Some(chrono::Utc::now().to_rfc3339());
                            s.events_processed += changeset.events.len() as u64;
                            s.files_reindexed += changeset.events.iter()
                                .filter(|e| matches!(e.kind, DeltaKind::Created | DeltaKind::Modified))
                                .count() as u64;
                            s.files_deleted += changeset.events.iter()
                                .filter(|e| matches!(e.kind, DeltaKind::Deleted))
                                .count() as u64;
                            s.avg_sync_ms = if sync_count > 0 { total_sync_ms / sync_count } else { 0 };
                            s.files_tracked = count_tracked_files(&path);

                            // Mark clusters as stale if enough files changed
                            if files_changed_since_cluster >= 10 {
                                s.cluster_status = "stale".to_string();
                            }
                        }

                        info!(
                            "Delta sync for '{}': {} events in {}ms",
                            workspace, changeset.events.len(), elapsed_ms
                        );
                    }
                }
            }

            // Debounce timer: flush any pending events after the window
            _ = tokio::time::sleep(debounce_duration) => {
                if !pending_events.is_empty() && last_batch_time.elapsed() >= debounce_duration {
                    let changeset = delta::classify_events(
                        &pending_events,
                        &workspace,
                        &root_path,
                    );
                    pending_events.clear();
                    last_batch_time = Instant::now();

                    if !changeset.events.is_empty() {
                        {
                            let mut s = status.write().await;
                            s.status = "syncing".to_string();
                        }

                        let sync_start = Instant::now();

                        for delta in &changeset.events {
                            let result = match delta.kind {
                                DeltaKind::Created | DeltaKind::Modified => {
                                    pipeline.reindex_file(&workspace, &path, &delta.file_path).await
                                }
                                DeltaKind::Deleted => {
                                    pipeline.delete_file(&workspace, &delta.file_path).await
                                }
                                DeltaKind::Renamed => {
                                    if let Some(old) = &delta.old_path {
                                        pipeline.rename_file(&workspace, old, &delta.file_path).await
                                    } else {
                                        pipeline.reindex_file(&workspace, &path, &delta.file_path).await
                                    }
                                }
                            };

                            let watch_event = WatchEvent {
                                workspace: workspace.clone(),
                                kind: format!("{:?}", delta.kind),
                                file_path: delta.file_path.clone(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                sync_ms: Some(sync_start.elapsed().as_millis() as u64),
                            };
                            let _ = event_tx.send(watch_event);

                            if let Err(e) = result {
                                warn!("Delta sync error for {}: {}", delta.file_path, e);
                            }
                        }

                        let elapsed_ms = sync_start.elapsed().as_millis() as u64;
                        total_sync_ms += elapsed_ms;
                        sync_count += 1;
                        files_changed_since_cluster += changeset.events.len() as u64;
                        last_event_processed_time = Instant::now();

                        {
                            let mut s = status.write().await;
                            s.status = "watching".to_string();
                            s.last_sync = Some(chrono::Utc::now().to_rfc3339());
                            s.events_processed += changeset.events.len() as u64;
                            s.files_reindexed += changeset.events.iter()
                                .filter(|e| matches!(e.kind, DeltaKind::Created | DeltaKind::Modified))
                                .count() as u64;
                            s.files_deleted += changeset.events.iter()
                                .filter(|e| matches!(e.kind, DeltaKind::Deleted))
                                .count() as u64;
                            s.avg_sync_ms = if sync_count > 0 { total_sync_ms / sync_count } else { 0 };
                            s.files_tracked = count_tracked_files(&path);
                            if files_changed_since_cluster >= 10 {
                                s.cluster_status = "stale".to_string();
                            }
                        }

                        info!(
                            "Delta sync (debounce flush) for '{}': {} events in {}ms",
                            workspace, changeset.events.len(), elapsed_ms
                        );
                    }
                }
            }

            // Periodic sweep for cross-filesystem deletions & tracked files refresh
            _ = sweep_interval.tick() => {
                let pruned = pipeline.prune_missing_files(&workspace, &path).await;
                let tracked = count_tracked_files(&path);
                let mut s = status.write().await;
                s.files_tracked = tracked;
                if !pruned.is_empty() {
                    s.files_deleted += pruned.len() as u64;
                    s.events_processed += pruned.len() as u64;
                    s.last_sync = Some(chrono::Utc::now().to_rfc3339());
                    for p in &pruned {
                        let _ = event_tx.send(WatchEvent {
                            workspace: workspace.clone(),
                            kind: "Deleted".to_string(),
                            file_path: p.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            sync_ms: Some(1),
                        });
                    }
                    info!("Periodic sweep pruned {} missing files in '{}'", pruned.len(), workspace);
                }

                // Quiescent auto-clustering: If files have changed and editing has paused for >= 3.5 seconds,
                // automatically recompute communities in memory, update DB, and emit SSE notification.
                if files_changed_since_cluster > 0 && last_event_processed_time.elapsed() >= Duration::from_millis(3500) {
                    info!("Quiescent period detected for '{}' ({} pending changes) -> auto-reclustering galaxies...", workspace, files_changed_since_cluster);
                    if let Ok((nodes, links)) = db.get_graph(Some(&workspace)).await {
                        if !nodes.is_empty() {
                            let mut seeds = HashMap::new();
                            for n in &nodes {
                                if let Some(c) = n.community {
                                    seeds.insert(n.id.clone(), c);
                                }
                            }
                            let assignments = CommunityDetector::detect_with_seeds(&nodes, &links, 15, Some(&seeds));
                            let (_, galaxy_records) = CommunityDetector::compute_galaxy_metrics(&workspace, &nodes, &links, &assignments);
                            if let Err(e) = db.update_communities(&assignments).await {
                                warn!("Failed to update communities during live auto-recluster for '{}': {}", workspace, e);
                            } else {
                                let _ = db.store_galaxies(&workspace, &galaxy_records).await;
                                info!("Auto-reclustered '{}' into {} active galaxies in background", workspace, galaxy_records.len());

                                let mut s = status.write().await;
                                s.cluster_status = "live".to_string();
                                let _ = event_tx.send(WatchEvent {
                                    workspace: workspace.clone(),
                                    kind: "ClustersRefreshed".to_string(),
                                    file_path: format!("{} galaxies computed", galaxy_records.len()),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    sync_ms: Some(0),
                                });
                            }
                        }
                    }
                    files_changed_since_cluster = 0;
                }
            }
        }
    }

    info!("Watcher loop ended for workspace '{}'", workspace);
}

/// Determine if a notify event should be processed
fn should_process_event(event: &Event, _root: &PathBuf) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    for path in &event.paths {
        // Check if any path component is an excluded directory
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy();
            if EXCLUDED_DIRS.iter().any(|d| name.as_ref() == *d) {
                return false;
            }
        }

        // Check file extension
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if EXCLUDED_EXTS.contains(&ext) {
                return false;
            }
        }
    }

    true
}
