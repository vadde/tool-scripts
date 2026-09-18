# ============================================================================
# Tool Makefile — [Tool Name]
#
# Standard targets for building, testing, and running this tool.
# Language: [python|go|node|rust|shell]
# ============================================================================

.PHONY: help setup test lint run build clean test-examples

.DEFAULT_GOAL := help

# ─── Configuration ──────────────────────────────────────────────────────────
TOOL_NAME := $(notdir $(CURDIR))

# ─── Help ───────────────────────────────────────────────────────────────────
help: ## Show this help message
	@echo "╔══════════════════════════════════════════════════╗"
	@echo "║  Tool: $(TOOL_NAME)"
	@echo "╚══════════════════════════════════════════════════╝"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# ─── Setup ──────────────────────────────────────────────────────────────────
setup: ## Install dependencies
	@echo "⚙️  Setting up $(TOOL_NAME)..."
	# Python: pip install -e ".[dev]"
	# Go:     go mod download
	# Node:   pnpm install
	# Rust:   cargo build
	@echo "✅ Setup complete."

# ─── Test ───────────────────────────────────────────────────────────────────
test: ## Run test suite
	@echo "🧪 Running tests for $(TOOL_NAME)..."
	# Python: python -m pytest tests/ -v --tb=short
	# Go:     go test ./... -v -race
	# Node:   pnpm test
	# Rust:   cargo test
	# Shell:  bats tests/
	@echo "✅ Tests complete."

# ─── Lint ───────────────────────────────────────────────────────────────────
lint: ## Run linters
	@echo "🔍 Linting $(TOOL_NAME)..."
	# Python: ruff check src/ && ruff format --check src/
	# Go:     golangci-lint run
	# Node:   pnpm lint
	# Rust:   cargo clippy -- -D warnings
	# Shell:  shellcheck src/*.sh
	@echo "✅ Lint complete."

# ─── Run ────────────────────────────────────────────────────────────────────
run: ## Run the tool
	@echo "🚀 Running $(TOOL_NAME)..."
	# Python: python src/main.py
	# Go:     go run src/main.go
	# Node:   pnpm start
	# Rust:   cargo run
	# Shell:  bash src/main.sh

# ─── Build ──────────────────────────────────────────────────────────────────
build: ## Build the tool (if applicable)
	@echo "📦 Building $(TOOL_NAME)..."
	# Go:   go build -o bin/$(TOOL_NAME) src/main.go
	# Rust: cargo build --release

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
clean: ## Remove build artifacts
	@echo "🧹 Cleaning $(TOOL_NAME)..."
	rm -rf build/ dist/ __pycache__ .pytest_cache .coverage .mypy_cache
	rm -rf node_modules/ target/ bin/
	@echo "✅ Clean complete."
