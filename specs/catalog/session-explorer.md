# Spec: Session Explorer

> **Status**: Review
> **Author**: @vadde
> **Created**: 2026-09-18
> **Last Updated**: 2026-09-18
> **Tool Path**: `tools/session-explorer/`

---

## 1. Overview

### 1.1 Problem Statement

Engineers using the Antigravity IDE accumulate rich conversational histories with AI agents — prompts, responses, tool invocations, reasoning traces, and artifacts — across multiple workspaces and projects. This data is invaluable for:

- **Knowledge retrieval**: "What did I ask the agent about Kubernetes last week?"
- **Prompt reuse**: "Find the prompt I used to scaffold that Python project"
- **Audit and reflection**: "What tools did the agent use for debugging?"
- **Learning**: "Show me all GenAI conversations to study agent reasoning patterns"

However, this data is buried in raw JSONL transcript files scattered across `~/.gemini/antigravity-ide/brain/<conversation-id>/` directories with no UI, no search, and no organization. Users of all technical levels — from junior engineers to principal architects to executives — need a fast, beautiful, and intuitive way to explore, search, and filter their agent session history.

### 1.2 Proposed Solution

**Session Explorer** is a self-contained CLI tool (single Go binary) that:

1. Scans the Antigravity IDE's `brain/` directory to discover all conversation sessions
2. Parses JSONL transcript files to extract user prompts, agent responses, tool calls, timestamps, and workspace context
3. Serves a stunning, premium web UI (embedded in the binary) on a local port
4. Automatically opens the user's default browser to the dashboard
5. Provides rich filtering, search, and visualization of session data

The tool delivers a **glassmorphic, liquid-glass, acrylic-inspired** UI that feels premium and modern — not a basic CRUD interface, but a polished product worthy of executive demos.

### 1.3 Target Audience

| Persona | Needs |
|---------|-------|
| **Junior Engineer** | Search past prompts, learn from agent responses, reuse effective prompts |
| **Senior Engineer** | Audit agent tool usage, compare approaches across sessions, filter by workspace |
| **Tech Lead / Architect** | Review team prompts (shared machines), analyze agent reasoning quality |
| **Engineering Manager** | Understand tool adoption, session frequency, conversation complexity |
| **Executive** | High-level dashboard of AI-assisted development activity |

### 1.4 Success Criteria

- Single command (`session-explorer`) launches a browser with all sessions visible
- Users can find any past prompt within 10 seconds using search/filter
- UI receives "wow" reactions — glassmorphism, smooth animations, responsive layout
- Works on macOS, Linux, and Windows without dependencies
- Handles 1000+ sessions and 100K+ transcript lines without lag

---

## 2. Requirements

### 2.1 Functional Requirements

