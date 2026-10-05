# 📓 Development Log — omni-graph

> Chronological record of all development sessions on this tool.
> **Append-only** — never delete entries, only add new ones at the top.
> Each entry captures what happened, what changed, and what to do next.

### 2026-10-05 — Fix Clippy CI Failures & Provide Local CI Make Targets

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Maintenance: CI Hardening & Clippy Compliance)
**Branch**: `main`
**Duration**: ~15m

#### Problem & Symptoms
- GitHub Actions CI workflow run `37072950720` (job `111056422594`) failed during `make lint` on `cargo clippy -- -D warnings` with 20 errors.
- Developers lacked granular, modular make commands to reproduce and verify CI steps locally (individual lint, test, build, and combined CI pipeline) prior to pushing.

#### Root Cause Analysis
- Code written in recent features contained patterns flagged by modern Clippy when `-D warnings` is enforced:
  1. `src/watcher/mod.rs`: `manual_checked_ops` on average sync ms division, `too_many_arguments` on `run_watcher`, unused fields in `ActiveWatcher`.
  2. `src/analysis/mod.rs`: `contains_key` followed by `insert` in seed label mapping, single pattern match on community assignments, dead code on `infer_neighbor_community`.
  3. `src/analytics/mod.rs`: manual char comparison in query param parsing, infinite error risk in `lines().filter_map` (replaced with `map_while`), map keys iteration, `sort_by` instead of `sort_by_key`, dead code on `scan_analytics`.
  4. `src/condenser/mod.rs`: unnecessary `format!` on string literal truncation banner.
  5. `src/db/mod.rs`: `results.get(0)` and `arr.get(0)` instead of `.first()`, dead code on `get_api_recent_calls`.
  6. `src/ingestion/mod.rs`: `clone` on `Copy` `u64` inside `sort_by_key`.
  7. `src/parser/mod.rs`: `too_many_arguments` on `traverse_node`, manual range contains on heading level and slice length, collapsible `if`, and `map_or` on first char.
- Local `make lint` lacked fallback to install clippy in the Docker fallback container, causing local developers to miss clippy warnings before pushing.

#### Solutions & Improvements
1. **Resolved All 20 Clippy Lints**: Refactored all affected locations to satisfy modern Clippy with zero warnings under `-D warnings`.
2. **Modular Make Commands in `tools/omni-graph/Makefile`**:
   - `make lint-rust`: Runs `cargo clippy -- -D warnings` (with automatic Docker fallback if local rustup lacks clippy).
   - `make lint-ui`: Runs `tsc --noEmit` via npm in `ui/`.
   - `make lint`: Composite target running `lint-rust` and `lint-ui`.
   - `make test-rust`: Runs Rust unit test suite (`cargo test --all -- --test-threads=1`).
   - `make test-ui`: Runs UI test suite (`npm test`).
   - `make test`: Composite target running `test-rust` and `test-ui`.
   - `make build-rust`: Compiles release binary locally or via Docker.
   - `make build-ui`: Builds static UI assets.
   - `make build`: Builds full 4-container Docker Compose stack.
   - `make ci`: Comprehensive local pipeline running `lint`, `test`, and `validate-specs`.
3. **Monorepo Integration**: Added `make ci-tool T=omni-graph` to root `Makefile`.
4. **Verification**: `make ci` and `make ci-tool T=omni-graph` executed cleanly (51/51 tests passing, 0 clippy warnings).

---

### 2026-10-02 — Repo-Wide Parity & Machine-Agnostic Agent Enforcement Setup

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Maintenance: Agent Parity & Portable Configuration)
**Branch**: `main`
**Duration**: ~10m

#### Context & Objectives
Ensure absolute parity between repo-level sources (`tools/omni-graph/`, `.agents/`) and machine configurations (`~/.gemini/config/`), and eliminate machine-specific hardcoded paths from `.agents/hooks.json` so collaborators cloning the repo receive an immediate, deterministic, error-free onboarding experience via `make setup-agent`.

#### What Was Done
1. **Source Parity Synchronization**:
   - Synchronized `tools/omni-graph/rules/00-omni-graph-mandatory-retrieval.md` with `.agents/rules/` and `~/.gemini/config/rules/` (including 450-line documentation ceiling and search line numbers).
   - Synchronized `tools/omni-graph/skills/omni-graph/scripts/omni.sh` with the latest CLI subcommands (`relate`, `watch start/stop/status/events`).
2. **Machine-Agnostic Portable Hook Execution**:
   - Converted `.agents/hooks.json` from user-specific absolute paths (`/Users/aparv/...`) to portable relative paths (`./scripts/hook_pre_tool.sh`, etc.), which Antigravity evaluates relative to the directory containing `hooks.json`.
   - Updated `tools/omni-graph/scripts/setup-agent.sh` so `make setup-agent` configures portable `./scripts/...` paths for workspace installations.
3. **End-to-End Verification**:
   - Verified `make setup-agent` executes with exit code 0.
   - Verified lifecycle hooks fire reliably using relative paths.
   - All 51 unit tests passing (`make test-tool T=omni-graph`).

---

### 2026-10-02 — Critical Hotfix: Resolve Watcher Self-Deadlock & UI Gateway 504 Timeouts

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Hotfix: WatchManager Deadlock & Polling Recovery)
**Branch**: `main`
**Duration**: ~15m

#### Problem & Symptoms
- Nginx reverse proxy logged persistent `504 Gateway Timeout` errors when requesting `GET /api/watch/status`.
- The Web UI froze completely: unable to refresh, start/stop watchers, or ingest new repositories.

#### Root Cause Analysis
1. **Unscoped `status.write()` Self-Deadlock (`src/watcher/mod.rs:518`)**:
   - In `run_watcher`, the 3-second periodic sweep acquired `let mut s = status.write().await;` at line 518 without an enclosing scope block.
   - When background file changes occurred, execution flowed directly into quiescent auto-clustering (lines 538–569), which attempted to acquire `let mut s = status.write().await;` a second time at line 556 on the exact same task.
   - Because `tokio::sync::RwLock` write locks are not re-entrant, the background watcher task deadlocked itself.
2. **Cascading Lock Contention in `WatchManager`**:
   - `get_status()` held `self.watchers.lock().await` while sequentially calling `.read().await` on each watcher's status lock.
   - Because `watcher.status` was permanently held by the deadlocked write lock, `get_status()` stalled indefinitely while holding `watchers.lock()`.
   - Every subsequent request to `/api/watch/status`, `/api/watch/start`, or `/api/watch/stop` piled up behind the mutex until Nginx timed out after 600s.
3. **Hyperactive Periodic Disk Sweep**:
   - A 3-second sweep interval was repeatedly invoking `prune_missing_files` and `count_tracked_files` (`WalkDir`) over mounted filesystems, causing excessive I/O contention.

#### Fixes Implemented
1. **Scoped Lock Retention**: Enclosed the line 518 status write lock in an explicit `{ let mut s = status.write().await; ... }` block, guaranteeing the write guard drops before quiescent auto-clustering executes.
2. **Decoupled Mutex Retention**: Rewrote `get_status` and `get_workspace_status` to clone the `Arc<RwLock<WatcherStatus>>` handles and release `self.watchers.lock()` immediately before awaiting reader locks.
3. **Calibrated Sweep Cadence**: Extended `sweep_interval` from 3s to 30s to prevent unnecessary disk thrashing.
4. **Verified Live**: Rebuilt `omni-rust-app` container; confirmed `GET /api/watch/status` responds in <5ms through Nginx on port 3000. All 51 unit tests passing.

---

### 2026-10-02 — First-Class Markdown Retrieval, Cross-Modal References & Calibrated Guardrails

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Markdown Semantic Retrieval & Guardrail Calibration)
**Branch**: `main`
**Duration**: ~35m

#### Context & Motivation
- Feedback from DSA coding agents highlighted context saturation from blind recursive grep over `*.md` files and whole-file 800-line dumps of narrative walkthroughs.
- Forensic audit revealed Omni-Graph already indexed 3,067 sections and 343 documents in DSA, but:
  1. `/api/search` omitted `line_start` and `line_end` from its SQL projection.
  2. `hook_pre_tool.sh` assigned blanket immunity to `.md` files without guiding agents to Omni-Graph.
  3. `parse_markdown` lacked relative hyperlink (`LINKS_TO`) and backtick code symbol (`REFERENCES`) extraction.

#### Changes Implemented
1. **Line-Number Precision in Vector Search (`src/db/mod.rs`)**:
   - Added `line_start: Option<usize>` and `line_end: Option<usize>` with `#[serde(default)]` to `SearchResult`.
   - Updated `search_vector` SQL projection to select `line_start, line_end`.
   - Verified live on `/api/search?q=Order-Agnostic+Binary+Search&workspace=DSA`: now returns exact line spans (`Lines: 294 - 303`).
2. **Hyperlinks & Cross-Modal Grounding (`src/parser/mod.rs`)**:
   - Extracted relative markdown links (`[text](target.md)`) into `LINKS_TO` edges.
   - Extracted backticked code identifiers (``` `ident` ```) into `REFERENCES` edges, linking documentation sections directly to AST symbols.
   - Added unit test `parse_markdown_links_and_references` in `tests/unit_tests.rs`. All 51 unit tests passing.
3. **Calibrated Documentation Guardrail (`hook_pre_tool.sh`)**:
   - Differentiated source code ($\le 150$ lines) from documentation ($\le 450$ lines).
   - Allows reading small documentation files ($\le 450$ lines) without requiring `EndLine`.
   - Blocks dumping massive $>450$-line markdown documents, nudging agents to `/api/search`.
   - Intercepts un-reconnoitered recursive `grep_search` on `*.md` across indexed workspaces, recommending `/api/search`.
   - Synced across `.agents/scripts/`, `tools/omni-graph/scripts/`, and `/Users/aparv/.gemini/config/scripts/`.
4. **Container Rebuild & Verification**:
   - Rebuilt `omni-rust-app` via `docker compose up -d --build rust-app`.
   - Verified `/api/health` and live vector search responses.

---

### 2026-10-02 — Fail-Safe, Non-Blocking Directory Browser & Escape Hatch

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Fix: Fail-Safe Directory Traversal & Non-Blocking Modal)
**Branch**: `main`
**Duration**: ~20m

#### Problem & User Impact
- The user reported getting trapped on an infinite `"Loading directory contents..."` spinner when opening or typing paths in the Directory Browser Modal.
- The UI completely hid all directory contents, provided zero cancel button, and even closing and reopening the modal restored the exact same stuck spinner state, locking the user out indefinitely.

#### Root Causes Identified
1. **Unbounded Fetch Without Timeout or Abort**: `fetchDirectory` had no `AbortSignal` or timeout. If a network request or filesystem stat call stalled, the promise remained unsettled and `isLoadingDir` stayed `true` forever.
2. **Debounce & Paste Race Conditions**: In `handleTargetPathChange` and `handleTargetPathPaste`, multiple rapid calls or simultaneous `paste` + `change` events fired overlapping fetches. Without request cancellation, stale responses or delayed errors could clobber state.
3. **Destructive UI Replacement**: In the JSX, when `isLoadingDir` was true, the entire directory list was unmounted and replaced with a full-height centered spinner with no cancel button, blinding the user.
4. **Stale Modal State On Reopen**: Closing the modal did not cancel pending background fetches or reset `isLoadingDir`. Reopening the modal directly or via toolbar re-triggered a fetch on the same stuck or problematic path.

#### Solutions Implemented (`ui/src/App.tsx`)
1. **Strict 3.5s Timeout & AbortController (`browseAbortController`)**:
   - `fetchDirectory` now cancels any prior in-flight request before dispatching a new one.
   - Enforces a strict 3.5-second timeout via `setTimeout(() => controller.abort(), 3500)`.
   - Guaranteed cleanup in `finally { if (browseAbortController.current === controller) setIsLoadingDir(false); }`.
2. **Non-Blocking UI & Preserved Entries**:
   - In-flight scans no longer hide or destroy existing directory entries. Previously loaded entries stay visible at `opacity: 0.6` under an elegant top progress shimmer bar.
   - Added an explicit **"Cancel Scan"** button in the header bar and a **"Stop Loading"** button in empty-state views.
   - Added quick recovery actions in error banners: **"Retry"** and **"Reset to /workspace"**.
3. **Safe Modal Lifecycle (`openBrowserModal` & `closeBrowserModal`)**:
   - `closeBrowserModal()` aborts any active network scan, clears debounce timers, and resets `isLoadingDir = false`.
   - `openBrowserModal()` guarantees a fresh start, safely falling back to `/workspace` if the previous path caused an error.
   - Updated all backdrop clicks, modal header `X` buttons, Cancel buttons, and HUD toolbar "+ Ingest Another" buttons to use `closeBrowserModal` and `openBrowserModal`.
4. **Debounce & Paste Protection**:
   - `handleTargetPathPaste` calls `e.preventDefault()` to prevent duplicate dispatch from the browser's subsequent `onChange` event.
   - Only triggers auto-exploration after a 600ms typing pause on valid path prefixes.
   - The user can press `Enter` or click `Browse ➔` to trigger directory exploration on demand.

#### Verification
- Built UI bundle (`npm run build`) in 2.91s without warnings.
- Rebuilt and restarted `graph-ui` container via `docker compose`.
- Verified `/api/browse` responds in <10ms via Nginx proxy.

---

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Directory Browser Reactive Path Navigation)
**Branch**: `main`
**Duration**: ~20m

#### Context & User Feedback
- When entering or pasting long paths (e.g. `/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge`) in `TARGET PATH`, the modal directory list stayed frozen at the previous browse path (`/workspace`). Users had to manually click breadcrumbs and drill down 10+ nested levels one by one.
- The user requested automatic adaptation: as soon as they paste or type a path, the palette should immediately fetch and render the subdirectories under that path.

#### What Was Done
1. **Interactive Path Synchronization (`ui/src/App.tsx`)**:
   - Implemented `handleTargetPathChange` with a 400ms debounce that triggers `fetchDirectory(path, false)` as the user types without resetting their input cursor.
   - Implemented `handleTargetPathPaste` to immediately capture pasted directory paths and trigger instant directory exploration.
   - Added `handleTargetPathKeyDown` (listening for `Enter`) and an explicit `Browse ➔` button beside `TARGET PATH` for instantaneous navigation.
2. **Current Directory Selection Banner**:
   - Added a prominent indicator above the directory entries list showing `Current Directory: <name> (X subfolders detected)` with a quick `Select This Folder` action.
   - Users can now seamlessly choose either the parent folder or any sub-codebase (`AWS`, `DSA`, `GenAI`, `GoLang`, `python`, `tool-scripts`, `tutor-intelligence`) with 1 click.
3. **Quick Jump Bookmarks**:
   - Added a dedicated 1-click bookmark chip for `📚 Interviews/Knowledge` (`/Users/aparv/.../knowledge`) alongside `/workspace`, `/Users`, and `/workspace/tools`.
4. **Error Handling & Visual Feedback**:
   - Added `dirError` display and spinning activity indicator `Exploring...` during dynamic path fetches.
5. **Verification**:
   - Built React UI bundle (`npm run build`) and restarted `graph-ui`.
   - Verified 50/50 unit tests passing (`make test`).

### 2026-10-02 — Persistent Active Ingestion HUD & Multi-Codebase In-Motion Tracker

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Active Ingestion HUD & Multi-Repo Tracker)
**Branch**: `main`
**Duration**: ~35m

