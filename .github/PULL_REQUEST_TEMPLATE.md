## Pull Request

### Description

_Brief description of the changes. What does this PR do and why?_

### Spec Reference

- **Spec**: `specs/catalog/<tool-name>.md`
- **Requirements**: R-001, R-002, ...

### Type of Change

- [ ] 🚀 New tool
- [ ] ✨ New feature (non-breaking)
- [ ] 🐛 Bug fix (non-breaking)
- [ ] 💥 Breaking change
- [ ] 📚 Documentation update
- [ ] 🔧 Maintenance / chore
- [ ] ♻️ Refactoring
- [ ] 🧪 Tests

### Checklist

- [ ] **Spec**: Links to the relevant spec in `specs/catalog/`
- [ ] **Tests**: All new/modified code has tests
- [ ] **Tests pass**: `make test-tool T=<name>` passes
- [ ] **Lint**: `make lint-tool T=<name>` passes
- [ ] **Docs**: README and examples updated
- [ ] **CHANGELOG**: Entry added to tool's `CHANGELOG.md`
- [ ] **STATUS**: Tool's `STATUS.md` reflects the change
- [ ] **No secrets**: No credentials or keys in the diff
- [ ] **Conventional commit**: PR title follows `type(scope): description`

### Testing

_Describe how you tested these changes._

```bash
make test-tool T=<name>
```

### Screenshots / Output

_If applicable, add screenshots or command output._