| ID | Requirement | Priority | Impl | Tested | Notes |
|----|------------|----------|------|--------|-------|
| R-001 | Discover and list all conversation sessions from `~/.gemini/antigravity-ide/brain/` | Must | ✅ | 🧪 | scanner.go ScanAll(), scanner_test.go |
| R-002 | Parse JSONL transcript files (`transcript.jsonl` and `transcript_full.jsonl`) to extract structured data | Must | ✅ | 🧪 | scanner.go scanSession(), scanner_test.go |
| R-003 | Extract user prompts (type: `USER_INPUT`, source: `USER_EXPLICIT`) with timestamps | Must | ✅ | 🧪 | scanner.go extractUserPrompt(), scanner_test.go |
| R-004 | Extract agent responses (type: `PLANNER_RESPONSE`) with content and tool calls | Must | ✅ | 🧪 | scanner.go scanSession(), scanner_test.go |
| R-005 | Extract workspace/project context from `ADDITIONAL_METADATA` in user messages | Must | ✅ | 🧪 | scanner.go extractWorkspace(), scanner_test.go |
| R-006 | Serve an embedded web UI on a local HTTP port (default: 9876) | Must | ✅ | 🧪 | server.go StartServer(), verified with embedded assets |
| R-007 | Auto-open the user's default browser on launch | Must | ✅ | 🧪 | main.go openBrowser() |
| R-008 | Dashboard view: show all sessions as cards with metadata (date, workspace, step count, size) | Must | ✅ | 🧪 | src/web/src/app.js renderSessionGrid() |
| R-009 | Session detail view: show full conversation timeline (user ↔ agent) with rendered content | Must | ✅ | 🧪 | src/web/src/app.js renderTimelineMessages() |
| R-010 | Filter sessions by date range (from/to date picker) | Must | ✅ | 🧪 | scanner.go GetSessions(), scanner_test.go |
| R-011 | Filter sessions by workspace/project | Must | ✅ | 🧪 | scanner.go GetSessions(), api_test.go |
| R-012 | Full-text search across all user prompts | Must | ✅ | 🧪 | scanner.go Search(), scanner_test.go |
| R-013 | Filter by message type (user prompts, agent responses, tool calls, errors) | Should | ✅ | 🧪 | src/web/src/app.js, api.go |
| R-014 | Sort sessions by date (newest/oldest), size, or step count | Should | ✅ | 🧪 | scanner.go GetSessions(), app.js |
| R-015 | Show tool call details (name, arguments) in a collapsible section | Should | ✅ | 🧪 | src/web/src/app.js details accordion |
| R-016 | Render markdown content in agent responses beautifully | Should | ✅ | 🧪 | marked.js + highlight.js integration |
| R-017 | Show session statistics: total sessions, total prompts, most active workspace, date distribution | Should | ✅ | 🧪 | scanner.go computeStats(), scanner_test.go |
| R-018 | Glassmorphic/acrylic/liquid-glass UI design with smooth animations | Must | ✅ | 🧪 | src/web/src/styles.css |
| R-019 | Responsive design: work on desktop, tablet, and mobile viewports | Should | ✅ | 🧪 | src/web/src/styles.css media queries |
| R-020 | Dark mode by default with optional light mode toggle | Should | ✅ | 🧪 | src/web/src/styles.css, app.js theme toggle |
| R-021 | Export a session's conversation to markdown or JSON | Could | ✅ | 🧪 | src/web/src/app.js exportCurrentSession() |
| R-022 | Keyboard shortcuts for navigation (Ctrl+K for search, Esc to close) | Could | ✅ | 🧪 | src/web/src/app.js keydown listener |
| R-023 | Configurable data directory path via CLI flag (`--data-dir`) | Must | ✅ | 🧪 | main.go flags, tested in demo.sh |
| R-024 | Configurable port via CLI flag (`--port`) | Must | ✅ | 🧪 | main.go flags, tested in demo.sh |
| R-025 | Show "most frequently used prompts" or prompt frequency analysis | Could | ✅ | 🧪 | scanner.go computeStats(), app.js Insights modal |
| R-026 | Copy prompt text to clipboard with one click | Should | ✅ | 🧪 | src/web/src/app.js copy buttons + toast |
| R-027 | Lazy loading / virtual scrolling for large conversation timelines | Should | ✅ | 🧪 | src/web/src/app.js DOM rendering |
| R-028 | Project and repository clustering: dynamically identify repos from tool paths and prompts | Must | ✅ | 🧪 | scanner.go discoverKnownProjects(), scanSession(), scanner_test.go |
| R-029 | Flexible user-driven grouping modes: Project Hub, Date Timeline, or Flat List | Must | ✅ | 🧪 | src/web/src/app.js renderView(), index.html segmented control |

**Priority levels**: Must (required for MVP), Should (important), Could (nice-to-have)

**Status legend**: ⬜ Not started · 🔨 In progress · ✅ Implemented · 🧪 Tested & verified · ❌ Blocked

### 2.2 Non-Functional Requirements

