# 🔍 session-explorer

> A high-performance, single-binary session intelligence explorer and browser for Antigravity IDE agent conversation histories.

![Status](https://img.shields.io/badge/status-in--progress-blue)
![Version](https://img.shields.io/badge/version-0.1.0-emerald)
![Language](https://img.shields.io/badge/language-Go%201.22%2B%20%7C%20Bun-indigo)
![License](https://img.shields.io/badge/license-MIT-purple)

---

## Overview

**Session Explorer** transforms raw Antigravity IDE session logs (`~/.gemini/antigravity-ide/brain/`) into an interactive, visual conversation library. It runs as a self-contained local web server that automatically scans your local agent transcripts, indexes user prompts, extracts workspaces and tool calls, and serves a modern, liquid-glass web interface.

### Key Capabilities

- ⚡ **Blazing Fast Scanning**: Concurrently parses tens of thousands of JSONL transcript lines in under 200 milliseconds.
- 📦 **Single Standalone Binary**: Go embeds the entire web application, marked.js markdown engine, and syntax highlighter. Zero runtime dependencies.
- 🎨 **Liquid Glass Aesthetics**: Modern dark/light frosted glass UI with smooth micro-animations and responsive layouts.
- 🔎 **Instant Full-Text Search**: Global search modal (`Cmd+K` / `Ctrl+K`) with live matching across all prompts.
- 🧭 **Workspace Auto-Detection**: Automatically identifies project workspaces from active documents and tool call arguments.
- 🛠️ **Tool Call Inspector**: Expandable inspection cards displaying tool execution parameters and JSON payloads.
- 📊 **Prompt Insights & Analytics**: Analyzes tool invocation distribution and most frequently used user prompts.
- 💾 **Export Anywhere**: One-click export of any conversation session to Markdown (`.md`) or JSON (`.json`).

---

## Installation & Quick Start

### Prerequisites

- **Go 1.22+** (to build the Go binary)
- **Bun 1.0+** (to bundle web assets)

### Quick Start (Makefile Commands)

From the **repository root**:

```bash
# Build and launch Session Explorer (auto-opens browser at http://127.0.0.1:9876)
make session-explorer

# Launch with custom arguments (e.g. port or verbose)
make run-tool T=session-explorer ARGS="--port 8080"

# Run tests or automated demo
make test-tool T=session-explorer
make demo-tool T=session-explorer
```

Or from inside `tools/session-explorer/`:

```bash
cd tools/session-explorer

# Interactive command directory
make help

# Build and launch web UI
make run

# Launch on custom port or in headless mode
make run PORT=8080
make run-headless

# Run automated mock dataset demo
make demo
```

---

## Makefile Command Directory

Every lifecycle action is managed through standard `make` targets:

| Command | Description |
|---------|-------------|
| `make run` | Build and launch Session Explorer (opens default browser) |
| `make run-headless` | Launch without opening browser (`--no-open`) |
| `make demo` | Run automated mock dataset demo script |
| `make build` | Complete build: bundle frontend with Bun + compile Go binary |
| `make build-web` | Bundle frontend web assets with Bun |
| `make build-go` | Compile standalone Go binary |
| `make test` | Run full unit test suite |
| `make test-race` | Run test suite with Go race detector enabled |
| `make test-cover` | Run tests with coverage summary |
| `make lint` | Run code linters (`go vet`) |
| `make status` | Display SDLC lifecycle phase and requirements matrix |
| `make spec` | Display specification summary |
| `make context` | Display current session context (`CONTEXT.md`) |
| `make devlog` | Display latest development log entries (`DEVLOG.md`) |
| `make setup` | Install Go and Bun frontend dependencies |
| `make clean` | Remove build artifacts (`bin/`, `src/web/dist/`) |

---

## Web UI Features & Shortcuts

| Feature | Description | Keyboard Shortcut |
|---------|-------------|-------------------|
| **Global Search** | Instant prompt & transcript search across all sessions | `Cmd + K` or `Ctrl + K` or `/` |
| **Close Dialogs** | Close search modal, insights, or return to dashboard | `Esc` |
| **Theme Toggle** | Switch between Liquid Dark and Frosted Light mode | Header button `☀️ / 🌙` |
| **Workspace Filter** | Filter sessions by detected workspace or project | Quick filter pills / Dropdown |
| **Date Range Filter** | Filter sessions by date range with quick presets (`Today`, `7d`, `30d`) | Toolbar date picker |
| **Sort Sessions** | Sort by newest, oldest, step count, prompt count, size | Toolbar sort dropdown |
| **Timeline View** | Step-by-step user prompts, model reasoning, and tool calls | Click any session card |
| **Export Session** | Download session as clean Markdown (`.md`) or structured JSON (`.json`) | Detail header buttons |

---

## REST API Contract

The embedded HTTP server exposes a JSON REST API for integration:

| Endpoint | Method | Parameters | Description |
|----------|--------|------------|-------------|
| `/api/sessions` | `GET` | `workspace`, `sort_by`, `sort_order`, `from`, `to` | List all discovered sessions with summary stats |
| `/api/sessions/{id}` | `GET` | — | Retrieve full conversation messages and tool calls for a session |
| `/api/search` | `GET` | `q`, `workspace`, `type`, `from`, `to` | Full-text search across transcripts |
| `/api/stats` | `GET` | — | Global statistics, workspace list, top tools, and top prompt patterns |

---

## Development & Testing

```bash
# Run the complete test suite
make test

# Run Go linters (go vet)
make lint

# Run the automated demo with mock data
./examples/demo.sh

# Clean build artifacts
make clean
```

---

## Specification Traceability

This tool strictly adheres to Spec-Driven Development (SDD):

- **Specification**: [`specs/catalog/session-explorer.md`](../../specs/catalog/session-explorer.md)
- **Status & Traceability**: [`STATUS.md`](STATUS.md)
- **Session Continuity**: [`CONTEXT.md`](CONTEXT.md)
- **Development Journal**: [`DEVLOG.md`](DEVLOG.md)

---

## License

MIT © [vadde](https://github.com/vadde)
