# Rule 03 — Commit Conventions & Git Workflow

> **Priority**: 🔴 Critical — All git operations must follow these conventions.

---

## Branch Strategy

```
main (protected)
  │
  ├── develop (integration branch)
  │     │
  │     ├── feat/tool-name/description    ← New tool or feature
  │     ├── fix/tool-name/description     ← Bug fix
  │     ├── docs/description              ← Documentation changes
  │     ├── refactor/tool-name/description ← Refactoring
  │     ├── test/tool-name/description    ← Test improvements
  │     └── chore/description             ← Maintenance tasks
  │
  └── release/vX.Y.Z (release candidates)
```

### Branch Naming Rules

| Pattern | Usage | Example |
|---------|-------|---------|
| `feat/<tool>/<desc>` | New tool or feature | `feat/json-validator/initial-impl` |
| `fix/<tool>/<desc>` | Bug fix | `fix/csv-merger/handle-empty-rows` |
| `docs/<desc>` | Documentation | `docs/update-getting-started` |
| `refactor/<tool>/<desc>` | Code restructuring | `refactor/math-utils/extract-parser` |
| `test/<tool>/<desc>` | Test additions/fixes | `test/json-validator/edge-cases` |
| `chore/<desc>` | Maintenance | `chore/update-ci-workflows` |
| `release/v<X.Y.Z>` | Release prep | `release/v1.2.0` |

### Branch Rules

1. **`main`** — Always stable, always deployable. Protected.
2. **`develop`** — Integration branch. PRs merge here first.
3. **Feature branches** — Branch from `develop`, merge back to `develop`.
4. **Release branches** — Branch from `develop`, merge to `main` AND `develop`.
5. **Hotfix branches** — Branch from `main`, merge to `main` AND `develop`.

---

## Commit Message Format

We use **Conventional Commits** (v1.0.0).

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Purpose | Triggers |
|------|---------|----------|
| `feat` | New feature or tool | Minor version bump |
| `fix` | Bug fix | Patch version bump |
| `docs` | Documentation only | No version bump |
| `style` | Formatting, whitespace | No version bump |
| `refactor` | Code restructuring | No version bump |
| `test` | Adding/fixing tests | No version bump |
| `chore` | Maintenance, deps | No version bump |
| `build` | Build system changes | No version bump |
| `ci` | CI/CD changes | No version bump |
| `perf` | Performance improvement | Patch version bump |
| `revert` | Reverting a commit | Depends on reverted commit |

### Scope

The scope is the **tool name** (from `tools/<name>/`) or a repo-level area:

| Scope | Context |
|-------|---------|
| `<tool-name>` | Changes to a specific tool |
| `specs` | Spec changes |
| `docs` | Documentation changes |
| `ci` | CI/CD pipeline changes |
| `agents` | Agent rules/skills changes |
| `sdlc` | SDLC governance changes |
| `repo` | Repository-wide changes |

### Examples

```bash
# New tool
feat(json-validator): add initial implementation with CLI interface

# Bug fix with body
fix(csv-merger): handle empty rows in input files

Previously, empty rows caused an IndexError. Now they are
skipped with a warning logged at DEBUG level.

Closes #42

# Breaking change
feat(math-utils)!: change API to accept numpy arrays

BREAKING CHANGE: The `compute()` function now requires numpy
arrays instead of plain lists. Use `np.array(data)` to convert.

# Documentation
docs(book): add chapter on advanced patterns

# CI change
ci: add security audit workflow for weekly scanning

# Chore
chore(deps): update ruff to v0.7.0
```

### Commit Rules

1. **Atomic commits** — Each commit does ONE thing
2. **Present tense** — "add feature" not "added feature"
3. **Imperative mood** — "fix bug" not "fixes bug"
4. **72-char subject line** — Keep it concise
5. **Body wraps at 100 chars** — Use the body for context
6. **Reference issues** — Use `Closes #N` or `Refs #N` in footer

---

## Pull Request Workflow

### PR Title Format

Same as commit format:
```
<type>(<scope>): <description>
```

### PR Checklist (included in template)

Every PR must satisfy:

- [ ] Spec reference: Links to the relevant spec in `specs/catalog/`
- [ ] Tests: All new/modified code has tests
- [ ] Tests pass: `make test-tool T=<name>` passes
- [ ] Lint: `make lint-tool T=<name>` passes
- [ ] Docs: README and examples updated
- [ ] CHANGELOG: Entry added to tool's `CHANGELOG.md`
- [ ] STATUS: Tool's `STATUS.md` reflects the change
- [ ] No secrets: No credentials or keys in the diff

### PR Size Guidelines

| Size | Lines Changed | Review Time |
|------|--------------|-------------|
| 🟢 Small | < 100 | < 30 min |
| 🟡 Medium | 100–300 | 30–60 min |
| 🟠 Large | 300–500 | 1–2 hours |
| 🔴 X-Large | > 500 | Split into smaller PRs |

### Merge Strategy

| Target Branch | Merge Type | Rationale |
|--------------|------------|-----------|
| `develop` ← feature | **Squash merge** | Clean linear history |
| `main` ← `develop` | **Merge commit** | Preserve integration context |
| `main` ← hotfix | **Merge commit** | Preserve fix context |

---

## Release & Tagging

### Semantic Versioning (SemVer 2.0)

```
MAJOR.MINOR.PATCH
  │      │     │
  │      │     └── Bug fixes, patches (backward-compatible)
  │      └──────── New features (backward-compatible)
  └─────────────── Breaking changes
```

### Tag Format

```
v<MAJOR>.<MINOR>.<PATCH>
```

Examples: `v1.0.0`, `v1.2.3`, `v2.0.0-rc.1`

### Release Process

1. **Create release branch** from `develop`:
   ```bash
   git checkout -b release/v1.2.0 develop
   ```

2. **Update versions**:
   - Tool `STATUS.md` → update `version` field
   - Root `sdlc/CHANGELOG.md` → add release section
   - Tool `CHANGELOG.md` → finalize "Unreleased" section

3. **Final testing**:
   ```bash
   make test
   make lint
   make validate-specs
   ```

4. **Merge to main**:
   ```bash
   git checkout main
   git merge --no-ff release/v1.2.0
   git tag -a v1.2.0 -m "Release v1.2.0: <summary>"
   git push origin main --tags
   ```

5. **Merge back to develop**:
   ```bash
   git checkout develop
   git merge --no-ff release/v1.2.0
   git push origin develop
   ```

6. **GitHub Release**: GHA workflow auto-creates a GitHub Release with notes

### Release Notes Format

```markdown
## What's New in v1.2.0

### 🚀 New Tools
- **json-validator** — Validate JSON files against schemas

### ✨ Enhancements
- **csv-merger** — Added support for TSV files (#45)

### 🐛 Bug Fixes
- **math-utils** — Fixed floating-point precision in `round_to` (#42)

### 📚 Documentation
- Added Chapter 07: Advanced Patterns to the docs eBook

### 🔧 Maintenance
- Updated CI to use Python 3.12
- Added security audit workflow
```