| ID | Requirement | Metric | Impl | Tested |
|----|------------|--------|------|--------|
| NF-001 | Performance — Initial scan | Index 1000 sessions in < 3 seconds | ✅ | 🧪 |
| NF-002 | Performance — Search | Full-text search returns results in < 200ms | ✅ | 🧪 |
| NF-003 | Performance — UI render | Dashboard loads in < 1 second after server start | ✅ | 🧪 |
| NF-004 | Portability | Single binary for macOS (arm64/amd64), Linux (amd64/arm64), Windows (amd64) | ✅ | 🧪 |
| NF-005 | Zero dependencies | Users need no runtime (no Node, no Python, no Docker) | ✅ | 🧪 |
| NF-006 | Memory | Handle 100K transcript lines with < 512MB RAM | ✅ | 🧪 |
| NF-007 | Reliability | Gracefully handle malformed JSONL, missing files, permission errors | ✅ | 🧪 |
| NF-008 | Security | Bind only to localhost (127.0.0.1), no external network exposure | ✅ | 🧪 |
| NF-009 | Startup time | Server ready in < 500ms before data scan completes | ✅ | 🧪 |

---

## 3. Interface Contract

### 3.1 CLI Interface

```
Usage: session-explorer [OPTIONS]

Options:
  -d, --data-dir <PATH>    Path to Antigravity brain directory
                            [default: ~/.gemini/antigravity-ide/brain]
  -p, --port <PORT>        Port to serve the web UI on
                            [default: 9876]
  --no-open                Don't auto-open browser
  -v, --verbose            Enable verbose logging
  -h, --help               Show help message
  --version                Show version
```

### 3.2 REST API (Internal, served by the Go backend)

#### `GET /api/sessions`

Returns list of all sessions with metadata.

**Response:**
```json
{
  "sessions": [
    {
      "id": "952f3203-4fe9-4e6c-9edb-a93984b1f3f0",
      "created_at": "2026-09-18T15:47:39Z",
      "last_message_at": "2026-09-18T16:31:30Z",
      "workspace": "/path/to/workspace",
      "project_name": "tool-scripts",
      "step_count": 169,
      "user_message_count": 6,
      "agent_response_count": 50,
      "tool_call_count": 120,
      "transcript_size_bytes": 524000,
      "first_user_prompt_preview": "I want to build a tool..."
    }
  ],
  "stats": {
    "total_sessions": 12,
    "total_user_messages": 150,
    "total_agent_responses": 400,
    "date_range": { "from": "2026-06-17", "to": "2026-09-18" },
    "workspaces": ["tool-scripts", "vadde.github.io", "DSA"]
  }
}
```

#### `GET /api/sessions/:id`

Returns full conversation timeline for a session.

**Response:**
```json
{
  "session": { "id": "...", "created_at": "...", "workspace": "..." },
  "messages": [
    {
      "step_index": 0,
      "source": "USER_EXPLICIT",
      "type": "USER_INPUT",
      "status": "DONE",
      "created_at": "2026-09-18T15:47:39Z",
      "content": "I want to build a tool...",
      "has_thinking": false,
      "tool_calls": null
    },
    {
      "step_index": 3,
      "source": "MODEL",
      "type": "PLANNER_RESPONSE",
      "status": "DONE",
      "created_at": "2026-09-18T15:48:00Z",
      "content": "Let me analyze...",
      "has_thinking": true,
      "tool_calls": [
        { "name": "list_dir", "args_preview": "{\"DirectoryPath\": \"/path...\"}" }
      ]
    }
  ]
}
```

#### `GET /api/search?q=<query>&workspace=<ws>&from=<date>&to=<date>&type=<type>`

Full-text search across all sessions.

**Response:**
```json
{
  "results": [
    {
      "session_id": "...",
      "step_index": 0,
      "type": "USER_INPUT",
      "content_preview": "...matching text...",
      "created_at": "2026-09-18T15:47:39Z",
      "workspace": "tool-scripts",
      "highlight_ranges": [[10, 25]]
    }
  ],
  "total_count": 42,
  "query": "kubernetes deployment"
}
```

### 3.3 Data Source Specification

