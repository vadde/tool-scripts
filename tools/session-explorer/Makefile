# ============================================================================
#  ╔══════════════════════════════════════════════════════════════════════╗
#  ║            🔍 session-explorer — Tool Command Center                 ║
#  ║                                                                      ║
#  ║  Single-binary conversation history explorer for Antigravity IDE.    ║
#  ║  Language: Go 1.22+ | Frontend: Vanilla HTML/CSS/JS (Bun bundled)   ║
#  ╚══════════════════════════════════════════════════════════════════════╝
# ============================================================================

.PHONY: help setup build-web build-go build test test-race test-cover lint \
        run run-headless demo clean status spec context devlog

.DEFAULT_GOAL := help

# ─── Configuration ──────────────────────────────────────────────────────────
TOOL_NAME    := session-explorer
PORT         ?= 9876
DATA_DIR     ?=
ARGS         ?=
BUN          ?= $(HOME)/.bun/bin/bun
PATH         := $(HOME)/.bun/bin:$(PATH)
export PATH

# ─── Colors & Styling ───────────────────────────────────────────────────────
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
	@printf "$(BOLD)║            🔍 session-explorer — Command Center                  ║$(NC)\n"
	@printf "$(BOLD)╚══════════════════════════════════════════════════════════════════╝$(NC)\n\n"
	@printf "  $(DIM)Quick links:$(NC)\n"
	@printf "  • Spec:    $(CYAN)specs/catalog/session-explorer.md$(NC)\n"
	@printf "  • Status:  $(CYAN)tools/session-explorer/STATUS.md$(NC)\n"
	@printf "  • Context: $(CYAN)tools/session-explorer/CONTEXT.md$(NC)\n\n"
	@printf "$(BOLD)🚀 Execution:$(NC)\n"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "run"          "Build & launch web UI (auto-opens browser at http://127.0.0.1:$(PORT))"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "run-headless" "Launch in headless mode without opening browser (--no-open)"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "demo"         "Run automated end-to-end demo script against mock brain data"
	@printf "\n"
	@printf "$(BOLD)📦 Build & Packaging:$(NC)\n"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "build"        "Complete build: bundle web UI with Bun + compile Go binary"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "build-web"    "Bundle frontend web assets with Bun into src/web/dist/"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "build-go"     "Compile standalone Go binary (embeds frontend assets)"
	@printf "\n"
	@printf "$(BOLD)🧪 Testing & Quality:$(NC)\n"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "test"         "Run full unit test suite (scanner, API, JSONL parser)"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "test-race"    "Run test suite with Go race detector enabled"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "test-cover"   "Run tests with coverage profile summary"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "lint"         "Run code linters and vet checks (go vet)"
	@printf "\n"
	@printf "$(BOLD)📋 SDD & Documentation:$(NC)\n"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "status"       "Display tool SDLC lifecycle phase and traceability matrix"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "spec"         "Display spec overview and requirements checklist"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "context"      "Display current development session context (CONTEXT.md)"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "devlog"       "Display latest development session log entries (DEVLOG.md)"
	@printf "\n"
	@printf "$(BOLD)⚙️ Setup & Maintenance:$(NC)\n"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "setup"        "Install dependencies (Go modules + Bun packages)"
	@printf "  $(CYAN)%-22s$(NC) %s\n" "clean"        "Remove build artifacts (bin/, src/web/dist/)"
	@printf "\n"
	@printf "  $(DIM)Custom parameters:$(NC)\n"
	@printf "    make run PORT=8080\n"
	@printf "    make run ARGS=\"--verbose --port 9000\"\n"
	@printf "    make run DATA_DIR=/path/to/custom/brain\n\n"

# ═══════════════════════════════════════════════════════════════════════════
#  Execution
# ═══════════════════════════════════════════════════════════════════════════

run: build ## Build and run Session Explorer
	@echo "$(GREEN)🚀 Launching $(TOOL_NAME) on port $(PORT)...$(NC)"
	@if [ -n "$(DATA_DIR)" ]; then \
		bin/$(TOOL_NAME) --port $(PORT) --data-dir "$(DATA_DIR)" $(ARGS); \
	else \
		bin/$(TOOL_NAME) --port $(PORT) $(ARGS); \
	fi

