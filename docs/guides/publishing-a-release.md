# Guide: Publishing a Release

> How to prepare and publish a tool release.

---

## Pre-Release Checklist

- [ ] All tests pass: `make test-tool T=<name>`
- [ ] Lint passes: `make lint-tool T=<name>`
- [ ] All spec requirements are implemented
- [ ] README is complete
- [ ] Examples are runnable
- [ ] CHANGELOG has entries under `[Unreleased]`
- [ ] No TODO/FIXME in source code

---

## Release Steps

### 1. Finalize CHANGELOG

Move `[Unreleased]` entries to a versioned section:

```markdown
## [1.0.0] - 2026-09-18

### Added
- Feature X (R-001)
- Feature Y (R-002)
```

### 2. Update STATUS.md

```yaml
status: released
version: 1.0.0
last_updated: 2026-09-18
```

### 3. Update Tool Catalog

Update the status in `tools/README.md`.

### 4. Commit

```bash
git commit -am "release(my-tool): v1.0.0"
```

### 5. Tag

```bash
git tag -a my-tool/v1.0.0 -m "my-tool v1.0.0: Brief description"
```

### 6. Push

```bash
git push origin main --tags
```

### 7. Verify

- GitHub Actions creates a Release automatically
- Check the release notes are correct
- Verify the status board is updated

---

## Version Bumping Guide

| Change | Bump | Example |
|--------|------|---------|
| Breaking API change | **MAJOR** | 1.0.0 → 2.0.0 |
| New feature (backward-compatible) | **MINOR** | 1.0.0 → 1.1.0 |
| Bug fix | **PATCH** | 1.0.0 → 1.0.1 |
| First release | — | 0.1.0 |

---

📎 See the [Release Tool Skill](../../.agents/skills/release-tool/SKILL.md) for the full agent procedure.