| Field | Type | Source |
|-------|------|--------|
| Session ID | UUID string | Directory name under `brain/` |
| Transcript | JSONL file | `brain/<id>/.system_generated/logs/transcript.jsonl` |
| Full Transcript | JSONL file | `brain/<id>/.system_generated/logs/transcript_full.jsonl` |
| Workspace | string (path) | Extracted from `ADDITIONAL_METADATA` in `USER_INPUT` entries |

**JSONL Entry Schema (per line):**

| Field | Type | Description |
|-------|------|-------------|
| `step_index` | int | Sequential index of the step |
| `source` | string | `USER_EXPLICIT`, `MODEL`, `SYSTEM` |
| `type` | string | `USER_INPUT`, `PLANNER_RESPONSE`, `VIEW_FILE`, `RUN_COMMAND`, `GREP_SEARCH`, `LIST_DIRECTORY`, `CODE_ACTION`, `SEARCH_WEB`, `READ_URL_CONTENT`, `BROWSER_SUBAGENT`, `ASK_QUESTION`, `CONVERSATION_HISTORY`, `KNOWLEDGE_ARTIFACTS`, `CHECKPOINT`, `EPHEMERAL_MESSAGE`, `ERROR_MESSAGE`, `GENERIC`, `SYSTEM_MESSAGE` |
| `status` | string | `DONE`, `ERROR` |
| `created_at` | ISO 8601 | Timestamp |
| `content` | string | Text content (may contain XML tags for USER_INPUT) |
| `thinking` | string | Agent's reasoning (PLANNER_RESPONSE only) |
| `tool_calls` | array | Tool invocations (PLANNER_RESPONSE only) |
| `is_truncated` | boolean | Whether content was truncated in compact transcript |

### 3.4 Error Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Data directory not found or not readable |
| 4 | Port already in use |

---

## 4. Constraints

### 4.1 Technical Constraints

- Must compile to a single static binary via Go (v1.22+)
- Frontend assets must be embedded via Go's `embed` package — no external file dependencies
- Must use only the Go standard library + minimal well-maintained dependencies
- HTML/CSS/JS frontend — vanilla JavaScript (no React/Vue/Angular build step)

### 4.2 Security Constraints

- HTTP server MUST bind to `127.0.0.1` only (localhost), never `0.0.0.0`
- Must not write to or modify any transcript files (read-only access)
- Must not transmit any data over the network
- Must sanitize rendered content to prevent XSS

### 4.3 Performance Constraints

- Initial page load: < 1 second
- Session scan: < 3 seconds for 1000 sessions
- Search latency: < 200ms for full-text search across all transcripts
- Memory: < 512MB for 100K transcript lines
- Binary size: < 20MB

---

## 5. Dependencies

### 5.1 External Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| Go standard library | 1.22+ | HTTP server, JSON parsing, embed, filepath, os |
| `github.com/pkg/browser` | latest | Cross-platform browser opening (optional, can use `exec.Command`) |

### 5.2 Internal Dependencies

| Tool | Purpose |
|------|---------|
| None | This is a standalone tool |

---

## 6. Acceptance Criteria

Tests that MUST pass for this spec to be considered satisfied:

| AC ID | Criteria | Traces To |
|-------|----------|-----------|
| AC-001 | Given a valid brain directory, when session-explorer runs, then all sessions are discovered and listed | R-001 |
| AC-002 | Given transcript.jsonl files, when parsed, then user messages and agent responses are correctly extracted with timestamps | R-002, R-003, R-004 |
| AC-003 | Given USER_INPUT entries with ADDITIONAL_METADATA, when parsed, then workspace/project paths are extracted | R-005 |
| AC-004 | Given the server starts, then it binds to localhost on the configured port and serves the web UI | R-006, R-008, NF-008 |
| AC-005 | Given the `--port` flag, when specified, then the server uses the custom port | R-024 |
| AC-006 | Given the `--data-dir` flag, when specified, then the custom path is used for session discovery | R-023 |
| AC-007 | Given the dashboard loads, then all sessions are displayed as cards with date, workspace, step count | R-008 |
| AC-008 | Given a session card is clicked, then the full conversation timeline is rendered with user/agent messages | R-009 |
| AC-009 | Given a date range filter is applied, then only sessions within that range are shown | R-010 |
| AC-010 | Given a workspace filter is selected, then only sessions from that workspace are shown | R-011 |
| AC-011 | Given a search query, then matching prompts across all sessions are returned with highlights | R-012 |
| AC-012 | Given the UI renders, then glassmorphic/acrylic styling is applied with smooth animations | R-018 |
| AC-013 | Given malformed JSONL lines, when parsing, then they are skipped gracefully without crashing | NF-007 |
| AC-014 | Given the server starts, then the default browser opens automatically (unless `--no-open`) | R-007 |
| AC-015 | Given the UI renders on mobile viewport, then layout adapts responsively | R-019 |

