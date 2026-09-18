# Guide: Running Tests

> How to run tests across the repository.

---

## Quick Commands

```bash
# Test everything
make test

# Test a specific tool
make test-tool T=json-validator

# Test from within a tool directory
cd tools/json-validator && make test
```

---

## Per-Language Testing

### Python

```bash
# Run tests
python -m pytest tests/ -v --tb=short

# With coverage
python -m pytest tests/ --cov=src --cov-report=term-missing

# Specific test
python -m pytest tests/test_main.py::test_R001_validate -v
```

### Go

```bash
# Run tests
go test ./... -v -race

# With coverage
go test ./... -coverprofile=coverage.out
go tool cover -html=coverage.out
```

### Node.js / TypeScript

```bash
# Run tests
pnpm test

# With coverage
pnpm test -- --coverage

# Watch mode
pnpm test -- --watch
```

### Rust

```bash
# Run tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

### Shell (bats)

```bash
# Run tests
bats tests/

# Verbose
bats tests/ --verbose-run
```

---

## CI Testing

Tests run automatically in GitHub Actions on:
- Every push to `main` or `develop`
- Every pull request

Only changed tools are tested (via path filtering).

---

## Troubleshooting

| Issue | Solution |
|-------|---------|
| Tests not found | Check `testpaths` in config or test file naming |
| Import errors | Ensure `src/` is on the path; check `__init__.py` |
| Fixture not found | Check `conftest.py` location |
| Flaky test | Look for shared state or timing dependencies |
