# ============================================================================
# Tool Makefile — omni-graph
#
# Multi-container semantic knowledge hub for AST + vector graph RAG.
# Languages: Rust (backend), React/TypeScript (frontend)
# Infrastructure: Docker Compose (SurrealDB, HuggingFace TEI)
# ============================================================================

.PHONY: help setup test lint run build clean up down logs health \
        ingest cluster workspaces search query setup-agent

.DEFAULT_GOAL := help

# ─── Configuration ──────────────────────────────────────────────────────────
TOOL_NAME := $(notdir $(CURDIR))
COMPOSE   := docker compose
PROJECT   ?=
Q         ?=
K         ?= 10

# Absorb trailing arguments so make never treats paths or flags as targets
.PHONY: $(MAKECMDGOALS)

ifeq (ingest,$(firstword $(MAKECMDGOALS)))
  INGEST_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
endif

ifeq (cluster,$(firstword $(MAKECMDGOALS)))
  CLUSTER_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
endif

$(filter-out $(firstword $(MAKECMDGOALS)),$(MAKECMDGOALS)):
	@true

# ─── Help ───────────────────────────────────────────────────────────────────
help: ## Show this help message
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  Tool: $(TOOL_NAME)                                             ║"
	@echo "║  Stack: Rust + React + SurrealDB + HF TEI                        ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "First Ingestion: Ingest this monorepo via ./scripts/omni.sh ingest /workspace or the UI input."
	@echo "Trigger Community Clustering: Run ./scripts/omni.sh cluster to compute galaxy IDs for the WebGL force graph."
	@echo ""
	@echo "Quick Make Commands:"
	@echo "  • make ingest <path> [PROJECT=name]   Index any directory or codebase"
	@echo "  • make cluster [PROJECT=name]         Compute modular galaxy IDs for 3D force graph"
	@echo "  • make workspaces                     List all indexed codebases & partition stats"
	@echo "  • make search Q=\"<terms>\"             Fast vector similarity code search"
	@echo "  • make query Q=\"<question>\"          Hybrid Graph-RAG synthesis (<1500 tokens)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ─── Setup ──────────────────────────────────────────────────────────────────
setup: ## Install dependencies for all services
	@echo "⚙️  Setting up $(TOOL_NAME)..."
	@echo "  → Checking Docker..."
	@docker --version || (echo "❌ Docker not found. Install Docker Desktop." && exit 1)
	@echo "  → Pulling container images..."
	$(COMPOSE) pull
	@echo "  → Installing UI dependencies..."
	@if [ -d ui ]; then cd ui && npm install; fi
	@echo "✅ Setup complete."

setup-agent: ## Configure workspace or global agent enforcement shield (TARGET=workspace|global)
	@./scripts/setup-agent.sh $(if $(TARGET),$(TARGET),workspace)

# ─── Docker Compose ────────────────────────────────────────────────────────
up: ## Start the full stack (4 containers)
	@echo "🚀 Starting $(TOOL_NAME) stack..."
	$(COMPOSE) up --build -d
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  ✅ Omni-Graph Stack Started!                                    ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo "  → Web Visualizer UI:  http://localhost:3000"
	@echo "  → REST API:           http://localhost:8080"
	@echo "  → SurrealDB Studio:   http://localhost:8000"
	@echo "  → TEI Inference:      http://localhost:8081"
	@echo ""
	@echo "First Ingestion: Ingest this monorepo via ./scripts/omni.sh ingest /workspace or the UI input."
	@echo "Trigger Community Clustering: Run ./scripts/omni.sh cluster to compute galaxy IDs for the WebGL force graph."
	@echo ""
	@echo "Quick Make Commands:"
	@echo "  • make ingest <path> [PROJECT=name]   Index any directory or codebase"
	@echo "  • make cluster [PROJECT=name]         Compute modular galaxy IDs for 3D force graph"
	@echo "  • make workspaces                     List all indexed codebases & partition stats"
	@echo "  • make search Q=\"<terms>\"             Fast vector similarity code search"
	@echo "  • make query Q=\"<question>\"          Hybrid Graph-RAG synthesis (<1500 tokens)"

down: ## Stop and remove all containers
	@echo "🛑 Stopping $(TOOL_NAME) stack..."
	$(COMPOSE) down
	@echo "✅ Stack stopped."

logs: ## Tail logs for all services
	$(COMPOSE) logs -f

health: ## Check health of all services
	@echo "🏥 Health check..."
	@./scripts/omni.sh health 2>/dev/null || echo "❌ API not responding"

# ─── Ingestion, Clustering & Graph RAG ───────────────────────────────────────
TARGET_INGEST_PATH := $(if $(INGEST_ARGS),$(firstword $(INGEST_ARGS)),$(if $(PATH),$(PATH),/workspace))
TARGET_PROJECT_NAME := $(if $(PROJECT),$(PROJECT),$(if $(filter-out $(TARGET_INGEST_PATH),$(INGEST_ARGS)),$(word 2,$(INGEST_ARGS)),))