---

## 7. Design Notes

### 7.1 Architecture

```
┌──────────────────────────────────────────────────────┐
│                    session-explorer                    │
│                                                        │
│  ┌──────────────┐    ┌──────────────┐                │
│  │   CLI Layer   │───▶│  HTTP Server │                │
│  │  (flags/args) │    │ (net/http)   │                │
│  └──────────────┘    └──────┬───────┘                │
│                             │                          │
│           ┌─────────────────┼─────────────────┐       │
│           │                 │                 │       │
│    ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼────┐  │
│    │  REST API   │  │  Static     │  │ WebSocket  │  │
│    │  /api/*     │  │  Assets     │  │ (optional) │  │
│    └──────┬──────┘  │  (embed)    │  └────────────┘  │
│           │         └─────────────┘                   │
│    ┌──────▼──────────────────────┐                    │
│    │     Session Index Engine    │                    │
│    │  ┌────────────────────────┐ │                    │
│    │  │ JSONL Parser (scanner) │ │                    │
│    │  │ In-Memory Search Index │ │                    │
│    │  │ Workspace Extractor    │ │                    │
│    │  └────────────────────────┘ │                    │
│    └──────┬──────────────────────┘                    │
│           │                                            │
│    ┌──────▼──────────────────────┐                    │
│    │   ~/.gemini/antigravity-ide │                    │
│    │   /brain/<id>/transcript.*  │                    │
│    └─────────────────────────────┘                    │
│                                                        │
└──────────────────────────────────────────────────────┘
```

### 7.2 UI Design Philosophy

- **Dark mode default** with deep navy/slate backgrounds (#0a0f1e, #111827)
- **Glassmorphism**: Semi-transparent cards with `backdrop-filter: blur()`, subtle borders
- **Liquid glass**: Gradient overlays with animated mesh backgrounds
- **Acrylic/Mica**: Frosted-glass effects on panels, modals, and navigation
- **Micro-animations**: Hover effects, card transitions, search result fade-ins
- **Typography**: Google Fonts (Inter or Outfit) for clean, modern readability
- **Color palette**: Indigo (#6366f1) → Violet (#8b5cf6) → Cyan (#06b6d4) gradient accents
- **Conversation bubbles**: User messages (right-aligned, accent color), Agent messages (left-aligned, glass)

### 7.3 Frontend Rendering Strategy

- Parse markdown in agent responses client-side using a lightweight renderer (e.g., `marked.js` via CDN or embedded)
- Syntax highlighting for code blocks (e.g., `highlight.js` or `prism.js` embedded)
- Virtual scrolling for large conversations (custom implementation or intersection observer)
- Responsive grid layout with CSS Grid/Flexbox

---

## 8. Open Questions

- [x] Tech stack: Go backend + vanilla JS frontend (decided)
- [ ] Should we support VSCode/Cursor/Claude Code transcript formats in a future version?
- [ ] Should we persist a search index to disk for faster subsequent launches?
- [ ] Should the tool support a "watch mode" that updates when new sessions appear?

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-09-18 | @vadde | Initial draft |
| 2026-09-18 | @vadde | Complete specification with all requirements |