#### Context & User Feedback
- The user requested a persistent status dock at the bottom of the screen mirroring the Live Watch HUD, specifically designed for ingestion progress reporting.
- Crucial UX requirements:
  1. Multi-codebase tracking: if two codebases are in-motion, the badge displays `INGESTING (2 IN MOTION)` and both repos appear as individual status chips.
  2. Hard browser refresh immunity: when a user reloads or hard-refreshes (`Cmd+Shift+R`), background re-ingestion must NOT be canceled, and the UI must instantly rehydrate in-motion progress.
  3. Live progress breakdown: display elapsed seconds, scanning/indexing counters, clustering phase, and a quick "✨ Explore" button upon completion.

#### What Was Done
1. **Nginx Client Abort Immunity (`ui/nginx.conf`)**:
   - Added `proxy_ignore_client_abort on;` inside `location /api/`.
   - Prevents client socket close (such as browser refresh) from propagating abort signals upstream to the backend Rust orchestrator.
2. **Backend Ingestion Job Tracker (`src/ingestion/mod.rs` & `src/api/mod.rs`)**:
   - Defined `IngestionJobStatus` struct tracking: `workspace`, `path`, `is_refresh`, `phase`, `files_scanned`, `files_indexed`, `total_nodes`, `total_edges`, `clusters_computed`, `started_at`, `elapsed_ms`, `completed`, and `error`.
   - Integrated `jobs: Arc<RwLock<HashMap<String, IngestionJobStatus>>>` into `IngestionPipeline`.
   - Updated `ingest_directory` to publish live status transitions (`purging` → `scanning` → `indexing` → `clustering` → `completed` / `failed`).
   - Exposed `GET /api/ingest/status` and `POST /api/ingest/dismiss` in `src/api/mod.rs`.
3. **Frontend Mission Control Ingestion HUD (`ui/src/App.tsx` & `ui/src/index.css`)**:
   - Built the Global Persistent Ingestion HUD floating dock with glassmorphism backdrop (`rgba(10, 15, 30, 0.94)`, `backdrop-filter: blur(20px)`, outer indigo/emerald glow).
   - Dynamic stacking: smoothly docks at `bottom: 74` when Live Watch HUD is active, or `bottom: 18` when idle, eliminating visual collision.
   - Dual-mode badge: animated radar dot with `INGESTING (N IN MOTION)` or `INGESTION COMPLETE`.
   - Multi-codebase chip list with rotating icons, clean re-ingest tag (`CLEAN`), real-time file counters, elapsed timer, `✨ Explore` quick-action button, and chip dismiss (`✕`).
   - Added `fetchIngestStatus` called on mount and every 2.5s, enabling instant rehydration across browser refreshes.
   - Added `@keyframes spin` and `.spinning-icon` to `ui/src/index.css`.
4. **Verification**:
   - Rebuilt Docker images for `rust-app` and `graph-ui`.
   - Verified `GET /api/ingest/status` returns active and completed jobs.
   - Verified `POST /api/ingest/dismiss` successfully clears jobs.
   - Verified 50/50 unit and integration tests passing (`cargo test --all -- --test-threads=1`).

### 2026-10-02 — Refresh Ingestion & Non-Blocking Ingestion UX Overhaul

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Refresh Ingestion & Non-Blocking UX)
**Branch**: `main`
**Duration**: ~30m

#### Context & Objectives
Addressed user's key concerns regarding codebase ingestion:
1. **Accumulated zombie nodes/stale data**: When significant changes happen offline or when re-indexing a workspace from scratch, there was no way to purge prior nodes, leading to duplicate/stale references and hash collision skips.
2. **Blocking modal & dead Cancel button**: During ingestion (which could take 30–120s on larger repos like DSA), the Cancel button was disabled (`disabled={isIngesting}`), locking the user on the modal screen with zero escape hatch.
3. **Unknown error / Client timeout**: Default browser fetch timeout resulted in "Ingestion failed: unknown error" when large codebases took >60s.

#### What Was Done
1. **Backend Refresh & Purge Pipeline**:
   - `src/db/mod.rs`: Implemented `purge_workspace(&self, workspace: &str)` executing an atomic 4-step purge (`DELETE linked_to WHERE in IN $nodes OR out IN $nodes`, `DELETE node WHERE workspace = $ws`, `DELETE galaxy WHERE workspace = $ws`, plus audit count).
   - `src/ingestion/mod.rs`: Added `clear(&self)` to `FileCache` and wired `refresh: bool` parameter through `ingest_directory`. When `refresh=true`, purges database records and resets in-memory staleness hashes so all files undergo clean, unskipped re-indexing.
   - `src/api/mod.rs`: Added `refresh: Option<bool>` to `IngestPayload` and routed through `ingest_handler`.
2. **Frontend UX & Non-Blocking Progress HUD**:
   - `ui/src/App.tsx`:
     - Added "Clean Re-ingest — purge existing data before fresh scan" toggle checkbox with red danger accent.
     - Dynamic button label & styling: switches from "Start Ingestion" to "Re-ingest Fresh" with subtle crimson glow when toggle is enabled.
     - Fixed Cancel button: Cancel is never disabled during ingestion and closes the modal immediately.
     - Instant modal dismissal: on trigger, modal immediately closes and shifts progress to a floating, non-blocking toast HUD (`ingestToast`) styled consistently with the Live Watch HUD.
     - Multi-phase animated progress toast: displays step transitions (`🔄 Purging stale data...` → `📂 Scanning AST nodes...` → `✨ Re-ingested X nodes & Y edges → Z galaxy clusters in Ws`).
     - Removed redundant `/api/cluster` post-ingest call since backend orchestrator already auto-computes galaxy communities.
     - Fixed large repo timeout by attaching an `AbortController` with a 5-minute timeout window.
3. **Code Quality & Compiler Cleanups**:
   - Eliminated unused variables and imports in `src/watcher/delta.rs` and `src/watcher/mod.rs`.
   - Verified clean zero-warning build for Rust orchestrator and TypeScript React bundle.

#### Verification
- Rebuilt Docker containers (`omni-rust-app` and `omni-graph-ui`).
- Tested end-to-end clean re-ingest on `session-explorer`: purged 68 stale nodes, rescanned 21 files fresh with 0 skipped, created 225 nodes, 999 edges, and 60 clusters in 9.1s.
- Tested template tool re-ingestion: completed in 1.03s with 55 nodes and 13 clusters.
- All 50 unit and integration tests passing (`cargo test --all -- --test-threads=1`).
- Automated visual validation via headless CDP captured:
  - `modal_opened.png`: clean modal layout with Clean Re-ingest toggle
  - `modal_clean_reingest_checked.png`: checked state with "Re-ingest Fresh" button
  - `toast_ingesting_progress.png`: non-blocking toast HUD floating over active graph view
  - `modal_after_cancel.png`: modal dismissal test verified.

#### Deep-Dive Investigation: "DSA" Ingestion "Unknown Error"
- **Symptom**: User attempted to re-ingest the `DSA` workspace and received `❌ Ingestion failed: Unknown error`.
- **Root Cause Analysis (Double-Culprit)**:
  1. **Nginx Reverse Proxy Default 60s Timeout**:
     In `ui/nginx.conf`, the `/api/` proxy lacked `proxy_read_timeout`. Nginx defaulted to 60s. For large repositories (DSA has 688 files, 5,662 AST nodes, 4,474 edges), embedding inference via CPU TEI + Louvain community clustering takes ~3.3 minutes (201s). At the 60s mark, Nginx dropped the connection with an HTML `504 Gateway Time-out`. The React UI attempted `await res.json()`, which threw on HTML, resulting in an empty fallback `{}` and `Unknown error`.
  2. **SurrealDB Client 30s Timeout & Unindexed Edge Purge**:
     In `src/db/mod.rs`, `reqwest::Client` had a hardcoded `timeout(Duration::from_secs(30))`. Furthermore, `purge_workspace` was using an unindexed subquery `DELETE linked_to WHERE in IN $nodes OR out IN $nodes;`, which did an unindexed scan across the edge table for 5,600+ node IDs, timing out at 30.02s before ingestion could even start.
- **Permanent Fixes Applied**:
  1. `ui/nginx.conf`: Configured `proxy_read_timeout 600s;`, `proxy_connect_timeout 600s;`, `proxy_send_timeout 600s;`, and `proxy_buffering off;`.
  2. `src/db/mod.rs`: Increased SurrealDB HTTP client timeout to 300s (5 minutes) and optimized `purge_workspace` to use the indexed `workspace` field (`DELETE linked_to WHERE workspace = $ws;`), slashing purge duration from >30s down to clean execution.
  3. `ui/src/App.tsx`: Increased `AbortController` timeout to 10 minutes (600,000ms) and added HTML fallback text parsing for non-JSON status codes (e.g. 504 Gateway Timeout) so the user receives clear, descriptive error messages.
- **End-to-End Validation**:
  - Successfully clean re-ingested `DSA` (`/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/DSA`):
    - Purged 5,662 stale nodes + edges + galaxies cleanly
    - Scanned 688 files, indexed 688 files, 0 skipped
    - Created 5,662 AST nodes & 4,474 edges
    - Computed 3,155 modular galaxy communities in 201.8s
    - Querying `/api/graph?workspace=DSA` returns all 5,662 nodes and 4,471 links cleanly.

---


### 2026-09-30 — Omni-Graph Galaxy Intelligence & Agent Relationship Augmentation

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Agent Relationship Augmentation & 3-Tier Reconnaissance Ladder)
**Branch**: `feat/agent-relationship-augmentation`
**Duration**: ~25m

#### Context & Objectives
Empowered coding agents across the system to actively augment the Omni-Graph knowledge base with inferred runtime/cross-service relationships and streamlined reconnaissance through the 3-Tier Omniverse Reconnaissance Ladder.

#### What Was Done
1. **Pre-Invocation Ephemeral Prompt Banner (Task 1)**:
   - Updated `.agents/scripts/hook_pre_invocation.sh`, `tools/omni-graph/scripts/hook_pre_invocation.sh`, and `~/.gemini/config/scripts/hook_pre_invocation.sh`.
   - Structured prompt injection into a 3-Tier Reconnaissance Ladder:
     - **Tier 1 (Graph-RAG Query)**: `POST /api/query`
     - **Tier 2 (Galaxy Clusters)**: `GET /api/galaxies?workspace={ws}`
     - **Tier 3 (AST Subgraph & Blast Radius)**: `GET /api/condense`, `GET /api/references`, `GET /api/symbol`, `GET /api/search`
2. **Agent Relationship Augmentation Endpoint (`POST /api/relationships` & `/api/relation`) (Task 2)**:
   - Added `RelationshipPayload` and `ReferenceResult` in `src/db/mod.rs`.
   - Implemented `add_relationship` in `DbClient` with SurrealQL atomic linking and automatic virtual component synthesis (`kind: "virtual_service"` / `kind: "endpoint"`) with 384-dim zero embeddings for external/unindexed symbols.
   - Enhanced `find_references` to return caller nodes along with `type`, `category` (`EXTRACTED` vs `INFERRED`), and `metadata` (`option<object>`).
   - Implemented `relationships_handler` in `src/api/mod.rs` with asynchronous agent telemetry recording (`/api/relationships`).
3. **CLI Helpers, Makefile Targets & Agent Skill Documentation (Task 3)**:
   - Added `relate` command to `omni.sh` (`./tools/omni-graph/scripts/omni.sh relate <src> <tgt> [type] [ws]`) across repo and user config skills.
   - Added `relate-graph` target in root `Makefile` and `tools/omni-graph/Makefile`:
     `make relate-graph SRC="ChatUI" TGT="FastAPI" TYPE="ROUTES_TO" PROJECT="QuarkDock"`
   - Updated `SKILL.md` (root, tool, and `~/.gemini/config`) with documentation on dynamic runtime dependency augmentation, REST API endpoints, and Recipe C.

#### Verification
- Docker multi-stage build succeeded cleanly with 0 compiler errors.
- `POST /api/relationships` verified end-to-end with runtime metadata.
- `GET /api/references` verified returning augmented relations with category and metadata.
- `make relate-graph` verified from repo root.
- Ephemeral pre-invocation prompt banner verified live on agent turn start.

---

### 2026-09-30 — Fix Heavy Subsystem List Flex Collapse & Consolidate Live Watch Status

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Dynamic Galaxy Clustering & Agent Primitives)
**Branch**: `feat/dynamic-galaxy-clustering-agent-primitives`
**Duration**: ~20m

#### Root Cause Analysis of Empty Subsystem Display List
- **The Issue**: In codebases with large numbers of clusters (e.g. `QuarkDock` with 252 total galaxies and 246 Domain Services), selecting "Domain Services" or "Total Galaxies" resulted in an empty/invisible card list, while small categories (3 Core Stable and 3 Leaves Orch) rendered normally.
- **Root Cause**:
  1. **Flex-Shrink Collapse**: Inside `SubsystemTopologyHub.tsx`, the card container was styled with `display: 'flex', flexDirection: 'column', gap: 8, maxHeight: 420, overflowY: 'auto'`. The individual galaxy cards lacked `flexShrink: 0` and `minHeight: 'fit-content'`. Under CSS flexbox default (`flex-shrink: 1`), fitting 246 cards into a 420px column compressed each card down to ~1.7px height with `overflow: 'hidden'`, rendering them invisible. With 3 cards, they easily fit into 420px and did not compress.
  2. **Non-Windowed DOM Overload**: All 246 cards and their nested metric subtrees were rendered simultaneously into the DOM without progressive windowing.
  3. **Stale Workspace Closure in Async Callbacks**: In `App.tsx`, SSE event listeners captured `selectedWorkspace` (`null` at mount) in closure, occasionally fetching Omniverse topology instead of workspace-scoped topology.

#### Solutions & Improvements
- **Windowed Progressive Rendering (`SubsystemTopologyHub.tsx`)**:
  - Added `visibleCount` state (default 40), automatically resetting on filter, sort, or workspace change.
  - Sliced rendered array with `filteredGalaxies.slice(0, visibleCount)`.
  - Added infinite scroll trigger (`onScroll`) and explicit `[ Load More (X remaining) ]` button at the bottom.
  - Added `flexShrink: 0` and `minHeight: 'fit-content'` on every card.
  - Replaced hardcoded `maxHeight: 420` with `flex: 1; minHeight: 0; overflowY: 'auto'` to dynamically expand across any viewport.
- **HUD Consolidation & Live Modularity Redundancy Removal (`App.tsx`)**:
  - Removed the redundant standalone `Live Modularity` badge from the top header action row.
  - Retained the primary `LIVE WATCH` button and HUD as the single source of truth for background sync and watching state.
  - Added a subtle `● Live Sync` indicator inside the Galaxy Subsystems drawer header next to the workspace name.
- **Closure State Integrity (`App.tsx`)**:
  - Added `selectedWorkspaceRef = useRef(selectedWorkspace)` to ensure SSE event handlers and async callbacks always query the active workspace.
  - Awaited `loadGalaxyTopology` in `loadGraph`.

#### Verification
- Verified via automated headless Chrome testing (`test_quarkdock_ui.js`):
  - Confirmed QuarkDock's 246 Domain Services and 252 Total Galaxies render at full height (`height: 73px, width: 324px`).
  - Confirmed infinite scrolling dynamically extends card count (40 -> 90 -> 140...).
  - Verified clean header with single `LIVE WATCH` indicator.
  - Captured verified screenshots: `quarkdock_subsystems_domain_services.png` and `quarkdock_subsystems_total_galaxies.png`.

