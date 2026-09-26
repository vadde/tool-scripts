# ============================================================================
#  ╔══════════════════════════════════════════════════════════════════════╗
#  ║                     tool-scripts — Root Command Center               ║
#  ║                                                                      ║
#  ║  The unified command interface for all tools, scripts, and plugins.  ║
#  ║  Run `make help` to see all available targets.                       ║
#  ╚══════════════════════════════════════════════════════════════════════╝
# ============================================================================

.PHONY: help setup new-tool test test-tool lint lint-tool build build-tool \
        run-tool demo-tool validate-specs catalog status clean \
        session-explorer omni-graph ingest cluster workspaces search-graph query-graph \
        graph-symbol graph-references graph-condense graph-galaxies graph-health health

.DEFAULT_GOAL := help

# ─── Configuration ──────────────────────────────────────────────────────────
SHELL       := /bin/bash
TOOLS_DIR   := tools
SCRIPTS_DIR := scripts
SPECS_DIR   := specs
T           ?=
ARGS        ?=
PROJECT     ?=
DIR         ?=
Q           ?=
K           ?= 10

# Absorb trailing arguments so make never treats paths or flags as targets
.PHONY: $(MAKECMDGOALS)

ifeq (ingest,$(firstword $(MAKECMDGOALS)))
  INGEST_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
endif

ifeq (cluster,$(firstword $(MAKECMDGOALS)))
  CLUSTER_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
endif

ifeq (setup-agent,$(firstword $(MAKECMDGOALS)))
  SETUP_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
endif

$(filter-out $(firstword $(MAKECMDGOALS)),$(MAKECMDGOALS)):
	@true

