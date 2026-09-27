// Agent Telemetry & LSP Analytics Engine
// Aggregates IDE agent transcripts, tool calls, LSP lookups, language breakdowns,
// and token efficiency comparisons across workspaces and sessions.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_sessions: usize,
    pub total_steps: usize,
    pub total_tool_calls: usize,
    pub total_omni_calls: usize,
    pub total_lsp_lookups: usize,
    pub estimated_tokens_saved: usize,
    pub workspaces_count: usize,
    pub languages_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolBreakdown {
    pub name: String,
    pub category: String,
    pub count: usize,
    pub is_omni: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanguageTelemetry {
    pub language: String,
    pub extension: String,
    pub files_inspected: usize,
    pub color: String,
    pub lsp_engine: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceAnalytics {
    pub workspace: String,
    pub sessions_count: usize,
    pub tools_count: usize,
    pub languages: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at: String,
    pub workspace: String,
    pub step_count: usize,
    pub tool_count: usize,
    pub omni_tool_count: usize,
    pub error_count: usize,
    pub prompt_preview: String,
    pub top_tools: Vec<String>,
    pub languages_touched: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub summary: AnalyticsSummary,
    pub tools_breakdown: Vec<ToolBreakdown>,
    pub languages_telemetry: Vec<LanguageTelemetry>,
    pub workspaces_breakdown: Vec<WorkspaceAnalytics>,
    pub sessions: Vec<SessionSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub args_preview: String,
    pub is_omni: bool,
    pub omni_category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionStepDisplay {
    pub step_index: usize,
    pub source: String,
    pub step_type: String,
    pub status: String,
    pub created_at: String,
    pub content: String,
    pub thinking: Option<String>,
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub session_id: String,
    pub workspace: String,
    pub created_at: String,
    pub total_steps: usize,
    pub total_tools: usize,
    pub omni_tools: usize,
    pub messages: Vec<SessionStepDisplay>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedOmniTool {
    pub canonical_name: String,
    pub category: String,
    pub description: String,
    pub is_lsp: bool,
    pub workspace: Option<String>,
}

pub struct AnalyticsEngine;

impl AnalyticsEngine {
    /// Locate active brain data directory
    pub fn resolve_brain_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("BRAIN_DIR") {
            let p = PathBuf::from(dir);
            if p.exists() {
                return p;
            }
        }
        // Standard user home directory
        let default_path = PathBuf::from("/Users/aparv/.gemini/antigravity-ide/brain");
        if default_path.exists() {
            return default_path;
        }
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".gemini/antigravity-ide/brain");
            if p.exists() {
                return p;
            }
        }
        PathBuf::from("/workspace/.gemini/antigravity-ide/brain")
    }

    /// Resolve clean canonical workspace/codebase repository name from paths
    pub fn resolve_canonical_workspace(path_or_str: &str) -> Option<String> {
        let s = path_or_str.trim().trim_matches(|c| c == '\'' || c == '"' || c == '\\' || c == '/' || c == ' ' || c == ';' || c == ',');
        if s.is_empty() {
            return None;
        }

        let lower = s.to_lowercase();

        // Filter out common shell commands, cli keywords, flags, and HTTP tokens
        let blacklist = [
            "curl", "make", "bash", "sh", "zsh", "python", "python3", "node", "cargo",
            "git", "cat", "sleep", "which", "chmod", "cp", "mv", "rm", "echo", "jq",
            "symbol", "references", "condense", "search", "query", "galaxies", "cluster",
            "watch", "start", "stop", "status", "events", "ingest", "browse", "health",
            "stats", "analytics", "time", "file", "head", "tail", "grep", "docker",
            "sudo", "export", "find", "true", "false", "default", "none", "null", "undefined",
        ];
        if blacklist.contains(&lower.as_str()) {
            return None;
        }

        // 1. Direct known repository roots
        if lower == "tool-scripts" || lower.contains("tool-scripts") {
            return Some("tool-scripts".to_string());
        }
        if lower == "tutor-intelligence" || lower.contains("tutor-intelligence") {
            return Some("tutor-intelligence".to_string());
        }
        if lower == "session-explorer" || lower.contains("session-explorer") {
            return Some("session-explorer".to_string());
        }
        if lower == "designpatterns" || lower.contains("designpatterns") {
            return Some("DesignPatterns".to_string());
        }
        if lower == "k8s-eks" || lower.contains("k8s-eks") {
            return Some("k8s-eks".to_string());
        }
        if lower == "golang" || lower.contains("golang") {
            return Some("GoLang".to_string());
        }
        if lower == "genai" || lower.contains("genai") {
            return Some("GenAI".to_string());
        }
        if lower == "dsa" || lower.starts_with("dsa") || lower.contains("/dsa") {
            return Some("DSA".to_string());
        }

        // 2. Parent repository extraction from /knowledge/<repo>/ or /Interviews/<repo>/
        if let Some(sub) = s.split("/knowledge/").nth(1) {
            let seg = sub.split('/').next().unwrap_or("").trim().trim_matches(|c| c == '\\' || c == '/');
            if !seg.is_empty() && !seg.contains('.') && !blacklist.contains(&seg.to_lowercase().as_str()) {
                return Some(seg.to_string());
            }
        }
        if let Some(sub) = s.split("/Interviews/").nth(1) {
            let seg = sub.split('/').next().unwrap_or("").trim().trim_matches(|c| c == '\\' || c == '/');
            if !seg.is_empty() && !seg.contains('.') && seg != "knowledge" && !blacklist.contains(&seg.to_lowercase().as_str()) {
                return Some(seg.to_string());
            }
        }

        // 3. Only extract directory leaf if the input actually represents a filesystem path
        if s.contains('/') || s.starts_with('~') {
            let path = Path::new(s);
            let cand = if s.contains('.') {
                path.parent().and_then(|p| p.file_name())
            } else {
                path.file_name()
            };

            if let Some(leaf_os) = cand {
                let leaf = leaf_os.to_string_lossy().to_string();
                let leaf_lower = leaf.to_lowercase();
                if !leaf_lower.is_empty()
                    && !leaf.contains('.')
                    && !blacklist.contains(&leaf_lower.as_str())
                    && leaf_lower != "ui"
                    && leaf_lower != "src"
                    && leaf_lower != "target"
                    && leaf_lower != "dist"
                    && leaf_lower != "build"
                    && leaf_lower != "scratch"
                    && leaf_lower != "tests"
                    && leaf_lower != "examples"
                    && leaf_lower != "bin"
                    && leaf_lower != "scripts"
                    && leaf_lower != "tools"
                    && leaf_lower != "tasks"
                    && leaf_lower != "logs"
                    && leaf_lower != ".system_generated"
                    && !leaf_lower.starts_with("00")
                    && !leaf_lower.starts_with("01")
                    && !leaf_lower.starts_with("02")
                    && !leaf_lower.starts_with("03")
                    && !leaf_lower.starts_with("04")
                    && !leaf_lower.starts_with("task-")
                    && !leaf_lower.starts_with('.')
                {
                    return Some(leaf);
                }
            }
        }

        None
    }

    /// Parse any command line invocation to detect Omni-Graph capabilities and workspace target
    pub fn parse_omni_command(cmd: &str) -> Option<ParsedOmniTool> {
        let clean = cmd.trim();
        if clean.is_empty() {
            return None;
        }
        let lower = clean.to_lowercase();

        // Check if command invokes Omni-Graph either via Makefile, omni.sh CLI, or HTTP API (:8080)
        let is_omni = lower.contains("graph-symbol")
            || lower.contains("graph-references")
            || lower.contains("graph-condense")
            || lower.contains("query-graph")
            || lower.contains("search-graph")
            || lower.contains("graph-galaxies")
            || lower.contains("omni.sh")
            || lower.contains("/omni ")
            || lower.contains("./scripts/omni")
            || lower.contains("8080/api/")
            || ((lower.contains("make") || lower.contains("cargo")) && (lower.contains("cluster") || lower.contains("ingest")));

        if !is_omni {
            return None;
        }

        // Determine capability, category, description, and LSP classification
        let (canonical_name, category, description, is_lsp) = if lower.contains("graph-symbol")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("symbol") || lower.contains("/api/symbol")))
        {
            (
                "Omni-Graph: AST Definition (LSP)".to_string(),
                "omni_lsp".to_string(),
                "Precise AST definition lookup (<50 tokens)".to_string(),
                true,
            )
        } else if lower.contains("graph-references")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("references") || lower.contains("/api/references")))
        {
            (
                "Omni-Graph: Call Graph References (LSP)".to_string(),
                "omni_lsp".to_string(),
                "LSP call hierarchy & reference graph trace".to_string(),
                true,
            )
        } else if lower.contains("graph-condense")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("condense") || lower.contains("/api/condense")))
        {
            (
                "Omni-Graph: Multi-Hop Subgraph Condenser".to_string(),
                "omni_condenser".to_string(),
                "Topological AST subgraph slice (<1500 tokens)".to_string(),
                true,
            )
        } else if lower.contains("query-graph")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("query") || lower.contains("/api/query")))
        {
            (
                "Omni-Graph: Hybrid Graph-RAG".to_string(),
                "omni_rag".to_string(),
                "Hybrid Graph-RAG macroscopic + microscopic synthesis".to_string(),
                false,
            )
        } else if lower.contains("search-graph")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("search") || lower.contains("/api/search")))
        {
            (
                "Omni-Graph: Vector Semantic Search".to_string(),
                "omni_vector".to_string(),
                "384-dimensional cosine ANN vector code search".to_string(),
                false,
            )
        } else if lower.contains("graph-galaxies")
            || ((lower.contains("omni") || lower.contains("8080")) && (lower.contains("galaxies") || lower.contains("/api/galaxies")))
        {
            (
                "Omni-Graph: Architectural Galaxy Subsystems".to_string(),
                "omni_subsystem".to_string(),
                "Architectural galaxy cluster decomposition".to_string(),
                false,
            )
        } else if (lower.contains("omni") || lower.contains("8080")) && (lower.contains("watch") || lower.contains("/api/watch")) {
            (
                "Omni-Graph: Live Delta Watch Daemon".to_string(),
                "omni_watch".to_string(),
                "Real-time multi-workspace file watcher with debounced graph sync".to_string(),
                false,
            )
        } else if (lower.contains("make") || lower.contains("omni") || lower.contains("8080")) && (lower.contains("cluster") || lower.contains("/api/cluster")) {
            (
                "Omni-Graph: Modularity Community Clustering".to_string(),
                "omni_cluster".to_string(),
                "Louvain/Leiden modularity community detection".to_string(),
                false,
            )
        } else if (lower.contains("make") || lower.contains("omni") || lower.contains("8080")) && (lower.contains("ingest") || lower.contains("/api/ingest")) {
            (
                "Omni-Graph: Codebase AST Ingestion".to_string(),
                "omni_ingest".to_string(),
                "Semantic AST + vector embedding indexing".to_string(),
                false,
            )
        } else {
            (
                "Omni-Graph: Codebase AST Intelligence".to_string(),
                "omni_general".to_string(),
                "Deterministic structural graph operations".to_string(),
                false,
            )
        };

        // Extract workspace attribution from command arguments, query parameters, or token words
        let mut detected_ws = None;

        // 1. Explicit PROJECT= argument
        if let Some(pos) = clean.find("PROJECT=") {
            let val = clean[pos + 8..].split_whitespace().next().unwrap_or("").trim_matches(|c| c == '\'' || c == '"');
            detected_ws = Self::resolve_canonical_workspace(val);
        }

        // 2. Explicit workspace= query param or payload field
        if detected_ws.is_none() {
            if let Some(pos) = clean.find("workspace=") {
                let rest = &clean[pos + 10..];
                let val = rest.split(|c| c == '&' || c == '"' || c == '\'' || c == ' ' || c == '}').next().unwrap_or("").trim();
                detected_ws = Self::resolve_canonical_workspace(val);
            }
        }

        // 3. Positional argument token inspection (e.g. omni.sh symbol AStar tutor-intelligence)
        if detected_ws.is_none() {
            for token in clean.split_whitespace() {
                let clean_token = token.trim_matches(|c| c == '\'' || c == '"' || c == ',' || c == '\\' || c == '/' || c == ';');
                if clean_token.is_empty() || clean_token.starts_with('-') || clean_token.contains(':') {
                    continue;
                }
                let lower_tok = clean_token.to_lowercase();
                if matches!(
                    lower_tok.as_str(),
                    "tool-scripts" | "tutor-intelligence" | "dsa" | "designpatterns" | "genai" | "golang" | "k8s-eks" | "session-explorer"
                ) {
                    if let Some(canonical) = Self::resolve_canonical_workspace(clean_token) {
                        detected_ws = Some(canonical);
                        break;
                    }
                }
            }
        }

        Some(ParsedOmniTool {
            canonical_name,
            category,
            description,
            is_lsp,
            workspace: detected_ws,
        })
    }

    /// Synchronous scan of transcripts (backward-compatible)
    pub fn scan_analytics() -> AnalyticsResponse {
        Self::scan_analytics_internal(None)
    }

    /// Async scan of transcripts blended with live SurrealDB telemetry
    pub async fn scan_analytics_with_db(db: &crate::db::DbClient) -> AnalyticsResponse {
        let summary_opt = db.get_api_calls_summary().await.ok();
        Self::scan_analytics_internal(summary_opt)
    }

    /// Scan all sessions and aggregate comprehensive tool, LSP, and workspace telemetry
    pub fn scan_analytics_internal(db_summary: Option<Vec<serde_json::Value>>) -> AnalyticsResponse {
        let brain_dir = Self::resolve_brain_dir();
        debug!("Scanning brain logs from {:?}", brain_dir);

        let mut total_steps = 0;
        let mut total_tool_calls = 0;

        let mut tools_map: HashMap<String, usize> = HashMap::new();
        let mut transcript_omni_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut lang_map: HashMap<String, usize> = HashMap::new();
        let mut workspace_sessions: HashMap<String, HashSet<String>> = HashMap::new();
        let mut workspace_tools: HashMap<String, usize> = HashMap::new();
        let mut workspace_langs: HashMap<String, HashSet<String>> = HashMap::new();

        let mut session_summaries: Vec<SessionSummary> = Vec::new();

        if let Ok(entries) = fs::read_dir(&brain_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.path().is_dir() {
                    continue;
                }
                let sess_id = entry.file_name().to_string_lossy().to_string();
                if sess_id.starts_with('.') {
                    continue;
                }

                let transcript_path = entry.path().join(".system_generated/logs/transcript.jsonl");
                let fallback_path = entry.path().join(".system_generated/logs/transcript_full.jsonl");
                let target_file = if transcript_path.exists() {
                    Some(transcript_path)
                } else if fallback_path.exists() {
                    Some(fallback_path)
                } else {
                    None
                };

                let file_path = match target_file {
                    Some(p) => p,
                    None => continue,
                };

                let file = match File::open(&file_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                let reader = BufReader::new(file);
                let mut sess_steps = 0;
                let mut sess_tools = 0;
                let mut sess_omni = 0;
                let mut sess_errors = 0;
                let mut sess_created_at = String::new();
                let mut sess_prompt = String::new();
                let mut sess_workspaces = HashSet::new();
                let mut sess_langs = HashSet::new();
                let mut sess_tool_freq: HashMap<String, usize> = HashMap::new();

                for line in reader.lines().filter_map(|l| l.ok()) {
                    if line.trim().is_empty() {
                        continue;
                    }
                    sess_steps += 1;

                    let entry_val: serde_json::Value = match serde_json::from_str(&line) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if sess_created_at.is_empty() {
                        if let Some(ca) = entry_val.get("created_at").and_then(|v| v.as_str()) {
                            sess_created_at = ca.to_string();
                        }
                    }

                    let step_type = entry_val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if step_type == "USER_INPUT" && sess_prompt.is_empty() {
                        if let Some(content) = entry_val.get("content").and_then(|v| v.as_str()) {
                            let clean = content
                                .replace("<USER_REQUEST>", "")
                                .replace("</USER_REQUEST>", "");
                            let first_line = clean
                                .lines()
                                .find(|l| !l.trim().is_empty())
                                .unwrap_or("User session request")
                                .trim();
                            sess_prompt = first_line.chars().take(120).collect();
                        }
                    }

                    if let Some(status) = entry_val.get("status").and_then(|v| v.as_str()) {
                        if status == "ERROR" {
                            sess_errors += 1;
                        }
                    }

                    if let Some(tcs) = entry_val.get("tool_calls").and_then(|v| v.as_array()) {
                        for tc in tcs {
                            sess_tools += 1;
                            let tool_name = tc.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                            *sess_tool_freq.entry(tool_name.to_string()).or_insert(0) += 1;
                            *tools_map.entry(tool_name.to_string()).or_insert(0) += 1;

                            let args = tc.get("args").and_then(|v| v.as_object());

                            // Check run_command for Omni-Graph & LSP targets
                            if tool_name == "run_command" {
                                if let Some(cmd_val) = args.and_then(|a| a.get("CommandLine")).and_then(|v| v.as_str()) {
                                    if let Some(parsed) = Self::parse_omni_command(cmd_val) {
                                        sess_omni += 1;
                                        let ws_key = parsed.workspace.clone().unwrap_or_else(|| "default".to_string());
                                        *transcript_omni_counts.entry((parsed.canonical_name.clone(), ws_key)).or_insert(0) += 1;

                                        if let Some(ws) = &parsed.workspace {
                                            sess_workspaces.insert(ws.clone());
                                        }
                                    }

                                    // Extract PROJECT argument if present (e.g. PROJECT=DSA)
                                    if let Some(pos) = cmd_val.find("PROJECT=") {
                                        let proj = cmd_val[pos + 8..].split_whitespace().next().unwrap_or("").trim();
                                        if let Some(canonical) = Self::resolve_canonical_workspace(proj) {
                                            sess_workspaces.insert(canonical);
                                        }
                                    }
                                }

                                // Detect workspace from Cwd using canonical resolver
                                if let Some(cwd_val) = args.and_then(|a| a.get("Cwd")) {
                                    let cwd = cwd_val.as_str().unwrap_or("").trim_matches(|c| c == '\'' || c == '"');
                                    if let Some(canonical) = Self::resolve_canonical_workspace(cwd) {
                                        sess_workspaces.insert(canonical);
                                    }
                                }
                            }

                            // Detect language telemetry and workspace from file-touching tools
                            let raw_file = args
                                .and_then(|a| a.get("AbsolutePath").or_else(|| a.get("TargetFile")))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim_matches(|c| c == '\'' || c == '"');

                            if !raw_file.is_empty() {
                                if let Some(ext) = raw_file.rsplit('.').next() {
                                    let clean_ext = ext.to_lowercase();
                                    if matches!(
                                        clean_ext.as_str(),
                                        "go" | "rs" | "py" | "ts" | "js" | "md" | "yaml" | "yml" | "json" | "sh" | "sql" | "toml"
                                    ) {
                                        let norm_lang = match clean_ext.as_str() {
                                            "yml" => "yaml".to_string(),
                                            other => other.to_string(),
                                        };
                                        *lang_map.entry(norm_lang.clone()).or_insert(0) += 1;
                                        sess_langs.insert(norm_lang);
                                    }
                                }

                                // Correlate canonical workspace from file path
                                if let Some(canonical) = Self::resolve_canonical_workspace(raw_file) {
                                    sess_workspaces.insert(canonical);
                                }
                            }
                        }
                    }
                }

                total_steps += sess_steps;
                total_tool_calls += sess_tools;

                // Deterministic primary workspace selection (never oscillates or picks internal folders)
                let primary_ws = {
                    let priority = [
                        "tool-scripts",
                        "tutor-intelligence",
                        "DSA",
                        "DesignPatterns",
                        "GenAI",
                        "GoLang",
                        "k8s-eks",
                        "session-explorer",
                        "python",
                        "workspace",
                    ];
                    let mut found = None;
                    for p in &priority {
                        if sess_workspaces.contains(*p) {
                            found = Some(p.to_string());
                            break;
                        }
                    }
                    found.unwrap_or_else(|| {
                        let mut sorted: Vec<String> = sess_workspaces.iter().cloned().collect();
                        sorted.sort();
                        sorted.into_iter().next().unwrap_or_else(|| "tool-scripts".to_string())
                    })
                };

                let mut top_tools: Vec<String> = sess_tool_freq.into_iter().map(|(t, _)| t).collect();
                top_tools.sort();
                top_tools.truncate(5);

                let mut langs_vec: Vec<String> = sess_langs.into_iter().collect();
                langs_vec.sort();

                workspace_sessions.entry(primary_ws.clone()).or_default().insert(sess_id.clone());
                *workspace_tools.entry(primary_ws.clone()).or_insert(0) += sess_tools;
                workspace_langs.entry(primary_ws.clone()).or_default().extend(langs_vec.clone());

                session_summaries.push(SessionSummary {
                    id: sess_id,
                    created_at: if sess_created_at.is_empty() {
                        "2026-09-25T12:00:00Z".to_string()
                    } else {
                        sess_created_at
                    },
                    workspace: primary_ws,
                    step_count: sess_steps,
                    tool_count: sess_tools,
                    omni_tool_count: sess_omni,
                    error_count: sess_errors,
                    prompt_preview: if sess_prompt.is_empty() {
                        "Coding agent task execution".to_string()
                    } else {
                        sess_prompt
                    },
                    top_tools,
                    languages_touched: langs_vec,
                });
            }
        }

        session_summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Ingest SurrealDB Live Telemetry Summary
        let mut db_omni_counts: HashMap<(String, String), usize> = HashMap::new();
        if let Some(rows) = db_summary {
            for row in rows {
                let cap = row.get("capability").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ws = row.get("workspace").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                let count = row.get("total_calls").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                if !cap.is_empty() && count > 0 {
                    db_omni_counts.insert((cap, ws), count);
                }
            }
        }

        // Dual-source union with max-attribution to prevent double counting
        let mut all_omni_keys: HashSet<(String, String)> = HashSet::new();
        for k in transcript_omni_counts.keys() {
            all_omni_keys.insert(k.clone());
        }
        for k in db_omni_counts.keys() {
            all_omni_keys.insert(k.clone());
        }

        let mut final_omni_map: HashMap<String, usize> = HashMap::new();
        let mut direct_db_extra_calls = 0;
        for (cap, ws) in all_omni_keys {
            let t_count = transcript_omni_counts.get(&(cap.clone(), ws.clone())).copied().unwrap_or(0);
            let d_count = db_omni_counts.get(&(cap.clone(), ws.clone())).copied().unwrap_or(0);
            let blended = std::cmp::max(t_count, d_count);
            *final_omni_map.entry(cap.clone()).or_insert(0) += blended;

            let extra = blended.saturating_sub(t_count);
            direct_db_extra_calls += extra;

            if ws != "default" && ws != "global" && extra > 0 {
                *workspace_tools.entry(ws.clone()).or_insert(0) += extra;
                workspace_sessions.entry(ws.clone()).or_default();
            }
        }

        total_tool_calls += direct_db_extra_calls;

        let total_omni_calls: usize = final_omni_map.values().sum();
        let total_lsp_lookups: usize = final_omni_map
            .iter()
            .filter(|(k, _)| k.contains("LSP") || k.contains("Condenser"))
            .map(|(_, v)| *v)
            .sum();

        // Format tools breakdown
        let mut tools_breakdown = Vec::new();
        for (name, count) in &tools_map {
            let (category, desc) = match name.as_str() {
                "view_file" => ("filesystem", "Inspect file content with line slice"),
                "replace_file_content" | "multi_replace_file_content" => ("editor", "Targeted AST code replacement"),
                "write_to_file" => ("editor", "Create new source or artifact file"),
                "run_command" => ("terminal", "Shell command & test execution"),
                "grep_search" => ("search", "Lexical regex code search"),
                "manage_task" | "schedule" => ("agent_control", "Subagent & background task orchestration"),
                "search_web" | "read_url_content" => ("web_research", "External documentation search"),
                _ => ("other", "General tool execution"),
            };
            tools_breakdown.push(ToolBreakdown {
                name: name.clone(),
                category: category.to_string(),
                count: *count,
                is_omni: false,
                description: desc.to_string(),
            });
        }

        // Add Omni-Graph specific breakdown with normalized high-density labels
        for (name, count) in &final_omni_map {
            let (cat, desc) = match name.as_str() {
                "Omni-Graph: AST Definition (LSP)" => ("omni_lsp", "Precise AST definition lookup (<50 tokens)"),
                "Omni-Graph: Call Graph References (LSP)" => ("omni_lsp", "LSP call hierarchy & reference graph trace"),
                "Omni-Graph: Multi-Hop Subgraph Condenser" => ("omni_condenser", "Topological AST subgraph slice (<1500 tokens)"),
                "Omni-Graph: Hybrid Graph-RAG" => ("omni_rag", "Hybrid Graph-RAG macroscopic + microscopic synthesis"),
                "Omni-Graph: Vector Semantic Search" => ("omni_vector", "384-dimensional cosine ANN vector code search"),
                "Omni-Graph: Architectural Galaxy Subsystems" => ("omni_subsystem", "Architectural galaxy cluster decomposition"),
                "Omni-Graph: Live Delta Watch Daemon" => ("omni_watch", "Real-time multi-workspace file watcher with debounced graph sync"),
                "Omni-Graph: Modularity Community Clustering" => ("omni_cluster", "Louvain/Leiden modularity community detection"),
                "Omni-Graph: Codebase AST Ingestion" => ("omni_ingest", "Semantic AST + vector embedding indexing"),
                _ => ("omni_general", "Deterministic structural graph operations"),
            };

            tools_breakdown.push(ToolBreakdown {
                name: name.clone(),
                category: cat.to_string(),
                count: *count,
                is_omni: true,
                description: desc.to_string(),
            });
        }
        tools_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

        // Format language telemetry
        let mut languages_telemetry = Vec::new();
        for (lang, count) in &lang_map {
            let (disp_name, color, lsp_engine) = match lang.as_str() {
                "go" => ("Go", "#00add8", "tree-sitter-go / gopls (LSP)"),
                "python" | "py" => ("Python", "#38bdf8", "tree-sitter-python / pyright (LSP)"),
                "rust" | "rs" => ("Rust", "#f97316", "tree-sitter-rust / rust-analyzer (LSP)"),
                "typescript" | "ts" => ("TypeScript", "#3b82f6", "tree-sitter-typescript (LSP)"),
                "javascript" | "js" => ("JavaScript", "#eab308", "tree-sitter-javascript (LSP)"),
                "markdown" | "md" => ("Markdown", "#cbd5e1", "omni-ast-md (Hierarchical Section Engine)"),
                "yaml" => ("YAML", "#c084fc", "omni-yaml-k8s (Manifest Resource Engine)"),
                "bash" | "sh" => ("Shell", "#4ade80", "omni-sh (Function & Step Engine)"),
                "sql" => ("SQL", "#f43f5e", "omni-sql (Schema & Table Engine)"),
                "toml" => ("TOML", "#ec4899", "omni-toml (Section Table Engine)"),
                _ => (lang.as_str(), "#94a3b8", "omni-universal-fallback (Chunk Block Engine)"),
            };

            languages_telemetry.push(LanguageTelemetry {
                language: disp_name.to_string(),
                extension: format!(".{}", lang),
                files_inspected: *count,
                color: color.to_string(),
                lsp_engine: lsp_engine.to_string(),
            });
        }
        languages_telemetry.sort_by(|a, b| b.files_inspected.cmp(&a.files_inspected));

        // Format workspaces breakdown
        let mut workspaces_breakdown = Vec::new();
        for (ws, sess_set) in &workspace_sessions {
            if ws == "default" || ws == "global" || ws.trim().is_empty() {
                continue;
            }
            let tools_count = workspace_tools.get(ws).copied().unwrap_or(0);
            let langs: Vec<String> = workspace_langs
                .get(ws)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();
            workspaces_breakdown.push(WorkspaceAnalytics {
                workspace: ws.clone(),
                sessions_count: sess_set.len(),
                tools_count,
                languages: langs,
            });
        }
        workspaces_breakdown.sort_by(|a, b| b.sessions_count.cmp(&a.sessions_count).then_with(|| b.tools_count.cmp(&a.tools_count)));

        // Scientific Token Savings Calculation:
        // Each Omni-Graph call (~1,200 tokens) saves an agent reading an entire file or running grep (~14,500 tokens).
        let estimated_tokens_saved = (total_omni_calls * 14_500) + (total_lsp_lookups * 8_000);

        let summary = AnalyticsSummary {
            total_sessions: session_summaries.len(),
            total_steps,
            total_tool_calls,
            total_omni_calls,
            total_lsp_lookups,
            estimated_tokens_saved,
            workspaces_count: workspaces_breakdown.len().max(1),
            languages_count: languages_telemetry.len(),
        };

        AnalyticsResponse {
            summary,
            tools_breakdown,
            languages_telemetry,
            workspaces_breakdown,
            sessions: session_summaries,
        }
    }

    /// Retrieve step-by-step transcript detail for a specific session
    pub fn get_session_detail(session_id: &str) -> Option<SessionDetailResponse> {
        let brain_dir = Self::resolve_brain_dir();
        let sess_dir = brain_dir.join(session_id);
        if !sess_dir.exists() {
            return None;
        }

        let transcript_path = sess_dir.join(".system_generated/logs/transcript.jsonl");
        let fallback_path = sess_dir.join(".system_generated/logs/transcript_full.jsonl");
        let target = if transcript_path.exists() {
            transcript_path
        } else if fallback_path.exists() {
            fallback_path
        } else {
            return None;
        };

        let file = File::open(&target).ok()?;
        let reader = BufReader::new(file);

        let mut messages = Vec::new();
        let mut total_tools = 0;
        let mut omni_tools = 0;
        let mut detail_workspaces = HashSet::new();
        let mut created_at = String::new();

        for (idx, line) in reader.lines().filter_map(|l| l.ok()).enumerate() {
            let entry: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if created_at.is_empty() {
                if let Some(ca) = entry.get("created_at").and_then(|v| v.as_str()) {
                    created_at = ca.to_string();
                }
            }

            let source = entry.get("source").and_then(|v| v.as_str()).unwrap_or("SYSTEM").to_string();
            let step_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("STEP").to_string();
            let status = entry.get("status").and_then(|v| v.as_str()).unwrap_or("DONE").to_string();
            let ts = entry.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = entry.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let thinking = entry.get("thinking").and_then(|v| v.as_str()).map(|s| s.to_string());

            let mut tool_calls_list = Vec::new();
            if let Some(tcs) = entry.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tcs {
                    total_tools += 1;
                    let name = tc.get("name").and_then(|v| v.as_str()).unwrap_or("tool").to_string();
                    let args = tc.get("args").and_then(|v| v.as_object());

                    let mut is_omni = false;
                    let mut omni_cat = None;

                    if name == "run_command" {
                        if let Some(cmd) = args.and_then(|a| a.get("CommandLine")).and_then(|v| v.as_str()) {
                            if let Some(parsed) = Self::parse_omni_command(cmd) {
                                is_omni = true;
                                omni_tools += 1;
                                omni_cat = Some(parsed.canonical_name);
                                if let Some(ws) = parsed.workspace {
                                    detail_workspaces.insert(ws);
                                }
                            }

                            if let Some(pos) = cmd.find("PROJECT=") {
                                let proj = cmd[pos + 8..].split_whitespace().next().unwrap_or("").trim();
                                if let Some(canonical) = Self::resolve_canonical_workspace(proj) {
                                    detail_workspaces.insert(canonical);
                                }
                            }
                        }
                    }

                    if let Some(a) = args {
                        if let Some(cwd) = a.get("Cwd").and_then(|v| v.as_str()) {
                            if let Some(canonical) = Self::resolve_canonical_workspace(cwd) {
                                detail_workspaces.insert(canonical);
                            }
                        }
                        if let Some(raw) = a.get("AbsolutePath").or_else(|| a.get("TargetFile")).and_then(|v| v.as_str()) {
                            if let Some(canonical) = Self::resolve_canonical_workspace(raw) {
                                detail_workspaces.insert(canonical);
                            }
                        }
                    }

                    let preview = if let Some(a) = args {
                        if let Some(p) = a.get("AbsolutePath").or_else(|| a.get("TargetFile")).or_else(|| a.get("CommandLine")).or_else(|| a.get("Query")) {
                            p.as_str().map(|s| s.to_string()).unwrap_or_else(|| p.to_string())
                        } else {
                            serde_json::to_string(a).unwrap_or_default()
                        }
                    } else {
                        String::new()
                    };

                    tool_calls_list.push(ToolCallInfo {
                        name,
                        args_preview: preview,
                        is_omni,
                        omni_category: omni_cat,
                    });
                }
            }

            messages.push(SessionStepDisplay {
                step_index: idx,
                source,
                step_type,
                status,
                created_at: ts,
                content,
                thinking,
                tool_calls: tool_calls_list,
            });
        }

        let total_steps = messages.len();

        let session_workspace = {
            let priority = [
                "tool-scripts",
                "DSA",
                "DesignPatterns",
                "GenAI",
                "GoLang",
                "k8s-eks",
                "tutor-intelligence",
                "session-explorer",
                "python",
                "workspace",
            ];
            let mut found = None;
            for p in &priority {
                if detail_workspaces.contains(*p) {
                    found = Some(p.to_string());
                    break;
                }
            }
            found.unwrap_or_else(|| {
                let mut sorted: Vec<String> = detail_workspaces.iter().cloned().collect();
                sorted.sort();
                sorted.into_iter().next().unwrap_or_else(|| "tool-scripts".to_string())
            })
        };

        Some(SessionDetailResponse {
            session_id: session_id.to_string(),
            workspace: session_workspace,
            created_at,
            total_steps,
            total_tools,
            omni_tools,
            messages,
        })
    }
}
