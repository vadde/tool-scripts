# Chapter 06 — CI/CD and Releases

> _Automated quality gates from commit to release._

---

## CI/CD Architecture

```
Developer/Agent pushes code
        │
        ▼
┌─────────────────────────────────┐
│  ci.yml (Main CI Pipeline)      │
│  ├── Detect changed tools       │
│  ├── For each changed tool:     │
│  │   └── Call tool-ci.yml       │
│  ├── Validate specs             │
│  └── Repo-level lint            │
└─────────────────┬───────────────┘
                  │
        ┌─────────▼──────────┐
        │  tool-ci.yml       │
        │  (Reusable)        │
        │  ├── Detect lang   │
        │  ├── Install deps  │
        │  ├── Lint           │
        │  ├── Test           │
        │  └── Coverage       │
        └─────────┬──────────┘
                  │
        ┌─────────▼──────────┐
        │  Results            │
        │  ├── Test report    │
        │  ├── Coverage       │
        │  └── Pass/Fail      │
        └────────────────────┘
```

---

## Workflows

### `ci.yml` — Main CI Pipeline

**Triggers:** Push to `main`/`develop`, Pull Requests

1. Detects which tools changed (via `git diff` path filtering)
2. Dispatches the reusable `tool-ci.yml` for each changed tool
3. Runs repo-level checks (spec validation, markdown lint)

### `tool-ci.yml` — Reusable Per-Tool CI

**Trigger:** Called by `ci.yml` via `workflow_call`

1. **Auto-detects language** by checking for indicator files
2. **Sets up the runtime** (Python, Go, Node.js, Rust)
3. **Installs dependencies** (pip, go mod, pnpm, cargo)
4. **Lints** (ruff, golangci-lint, eslint, clippy)
5. **Tests** (pytest, go test, vitest, cargo test)
6. **Reports coverage** as a PR comment

### `release.yml` — Release Workflow

**Trigger:** Tag push (`v*`)

1. Validates the tag format
2. Extracts release notes from CHANGELOG.md
3. Creates a GitHub Release with assets
4. Updates the status board

### `security-audit.yml` — Security Scanning

**Trigger:** Weekly schedule + PR

1. Scans dependencies for known vulnerabilities
2. Runs secret detection
3. Reports findings as PR comments

### `docs-publish.yml` — Documentation

**Trigger:** Merge to `main`

1. Builds documentation
2. Publishes to GitHub Pages

---

## The Root Makefile

The Makefile is the **unified command interface** for all operations:

```bash
make help              # Show all targets
make setup             # Install pre-commit hooks
make new-tool NAME=x   # Scaffold a new tool
make test              # Run all tests
make test-tool T=name  # Run tests for one tool
make lint              # Lint everything
make lint-tool T=name  # Lint one tool
make validate-specs    # Validate all specs
make catalog           # Regenerate tool catalog
make status            # Show SDLC dashboard
make clean             # Clean all artifacts
```

---

## Release Process

### 1. Prepare

```bash
# Ensure all tests pass
make test-tool T=my-tool
make lint-tool T=my-tool

# Update CHANGELOG.md
$EDITOR tools/my-tool/CHANGELOG.md
```

### 2. Version & Commit

```bash
# Update STATUS.md version
$EDITOR tools/my-tool/STATUS.md

# Commit
git add tools/my-tool
git commit -m "release(my-tool): v1.0.0"
```

### 3. Tag & Push

```bash
git tag -a my-tool/v1.0.0 -m "my-tool v1.0.0: Initial release"
git push origin main --tags
```

### 4. Automated Release

The `release.yml` workflow automatically:
- Creates a GitHub Release
- Attaches release notes from CHANGELOG
- Updates the status board

---

📎 **Related:**
- [Commit Conventions](../../.agents/rules/03-commit-conventions.md)
- [Release Skill](../../.agents/skills/release-tool/SKILL.md)
- [Testing Strategy](../../.agents/rules/05-testing-strategy.md)

---

_[← Agent Collaboration](05-agent-collaboration.md) | [Chapter 07: Advanced Patterns →](07-advanced-patterns.md)_
