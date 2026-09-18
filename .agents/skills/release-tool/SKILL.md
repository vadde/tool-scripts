---
name: release-tool
description: Prepare a tool for release. Use when a tool has passed testing and review and is ready to be published as a stable version.
---

# Skill: Release a Tool

## When to Use

Use this skill when a tool's STATUS.md shows `status: review` and all
review criteria have been met.

---

## Pre-Release Checklist

Before starting the release process, verify ALL of these:

- [ ] `STATUS.md` shows `status: review`
- [ ] ALL spec requirements (R-XXX) are implemented
- [ ] ALL tests pass: `make test-tool T=<name>`
- [ ] Lint passes: `make lint-tool T=<name>`
- [ ] ALL examples in `examples/` are runnable
- [ ] README.md is complete (all sections filled)
- [ ] CHANGELOG.md has entries under `[Unreleased]`
- [ ] No TODO/FIXME/HACK comments in source code
- [ ] No hardcoded paths or secrets
- [ ] Code review completed (if human reviewer available)

---

## Release Procedure

### Step 1: Finalize Version

Determine the version bump based on changes:

| Change Type | Version Bump | Example |
|------------|-------------|---------|
| Breaking API change | MAJOR | 0.1.0 → 1.0.0 |
| New feature | MINOR | 0.1.0 → 0.2.0 |
| Bug fix only | PATCH | 0.1.0 → 0.1.1 |
| First release | — | 0.1.0 |

### Step 2: Update CHANGELOG.md

Move entries from `[Unreleased]` to a versioned section:

```markdown
## [Unreleased]
(empty — move items below)

## [0.2.0] - 2026-XX-XX

### Added
- Feature X (R-003)
- Feature Y (R-004)

### Fixed
- Bug in input parsing (#42)
```

### Step 3: Update STATUS.md

```yaml
---
tool: <tool-name>
status: released
version: 0.2.0
language: python
category: Utilities
created: 2026-01-15
last_updated: 2026-XX-XX
owner: "@vadde"
spec: ../../specs/catalog/<tool-name>.md
---
```

### Step 4: Update Global Changelog

Add entry to `sdlc/CHANGELOG.md`:

```markdown
## [YYYY-MM-DD]

### Released
- **<tool-name>** v0.2.0 — Brief description of what's new
```

### Step 5: Update Tool Catalog

Update the status column in `tools/README.md`:

```markdown
| [tool-name](tool-name/) | Category | Python | `released` v0.2.0 | Description |
```

### Step 6: Final Test Run

```bash
make test-tool T=<name>
make lint-tool T=<name>
make validate-specs
```

### Step 7: Create Release Commit

```bash
git add tools/<name> specs/catalog/<name>.md sdlc/CHANGELOG.md tools/README.md
git commit -m "release(<name>): v0.2.0

Released <tool-name> v0.2.0 with:
- Feature X (R-003)
- Feature Y (R-004)
- Bug fix for input parsing

All tests pass. All spec requirements satisfied."
```

### Step 8: Tag the Release

```bash
git tag -a <name>/v0.2.0 -m "<tool-name> v0.2.0"
git push origin main --tags
```

### Step 9: Verify Release

- [ ] Tag exists on remote
- [ ] GitHub Release created (via GHA workflow)
- [ ] Release notes are accurate
- [ ] Status board updated

---

## Post-Release

1. **Announce** — Update any relevant documentation or channels
2. **Monitor** — Watch for early bug reports
3. **Plan** — Move next items from ROADMAP to spec drafts