# ─── Colors & Formatting ────────────────────────────────────────────────────
CYAN   := \033[0;36m
GREEN  := \033[0;32m
YELLOW := \033[0;33m
BLUE   := \033[0;34m
PURPLE := \033[0;35m
BOLD   := \033[1m
DIM    := \033[2m
NC     := \033[0m

# ═══════════════════════════════════════════════════════════════════════════
#  Help
# ═══════════════════════════════════════════════════════════════════════════

help: ## Show this interactive command directory
	@printf "\n"
	@printf "$(BOLD)╔══════════════════════════════════════════════════════════════════╗$(NC)\n"
	@printf "$(BOLD)║               🔧 tool-scripts — Command Center                   ║$(NC)\n"
	@printf "$(BOLD)╚══════════════════════════════════════════════════════════════════╝$(NC)\n\n"
	@printf "$(BOLD)🚀 Tool Execution:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "run-tool T=<name>"         "Launch a tool (e.g. make run-tool T=session-explorer)"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "demo-tool T=<name>"        "Run automated demo for a tool (e.g. make demo-tool T=session-explorer)"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "session-explorer"          "Quick launcher: build and run Session Explorer web UI"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "omni-graph"                "Quick launcher: start Omni-Graph semantic knowledge stack"
	@printf "\n"
	@printf "$(BOLD)🧠 Omni-Graph Semantic Hub & Graph RAG:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "ingest <path>"             "Index a codebase or monorepo into SurrealDB (e.g. make ingest /workspace)"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "cluster [project]"         "Compute Louvain/Leiden galaxy community clusters for 3D force graph"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "workspaces"                "List all partitioned codebases and stats in Omni-Graph"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "search-graph Q=\"...\""     "Fast vector semantic code search in knowledge hub"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "query-graph Q=\"...\""      "Hybrid Graph-RAG synthesis (<1500 tokens for agents)"
	@printf "\n"
	@printf "$(BOLD)📦 Build & Compilation:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "build"                     "Build ALL tools across the monorepo"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "build-tool T=<name>"       "Build a specific tool (e.g. make build-tool T=session-explorer)"
	@printf "\n"
	@printf "$(BOLD)🧪 Testing & Quality:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "test"                      "Run test suites for ALL tools"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "test-tool T=<name>"        "Run tests for a specific tool (e.g. make test-tool T=session-explorer)"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "lint"                      "Run linters across all tools"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "lint-tool T=<name>"        "Run linters for a specific tool (e.g. make lint-tool T=session-explorer)"
	@printf "\n"
	@printf "$(BOLD)📋 SDD Specifications & Governance:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "validate-specs"            "Validate all SDD specs, checklists, and status tracking"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "status"                    "Display SDLC status dashboard for all tools"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "catalog"                   "Regenerate tool catalog in tools/README.md"
	@printf "\n"
	@printf "$(BOLD)🔨 Development & Scaffold:$(NC)\n"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "new-tool NAME=<name>"      "Scaffold a new tool from canonical template"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "setup"                     "Install global repo pre-commit hooks and tools"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "setup-agent"               "Deploy Omni-Graph agent shield (TARGET=workspace|global)"
	@printf "  $(CYAN)%-24s$(NC) %s\n" "clean"                     "Clean build artifacts across all tools"
	@printf "\n"
	@printf "  $(DIM)Examples:$(NC)\n"
	@printf "    make omni-graph\n"
	@printf "    make ingest /workspace\n"
	@printf "    make cluster\n"
	@printf "    make search-graph Q=\"CommunityDetector\"\n"
	@printf "    make query-graph Q=\"How does AST parsing work?\"\n\n"

# ═══════════════════════════════════════════════════════════════════════════
#  Tool Execution & Quick Launchers
# ═══════════════════════════════════════════════════════════════════════════

run-tool: ## Run a specific tool (T=tool-name [ARGS="..."])
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make run-tool T=<tool-name> [ARGS=\"...\"]"; \
		echo "   Example: make run-tool T=session-explorer"; \
		exit 1; \
	fi
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		printf "$(GREEN)🚀 Running tool: $(T)...$(NC)\n"; \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" run ARGS="$(ARGS)"; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

demo-tool: ## Run automated demo for a tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make demo-tool T=<tool-name>"; \
		echo "   Example: make demo-tool T=session-explorer"; \
		exit 1; \
	fi
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		printf "$(GREEN)📝 Running demo for: $(T)...$(NC)\n"; \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" demo; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

session-explorer: ## Quick launcher for Session Explorer
	@$(MAKE) run-tool T=session-explorer ARGS="$(ARGS)"

omni-graph: ## Quick launcher: start Omni-Graph semantic knowledge stack
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph up

ingest: ## Index a codebase into Omni-Graph: make ingest [<dir>] [DIR=<dir>] [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph ingest $(if $(INGEST_ARGS),$(INGEST_ARGS),$(if $(DIR),TARGET_DIR="$(DIR)",)) PROJECT="$(PROJECT)"

cluster: ## Compute galaxy IDs: make cluster [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph cluster $(if $(CLUSTER_ARGS),$(CLUSTER_ARGS),) PROJECT="$(PROJECT)"

workspaces: ## List all partitioned codebases in Omni-Graph
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph workspaces

search-graph: ## Search Omni-Graph knowledge hub: make search-graph Q="<terms>" [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph search Q="$(Q)" PROJECT="$(PROJECT)" K="$(K)"

query-graph: ## Hybrid Graph-RAG retrieval: make query-graph Q="<question>" [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph query Q="$(Q)" PROJECT="$(PROJECT)" K="$(K)"

graph-symbol: ## LSP definition lookup: make graph-symbol SYM=<name> [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph symbol SYM="$(SYM)" PROJECT="$(PROJECT)"

graph-references: ## LSP reference callers: make graph-references SYM=<name> [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph references SYM="$(SYM)" PROJECT="$(PROJECT)"

graph-condense: ## Condense AST slice (<1500 tokens): make graph-condense SYM=<name> [HOPS=2] [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph condense SYM="$(SYM)" HOPS="$(HOPS)" PROJECT="$(PROJECT)"

graph-galaxies: ## Inspect architectural subsystems: make graph-galaxies [PROJECT=name]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph galaxies PROJECT="$(PROJECT)"

graph-health health: ## Check health of Omni-Graph services
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph health

setup-agent: ## Configure agent enforcement shield: make setup-agent [<dir>|DIR=<dir>|TARGET=<dir>|global]
	@$(MAKE) -C $(TOOLS_DIR)/omni-graph setup-agent $(if $(SETUP_ARGS),$(SETUP_ARGS),) DIR="$(DIR)" TARGET="$(TARGET)"

# ═══════════════════════════════════════════════════════════════════════════
#  Build & Packaging
# ═══════════════════════════════════════════════════════════════════════════

build: ## Build ALL tools across the monorepo
	@printf "$(BLUE)📦 Building all tools...$(NC)\n"
	@for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		if [ -f "$$tool_dir/Makefile" ] && grep -q '^build:' "$$tool_dir/Makefile"; then \
			echo "  Building: $$tool_name"; \
			$(MAKE) -C "$$tool_dir" build || exit 1; \
		fi; \
	done
	@printf "$(GREEN)✅ All tools built successfully!$(NC)\n"

build-tool: ## Build a specific tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make build-tool T=<tool-name>"; \
		echo "   Example: make build-tool T=session-explorer"; \
		exit 1; \
	fi
	@printf "$(BLUE)📦 Building: $(T)...$(NC)\n"
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" build; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

# ═══════════════════════════════════════════════════════════════════════════
#  Testing
# ═══════════════════════════════════════════════════════════════════════════

test: ## Run tests for ALL tools
	@$(SCRIPTS_DIR)/run-all-tests.sh

test-tool: ## Run tests for a specific tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make test-tool T=<tool-name>"; \
		echo "   Example: make test-tool T=session-explorer"; \
		exit 1; \
	fi
	@printf "$(YELLOW)🧪 Testing: $(T)...$(NC)\n"
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" test; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

# ═══════════════════════════════════════════════════════════════════════════
#  Linting
# ═══════════════════════════════════════════════════════════════════════════

lint: ## Lint ALL tools
	@printf "$(BLUE)🔍 Linting all tools...$(NC)\n"
	@for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		if [ -f "$$tool_dir/Makefile" ] && grep -q '^lint:' "$$tool_dir/Makefile"; then \
			echo "  Linting: $$tool_name"; \
			$(MAKE) -C "$$tool_dir" lint || exit 1; \
		fi; \
	done
	@printf "$(GREEN)✅ All linting passed!$(NC)\n"

lint-tool: ## Lint a specific tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make lint-tool T=<tool-name>"; \
		exit 1; \
	fi
	@printf "$(BLUE)🔍 Linting: $(T)...$(NC)\n"
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" lint; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

# ═══════════════════════════════════════════════════════════════════════════
#  SDD Specifications & Governance
# ═══════════════════════════════════════════════════════════════════════════

validate-specs: ## Validate all SDD specs are complete
	@$(SCRIPTS_DIR)/validate-specs.sh

catalog: ## Regenerate the tool catalog in tools/README.md
	@$(SCRIPTS_DIR)/update-catalog.sh

status: ## Show SDLC status dashboard for all tools
	@printf "\n"
	@printf "$(BOLD)╔══════════════════════════════════════════════════════════╗$(NC)\n"
	@printf "$(BOLD)║          📊 SDLC Status Dashboard                      ║$(NC)\n"
	@printf "$(BOLD)╚══════════════════════════════════════════════════════════╝$(NC)\n\n"
	@printf "  $(BOLD)%-25s %-15s %-10s %-12s$(NC)\n" "Tool" "Status" "Version" "Language"
	@printf "  %-25s %-15s %-10s %-12s\n" "─────────────────────────" "───────────────" "──────────" "────────────"
	@found=0; \
	for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		status_file="$$tool_dir/STATUS.md"; \
		if [ -f "$$status_file" ]; then \
			found=1; \
			status=$$(grep '^status:' "$$status_file" | head -1 | sed 's/^status: *//'); \
			version=$$(grep '^version:' "$$status_file" | head -1 | sed 's/^version: *//'); \
			language=$$(grep '^language:' "$$status_file" | head -1 | sed 's/^language: *//'); \
			printf "  %-25s %-15s %-10s %-12s\n" "$$tool_name" "$$status" "$$version" "$$language"; \
		fi; \
	done; \
	if [ "$$found" -eq 0 ]; then \
		echo "  No tools found. Create one with: make new-tool NAME=<name>"; \
	fi
	@printf "\n"

# ═══════════════════════════════════════════════════════════════════════════
#  Scaffold & Setup
# ═══════════════════════════════════════════════════════════════════════════

new-tool: ## Scaffold a new tool (NAME=tool-name)
	@if [ -z "$(NAME)" ]; then \
		echo "❌ Usage: make new-tool NAME=<tool-name>"; \
		echo "   Example: make new-tool NAME=json-validator"; \
		exit 1; \
	fi
	@$(SCRIPTS_DIR)/scaffold-tool.sh "$(NAME)"

setup: ## Install pre-commit hooks and repo dependencies
	@printf "$(BLUE)⚙️  Setting up tool-scripts...$(NC)\n"
	@if command -v pre-commit &>/dev/null; then \
		pre-commit install && \
		pre-commit install --hook-type commit-msg && \
		printf "$(GREEN)✅ Pre-commit hooks installed$(NC)\n"; \
	else \
		printf "$(YELLOW)⚠️  pre-commit not found. Install with: pip install pre-commit$(NC)\n"; \
	fi
	@printf "$(GREEN)✅ Setup complete!$(NC)\n"

clean: ## Clean all build artifacts across all tools
	@printf "$(YELLOW)🧹 Cleaning all tools...$(NC)\n"
	@for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		if [ -f "$$tool_dir/Makefile" ] && grep -q '^clean:' "$$tool_dir/Makefile"; then \
			$(MAKE) -C "$$tool_dir" clean; \
		fi; \
	done
	@printf "$(GREEN)✅ Clean complete!$(NC)\n"
