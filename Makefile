# ============================================================================
#  ╔══════════════════════════════════════════════════════════════════════╗
#  ║                     tool-scripts — Root Makefile                    ║
#  ║                                                                    ║
#  ║  The unified command interface for the entire repository.          ║
#  ║  Run `make help` to see all available targets.                     ║
#  ╚══════════════════════════════════════════════════════════════════════╝
# ============================================================================

.PHONY: help setup new-tool test test-tool lint lint-tool validate-specs \
        catalog status clean

.DEFAULT_GOAL := help

# ─── Configuration ──────────────────────────────────────────────────────────
SHELL := /bin/bash
TOOLS_DIR := tools
SCRIPTS_DIR := scripts
SPECS_DIR := specs

# ─── Colors ─────────────────────────────────────────────────────────────────
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BOLD := \033[1m
NC := \033[0m

# ═══════════════════════════════════════════════════════════════════════════
#  Help
# ═══════════════════════════════════════════════════════════════════════════

help: ## Show this help message
	@echo ""
	@echo "$(BOLD)╔══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BOLD)║          🔧 tool-scripts — Command Center              ║$(NC)"
	@echo "$(BOLD)╚══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(BOLD)Usage:$(NC)"
	@echo "  make $(CYAN)<target>$(NC) [ARGS]"
	@echo ""
	@echo "$(BOLD)Targets:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(BOLD)Examples:$(NC)"
	@echo "  make new-tool NAME=json-validator"
	@echo "  make test-tool T=json-validator"
	@echo "  make lint-tool T=json-validator"
	@echo ""

# ═══════════════════════════════════════════════════════════════════════════
#  Setup
# ═══════════════════════════════════════════════════════════════════════════

setup: ## Install pre-commit hooks and repo dependencies
	@echo "⚙️  Setting up tool-scripts..."
	@if command -v pre-commit &>/dev/null; then \
		pre-commit install && \
		pre-commit install --hook-type commit-msg && \
		echo "$(GREEN)✅ Pre-commit hooks installed$(NC)"; \
	else \
		echo "$(YELLOW)⚠️  pre-commit not found. Install with: pip install pre-commit$(NC)"; \
	fi
	@echo "$(GREEN)✅ Setup complete!$(NC)"

# ═══════════════════════════════════════════════════════════════════════════
#  Tool Management
# ═══════════════════════════════════════════════════════════════════════════

new-tool: ## Scaffold a new tool (NAME=tool-name)
	@if [ -z "$(NAME)" ]; then \
		echo "❌ Usage: make new-tool NAME=<tool-name>"; \
		echo "   Example: make new-tool NAME=json-validator"; \
		exit 1; \
	fi
	@$(SCRIPTS_DIR)/scaffold-tool.sh "$(NAME)"

# ═══════════════════════════════════════════════════════════════════════════
#  Testing
# ═══════════════════════════════════════════════════════════════════════════

test: ## Run tests for ALL tools
	@$(SCRIPTS_DIR)/run-all-tests.sh

test-tool: ## Run tests for a specific tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make test-tool T=<tool-name>"; \
		echo "   Example: make test-tool T=json-validator"; \
		exit 1; \
	fi
	@echo "🧪 Testing: $(T)"
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
	@echo "🔍 Linting all tools..."
	@for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		if [ -f "$$tool_dir/Makefile" ] && grep -q '^lint:' "$$tool_dir/Makefile"; then \
			echo "  Linting: $$tool_name"; \
			$(MAKE) -C "$$tool_dir" lint || exit 1; \
		fi; \
	done
	@echo "$(GREEN)✅ All linting passed!$(NC)"

lint-tool: ## Lint a specific tool (T=tool-name)
	@if [ -z "$(T)" ]; then \
		echo "❌ Usage: make lint-tool T=<tool-name>"; \
		exit 1; \
	fi
	@echo "🔍 Linting: $(T)"
	@if [ -f "$(TOOLS_DIR)/$(T)/Makefile" ]; then \
		$(MAKE) -C "$(TOOLS_DIR)/$(T)" lint; \
	else \
		echo "❌ No Makefile found in tools/$(T)/"; \
		exit 1; \
	fi

# ═══════════════════════════════════════════════════════════════════════════
#  Spec Validation
# ═══════════════════════════════════════════════════════════════════════════

validate-specs: ## Validate all SDD specs are complete
	@$(SCRIPTS_DIR)/validate-specs.sh

# ═══════════════════════════════════════════════════════════════════════════
#  Catalog & Status
# ═══════════════════════════════════════════════════════════════════════════

catalog: ## Regenerate the tool catalog in tools/README.md
	@$(SCRIPTS_DIR)/update-catalog.sh

status: ## Show SDLC status dashboard for all tools
	@echo ""
	@echo "$(BOLD)╔══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BOLD)║          📊 SDLC Status Dashboard                      ║$(NC)"
	@echo "$(BOLD)╚══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
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
	@echo ""

# ═══════════════════════════════════════════════════════════════════════════
#  Cleanup
# ═══════════════════════════════════════════════════════════════════════════

clean: ## Clean all build artifacts across all tools
	@echo "🧹 Cleaning all tools..."
	@for tool_dir in $(TOOLS_DIR)/*/; do \
		tool_name=$$(basename "$$tool_dir"); \
		if [ "$$tool_name" = "_template" ]; then continue; fi; \
		if [ -f "$$tool_dir/Makefile" ] && grep -q '^clean:' "$$tool_dir/Makefile"; then \
			$(MAKE) -C "$$tool_dir" clean; \
		fi; \
	done
	@echo "$(GREEN)✅ Clean complete!$(NC)"
