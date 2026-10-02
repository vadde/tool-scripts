// Delta event classifier: transforms raw notify events into semantic delta changesets
// Handles deduplication, rename tracking, and noise filtering

use chrono::{DateTime, Utc};
use notify::{
    event::{ModifyKind, RenameMode},
    Event, EventKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    Created,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaEvent {
    pub kind: DeltaKind,
    pub file_path: String,        // Relative to workspace root
    pub workspace: String,
    pub old_path: Option<String>,  // Only for Renamed
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaChangeset {
    pub workspace: String,
    pub events: Vec<DeltaEvent>,
    pub computed_at: DateTime<Utc>,
}

/// Classify and coalesce a batch of raw notify events into a unified changeset
pub fn classify_events(events: &[Event], workspace: &str, root: &Path) -> DeltaChangeset {
    let now = Utc::now();
    // Map of rel_path -> (DeltaKind, Option<old_rel_path>)
    let mut file_states: HashMap<String, (DeltaKind, Option<String>)> = HashMap::new();

    for event in events {
        match &event.kind {
            // Explicit rename events from notify (RenameMode::Both or From/To)
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if event.paths.len() >= 2 => {
                let old_rel = to_relative(&event.paths[0], root);
                let new_rel = to_relative(&event.paths[1], root);
                if let (Some(old_p), Some(new_p)) = (old_rel, new_rel) {
                    file_states.remove(&old_p);
                    file_states.insert(new_p.clone(), (DeltaKind::Renamed, Some(old_p)));
                }
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::From)) if !event.paths.is_empty() => {
                if let Some(old_p) = to_relative(&event.paths[0], root) {
                    file_states.insert(old_p.clone(), (DeltaKind::Deleted, None));
                }
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::To)) if !event.paths.is_empty() => {
                if let Some(new_p) = to_relative(&event.paths[0], root) {
                    file_states.insert(new_p, (DeltaKind::Created, None));
                }
            }
            EventKind::Create(_) => {
                for path in &event.paths {
                    if let Some(rel) = to_relative(path, root) {
                        // If file was previously marked Deleted, treat as Modified
                        if let Some((prev_kind, _)) = file_states.get(&rel) {
                            if *prev_kind == DeltaKind::Deleted {
                                file_states.insert(rel, (DeltaKind::Modified, None));
                                continue;
                            }
                        }
                        file_states.insert(rel, (DeltaKind::Created, None));
                    }
                }
            }
            EventKind::Modify(_) => {
                for path in &event.paths {
                    if let Some(rel) = to_relative(path, root) {
                        // If already marked Created or Renamed, keep that status
                        if let Some((prev_kind, _)) = file_states.get(&rel) {
                            if matches!(prev_kind, DeltaKind::Created | DeltaKind::Renamed) {
                                continue;
                            }
                        }
                        file_states.insert(rel, (DeltaKind::Modified, None));
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    if let Some(rel) = to_relative(path, root) {
                        // If it was created and deleted in the same debounce window, omit it entirely
                        if let Some((prev_kind, _)) = file_states.get(&rel) {
                            if *prev_kind == DeltaKind::Created {
                                file_states.remove(&rel);
                                continue;
                            }
                        }
                        file_states.insert(rel, (DeltaKind::Deleted, None));
                    }
                }
            }
            _ => {}
        }
    }

    // Convert coalesced states into DeltaEvents
    let mut events = Vec::new();
    for (file_path, (kind, old_path)) in file_states {
        events.push(DeltaEvent {
            kind,
            file_path,
            workspace: workspace.to_string(),
            old_path,
            timestamp: now,
        });
    }

    // Stable sort by path for deterministic processing
    events.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    DeltaChangeset {
        workspace: workspace.to_string(),
        events,
        computed_at: now,
    }
}

/// Convert an absolute path to a normalized relative path from the workspace root
fn to_relative(path: &Path, root: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
}