run-headless: build ## Launch in headless mode without opening browser
	@echo "$(GREEN)🚀 Launching $(TOOL_NAME) in headless mode (--no-open)...$(NC)"
	@if [ -n "$(DATA_DIR)" ]; then \
		bin/$(TOOL_NAME) --port $(PORT) --no-open --data-dir "$(DATA_DIR)" $(ARGS); \
	else \
		bin/$(TOOL_NAME) --port $(PORT) --no-open $(ARGS); \
	fi

demo: ## Run automated demo against sample mock data
	@echo "$(GREEN)📝 Executing automated demo...$(NC)"
	@chmod +x examples/demo.sh
	@examples/demo.sh

test-examples: demo ## Run all examples

# ═══════════════════════════════════════════════════════════════════════════
#  Build
# ═══════════════════════════════════════════════════════════════════════════

build-web: ## Bundle frontend assets using Bun
	@echo "$(BLUE)🎨 Bundling frontend web assets with Bun...$(NC)"
	@cd src/web && $(BUN) run build
	@echo "$(GREEN)✅ Frontend built in src/web/dist/$(NC)"

build-go: ## Compile Go standalone binary embedding frontend assets
	@echo "$(BLUE)📦 Compiling standalone Go binary...$(NC)"
	@mkdir -p bin
	@cd src && go build -ldflags="-s -w" -o ../bin/$(TOOL_NAME) .
	@echo "$(GREEN)✅ Binary compiled: bin/$(TOOL_NAME)$(NC)"

build: build-web build-go ## Full build: web UI + Go binary

# ═══════════════════════════════════════════════════════════════════════════
#  Testing & Linting
# ═══════════════════════════════════════════════════════════════════════════

test: ## Run test suite
	@echo "$(YELLOW)🧪 Running test suite...$(NC)"
	@cd src && go test -v ./...
	@echo "$(GREEN)✅ All tests passed!$(NC)"

test-race: ## Run test suite with race detector
	@echo "$(YELLOW)🧪 Running tests with race detector...$(NC)"
	@cd src && go test -v -race ./...
	@echo "$(GREEN)✅ Race detector checks passed!$(NC)"

test-cover: ## Run tests with code coverage profile
	@echo "$(YELLOW)🧪 Running tests with coverage...$(NC)"
	@cd src && go test -coverprofile=coverage.out ./... && go tool cover -func=coverage.out
	@rm -f src/coverage.out
	@echo "$(GREEN)✅ Coverage analysis complete!$(NC)"

lint: ## Run linters (go vet)
	@echo "$(BLUE)🔍 Running linters (go vet)...$(NC)"
	@cd src && go vet ./...
	@echo "$(GREEN)✅ Linting passed!$(NC)"

# ═══════════════════════════════════════════════════════════════════════════
#  SDD & Governance
# ═══════════════════════════════════════════════════════════════════════════

status: ## Show SDLC status and traceability
	@echo ""
	@head -n 25 STATUS.md
	@echo ""

spec: ## Show specification summary
	@echo ""
	@head -n 35 ../../specs/catalog/$(TOOL_NAME).md
	@echo ""

context: ## Show current session continuity context
	@echo ""
	@head -n 35 CONTEXT.md
	@echo ""

devlog: ## Show latest devlog session entries
	@echo ""
	@head -n 50 DEVLOG.md
	@echo ""

# ═══════════════════════════════════════════════════════════════════════════
#  Setup & Cleanup
# ═══════════════════════════════════════════════════════════════════════════

setup: ## Install dependencies (Go + Bun)
	@echo "$(BLUE)⚙️  Setting up $(TOOL_NAME)...$(NC)"
	@cd src && go mod download
	@cd src/web && $(BUN) install
	@echo "$(GREEN)✅ Setup complete.$(NC)"

clean: ## Clean build artifacts
	@echo "$(YELLOW)🧹 Cleaning build artifacts...$(NC)"
	@rm -rf bin/ src/web/dist/
	@echo "$(GREEN)✅ Clean complete.$(NC)"