---

### 2026-09-30 — Dynamic Live Galaxy Clustering & Agent Actionable Boundary Contracts

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Feature: Dynamic Galaxy Clustering & Agent Primitives)
**Branch**: `feat/dynamic-galaxy-clustering-agent-primitives`
**Duration**: ~45m

#### Problem & Architectural Motivation
- While Omni-Graph computed file-level delta AST updates in real-time, galaxy clusters (communities) remained static unless manually triggered via `/api/cluster`.
- More critically, community data was previously only consumed by the WebGL visualizer as raw integers (`community: 42`). AI coding agents had no actionable primitives to query macro subsystem boundaries, cross-boundary blast radii, or coupling metrics before refactoring symbols.

#### What Was Done
- **Schema-Full `galaxy` Table (`src/db/schema.surql`)**:
  - Defined table `galaxy` storing macro architectural subsystem entities: `workspace`, `galaxy_id`, `name`, `dominant_path`, `node_count`, `internal_edges`, `external_edges`, `afferent_coupling` ($C_a$), `efferent_coupling` ($C_e$), `instability` ($I$), `role`, `key_symbols`, `languages`, and `updated_at`.
- **Dual-Phase Dynamic Clustering**:
  - **Phase 1 (Instant Single-File Inheritance in `watcher/pipeline.rs`)**:
    - During incremental `reindex_file`, queries the previous community of the file (`get_file_community`) before pruning old nodes.
    - Updated `store_nodes` to use SurrealQL `MERGE` rather than `CONTENT` to guarantee `community` assignments are never wiped during incremental AST updates.
  - **Phase 2 (Quiescent Seed-Preserving Re-Clustering in `watcher/mod.rs` & `analysis/mod.rs`)**:
    - Background task detects when file edits pause for $\ge 3.5$s (`last_event_processed_time`).
    - Executes `detect_with_seeds`: edge-weighted Label Propagation Algorithm (`IMPLEMENTS` = 3.0, `CALLS` = 2.0, `TYPE_REF` = 1.5, `IMPORTS` = 1.0) with seed persistence to prevent cluster ID thrashing.
    - Computes Robert C. Martin coupling metrics and subsystem roles (`"Core Foundation"` for $I \le 0.25$, `"Domain Service"` for $I \le 0.65$, `"Orchestrator / Leaf"` for $I > 0.65$).
    - Persists updated `galaxy` records and emits a `"ClustersRefreshed"` SSE `WatchEvent`.
- **Actionable Agent Endpoints (`src/api/mod.rs` & `src/db/mod.rs`)**:
  - `GET /api/galaxy/topology?workspace=<ws>`: Returns high-level architectural map with coupling metrics and cross-galaxy dependency edges.
  - `GET /api/galaxy/boundary?symbol=<sym>&workspace=<ws>`: Returns symbol architectural contract, home galaxy, internal vs foreign callers, and synthesized `agent_actionable_advice` (`risk_level`, `summary`, `rule_of_thumb`).
  - Optimized `find_symbols` to rank exact definition matches ahead of import statements.
- **UI/UX Architecture Overhaul (`ui/src/`)**:
  - **`BoundaryContractCard.tsx`**: Integrated into Node Inspector drawer. Displays architectural containment badge (`🛡️ Contained Internal`, `⚠️ Boundary Crossing`, `🔌 Isolated`), home galaxy with role badge, internal vs foreign cross-subsystem callers with click-to-navigate action, glowing risk advice banner (`HIGH`, `MEDIUM`, `LOW`), and one-click "Copy Boundary Contract for Agent" button.
  - **`SubsystemTopologyHub.tsx`**: Integrated into Galaxy Subsystems drawer with View Mode Toggle (`Macro Topology & Coupling` vs `Flat Explorer`). Features global subsystem metric cards (Core Foundation, Domain Services, Leaves), Instability ($I$) progress meters with gradient visualization, coupling counts ($C_a, C_e$, internal links), key exported symbols, and direct `[ 🎯 Isolate in 3D ]` action.
  - **Top Navigation HUD**: Added `● Live Modularity` watch chip that pulses when active and links directly to the macro topology hub. Added real-time SSE listener for `cluster` events.
- **Verification**:
  - All 50/50 unit and integration tests passing in Docker (`unit_tests.rs`).
  - Frontend production build (`tsc && vite build`) passed with zero errors in 2.99s.
  - Verified `/api/cluster`, `/api/galaxy/topology`, and `/api/galaxy/boundary` with live queries against `tool-scripts`.
  - Rebuilt and restarted both `omni-rust-app` and `omni-graph-ui` containers successfully.

---

### 2026-09-30 — Architectural Root Prevention: Session-Scoped Recon Gate & Lifecycle Hooks

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Core Prevention & Enforcement Architecture)
**Duration**: ~20m

#### Problem & Architectural Audit
- Prior enforcement mechanisms treated symptoms (magnitude checks such as line counts or regex query filters) rather than root cause:
  - An agent could bypass Omni-Graph entirely by chunking `view_file` calls (e.g. 150 lines at a time) or running targeted `grep_search` calls, touching raw source code without ever consulting the AST knowledge graph.
  - Rule 00 invariant states: *"Never read whole source code files without prior AST symbol localization."* The core flaw was lack of sequence enforcement (*order* rather than magnitude).

#### What Was Done
- **Session-Scoped Recon Gate (`hook_pre_tool.sh`)**:
  - Enforced the architectural invariant: **Do A before B**. Source code access (`view_file` and `grep_search` targeting code) is gated until the agent queries Omni-Graph at least once in the session.
  - Implemented `/tmp/omni_recon_<workspace>.marker` with a 2-hour TTL (7,200s).
  - Automatically detects Omni-Graph AST queries in `run_command` (`localhost:8080/api/symbol`, `/condense`, `/references`, `/search`, `/galaxy`, `/ast`, `/workspaces`, `/ingest`, `make graph-*`, `make workspaces`, `omni.sh`) and records the recon marker.
  - Preserved complete exemptions for non-code files (`.md`, `.json`, `.yaml`, `.toml`, `Makefile`, configs) and agent infrastructure (`/.agents/`, `/.gemini/`, `/.git/`) so orientation and rule checks remain friction-free.
  - Kept the 150-line gate as independent defense-in-depth after recon is unlocked.
- **PreInvocation Hook Hardening (`hook_pre_invocation.sh`)**:
  - Added upfront banner: `⚠️ Source code access (grep_search, view_file) is gated until you query Omni-Graph.`
  - Added fast workspace caching (`/tmp/omni_indexed_<workspace>.flag` and `/tmp/omni_indexed_workspaces.json`) for sub-millisecond PreToolUse evaluation without HTTP roundtrips.
- **Robust Bash Heredoc Invocation**:
  - Switched execution to `python3 - "$PAYLOAD" << 'EOF'` across hook scripts, preventing bash quote collisions on regex patterns or complex strings.
- **Verification & Deployment**:
  - Validated all 9 lifecycle test cases covering: pre-recon blocking on source view and grep, doc exemptions, agent script exemptions, recon recording via API command, post-recon access, and post-recon 150-line gate enforcement.
  - Deployed and synchronized machine-wide via `setup-agent.sh workspace` and `setup-agent.sh global`.

---

### 2026-09-29 — Workspace Sanitization & Deduplication in Omni-Graph Backend and UI

**Agent/Author**: Antigravity (Google DeepMind)
**SDLC Phase**: `in-progress` (Deduplication, DB Coalescence & UI Hardening)
**Duration**: ~15m

#### What Was Done
- **Backend Workspace Coalescence (`db/mod.rs`)**:
  - Overhauled `get_workspaces(&self)`:
    1. Legacy or uncanonical `workspace = "workspace"` entries are automatically normalized and merged into `"tool-scripts"`.
    2. Invalid, empty, `"default"`, or `"global"` workspace names are excluded.
    3. Merged total node counts, language arrays, and file arrays using a clean `HashMap<String, serde_json::Value>` to guarantee strict uniqueness.
    4. Deterministically sorts workspaces alphabetically.
- **Frontend Deduplication & Guardrails (`App.tsx`)**:
  - In `loadWorkspaces`: added case-insensitive `Set` deduplication on incoming `/api/workspaces` payload.
  - Filtered out `"workspace"`, `"default"`, and `"global"` entries so dropdowns and cluster pickers only present bona fide codebases.
- **Verification**:
  - Rebuilt `omni-graph-ui` via Vite (`npm run build`), passing cleanly in 3.04s.
  - Rebuilt and restarted `omni-rust-app` and `omni-graph-ui` via Docker Compose.
  - Verified `curl -s http://localhost:8080/api/workspaces` returns clean, unique codebases with 1,591 nodes for `tool-scripts` and zero duplicate entries.

---

### 2026-09-29 — Unbypassable Polyglot Guardrails, 150-Line Source Gate & Top-Tier Rule 00

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~25m

#### Problem & Agent Feedback
- Feedback from coding agents operating across sister workspaces (e.g. `tutor-intelligence`) revealed guardrail bypass paths:
  1. `PreToolUse` matcher only intercepted `run_command|grep_search`. Agents could freely bypass it by calling `view_file` on large source files (>500 lines) or dumping full files into context.
  2. `PreInvocation` emitted passive text rather than dynamic, workspace-scoped actionable commands.
  3. `08-omni-graph-enforcement.md` was susceptible to rule shadowing when downstream repositories defined their own `08-*` rules.
  4. Rules hardcoding language extensions failed across polyglot ecosystems (10–30 programming languages).

#### What Was Done
- **Expanded PreToolUse Interception (`hooks.json`)**:
  - Updated matcher to `"run_command|grep_search|view_file"` across `~/.gemini/config/hooks.json`, `tools/omni-graph/hooks.json`, and `.agents/hooks.json`.
- **Language-Agnostic 150-Line Source Gate (`hook_pre_tool.sh`)**:
  - Implemented fail-open safeguard: if `http://localhost:8080/api/health` does not respond within 500ms, immediately allows all tools to guarantee agents are never blocked when the daemon is offline.
  - Implemented negative exclusion list for non-code files (`.md`, `.json`, `.yaml`, `.toml`, `.txt`, `.csv`, `.lock`, `.sum`, images, media, etc.).
  - All source code files across Go, Rust, Python, TypeScript, JavaScript, C/C++, Java, Zig, Elixir, Scala, Swift, Kotlin, etc., are actively gated.
  - Calls to `view_file` on source code without `EndLine` specified or spanning `> 150 lines` are automatically DENIED with clear guidance to query `/api/condense` or `/api/symbol`. Slices `<= 150 lines` and all non-code files are allowed.
- **Dynamic PreInvocation Context Injector (`hook_pre_invocation.sh`)**:
  - Dynamically determines the active repository/workspace from `git rev-parse --show-toplevel` or current working directory.
  - Queries `GET http://localhost:8080/api/workspaces` and injects exact workspace-scoped commands with live AST node counts (e.g. `tutor-intelligence (8,239 AST nodes)` or `tool-scripts (1,397 AST nodes)`).
- **Rule Renaming & Top-Tier Precedence (`00-omni-graph-mandatory-retrieval.md`)**:
  - Renamed `08-omni-graph-enforcement.md` → `00-omni-graph-mandatory-retrieval.md` with priority `🔴 CRITICAL — NON-NEGOTIABLE GLOBAL INVARIANT`.
  - Prefix `00-` ensures it is loaded first in system context and eliminates rule shadowing across all 100+ codebases.
  - Mandated caller blast-radius verification via `/api/references` before modifying public signatures or deleting symbols.
  - Synced across `~/.gemini/config/rules/`, `tools/omni-graph/rules/`, and `.agents/rules/`.
- **Automated Bootstrapper (`setup-agent.sh`)**:
  - Updated `setup-agent.sh` to install `00-omni-graph-mandatory-retrieval.md` and configure `run_command|grep_search|view_file`.
- **Verification**:
  - Tested `view_file` denials across `.go`, `.rs`, `.py`, `.ts`, `.cpp` (>150 lines) -> Verified DENIED.
  - Tested `view_file` slices <= 150 lines -> Verified ALLOWED.
  - Tested `view_file` on `.md`, `.json`, `.yaml`, `.txt` (unbounded / 500 lines) -> Verified ALLOWED.
  - Tested daemon offline fail-open safeguard -> Verified ALLOWED.
  - 49/49 unit tests passing, specs validated with 0 errors and 0 warnings.

---

### 2026-09-26 — Dual-Source Live Agent Telemetry & Dynamic Workspace Capability Attribution

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~35m

#### Problem & User Feedback
- The Agent Analytics & LSP telemetry dashboard was originally relying on manual `make` invocations run within `tool-scripts` or basic regex transcript parsing.
- Omni-Graph tool calls made by coding agents working in other active workspaces/codebases (such as `tutor-intelligence`, `DSA`, `GenAI`, `GoLang`, `session-explorer`, `k8s-eks`) were not dynamically accounted for in real time if called directly via HTTP or outside repository-local transcripts.
- Heuristic leaf extraction occasionally caught bare shell command tokens (`curl`, `make`, `cat`), contaminating `workspaces_breakdown` with pseudo-workspaces.

#### What Was Done
- **SurrealDB Telemetry Schema (`src/db/schema.surql`)**:
  - Defined the `agent_api_call` table to record every agent API invocation with fields: `endpoint`, `workspace`, `capability`, `query_param`, `caller`, `duration_ms`, `created_at`.
  - Added performance indexes on `workspace`, `capability`, and `created_at`.
- **Database Client Telemetry Engine (`src/db/mod.rs`)**:
  - Implemented `record_agent_api_call` for non-blocking telemetry logging.
  - Implemented `get_api_calls_summary` aggregating calls by `(capability, workspace)` using SurrealDB `GROUP BY capability, workspace`.
  - Implemented `get_api_recent_calls` for real-time audit tracing.
- **REST Endpoints Instrumentation (`src/api/mod.rs`)**:
  - Instrumented all primary Omni-Graph endpoints (`/api/symbol`, `/api/references`, `/api/condense`, `/api/search`, `/api/query`, `/api/galaxies`, `/api/cluster`, `/api/watch/start`, `/api/ingest`).
  - Wrapped recording in non-blocking `tokio::spawn` with duration timing, guaranteeing sub-millisecond response overhead.
  - Passed `State(state)` into `/api/analytics` to invoke `AnalyticsEngine::scan_analytics_with_db(&state.db).await`.
- **Dual-Source Aggregation & Noise Filtering (`src/analytics/mod.rs`)**:
  - Implemented `ParsedOmniTool` and `parse_omni_command` with canonical capability classification and strict noise blacklisting.
  - Implemented dual-source union taking `max(transcript_count, db_count)` per `(capability, workspace)` to seamlessly blend IDE session history with live SurrealDB events while strictly preventing double counting.
  - Preserved traditional tool tracking (`view_file`, `run_command`, `replace_file_content`, etc.) while cleanly isolating `Omni AST` high-density tools.