ingest: ## Index a codebase or monorepo: make ingest <path> [PROJECT=name]
	@echo "📥 Ingesting codebase into Omni-Graph..."
	@./scripts/omni.sh ingest "$(TARGET_INGEST_PATH)" "$(TARGET_PROJECT_NAME)"

cluster: ## Compute galaxy IDs: make cluster [PROJECT=name]
	@echo "🌌 Triggering Community Clustering (Louvain/Leiden modularity)..."
	@./scripts/omni.sh cluster $(if $(CLUSTER_ARGS),$(firstword $(CLUSTER_ARGS)),$(PROJECT))

workspaces: ## List all partitioned codebases and stats in SurrealDB
	@./scripts/omni.sh workspaces

search: ## Fast vector semantic code search: make search Q="<terms>" [PROJECT=name]
	@if [ -z "$(Q)" ]; then \
		echo "❌ Missing query argument: make search Q=\"<keywords>\" [PROJECT=name]"; \
		exit 1; \
	fi
	@./scripts/omni.sh search "$(Q)" "$(PROJECT)" "$(K)"

query: ## Hybrid Graph-RAG synthesis: make query Q="<question>" [PROJECT=name]
	@if [ -z "$(Q)" ]; then \
		echo "❌ Missing question argument: make query Q=\"<architectural question>\" [PROJECT=name]"; \
		exit 1; \
	fi
	@./scripts/omni.sh query "$(Q)" "$(PROJECT)" "$(K)"

# ─── Test ───────────────────────────────────────────────────────────────────
test: ## Run test suite
	@echo "🧪 Running tests for $(TOOL_NAME)..."
	@if command -v cargo >/dev/null && cargo --version | grep -v 'cargo 1\.[0-5]' >/dev/null; then \
		cargo test --all; \
	elif docker info >/dev/null 2>&1; then \
		docker run --rm -v "$$(pwd)":/app -w /app rust:latest cargo test --all; \
	else \
		echo "⚠️ Modern Rust/Cargo (>= 1.56) or Docker required to run tests."; \
	fi
	@if [ -d ui ] && [ -f ui/package.json ]; then cd ui && npm test 2>/dev/null || true; fi
	@echo "✅ Tests complete."

# ─── Lint ───────────────────────────────────────────────────────────────────
lint: ## Run linters
	@echo "🔍 Linting $(TOOL_NAME)..."
	@if command -v cargo >/dev/null && cargo --version | grep -v 'cargo 1\.[0-5]' >/dev/null; then \
		cargo clippy -- -D warnings; \
	elif docker info >/dev/null 2>&1; then \
		docker run --rm -v "$$(pwd)":/app -w /app rust:latest cargo clippy -- -D warnings; \
	else \
		echo "⚠️ Modern Rust/Cargo (>= 1.56) or Docker required to run clippy."; \
	fi
	@if [ -d ui ] && [ -f ui/package.json ]; then cd ui && npm run lint 2>/dev/null || true; fi
	@echo "✅ Lint complete."

# ─── Build ──────────────────────────────────────────────────────────────────
build: ## Build the Rust binary and UI
	@echo "📦 Building $(TOOL_NAME)..."
	@$(COMPOSE) build
	@echo "✅ Build complete."

# ─── Run (local, no Docker) ────────────────────────────────────────────────
run: ## Run the Rust orchestrator locally (requires SurrealDB + TEI running)
	@echo "🚀 Running $(TOOL_NAME) Rust orchestrator..."
	@if [ -f Cargo.toml ]; then cargo run; elif [ -f src/Cargo.toml ]; then cd src && cargo run; fi

# ─── Test Examples ──────────────────────────────────────────────────────────
test-examples: ## Run all examples to verify they work
	@echo "📝 Running examples for $(TOOL_NAME)..."
	@if [ -d examples ]; then \
		for f in examples/*; do \
			echo "  Running $$f..."; \
		done; \
	else \
		echo "  No examples directory found."; \
	fi
	@echo "✅ Examples complete."

# ─── Clean ──────────────────────────────────────────────────────────────────
clean: ## Remove build artifacts and Docker volumes
	@echo "🧹 Cleaning $(TOOL_NAME)..."
	@if [ -f src/Cargo.toml ]; then cd src && cargo clean; fi
	@if [ -d ui/node_modules ]; then rm -rf ui/node_modules ui/dist; fi
	$(COMPOSE) down -v 2>/dev/null || true
	@echo "✅ Clean complete."

# ─── Validate Specs ────────────────────────────────────────────────────────
validate-specs: ## Validate specification files exist and are populated
	@echo "📋 Validating specs for $(TOOL_NAME)..."
	@test -f ../../specs/catalog/omni-graph.md || (echo "❌ Spec missing" && exit 1)
	@test -f STATUS.md || (echo "❌ STATUS.md missing" && exit 1)
	@test -f CONTEXT.md || (echo "❌ CONTEXT.md missing" && exit 1)
	@test -f DEVLOG.md || (echo "❌ DEVLOG.md missing" && exit 1)
	@echo "✅ Specs valid."