- **Verification & Test Suite**:
  - Updated unit test assertions in `tests/unit_tests.rs` to reflect the enhanced canonical capability naming.
  - Verified 49/49 unit tests passing in Docker (`test result: ok. 49 passed; 0 failed`).
  - Verified live analytics endpoint (`GET /api/analytics`): 14 sessions, 53,400+ steps, 20,700+ tool calls, 169 omni calls, 37 LSP lookups, 2.74M+ estimated tokens saved across 5 active workspaces with zero dirty artifacts.

---

### 2026-09-26 — Workspace Navbar Dropdown Outside-Click & Escape Dismissal UX

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~10m

#### Problem & User Feedback
- When clicking the "Workspaces" filter button in the top navigation bar to open the dropdown menu, clicking outside the dropdown or interacting with the canvas did not close the dropdown. The user had to specifically re-click the trigger button to dismiss it, violating standard desktop UX expectations.

#### What Was Done
- **Outside-Click & Escape Key Dismissal (`ui/src/App.tsx`)**:
  - Attached a container `useRef<HTMLDivElement>(null)` to the workspace dropdown parent wrapper.
  - Implemented an active `useEffect` listener registering `mousedown`, `touchstart` (with event capture to reliably capture clicks over the 3D WebGL canvas or drawers), and `keydown` for `Escape`.
  - Automatically dismisses the dropdown when clicks occur outside the component or when `Escape` is pressed.
  - Added smooth chevron rotation (`transform: rotate(180deg)`) on toggle.
  - Enhanced item hover states with micro-transitions and bounded dropdown height (`maxHeight: 70vh`, `overflowY: auto`).
- **Build & Verification**:
  - Built production bundle (`npm run build`) in `ui/` cleanly in 2.80s.
  - Rebuilt and restarted `omni-graph-graph-ui` Docker container on port 3000.

---

### 2026-09-26 — Watch Modal Persistence & High-Contrast Debounce UX Polish

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### Problem & User Feedback
1. **Modal Prematurely Closing**: Clicking "+ Watch" on a codebase in the Quick Select palette immediately dismissed the Watch Manager modal and sent the user back to the home page, preventing them from watching multiple codebases or checking telemetry.
2. **Debounce Window Contrast Issue**: In the modal, default browser select styles caused dark text to render on a dark background (`#0b1120`), making the selected 500ms recommended debounce window illegible unless actively clicked.

#### What Was Done
- **Modal Persistence & Explicit User Dismissal (`ui/src/App.tsx`)**:
  - Removed automatic modal dismissal (`setIsWatchModalOpen(false)`) from `handleStartWatch`.
  - The modal now remains open so users can watch multiple codebases sequentially, observe real-time telemetry card updates, and inspect status.
  - Added a dedicated "Done / Close" button in the modal footer alongside the top-right `[✕]` button.
- **High-Contrast Form Inputs & Debounce Dropdown (`ui/src/index.css`, `ui/src/App.tsx`)**:
  - Defined explicit `.cyber-input` and `.cyber-select` CSS rules with `#0f172a` dark background, `#f8fafc` crisp white text, and `color-scheme: dark` to prevent OS/browser inversion bugs.
  - Added real-time badge in the label: `Debounce Window: <span color="#38bdf8">{watchDebounceMs}ms selected</span>`.
  - Added styled options with clear icons and highlighted `⭐ 500ms (Recommended)`.
- **Build & Verification**:
  - Built production bundle (`npm run build`) in `ui/` cleanly in 2.85s.
  - Rebuilt and restarted `omni-graph-graph-ui` Docker container on port 3000.

---

### 2026-09-26 — Workspace Root Path Resolution & Robust Multi-Repository Live Watch Fix

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### Problem & Root Cause
- When selecting `tool-scripts` from the "Quick select from Ingested Codebases" palette in the Watch Manager modal, the system failed with:
  `Failed to start watch: Path does not exist or is not a directory: /workspace/tools/tool-scripts`.
- **Root Cause**:
  1. The monorepo root is mounted into Docker at `/workspace`, while internal tools reside in `/workspace/tools/<tool>`, and sister codebases (`DSA`, `k8s-eks`, `tutor-intelligence`) reside at their host directory paths (`/Users/.../knowledge/<name>`). The UI initially used an ad-hoc fallback `/workspace/tools/${ws.workspace}`.
  2. The database schema previously grouped nodes by workspace alias without persisting the canonical filesystem root directory that was ingested.

#### What Was Done
- **Engineered Workspace Root Metadata Engine (`src/db/schema.surql`, `src/db/mod.rs`)**:
  - Defined `workspace_meta` table in SurrealDB schema to store `(workspace, root_path, updated_at)`.
  - Added `record_workspace_root` and `get_workspace_root` methods in `DbClient`.
  - Updated `IngestionPipeline` to automatically record the canonical root directory on every ingestion run.
  - Implemented `resolve_disk_path` heuristic testing candidate mounts (`/workspace`, `/workspace/tools/*`, `/Users/.../knowledge/*`) against actual files present in the database.
  - Updated `get_workspaces()` API to enrich every workspace with its exact, verified `root_path`.
- **Auto-Resolution & Path Guardrails (`src/api/mod.rs`)**:
  - Enhanced `POST /api/watch/start` handler to detect non-existent directory requests, automatically strip rogue suffixes (e.g. `/tools/tool-scripts` -> `/workspace`), query `workspace_meta`, and auto-resolve the directory on disk before launching the watch daemon.
- **Frontend Palette & Quick-Select Update (`ui/src/App.tsx`)**:
  - Updated quick-select buttons in Watch Manager to utilize verified `ws.root_path` from `/api/workspaces`, rendering the exact resolved path under each workspace button.
- **Verification & Deployment**:
  - Rebuilt and deployed `omni-graph-rust-app` and `omni-graph-graph-ui` containers.
  - Verified `POST /api/watch/start` with `/workspace/tools/tool-scripts` now seamlessly auto-resolves to `/workspace` and starts watching 160 files without error.
  - Verified `GET /api/workspaces` returns verified `root_path` for all 10 ingested codebases (`tool-scripts`, `DSA`, `k8s-eks`, `session-explorer`, `python`, etc.).

---

### 2026-09-26 — Dynamic Live Delta Sync, File Watcher Daemon & Global Persistent Live Recording HUD

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~45m

#### What Was Done
- **Engineered Live Delta Sync Engine (`src/watcher/mod.rs`, `delta.rs`, `pipeline.rs`)**:
  - Implemented multi-workspace file watcher daemon utilizing native OS events (`notify` / `inotify` / FSEvents) with configurable debounce windows (default 500ms).
  - Coalesces rapid keystrokes and file events into clean semantic changesets (`Created`, `Modified`, `Deleted`, `Renamed`).
  - Implemented single-file incremental pipeline: reads changed files, parses AST via Tree-sitter, computes HuggingFace TEI vector embeddings, updates SurrealDB graph nodes and edges in sub-100ms.
  - Implemented atomic prune for deleted files: cascade cleans connected graph edges (`linked_to`) and AST nodes in SurrealDB when files are removed.
  - Implemented background interval sweep (`tokio::time::interval`) to guarantee 100% resilient deletion detection across host/Docker virtualization boundaries (VirtioFS / gRPC-FUSE).
- **REST & SSE Live Telemetry API (`src/api/mod.rs`)**:
  - `POST /api/watch/start`: Starts real-time watcher on any directory path with workspace alias and debounce settings.
  - `POST /api/watch/stop`: Gracefully terminates a watcher task.
  - `GET /api/watch/status`: Returns live telemetry for all active watchers (tracked files, total events, re-indexed count, deleted count, avg latency ms, cluster status).
  - `GET /api/watch/status/:workspace`: Returns status for a single workspace.
  - `GET /api/watch/events`: Server-Sent Events (SSE) streaming live delta events to browser in real time.
- **CLI Commands (`scripts/omni.sh`)**:
  - `omni.sh watch start <path> [project] [debounce_ms]`
  - `omni.sh watch stop <workspace>`
  - `omni.sh watch status`
  - `omni.sh watch events`
- **Global Persistent Live Recording HUD & Watch Manager UI (`ui/src/App.tsx`)**:
  - **Always-Visible Global Floating HUD**: Pinned at bottom center across ALL pages (`3D Graph Studio` and `Agent Analytics & LSP`), displaying:
    - Glowing badge `🔴 LIVE RECORDING`
    - Real-time chips for each watched workspace with tracked file counts, event counts, and latency
    - Instant `[×]` stop buttons on each chip
    - Quick `+ Watch More` and minimize/expand toggles
  - **Header Live Watch Pill**: Displays `[ 🔴 LIVE WATCH (N Active) ]` or `[ 📡 Live Watch (Idle) ]` in the top header navigation.
  - **Interactive Watch Manager Modal**: Full flyout modal to start watching any ingested codebase with 1 click, custom path entry, debounce settings, and live telemetry cards.
  - **Real-Time SSE Delta Toast**: Instant notification slide-up when files are modified/created/deleted in the active codebase.

#### Files Changed
- `src/watcher/mod.rs` — New: WatchManager daemon orchestrating concurrent workspace watchers
- `src/watcher/delta.rs` — New: Event coalescing, rename tracking, and delta classification
- `src/watcher/pipeline.rs` — New: Incremental AST re-indexing and atomic file pruning
- `src/db/mod.rs` — Added `delete_file` and `rename_file` methods for atomic SurrealQL graph pruning
- `src/ingestion/mod.rs` — Added `remove`, `rename`, and `get_all_paths` to `FileCache`
- `src/api/mod.rs` — Registered `/api/watch/*` routes and SSE event handler
- `src/lib.rs` & `src/main.rs` — Registered watcher module and passed `watch_manager` into `AppState`
- `Cargo.toml` — Added `notify = "6.1"`, `chrono = "0.4"`, `tokio-stream = "0.1"`
- `scripts/omni.sh` — Added `watch start/stop/status/events` subcommands
- `ui/src/App.tsx` — Global persistent Live Watch HUD, header indicator, and Watch Manager modal
- `CONTEXT.md` & `DEVLOG.md` — Updated session state

---

### 2026-09-25 — Omni-DB Agent Analytics, Live Dynamic Polling & LSP Grounding Paradigm Hub

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~35m

#### What Was Done
- **Built Live Real-Time Analytics Engine (`src/analytics/mod.rs`)**:
  - Dynamically scans active agent transcripts across all IDE sessions in `~/.gemini/antigravity-ide/brain/` live on each request.
  - Aggregates multi-workspace metrics: 14 active trajectories, 47,400+ reasoning steps, 18,900+ tool invocations, ~863,000+ estimated tokens saved via AST condensation vs raw multi-file reading.
  - Extracts LSP engine telemetry across 10 programming languages (Go, Markdown, Python, Rust, JavaScript, TypeScript, YAML, Shell, SQL, TOML).
  - Traversal timeline inspector endpoint `GET /api/analytics/session/:id` returning step-by-step thinking blocks and tool call arguments.
  - Unit test `test_analytics_scan_and_detail_with_synthetic_session` passing; **49/49 unit tests passing** across entire Rust test suite.
- **Frontend Architecture & UX (`ui/src/AgentAnalytics.tsx` & `ui/src/App.tsx`)**:
  - **Dynamic Top Switcher**: Smooth navigation between `[ 🌌 3D Graph Studio ]` and `[ 📊 Agent Analytics & LSP (Omni-DB) ]`.
  - **Live Dynamic Polling (6s)**: Automatic background sync loop with live indicator `[ ● Live Dynamic Sync (6s) ]` and timestamp that automatically updates the dashboard as coding agents generate new thoughts, prompts, and tool calls.
  - **The Grounding Paradigm Matrix**: 4-card matrix directly contrasting legacy agent tool calls with Omni-Graph AST retrievals:
    1. `❌ view_file / cat (4k–15k tokens)` ➔ `⚡ make graph-symbol SYM=name` (<50 tokens, 99% context reduction).
    2. `❌ grep_search (noisy lexical regex)` ➔ `⚡ make graph-references SYM=name` (compiler semantic call graph & references).
    3. `❌ Multi-file reading (5–10 files, 25k–50k tokens)` ➔ `⚡ make graph-condense SYM=root` (topological 2-hop AST slice <1,500 tokens).
    4. `❌ list_dir / find (blind directory walk)` ➔ `⚡ make query-graph Q="question"` (Hybrid Graph-RAG macro + micro retrieval).
  - **Tool Spectrum Segmentation**: Categorized into `⚡ Omni AST (High-Density RAG)` vs `🛑 Traditional (Baseline Ops)`.
  - **Deep Session Traversal Inspector Modal**: Step-by-step timeline with persona filter tabs (`ALL`, `PROMPTS`, `THOUGHTS`, `TOOLS`, `OMNI`).
- **Zero-Downtime Hot Deploy**:
  - Maintained 100% container uptime for SurrealDB and HF TEI; hot-compiled and hot-copied release binary and Vite bundle into `omni-rust-app` and `omni-graph-ui`.
  - Automated browser verification with Google Chrome: captured verified screenshots `agent_analytics_dashboard.png` and `agent_analytics_sessions_stream.png`.

#### Files Changed
- `src/analytics/mod.rs` — New: Analytics engine and live transcript scanner
- `src/lib.rs` & `src/main.rs` — Registered `pub mod analytics`
- `src/api/mod.rs` — Registered `/api/analytics` and `/api/analytics/session/:id` routes
- `tests/unit_tests.rs` — Added analytics engine unit test
- `ui/src/AgentAnalytics.tsx` — New: Agent Analytics & LSP dashboard component
- `ui/src/App.tsx` — Mode switcher HUD and navigation integration
- `CONTEXT.md` & `DEVLOG.md` — Updated session state

---

### 2026-09-25 — Rigid Pipeline Hardening: Universal Multi-Tier Parsing, Auto-Clustering, Zero-Drop Guarantee & Makefile Path Fix

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~35m

#### What Was Done
- **Universal Multi-Tier Parsing & Zero-Drop Guarantee (`src/parser/mod.rs`)**:
  - Eliminated the "all-or-nothing" fragility where non-standard extensions or files without function/struct signatures were silently dropped.
  - Implemented multi-tier extraction:
    - **Tier 1 (Code AST)**: Tree-Sitter for Rust, Python, Go, JavaScript, TypeScript.
    - **Tier 2 (Documentation & Notes)**: Markdown & MDX parser extracting sections, titles, and embedded code blocks.
    - **Tier 3 (Infrastructure, Manifests & Configs)**:
      - `parse_yaml`: Extracts Kubernetes manifests (`Deployment`, `Service`, `Ingress`, `ConfigMap`), resource names, and `CONTAINS` edges.
      - `parse_json`: Extracts top-level configuration objects, scripts, and dependencies.
      - `parse_shell`: Extracts bash/shell functions (`function foo()` and `foo() {`) and execution steps.
      - `parse_sql`: Extracts `CREATE TABLE`, `CREATE VIEW`, `CREATE PROCEDURE/FUNCTION`.
      - `parse_toml`: Extracts `[table]` sections.
    - **Tier 4 (Universal Fallback Chunking)**:
      - `parse_fallback`: Extracts root `file` node and breaks files >40 lines into coherent chunk blocks with `CONTAINS` edges.
      - Guarantees 100% of text and code files are indexed into SurrealDB and embedded into vector space via TEI.
- **Auto-Clustering on Ingestion (`src/ingestion/mod.rs`)**:
  - Upgraded `IngestionPipeline::ingest_directory` to automatically trigger `CommunityDetector::detect` and update SurrealDB community IDs immediately upon completing ingestion when nodes exist.
  - No manual second step (`make cluster`) required for graph partitioning.
  - Returns `clusters_computed` in `IngestResult`.
- **Structural Directory Subsystem Fallback (`src/api/mod.rs` & `ui/src/App.tsx`)**:
  - In `galaxies_handler` (`GET /api/galaxies`), if Louvain/Leiden modular communities are not yet computed or empty, automatically partitions nodes by their parent directory path.
  - In React Cosmograph UI (`App.tsx`), `clusters` memo falls back to directory partitioning, and `getNodeColor` hashes directory names so the 3D WebGL visualizer clusters nodes visually even before community detection.
  - The UI and API NEVER display an empty galaxy drawer or 0 galaxies when nodes exist.
- **SurrealDB Transport Payload Batching (`src/db/mod.rs`)**:
  - `store_nodes`: Chunked in batches of 50 nodes per HTTP query.
  - `store_edges`: Chunked in batches of 30 edges (90 statements) per HTTP query.
  - Eliminates request timeout and buffer overflow errors on large codebases.
- **Makefile UNIX `$PATH` Variable Collision Fix**:
  - Discovered that passing `PATH="$(DIR)"` in `Makefile` and `tools/omni-graph/Makefile` was overwriting the shell's `$PATH` environment variable, stripping `/bin` and causing `env: bash: No such file or directory`.
  - Renamed variable to `TARGET_DIR` across root and tool Makefiles, completely restoring CLI stability.
- **Test Suite**:
  - Added unit tests for YAML Kubernetes manifests, JSON configs, Shell functions, SQL schemas, TOML tables, and Fallback chunkers.
  - **48/48 unit tests passing** in Docker.
- **Zero-Downtime Hot Update**:
  - Compiled release binary via Docker and hot-updated `omni-rust-app` and `omni-graph-ui` without restarting SurrealDB or TEI inference engines.

#### Files Changed
- `src/parser/mod.rs` — Added `parse_yaml`, `parse_json`, `parse_shell`, `parse_sql`, `parse_toml`, and `parse_fallback`
- `src/ingestion/mod.rs` — Binary exclusion filter, universal fallback parsing, auto-clustering upon ingestion
- `src/db/mod.rs` — Chunked batching in `store_nodes` (50) and `store_edges` (30)
- `src/api/mod.rs` — Directory subsystem fallback in `galaxies_handler`
- `ui/src/App.tsx` — Directory subsystem fallback in `clusters` memo and `getNodeColor`
- `scripts/omni.sh` — Added `clusters_computed` display
- `Makefile` & `tools/omni-graph/Makefile` — Fixed `$PATH` variable collision
- `tests/unit_tests.rs` — Added 6 new unit tests (48 tests total)
- `CONTEXT.md` & `DEVLOG.md` — Updated state, metrics, and journal

---

### 2026-09-25 — Markdown AST & Knowledge Base Ingestion: GoLang, GenAI & k8s-eks

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### What Was Done
- **Markdown Semantic AST Extractor (`parse_markdown`)**:
  - Implemented deterministic markdown AST parsing in `src/parser/mod.rs`.
  - Extracts document titles, sections, chapters, sub-sections (`#`, `##`, `###`, `####`), and fenced code blocks (`go`, `python`, `yaml`, `bash`, etc.).
  - Extracts function definitions (`func`, `def`, `fn`, `class`, `k8s:Kind`) embedded inside code blocks.
  - Automatically constructs hierarchical `CONTAINS` edges from Documents -> Sections -> Subsections -> Code Snippets.
  - Added support for `.md` and `.markdown` in `src/ingestion/mod.rs`.
- **Knowledge Base Ingestion & Community Clustering**:
  - Ingested **`GoLang`**: 16 files, 325 AST nodes, 309 edges, 75 communities detected.
  - Ingested **`GenAI`**: 12 files, 452 AST nodes, 440 edges, 79 communities detected.
  - Ingested **`k8s-eks`**: 52 files, 696 AST nodes, 644 edges, 79 communities detected.
  - Total Knowledge Nodes: **1,473 nodes, 1,393 edges, 233 clusters** with 384-d TEI embeddings.
- **Continuous Container Operation**:
  - Maintained 100% container uptime for SurrealDB and TEI inference engines; hot-updated orchestrator binary in place without killing or re-initializing data services.
- **Spot-Checks & Verification**:
  - Verified `make graph-symbol SYM=Goroutines PROJECT=GoLang` returns exact markdown chapters, sections, and embedded code blocks.
  - Verified `make query-graph` synthesizes hybrid Graph-RAG answers for:
    - GoLang: Goroutines, channels, M:N scheduler, and interview Q&As.
    - GenAI: RAG architectures, chunking, re-ranking, and agentic RAG.
    - k8s-eks: AWS IRSA, EKS Access Entries, IAM Roles for Service Accounts.
- **Test Suite**:
  - Added unit tests `parse_markdown_sections_and_snippets` and `parse_markdown_python_code_block`.
  - 42/42 tests passing in Docker.

#### Files Changed
- `src/parser/mod.rs` — Added `parse_markdown()` and markdown extension dispatch
- `src/ingestion/mod.rs` — Added `md` and `markdown` to allowed scanner extensions
- `tests/unit_tests.rs` — Added 2 markdown parser unit tests; updated unsupported extension test (42 tests total)
- `DEVLOG.md` — This entry

---

### 2026-09-25 — Audit Backlog Resolution: SQL Injection, File Hash Persistence, LPA Shuffle, Singleton Noise Filter & Host Portability

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~30m

#### What Was Done
- **Finding #2 — SQL Injection Hardening**:
  - Implemented centralized `surql_escape()` in `src/db/mod.rs` escaping backslashes, single quotes, control characters, null bytes, and converting newlines.
  - Refactored all queries (`store_nodes`, `store_edges`, `search_vector`, `get_graph`, `find_symbols`, `find_references`, `update_communities`, `get_galaxy_aggregation`, `get_file_hashes`) to use `surql_escape()`.
  - Added unit test suite `escape_tests` verifying escaping and defeat of SQL injection payloads.
- **Finding #4 — Persisted File Hash Staleness Tracking**:
  - Updated `DbNode` and `store_nodes()` to persist `file_hash` directly into SurrealDB node records.
  - Added `get_file_hashes()` in `src/db/mod.rs` to fetch existing hashes on startup.
  - Added `FileCache::populate()` in `src/ingestion/mod.rs` to restore cached hashes on scan start.
  - Switched `FileCache` keys to canonical relative paths matching SurrealDB `file_path`.
- **Finding #5 — LPA Non-Random Iteration Order Fix**:
  - Implemented `SimpleRng` (lightweight, zero-dependency Xorshift64 PRNG) in `src/analysis/mod.rs`.
  - Added Fisher-Yates shuffle to node index order in each iteration of `CommunityDetector::detect()`, eliminating traversal bias while remaining deterministic and reproducible.
- **Finding #6 — Singleton Cluster Noise Elimination**:
  - Implemented `CommunityDetector::summarize_filtered(nodes, assignments, min_size)`.
  - Added `min_size` query parameter support to `GET /api/galaxies` and `POST /api/cluster`.
  - Added unit test suite `cluster_filter_tests` verifying singletons are dropped when `min_size >= 2`.
- **Finding #7 — Server-Side Galaxy Aggregation Query**:
  - Added `get_galaxy_aggregation()` in `src/db/mod.rs` leveraging SurrealDB `GROUP BY community`.
  - Updated `galaxies_handler` in `src/api/mod.rs` to prioritize server-side aggregation, bypassing memory-heavy full topology scans.
- **Finding #8 — Host Mount Portability**:
  - Updated `docker-compose.yml` to use `${HOST_MOUNT:-/Users}` for both directory mount and `BROWSE_ROOTS`.
  - Created `.env.example` documenting configuration for macOS, Linux, and Windows WSL.
- **Test Suite Expansion**:
  - Expanded `tests/unit_tests.rs` from 32 to 40 tests across 7 modules (parser, condenser, community, API normalization, SQL escape, PRNG shuffle, cluster filter).
  - Verified 40/40 tests passing in 0.01s via Docker `rust:latest`.
  - Verified `make test`, `make test-tool T=omni-graph`, and `make validate-specs` all pass cleanly.

#### Files Changed
- `src/db/mod.rs` — Centralized `surql_escape()`, `file_hash` persistence, server-side galaxy aggregation
- `src/ingestion/mod.rs` — `FileCache::populate()`, startup hash restoration, relative path indexing
- `src/analysis/mod.rs` — `SimpleRng` Xorshift64 PRNG, shuffled LPA iterations, `summarize_filtered()`
- `src/api/mod.rs` — `min_size` parameter in `GraphParams` / `ClusterPayload`, server-side aggregation in `galaxies_handler`
- `docker-compose.yml` — `${HOST_MOUNT:-/Users}` volume mount and BROWSE_ROOTS
- `.env.example` — Environment template for host directory mount configuration
- `tests/unit_tests.rs` — Expanded to 40 tests (added `escape_tests`, `lpa_rng_tests`, `cluster_filter_tests`)
- `CONTEXT.md` — Updated test coverage to 40/40 verified in Docker
- `STATUS.md` — Updated progress to 100% and test references
- `DEVLOG.md` — This entry

---

### 2026-09-25 — Codebase Audit: Test Suite, Security Hardening, Condenser Cap & Governance Fix

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~25m

#### What Was Done
- **Full Codebase Audit (Fresh-Eyes Pass)**:
  - Deep-reviewed every Rust module (api, db, parser, condenser, analysis, embedder, ingestion, config), UI (App.tsx), shell scripts, Docker infra, SurrealDB schema, hooks, and agent rules/skills.
  - Documented 15 findings across 3 severity tiers (3 Critical, 6 Important, 6 Minor).
- **Finding #1 — Created Comprehensive Test Suite (32 tests, all passing)**:
  - Created `src/lib.rs` to re-export modules for integration test access.
  - Created `tests/unit_tests.rs` with 4 test modules:
    - `parser_tests` (11 tests): Rust, Python, Go, JS, TS parsing, struct extraction, imports, edge detection, text truncation, empty file, unsupported extension.
    - `condenser_tests` (5 tests): Root symbol location, multi-hop BFS, callers/callees, nonexistent symbol, token estimate.
    - `community_tests` (7 tests): Node assignment, connected components, disconnected components, compact IDs, empty graph, isolated nodes, summarize correctness/ordering.
    - `api_normalize_tests` (8 tests): Full path, trailing slash, bare name, empty, whitespace, None, nested path.
  - Verified: `cargo test` → 32/32 passed in 0.01s via Docker `rust:latest`.
- **Finding #3 — Fixed Browse Path Traversal Vulnerability**:
  - Added `allowed_browse_roots()` function with `BROWSE_ROOTS` env var override.
  - Added `is_path_allowed()` which canonicalizes paths (resolving symlinks) and checks against the allowlist.
  - Parent path navigation now stops at allowlist boundaries (returns `null` instead of exposing parent directories).
  - Added `BROWSE_ROOTS=/workspace,/Users` to `docker-compose.yml` rust-app environment.
  - Unapproved paths return HTTP 403 Forbidden with clear error message.
- **Finding #9 — Added Condenser Hard Cap**:
  - BFS expansion now capped at `MAX_CONDENSED_NODES = 60`.
  - Markdown output truncated at 6000 chars (~1500 tokens) with warning message.
  - Prevents context window blowout on highly-connected utility symbols.
- **Finding #12 — Corrected Governance Metrics**:
  - CONTEXT.md test coverage metric changed from "90% Verified ✅" to "Unit tests written 🟡 Pending Docker verification".
- **Finding #13 — Added python3 Dependency Check**:
  - `omni.sh` now checks for `python3` at startup and fails with clear message if missing.
  - Synced to `.agents/skills/omni-graph/scripts/omni.sh` and `tools/omni-graph/skills/omni-graph/scripts/omni.sh`.

#### Files Changed
- `tests/unit_tests.rs` — New: 32 unit tests across parser, condenser, community, API
- `src/lib.rs` — New: library re-exports for test crate access
- `src/api/mod.rs` — Security: browse allowlist, canonicalization, symlink protection
- `src/condenser/mod.rs` — Hard cap: 60 nodes BFS, 6000 chars output
- `docker-compose.yml` — Added `BROWSE_ROOTS` env var
- `scripts/omni.sh` — Added python3 availability check
- `.agents/skills/omni-graph/scripts/omni.sh` — Synced
- `tools/omni-graph/skills/omni-graph/scripts/omni.sh` — Synced
- `CONTEXT.md` — Corrected test coverage metrics
- `DEVLOG.md` — This entry

---

### 2026-09-25 — Restore session-explorer Launcher, Implement /api/galaxies, and Beef Up Agent Reconnaissance Ladder

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~35m

#### What Was Done
- **Restored `make session-explorer` & Root Makefile Health**:
  - Identified root cause of `make session-explorer` failure: `run-tool` and `demo-tool` recipe blocks were omitted when inserting omni-graph shortcuts, causing `make` to report "Nothing to be done for 'run-tool'".
  - Resolved variable collision where `PATH ?=` in the root Makefile shadowed the user's system `$PATH` environment variable, corrupting tool binary execution paths; renamed Makefile parameter to `DIR ?=`.
  - Verified `make session-explorer ARGS="--help"`: successfully bundled 198 Bun frontend modules in 25ms, compiled the Go binary, and cleanly executed `--help`.
- **Implemented Read-Only Subsystem Inspection (`GET /api/galaxies`)**:
  - In `tools/omni-graph/src/api/mod.rs`, created `galaxies_handler` and registered `GET /api/galaxies`.
  - Aggregates existing community assignments (community ID, node count, dominant path, languages, and top symbols) without triggering an expensive write or re-clustering.
  - Rebuilt Docker Rust backend image (`omni-graph-rust-app`).
- **Added CLI Commands & Root Makefile Targets**:
  - Added `galaxies` subcommand to `tools/omni-graph/scripts/omni.sh` and synced to `.agents/skills/omni-graph/scripts/omni.sh`.
  - Added `make graph-symbol`, `make graph-references`, `make graph-condense`, and `make graph-galaxies` to root `Makefile` and tool-local `Makefile`.
- **Beefed Up Agent Reconnaissance Ladder & Documentation**:
  - Upgraded `.agents/rules/08-omni-graph-enforcement.md` and `tools/omni-graph/rules/08-omni-graph-enforcement.md` with the 3-Tier Omniverse Reconnaissance Ladder:
    - Tier 1 (Macro): `workspaces` (discover ingested codebases).
    - Tier 2 (Meso): `graph-galaxies` (discover high-level subsystem communities and architectural boundaries).
    - Tier 3 (Micro): `graph-symbol`, `graph-references`, `graph-condense` (extract token-budgeted <1500 AST subgraphs).
  - Upgraded `.agents/skills/omni-graph/SKILL.md` and `tools/omni-graph/skills/omni-graph/SKILL.md` with multi-workspace guides, token budgeting rules, and practical agent recipes.
  - Updated `AGENTS.md` orientation checklist and Key Commands table to enforce AST/Omni-Graph reconnaissance over brute-force grep.
- **Verification & Governance**:
  - Verified `make graph-galaxies PROJECT=session-explorer` returns 15 detected subsystems.
  - Verified `make graph-condense SYM=extractWorkspace PROJECT=session-explorer` returns <1500 tokens of AST markdown.
  - Verified `make validate-specs` (2 tools checked, 0 errors, 0 warnings).
  - Gracefully stopped Docker containers via `docker compose down`.

#### Files Changed
- `Makefile` — Restored `run-tool`, `demo-tool`, renamed `PATH` to `DIR`, added graph convenience targets
- `tools/omni-graph/Makefile` — Added `galaxies`, `condense`, `symbol`, `references` shortcuts
- `tools/omni-graph/src/api/mod.rs` — Added `GET /api/galaxies` endpoint & handler
- `tools/omni-graph/scripts/omni.sh` — Added `galaxies` CLI command
- `.agents/skills/omni-graph/scripts/omni.sh` — Synced `galaxies` CLI command
- `tools/omni-graph/skills/omni-graph/scripts/omni.sh` — Synced `galaxies` CLI command
- `.agents/rules/08-omni-graph-enforcement.md` — Beefed up with 3-Tier Reconnaissance Ladder & token budgeting
- `tools/omni-graph/rules/08-omni-graph-enforcement.md` — Synced rule updates
- `.agents/skills/omni-graph/SKILL.md` — Complete agent guide for omniverse navigation
- `tools/omni-graph/skills/omni-graph/SKILL.md` — Synced skill documentation
- `.agents/AGENTS.md` — Updated orientation checklist and Key Commands table
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-25 — Eliminate Nested Scroll Trap & Dynamically Adapt Cluster Symbol List

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### What Was Done
- **Root Cause Analysis of Cluster Accordion Scroll Trap**:
  - Inside the expanded galaxy cluster card, the symbol list container had hardcoded `maxHeight: 200` with `overflowY: 'auto'`.
  - This caused two major UX flaws:
    1. **Nested Scroll Chaining / Trap**: When mouse/trackpad scrolling occurred over the expanded card, wheel events were captured by the child container (`outer.scrollTop` remained locked at `0`). The outer drawer could never scroll down to view clusters 10, 11, ... 20+, making it feel as if the user could only see 7–10 clusters.
    2. **Artificial Content Clamping**: Despite a cluster containing 20+, 50+, or 67 symbols, only 7–8 items fit inside the 200px box, forcing micro-scrolling in a tiny nested box.
- **Implemented Dynamic Adaptation & Unified Flow**:
  - Removed `maxHeight: 200` and `overflowY: 'auto'` from the symbol list container.
  - Allowed symbols to flow naturally into the drawer's primary scroll container, establishing a **single unified scroll context**.
  - Added dynamic pagination: initial slice displays up to 30 symbols directly (adapting height dynamically to 10, 15, 20, 25, 30 items without awkward cutoff).
  - For larger clusters (> 30 items), added a sleek `+ Show all X symbols (Y more)` toggle that expands to show all items on demand with a `Show less (first 30)` toggle.
- **Verified via Automated Headless Browser Testing**:
  - Simulated mouse wheel scrolling (`deltaY: 600`, `deltaY: 800`) directly over the expanded cluster card.
  - Confirmed `outer.scrollTop` increased from `0` to `600px` and `1400px`, scrolling smoothly through all 20+ clusters without trapping.
  - Tested clicking `Show all 67 symbols`: verified all 67 items render in the flow (`cardHeight: 1985px`) and `Show less` button renders.
  - Captured verified screenshots: `cluster_expanded_dynamic.png`, `cluster_scrolled_deep.png`, `cluster_show_all_symbols.png`.

#### Files Changed
- `tools/omni-graph/ui/src/App.tsx` — Replaced fixed `maxHeight: 200` and `overflowY: 'auto'` with dynamic adaptation and `Show all / Show less` toggle
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-25 — Fix Cluster Card Flexbox Collapse & Add Windowed Subsystem Rendering

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### What Was Done
- **Root Cause Analysis of "Blank Lines" on Large Workspaces (e.g., DSA with 902 Clusters)**:
  - In `tools/omni-graph/ui/src/App.tsx`, the cluster list container is a flex column (`display: flex; flex-direction: column; overflow-y: auto; height: 515px`).
  - By default in CSS Flexbox, child items have `flex-shrink: 1`. Each card also possessed `overflow: hidden`, causing its minimum content height to resolve to `0`.
  - In small workspaces like `session-explorer` (15 items), 15 * 36px = 540px roughly fits inside the container without squishing.
  - In large workspaces like `DSA` (902 items), flexbox distributed the height deficit across all 902 children with `flex-shrink: 1`, squishing every single card down to `2px` (the 1px top border + 1px bottom border), while `overflow: hidden` clipped the 34px content. This caused 900+ cluster cards to appear as identical stacked "blank lines".
- **Implemented CSS Flexbox & Layout Hardening**:
  - Added `flexShrink: 0` and `minHeight: 'fit-content'` to every cluster card so cards never collapse regardless of list length.
  - Added `minWidth: 0, flex: 1` to cluster titles to ensure long package/file paths truncate cleanly with ellipsis rather than pushing or breaking the node count badge.
  - Enhanced single-node cluster labeling in `clusters` `useMemo`: single-node clusters now display the symbol label (`Cluster #${cid}: ${dominant} • ${clusterNodes[0].label}`) instead of repetitive identical directory strings.
  - Added empty search state: `No clusters or symbols match "${clusterSearchTerm}"`.
- **Implemented Windowed/Virtualized List Rendering**:
  - Implemented `visibleClusterCount` (initial 60 items) with progressive scroll loading (`onScroll` auto-increments by 40 when within 120px of bottom) and an explicit `[ Load More (X remaining) ]` button.
  - This ensures silky-smooth 60fps performance and zero DOM lag when browsing repositories with 1,000+ detected clusters.
- **End-to-End Browser Verification**:
  - Automated headless Chrome testing with WebGL angle rendering.
  - Verified `computedHeight: "36px"` across all sample cluster cards for `session-explorer` and `DSA`.
  - Verified both Omniverse view (`All Workspaces`, 1627 clusters) and isolated workspace view (`DSA`, 902 clusters) render cards with colors, cluster names, node counts, and chevrons.
  - Saved verified screenshot artifacts to brain directory (`dsa_drawer_verified.png`, `dsa_selected_drawer.png`).

#### Files Changed
- `tools/omni-graph/ui/src/App.tsx` — Added `flexShrink: 0`, title ellipsis truncation, single-node symbol labeling, and windowed list loading
- `tools/omni-graph/DEVLOG.md` — This entry
- `tools/omni-graph/CONTEXT.md` — Updated last session date and agent state

---

### 2026-09-24 — Fix SurrealDB Record ID Delimiters & Community Node Persistence

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### What Was Done
- **Root Cause Analysis of "Galaxies 0 & Clustered Nodes 0 / 2547"**:
  - SurrealDB v2 record IDs when returned from `SELECT` enclose IDs containing spaces, dashes, or colons with backticks (e.g. `node:\`DSA:00 - Fundamentals/001 - Queue and Stack/queue_and_stack.py:Node:38\``).
  - In `src/db/mod.rs`, `update_communities` only stripped `"node:"`, leaving literal backticks in `clean_id`.
  - Passing `'`clean_id`'` into `type::thing('node', '{}')` caused SurrealQL to search for record IDs literally starting and ending with backticks, matching 0 records and updating nothing (while returning HTTP 200 OK without errors).
  - Consequently, after running `POST /api/cluster`, all nodes in SurrealDB remained with `community: null`, causing the React UI's galaxy grouping to calculate 0 clusters and `0 / 2547` clustered nodes.
- **Implemented Fix in `src/db/mod.rs`**:
  - Enhanced `update_communities` to strip `node:` prefix and trim all delimiter wrappers (`` ` ``, `⟨`, `⟩`, `"`, `'`).
  - Added string character escaping for backslashes and single quotes.
  - Implemented batch chunking (150 statements per `/sql` HTTP request) to ensure reliable query execution across thousands of nodes without SurrealDB request size issues or JSON memory spikes.
- **Added UI Cache-Busting**:
  - In `tools/omni-graph/ui/src/App.tsx`, added dynamic cache-busting timestamp `_t=${Date.now()}` to `loadGraph` so browser HTTP caching never returns stale node objects without updated community integers.
- **Rebuilt & Verified**:
  - Rebuilt `omni-rust-app` and `omni-graph-ui` containers.
  - Tested `POST /api/cluster` for `tutor-intelligence` workspace: all 231/231 nodes partitioned into 98 communities and verified populated in SurrealDB.
  - Tested `POST /api/cluster` for full Omniverse: all 2,547/2,547 nodes partitioned into 1,632 galaxy clusters.
  - Verified `GET /api/graph` returns 2,547/2,547 clustered nodes with populated `community` values.

#### Files Changed
- `tools/omni-graph/src/db/mod.rs` — Trim delimiters and batch update statements in `update_communities`
- `tools/omni-graph/ui/src/App.tsx` — Added cache-busting query parameter to `loadGraph`
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-24 — Path Normalization for Workspace Clustering & Queries

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~10m

#### What Was Done
- **Root Cause Analysis of 0 Communities Detected**:
  - Ingestion assigns workspaces by their project/folder name (e.g., `tutor-intelligence`, `DSA`, `session-explorer`), not the full absolute host path.
  - When users ran `make cluster /Users/aparv/.../tutor-intelligence`, the full directory path was passed verbatim as the workspace partition name, causing SurrealDB to query `workspace = '/Users/...'` which returned 0 nodes.
- **Implemented Polyglot Workspace Path Normalization**:
  - Added `normalize_workspace` helper in [`src/api/mod.rs`](file:///Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/tool-scripts/tools/omni-graph/src/api/mod.rs) to cleanly convert directory paths into canonical workspace folder names across all endpoints: `graph_handler`, `search_handler`, `condense_handler`, `symbol_handler`, `references_handler`, `query_handler`, and `cluster_handler`.
  - Added `norm_ws` shell helper in `scripts/omni.sh` across all subcommands (`cluster`, `search`, `symbol`, `references`, `condense`, `query`).
  - Synced scripts to `.agents/skills/omni-graph/scripts/omni.sh`.
- **Verified Successful Execution**:
  - `make cluster /Users/aparv/.../tutor-intelligence` now successfully detects **98 communities** across 231 AST nodes.
  - `make cluster /Users/aparv/.../DSA` now successfully detects modular pattern clusters across 1,184 AST nodes.
- **Container Cleanup**:
  - Stopped all running containers via `docker compose down` per user constraint.

#### Files Changed
- `tools/omni-graph/src/api/mod.rs` — Added `normalize_workspace` and wired into all handlers
- `tools/omni-graph/scripts/omni.sh` — Added `norm_ws` helper
- `.agents/skills/omni-graph/scripts/omni.sh` — Synchronized with root tool script
- `tools/omni-graph/skills/omni-graph/scripts/omni.sh` — Synchronized with root tool script
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-24 — UI Galaxy Clustering Suite & Subsystem Isolation Mode

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### What Was Done
- **UI Auto-Clustering upon Ingestion**:
  - Added `Auto-compute Galaxy Clusters immediately after ingestion` checkbox (enabled by default) directly in the Directory Traversal & Ingest modal.
  - Multi-stage progress tracking in the UI:
    1. Parsing AST structures and computing 384-d vector embeddings (`POST /api/ingest`)
    2. Automatically computing Louvain/Leiden modular community clusters (`POST /api/cluster`) for the ingested workspace
    3. Auto-refreshing the Cosmograph canvas and automatically sliding open the Galaxy Subsystems Navigator.
- **On-Demand Galaxy Clustering Command Center**:
  - Added primary cyber button `[ ⚡ Compute / Re-cluster Galaxies ]` inside the Galaxy Drawer with live clustering animation and toast feedback.
  - Added workspace-aware clustering: one-click to re-cluster the active workspace or partition the entire Omniverse.
  - Added quick `[ ⚡ Cluster Universe ]` trigger button in top navigation bar when unclustered nodes are detected.
  - Added Subsystem Metrics summary card in the drawer showing total detected galaxies and clustered node ratio.
- **Galaxy Subsystem Isolation Mode (`[ 🔭 Isolate Galaxy ]`)**:
  - Users can click "Isolate Galaxy" on any cluster card to isolate that specific subsystem in 3D WebGL space. All non-cluster nodes and external links are filtered out, providing zero-distraction architectural inspection.
  - Floating top banner indicating active isolation with single-click `[ ✕ Exit Isolation (Show Omniverse) ]`.
- **Subsystem Architecture Export**:
  - Added `[ 📋 Copy Architecture ]` action to every galaxy card, generating dense markdown summaries of all constituent functions and types ready for AI agent prompting.
  - Added symbol kind filter chips (`All`, `function`, `struct`, `class`, `import`) inside each galaxy cluster accordion.
- **Styles & Verification**:
  - Added purple cyber glow buttons, symbol kind badges, and isolation banner styles in `index.css`.
  - Verified `npm run build` passes cleanly with 0 TypeScript errors (3.08s).
  - Rebuilt Docker image `omni-graph-ui` via `docker compose build graph-ui`.
  - Shut down all containers via `docker compose down` releasing host ports.

#### Files Changed
- `tools/omni-graph/ui/src/App.tsx` — Full galaxy clustering suite, auto-cluster after ingest, isolation mode
- `tools/omni-graph/ui/src/index.css` — Purple cyber buttons, kind badges, isolation styling
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-24 — UI/UX Evolution: Workspace Filter, Galaxy Navigator & Directory Traversal Modal

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### What Was Done
- **Implemented Interactive Filesystem Directory Traversal API (`GET /api/browse`)**:
  - Added [`browse_handler`](file:///Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/tool-scripts/tools/omni-graph/src/api/mod.rs) to Axum backend.
  - Dynamically lists child directories, parent paths, and automatically flags detected codebases (detecting `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `.git`, `Makefile`) with language tags.
- **Redesigned Cosmograph WebGL Frontend (`http://localhost:3000`)**:
  - **Workspace & Codebase Selector Dropdown**:
    - Replaced the chaotic "all-nodes-at-once" view with an interactive Workspace Dropdown in the top HUD.
    - Users can select `🌐 All Workspaces (Omniverse)` or focus on a specific project (e.g. `📁 session-explorer`, `📁 DSA`, `📁 DesignPatterns`). Selecting a workspace dynamically refetches `/api/graph?workspace=...` and isolates that codebase.
  - **Collapsible Galaxy Subsystems Navigator (Left Drawer)**:
    - Added floating `[ 🌌 Galaxies ]` toggle button with community count badge.
    - Displays all detected Louvain/Leiden modular clusters with neon color dots, cluster IDs, and node counts.
    - Clicking a cluster card highlights and zooms into those nodes in the WebGL Cosmograph canvas and expands an accordion listing all constituent functions/structs with real-time fuzzy filtering.
    - Clicking any function card instantly focuses that node and opens the Node Inspector.
  - **Futuristic Directory Traversal & Ingestion Modal**:
    - Replaced the static path text input with a Cyber-Glass Modal (`[ ⚡ Ingest Codebase ]`).
    - Features clickable breadcrumbs (`root / Users / aparv / ...`), quick-jump bookmarks (`[ 💻 Monorepo ]`, `[ 👤 Host Users ]`, `[ 📦 Tools ]`), and "Up one level" navigation.
    - Lists directories with folder icons, codebase badges, and direct `[ Ingest ]` buttons without forcing the user to type manual file paths.
  - **Node Inspector & Context Condenser Drawer (Right Drawer)**:
    - Slides out when clicking any node in the graph or galaxy list.
    - Displays symbol name, kind, file path, line range, community galaxy, code snippet preview, and one-click **"Copy Prompt Slice (<1500 tokens)"** for AI coding agents.
  - **Clean Clutter-Free Aesthetics**:
    - All drawers and modals are collapsible, floating over the full-viewport 3D WebGL canvas with deep glassmorphism and cyber-neon accents.
- **Build & Container Verification**:
  - `npm run build` compiled cleanly with 0 TypeScript errors.
  - Rebuilt `omni-rust-app` and `omni-graph-ui` containers.
  - Verified `GET /api/browse?path=/workspace` and `/workspace/tools`.
  - Shut down all containers (`docker compose down`) per user directive to release host ports.

#### Files Changed
- `tools/omni-graph/src/api/mod.rs` — Added `BrowseParams`, `DirEntry`, `BrowseResponse`, and `browse_handler`
- `tools/omni-graph/ui/src/App.tsx` — Full UI redesign with workspace filter, galaxy drawer, and folder browser modal
- `tools/omni-graph/ui/src/index.css` — Modern glassmorphism, drawer transitions, and cyber button styling
- `tools/omni-graph/DEVLOG.md` — This entry

---

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### What Was Done
- **Fixed Ingestion Connection Failure & Silent SurrealDB Rejections**:
  - Root-caused `Remote end closed connection without response` and silent database insert failures:
    - SurrealDB v2 strictly requires lowercase headers `surreal-ns: omni` and `surreal-db: graph`. Sending v1 headers `NS` and `DB` caused SurrealDB to return `status: "ERR", result: "Specify a namespace to use"`, which returned HTTP 200 and was silently dropped.
    - Updated `src/db/mod.rs` to send `surreal-ns` and `surreal-db` headers, and added explicit error-checking inspecting response JSON for `status: "ERR"` and returning actionable Rust errors.
- **Fixed HuggingFace TEI Batch Size Limit (422 Unprocessable Entity)**:
  - TEI enforces a maximum batch size of 32 on CPU/ARM64. Extracted files with > 32 AST nodes (such as `session-explorer/src/web/src/app.js` with 40 nodes) were rejected with `batch size 40 > maximum allowed batch size 32`.
  - Updated `src/embedder/mod.rs` to chunk texts into slices of `<= 32` before sending to `/embed`, validating dimensions (384-dim) per chunk and aggregating results.
- **Fixed Ingestion Cache Invalidation on Error**:
  - Updated `FileCache` in `src/ingestion/mod.rs` so file hashes are only committed to cache once AST nodes and graph edges are successfully persisted to SurrealDB.
- **Eliminated GNU Make Target Collisions (`make: '...' is up to date`)**:
  - Configured trailing argument absorption in root `Makefile` and `tools/omni-graph/Makefile` using `.PHONY: $(MAKECMDGOALS)` and `$(filter-out $(firstword $(MAKECMDGOALS)),$(MAKECMDGOALS)): @true`.
  - Now running `make ingest /path/to/folder` cleanly executes ingestion without emitting Make file status noise.
- **End-to-End Verification Completed**:
  - Ingested `session-explorer` (`make ingest /Users/aparv/.../tools/session-explorer`): 8 files scanned, 7 indexed, 68 AST nodes, 750 directional graph edges created in 3.3s.
  - Ran `make workspaces`: Confirmed `session-explorer` partition with 68 nodes across Go and JavaScript.
  - Ran `make cluster`: Detected 14 modular communities via label propagation and assigned galaxy IDs.
  - Ran `make search-graph Q="Scanner"`: Verified 24ms vector similarity ANN search across HNSW index.
  - Ran `make query-graph Q="How does session-explorer search work?"`: Verified macroscopic architecture synthesis and expanded AST call chain condensation.

#### Files Changed
- `tools/omni-graph/src/db/mod.rs` — Added `surreal-ns`/`surreal-db` headers and `status: "ERR"` validation
- `tools/omni-graph/src/embedder/mod.rs` — Chunked embedding requests into batches of `<= 32`
- `tools/omni-graph/src/ingestion/mod.rs` — Fixed `FileCache` to record hashes only upon successful persistence
- `tools/omni-graph/scripts/omni.sh` — Added urllib timeout=300 and clear error reporting
- `tools/omni-graph/Makefile` — Clean trailing argument absorption and modern Docker test fallback
- `Makefile` — Clean trailing argument absorption for root `make ingest` and `make cluster`
- `tools/omni-graph/DEVLOG.md` — This entry

---

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### What Was Done
- **Resolved Docker Compose Rust Build Failure (`exit code 101`)**:
  - Upgraded Rust builder to `FROM rust:latest` (Rust 1.85+ supports `edition2024` required by dependencies like `idna_adapter v1.2.2`).
  - Fixed Tree-sitter borrowing issue (`set_language(&Language)`).
  - Wired missing `workspace` arguments across all endpoints and modules (`db/mod.rs`, `ingestion/mod.rs`, `analysis/mod.rs`, `api/mod.rs`).
  - Cleaned all unused imports and warnings across all Rust source files; `docker compose build rust-app` now compiles cleanly in release mode with 0 errors and 0 warnings.
- **Resolved SurrealDB Startup & Health Check Failure (`graph-db failed to start`)**:
  - Fixed volume permission issue on `/data`: Added `user: "0:0"` so SurrealDB runs with root permissions and can create RocksDB/SurrealKV directories in named volumes.
  - Upgraded embedded storage engine from deprecated `file:/data/omni.db` to production `surrealkv:/data/omni.db`.
  - Fixed distroless health check: `surrealdb/surrealdb` image is a scratch image without `/bin/sh`. Replaced `CMD-SHELL` with exec form `["CMD", "/surreal", "isready", "--endpoint", "http://127.0.0.1:8000"]`. Verified container reports `"healthy"`.
- **Implemented Multi-Workspace Database Schema & Partitioning**:
  - Updated `src/db/schema.surql`: Added `workspace` field and index `idx_node_workspace`, `idx_edge_workspace`.
  - Partitioned node IDs: `node:{workspace}:{file_path}:{name}:{line}` with safe SurrealDB `type::thing('node', ...)`.
  - Added multi-workspace filtering to vector search (`search_vector`), graph retrieval (`get_graph`), symbol lookup (`find_symbols`), and reference callers (`find_references`).
  - Added new `/api/workspaces` endpoint to list all ingested codebases and their node/file partition statistics.
- **Enhanced Host Filesystem Access in Docker**:
  - Mounted `/Users:/Users:ro` in `docker-compose.yml` so any directory or codebase across the host Mac can be ingested directly without path translations.
- **Implemented User-Requested Unified Make Interface**:
  - Updated `tools/omni-graph/Makefile` and root `Makefile` with clean trailing argument capture:
    - `make ingest <path> [PROJECT=name]` (e.g. `make ingest /workspace` or `make ingest /Users/.../my-project`)
    - `make cluster [PROJECT=name]` (executes Louvain/Leiden community detection to assign galaxy IDs)
    - `make workspaces` (inspects partitioned codebases)
    - `make search Q="..." [PROJECT=name]` (vector similarity search)
    - `make query Q="..." [PROJECT=name]` (hybrid Graph-RAG synthesis)
  - Preserved the user's requested startup banner in `make up` and `make help`.
- **Created Comprehensive Documentation**:
  - Fully documented the multi-workspace database schema and Louvain/Leiden community clustering computation in `tools/omni-graph/README.md`.
  - Updated `omni.sh` CLI with `workspaces` and `cluster` subcommands.
  - Verified monorepo governance with `make validate-specs` (0 errors, 0 warnings).

#### Requirements Addressed
- R-001, R-002, R-003, R-004, R-005, R-006, R-007, R-008, R-009, R-010, R-012, R-022, R-023, R-026, R-028

#### Files Changed
- `tools/omni-graph/Dockerfile` — Upgraded to `rust:latest`
- `tools/omni-graph/docker-compose.yml` — Added `/Users:/Users:ro` volume mount
- `tools/omni-graph/src/db/schema.surql` — Added `workspace` field & secondary indexes
- `tools/omni-graph/src/parser/mod.rs` — Fixed Tree-sitter borrowing & added workspace to AST nodes/edges
- `tools/omni-graph/src/db/mod.rs` — Added workspace isolation, safe record ID handling, and `get_workspaces`
- `tools/omni-graph/src/ingestion/mod.rs` — Added workspace derivation and relative path extraction
- `tools/omni-graph/src/analysis/mod.rs` — Wired workspace filtering into `GraphRagEngine::query`
- `tools/omni-graph/src/api/mod.rs` — Wired workspace query/payload parameters & added `/api/workspaces`
- `tools/omni-graph/src/main.rs` — Cleaned unused imports
- `tools/omni-graph/scripts/omni.sh` — Added multi-workspace ingestion, clustering, and workspaces commands
- `tools/omni-graph/skills/omni-graph/scripts/omni.sh` — Synced updated CLI
- `tools/omni-graph/Makefile` — Added easy `ingest`, `cluster`, `workspaces`, `search`, `query` targets & banner
- `Makefile` — Added root-level `omni-graph`, `ingest`, `cluster`, `workspaces`, `search-graph`, `query-graph` targets
- `tools/omni-graph/README.md` — Documented multi-workspace pattern and community clustering in depth
- `tools/omni-graph/CONTEXT.md` — Updated next steps and state
- `tools/omni-graph/DEVLOG.md` — This entry

#### Next Steps
- User runs `make up` from terminal to launch the multi-container stack.
- Ingest monorepo: `make ingest /workspace`.
- Compute galaxy clustering: `make cluster`.

---

### 2026-09-24 — LSP Symbolic Tools, Community Detection & Graph RAG Engine

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~25m

#### What Was Done
- Implemented Serena-equivalent symbolic LSP endpoints in Axum & SurrealDB:
  - `GET /api/symbol?name=...` (`textDocument/definition` equivalent)
  - `GET /api/references?symbol=...` (`textDocument/references` equivalent)
- Implemented Microsoft GraphRAG-inspired Community Detection and Hybrid Retrieval:
  - `src/analysis/mod.rs`: Label propagation modular clustering (Louvain/Leiden equivalent) partitioning AST graph into architectural community clusters
  - `POST /api/cluster`: Computes & stores `community_id` across SurrealDB nodes
  - `POST /api/query`: Unified Graph-RAG retrieval pipeline combining microscopic vector similarity, macroscopic community summaries, and expanded AST call subgraphs
- Extended agent CLI `omni.sh` with `symbol`, `references`, `query`, and `cluster` subcommands
- Synced `setup-agent` command to root `Makefile` and `tools/omni-graph/Makefile`
- Integrated Rule 08 (`08-omni-graph-enforcement.md`) and `omni-graph` skill into `.agents/AGENTS.md`
- Rewrote `tools/omni-graph/README.md` with complete architectural documentation, 4-tier enforcement shield, and full API reference
- Verified spec compliance with `make validate-specs` (0 errors, 0 warnings)

#### Requirements Addressed
- R-003, R-005, R-006, R-007, R-008, R-009, R-010, R-012, R-025, R-026, R-027, R-028

#### Files Changed
- `tools/omni-graph/src/db/mod.rs` — Added `find_symbols`, `find_references`, `update_communities`
- `tools/omni-graph/src/analysis/mod.rs` — New: Community detection & Graph-RAG engine
- `tools/omni-graph/src/api/mod.rs` — Added routes & handlers for `/api/symbol`, `/api/references`, `/api/query`, `/api/cluster`
- `tools/omni-graph/src/main.rs` — Registered `mod analysis`
- `tools/omni-graph/skills/omni-graph/scripts/omni.sh` — Added symbolic and Graph-RAG commands
- `tools/omni-graph/skills/omni-graph/SKILL.md` — Documented symbolic LSP and Graph-RAG recipes
- `tools/omni-graph/README.md` — Rewritten with comprehensive guides and API reference
- `tools/omni-graph/Makefile` — Added `setup-agent`
- `Makefile` — Added root `setup-agent` target and help docs
- `.agents/AGENTS.md` — Added Rule 08 and `omni-graph` skill to tables
- `tools/omni-graph/DEVLOG.md` — Updated: this entry

#### Decisions Made
- **Hybrid Graph-RAG Pipeline**: Combined vector similarity seeds with directed AST 1-hop expansion and macro community cluster summaries to deliver dense, hallucination-resistant prompt contexts.
- **Direct Symbolic Endpoints**: Added dedicated `/api/symbol` and `/api/references` endpoints mirroring LSP semantics so coding agents avoid regex text search.

#### Next Steps
- User test run: `make up` to launch the Docker Compose cluster (SurrealDB + HuggingFace TEI + Rust App + WebGL UI)
- Ingest a local codebase folder (`make setup-agent` + `./scripts/omni.sh ingest <path>`)
- Verification and testing phase transition (`in-progress` → `testing`)

---

### 2026-09-24 — Agent Enforcement Shield & Antigravity Hook Architecture

**Agent/Author**: Antigravity
**SDLC Phase**: `draft`
**Duration**: ~20m

#### What Was Done
- Deeply analyzed the agent enforcement gap raised by user (preventing LLM cognitive drift / blind `grep`/`cat` context burning)
- Researched Serena's LSP symbolic tool model and ECC's deterministic pre-tool execution guardrails
- Researched Microsoft GraphRAG hierarchical community detection and RAGFlow deep document chunking
- Studied Antigravity customization specification (`skills/`, `rules/`, `hooks.json`, MCP configs)
- Extended `specs/catalog/omni-graph.md` with:
  - 4 new functional requirements: R-025 (Lifecycle hooks), R-026 (Native Agent Interface/MCP/Skill), R-027 (Automated Setup CLI), R-028 (Context Condenser)
  - 4 new acceptance criteria (AC-015 through AC-018)
  - Section 7.7: Multi-Layered Agent Enforcement Architecture (4-tier shield)
  - Section 7.8: Antigravity Hooks (`hooks.json`), Skills (`skills/omni-graph/`), Rules, and Setup bootstrapper specification
- Synchronized `STATUS.md` and `CONTEXT.md` traceability matrices (28 R-XXX, 18 AC-XXX, 11 NF-XXX)

#### Requirements Addressed
- Specification phase expansion (R-025 → R-028, AC-015 → AC-018)

#### Files Changed
- `specs/catalog/omni-graph.md` — Updated: added R-025..R-028, AC-015..AC-018, Sections 7.7 and 7.8
- `tools/omni-graph/STATUS.md` — Updated: metrics (28 reqs, 18 ACs) and traceability matrix
- `tools/omni-graph/CONTEXT.md` — Updated: progress metrics, reference projects, and traceability summary
- `tools/omni-graph/DEVLOG.md` — Updated: this entry

#### Decisions Made
- **Deterministic PreToolUse Hook**: Hard-block or rewrite whole-codebase `grep`/`cat` tool calls to force agents through the Omni-Graph AST and vector search endpoints.
- **Symbolic Native Tooling**: Provide an Antigravity skill and MCP server so agents have first-class semantic actions (`trace_call_chain`, `semantic_search`, `get_subgraph`).
- **Interactive Multi-Target Setup**: Support both workspace-level (`.agents/`) and global (`~/.gemini/config`) target configuration via `make setup-agent`.

#### Blockers Encountered
- **None**

#### Next Steps (for the next session)
- Human decision on setup configuration target preference
- Transition SDLC status from `draft` to `spec-review`
- Begin Task 1: `docker-compose.yml` infrastructure

---

### 2026-09-24 — Project Genesis & Specification

**Agent/Author**: @antigravity-opus
**SDLC Phase**: `—` → `draft`
**Duration**: ~30m

#### What Was Done
- Scaffolded tool directory from `_template`
- Conducted deep research on 4 reference architectures (Graphify, CodeGraph, Serena, ECC)
- Researched SurrealDB v2 vector indexing (HNSW, not MTREE — critical correction)
- Researched HuggingFace TEI ARM64 compatibility (CPU-only in Docker, Metal via native only)
- Researched Cosmograph React/TypeScript WebGL graph visualization
- Authored comprehensive specification (`specs/catalog/omni-graph.md`):
  - 24 functional requirements (R-001 → R-024)
  - 14 acceptance criteria (AC-001 → AC-014)
  - 11 non-functional requirements (NF-001 → NF-011)
  - Full REST API contract (health, ingest, graph, search, stats)
  - SurrealQL schema definition with HNSW index
  - Architecture diagrams and edge taxonomy
- Initialized STATUS.md with full traceability matrix
- Initialized CONTEXT.md with architecture summary and key decisions
- Updated tool catalog entry

#### Requirements Addressed
- Specification phase only — no implementation requirements started

#### Files Changed
- `tools/omni-graph/` — New: entire tool directory (scaffolded from template)
- `specs/catalog/omni-graph.md` — New: full specification
- `tools/omni-graph/STATUS.md` — Updated: 24 requirements, 14 ACs, 11 NFRs
- `tools/omni-graph/CONTEXT.md` — Updated: architecture, decisions, references
- `tools/omni-graph/DEVLOG.md` — Updated: this entry
- `tools/omni-graph/README.md` — Updated: project overview
- `tools/omni-graph/CHANGELOG.md` — Updated: initial scaffolding
- `tools/omni-graph/spec.md` — Updated: pointer to catalog spec
- `tools/omni-graph/Makefile` — Updated: Rust + Docker targets
- `tools/README.md` — Updated: catalog entry

#### Decisions Made
- **HNSW over MTREE**: SurrealDB v2 uses HNSW for production ANN — MTREE is deprecated/experimental
- **TEI CPU-only in Docker**: macOS Docker lacks GPU passthrough; Metal only via native Homebrew install
- **384-dim bge-small-en-v1.5**: Lightweight, high-quality BERT embeddings optimized for ARM64 SIMD
- **Cosmograph WebGL**: GPU-accelerated force layout, supports 100K+ nodes at 60fps
- **EXTRACTED vs INFERRED edges**: Following Graphify's taxonomy for relationship categorization

#### Blockers Encountered
- **None**

#### Next Steps (for the next session)
1. Human review of specification → transition to `spec-review`
2. After approval, begin Task 1: `docker-compose.yml`
3. Task 2: Rust orchestrator (`Cargo.toml` + `src/main.rs` skeleton)
4. Task 3: SurrealQL schema initialization script
5. Task 4: React + Cosmograph UI scaffold (Vite + TypeScript)

---

### 2026-09-25 — Agent Analytics & LSP Telemetry Dashboard in Omni-Graph

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~45m

#### What Was Done
- **Engineered Omni-Graph Analytics & Telemetry Engine (`src/analytics/mod.rs`)**:
  - Implemented `AnalyticsEngine::scan_analytics()`: scans active IDE agent trajectories in `~/.gemini/antigravity-ide/brain/` (47,000+ steps across 14 sessions).
  - Categorizes tool calls into Omni-Graph AST tools (`graph-symbol`, `graph-references`, `graph-condense`, `query-graph`, `graph-galaxies`), Filesystem tools (`view_file`, `list_dir`), Editor tools (`replace_file_content`, `write_to_file`), and Execution tools (`run_command`).
  - Implemented language telemetry tracking: extracts programming language extensions (`.go`, `.rs`, `.py`, `.ts`, `.md`, `.yaml`, `.sql`, etc.) and maps them to their respective active LSP parse engines (e.g. `tree-sitter-go / gopls (LSP)`, `tree-sitter-rust / rust-analyzer (LSP)`, `omni-ast-md (Hierarchical Section Engine)`).
  - Implemented token savings formula: computes ~863,000 tokens preserved by contrasting targeted <1,500 token AST subgraph condensation against ~15,000 token raw multi-file context dumps.
  - Implemented `AnalyticsEngine::get_session_detail(session_id)`: provides step-by-step transcript timeline with user requests, agent thinking/reasoning blocks, tool arguments, and Omni-Graph call highlights.
- **Exposed REST API Endpoints in Axum (`src/api/mod.rs`)**:
  - Registered `GET /api/analytics` and `GET /api/analytics/session/:id`.
  - Added unit test `test_analytics_scan_and_detail_with_synthetic_session` in `tests/unit_tests.rs`.
  - Verified 49/49 unit and integration tests passing (100% pass rate).
  - Hot-swapped release binary into `omni-rust-app` (`docker cp` + `docker restart omni-rust-app` without restarting SurrealDB or TEI).
- **Built Scientist/Researcher-Grade UI Dashboard (`ui/src/AgentAnalytics.tsx`)**:
  - Designed interactive UI matching Cosmograph dark aesthetic with rich glassmorphism.
  - Added Persona Selector:
    - `Lead AI Architect`: Focus on AST condensation efficiency, Rule 08 compliance, token reduction, and macro subsystem clustering.
    - `Systems Researcher`: Focus on code traversal patterns, language telemetry, LSP symbol lookups vs file reads, and prompt reasoning progression.
    - `Token & Cost Auditor`: Focus on context window consumption, file dumps vs targeted AST queries, error rates, and cost prevention.
  - Hero Metric Ribbon: 8 scientist-grade cards (Active Trajectories, Steps, Tool Invocations, Tokens Saved, Omni Calls, Monitored Workspaces, LSP Syntax Engines).
  - Efficiency Visualizer: Interactive bar comparing 2-hop topological call graphs (<1,500 tokens) vs full-context file dumps (~15,000 tokens).
  - Multi-Faceted Filters: Live keyword search, workspace filter chips (All, DSA, GoLang, tool-scripts, AWS, GenAI, etc.), language filters, tool category filters (All, Omni-Graph AST, Filesystem, Terminal), and "Has Omni-Graph Calls" quick toggle.
  - Deep Session Traversal Inspector Modal: Allows researchers to inspect individual session timelines, agent thought processes, tool arguments, and Omni-Graph AST lookups.
- **Top Header Integration (`ui/src/App.tsx`)**:
  - Added mode toggle in top navigation: `[ 🌌 3D Graph Studio ]` vs `[ 📊 Agent Analytics & LSP ]`.
  - Built production bundle with `tsc && vite build` and hot-copied to `omni-graph-ui:/usr/share/nginx/html/`.
- **Automated Verification**:
  - Headless Chrome testing verified metric rendering, persona switching, and session inspector modal.
  - Captured verified screenshots: `agent_analytics_dashboard.png`, `session_traversal_inspector.png`, `agent_analytics_sessions_stream.png`.

#### Files Changed
- `tools/omni-graph/src/analytics/mod.rs` — Created AnalyticsEngine with transcript parser and token metrics
- `tools/omni-graph/src/lib.rs` — Exported `pub mod analytics;`
- `tools/omni-graph/src/main.rs` — Declared `mod analytics;`
- `tools/omni-graph/src/api/mod.rs` — Added `/api/analytics` and `/api/analytics/session/:id` routes
- `tools/omni-graph/tests/unit_tests.rs` — Added synthetic session analytics unit tests
- `tools/omni-graph/ui/src/AgentAnalytics.tsx` — Built scientist-grade analytics dashboard
- `tools/omni-graph/ui/src/App.tsx` — Added mode switcher and view toggle
- `tools/omni-graph/CONTEXT.md` — Updated SDLC status, test count, and telemetry capabilities
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-25 — Session Traversal Timeline Filtering, Canonical Workspaces & Configurable Dynamic Sync

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~35m

#### What Was Done
- **Enhanced Traversal Timeline Inspector Modal (Session Explorer Equivalent)**:
  - Added multi-criteria filtering and searching inside the "Inspect Traversal Timeline" modal:
    - Real-time search query input matching user requests, agent thinking/thoughts, tool names, and tool arguments.
    - Chronological sort toggle (`Oldest (#0 ➔ #N)` vs `Newest (#N ➔ #0)`), verified on 10,900+ step trajectory.
    - Step category badges with live counts (`ALL`, `PROMPTS`, `THOUGHTS`, `TOOLS`, `OMNI AST`, `ERRORS`).
    - Dynamic tool selector dropdown populated from tools specifically invoked in the active session.
    - Global `[ Expand All Details ]` / `[ Collapse Details ]` toggle.
    - Quick-jump navigation buttons: `[ ⤒ Top ]` and `[ ⤓ Bottom ]`.
    - Failing step indicators (`ERROR` status badges and warning borders).
- **Canonical Workspace & Repository Name Resolution (`src/analytics/mod.rs`)**:
  - Implemented `resolve_canonical_workspace(&str) -> Option<String>` to strictly normalize raw filepaths and relative folders to canonical repositories (`tool-scripts`, `DSA`, `GenAI`, `GoLang`, `k8s-eks`, `tutor-intelligence`, `session-explorer`, `python`, `workspace`).
  - Filtered out internal subdirectories (`ui`, `src`, `001 - Queue and Stack`, `target`, `dist`, etc.) and non-workspace filenames (`task-*.log`, `*.py`, `*.png`).
  - Eliminated non-deterministic `HashSet` iteration in Rust by sorting and prioritizing canonical names, ensuring 100% deterministic workspace breakdowns across refreshes.
- **Configurable Dynamic Sync Interval (1m Default)**:
  - Implemented interval selector in the Analytics HUD with options: `10s`, `30s`, `1m (default)`, `3m`, `5m`, and `Paused (Off)`.
  - Configured 1-minute default (`1m`) to prevent aggressive polling overhead.
  - Added live 1-second countdown ticker displaying time remaining until next sync.
  - Added smooth initial loading skeleton so dashboard never flashes 0 sessions during fetch transitions.
- **Verification**:
  - Hot-swapped Rust binary in `omni-rust-app` and rebuilt UI bundle in `omni-graph-ui`.
  - Verified with headless Chrome: `modal_filtered_timeline.png`, `modal_sorted_timeline.png`, `agent_analytics_final_dashboard.png`.

#### Files Changed
- `tools/omni-graph/src/analytics/mod.rs` — Canonical workspace resolution and deterministic sorting
- `tools/omni-graph/ui/src/AgentAnalytics.tsx` — Traversal modal filtering, dynamic sync interval selector, loading skeleton
- `tools/omni-graph/CONTEXT.md` — Updated state
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-26 — Multi-Target Agent Enforcement Shield (`make setup-agent`)

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~15m

#### What Was Done
- **Universal Multi-Target Agent Shield Bootstrapper (`scripts/setup-agent.sh`)**:
  - Enhanced `setup-agent.sh` to accept:
    1. **Any user-specified folder/codebase** (e.g. `make setup-agent /path/to/codebase` or `DIR=/path/to/codebase`): Resolves absolute paths, installs `.agents/rules/08-omni-graph-enforcement.md`, `.agents/skills/omni-graph/`, `.agents/scripts/` (executable hook scripts), writes `.agents/hooks.json` configured with absolute script paths, and creates/updates `AGENTS.md`.
    2. **Global system-level profile** (`make setup-agent global`): Deploys rules, skills, hook scripts, and `hooks.json` to `~/.gemini/config/` with global-safe paths.
    3. **Current workspace** (`make setup-agent`): Deploys to repo-level `.agents/`.
- **Generalized PreToolUse Guardrail Hook (`hook_pre_tool.sh`)**:
  - Removed hardcoded workspace string checks so broad blanket grep searches are intercepted across any directory/codebase.
- **Unified Makefile Interfaces**:
  - Updated both root `Makefile` and `tools/omni-graph/Makefile` with trailing argument absorption for `setup-agent`.
  - Documented setup commands in `tools/omni-graph/README.md`.
- **Verification**:
  - Tested on `tool-scripts` workspace: verified `.agents/hooks.json`.
  - Tested on external sibling codebase `/Users/aparv/.../DSA`: verified `.agents/` and `AGENTS.md` creation.
  - Tested on global profile `~/.gemini/config/`: verified rules, skills, scripts, and `hooks.json`.

#### Files Changed
- `tools/omni-graph/scripts/setup-agent.sh` — Multi-target installer supporting arbitrary paths, global, and workspace
- `tools/omni-graph/scripts/hook_pre_tool.sh` — Generalized guardrail hook for all workspaces
- `tools/omni-graph/Makefile` — Trailing argument absorption and target parameters
- `Makefile` — Added `setup-agent` target with trailing argument absorption
- `tools/omni-graph/README.md` — Documented `make setup-agent` usage
- `tools/omni-graph/DEVLOG.md` — This entry

---

### 2026-09-30 — Node Inspector Breadcrumbs, Caller History Stack & One-Click Back Navigation

**Agent/Author**: Antigravity
**SDLC Phase**: `in-progress`
**Duration**: ~20m

#### What Was Done
- **Navigation History Stack & Breadcrumbs (`tools/omni-graph/ui/src/App.tsx`)**:
  - Implemented `nodeHistory: GraphNode[]` state stack tracking the traversal path when exploring symbols and callers.
  - Added dedicated navigation bar at the top of the Node Inspector (Right Drawer):
    - `← Back to [Previous Symbol]` one-click return button.
    - `Origin` shortcut to return immediately to the root symbol of the exploration session.
    - Interactive breadcrumb trail (`Symbol A › Symbol B › Current Symbol`) allowing agents and users to jump directly to any ancestor node.
  - Reset navigation history cleanly when selecting a new root node via Macro Topology, Flat Explorer, search, or workspace change.
- **In-Card Contextual Return Affordance (`tools/omni-graph/ui/src/BoundaryContractCard.tsx`)**:
  - Extended `BoundaryContractCardProps` with `previousSymbol` and `onNavigateBack`.
  - Added an in-card "Return to caller: `[previousSymbol]`" button with an `ArrowLeft` icon placed right below the containment status badge.
  - Provides dual affordance: users can navigate back from the top drawer header or directly within the boundary contract card.
- **Robust Cross-Boundary Node Resolution**:
  - Enhanced `onSelectCaller` and `onSelectSymbol` with fallback resolvers so navigating to foreign callers across workspaces/subsystems always synthesizes valid `GraphNode` objects and opens the Node Inspector reliably.
- **Verification**:
  - Rebuilt production UI bundle with Vite (`npm run build` in 3.21s).
  - Recreated Docker container `omni-graph-ui` via `docker compose build graph-ui && docker compose up -d graph-ui`.
  - Tested navigation flow in Chrome: verified origin symbol (`detect`), drilldown to caller (`detect_compact_ids_start_at_zero`), breadcrumb display, and one-click return to origin.

#### Files Changed
- `tools/omni-graph/ui/src/App.tsx` — Navigation history stack, breadcrumb trail, top-level back button, fallback symbol resolution
- `tools/omni-graph/ui/src/BoundaryContractCard.tsx` — `previousSymbol` and `onNavigateBack` props, in-card return button
- `tools/omni-graph/DEVLOG.md` — This entry

---
