# Rule 05 — Testing Strategy

> **Priority**: 🟡 Important — Every tool must be tested before release.

---

## Testing Pyramid

```
                    ╱╲
                   ╱  ╲
                  ╱ E2E╲          Few, slow, high-confidence
                 ╱──────╲
                ╱        ╲
               ╱Integration╲     Some, medium speed
              ╱──────────────╲
             ╱                ╲
            ╱    Unit Tests    ╲  Many, fast, focused
           ╱────────────────────╲
```

### Coverage Targets

| Test Type | Coverage Target | Required For |
|-----------|----------------|--------------|
| Unit | ≥ 80% line coverage | All tools |
| Integration | Key workflows covered | Tools with dependencies |
| E2E / Examples | All examples runnable | All tools |

---

## Test Organization

```
tools/<name>/tests/
├── test_main.py          # Unit tests for main module
├── test_utils.py         # Unit tests for utilities
├── test_cli.py           # CLI integration tests
├── test_integration.py   # Integration tests (if needed)
├── conftest.py           # Shared fixtures (Python)
└── fixtures/             # Test data files
    ├── valid_input.json
    └── invalid_input.json
```

### Test Naming Convention

Tests MUST reference spec requirements where applicable:

```python
# Pattern: test_<requirement_or_function>_<scenario>_<expected>

def test_R001_validate_schema_valid_input_returns_true():
    """R-001: Schema validation accepts valid JSON."""
    ...

def test_R001_validate_schema_invalid_input_returns_false():
    """R-001: Schema validation rejects invalid JSON."""
    ...

def test_R002_cli_format_flag_outputs_json():
    """R-002: CLI --format=json produces JSON output."""
    ...

def test_parse_input_empty_string_raises_value_error():
    """Edge case: empty input should raise ValueError."""
    ...
```

---

## Testing Requirements by Language

### Python

```toml
# In pyproject.toml
[tool.pytest.ini_options]
testpaths = ["tests"]
addopts = [
    "--strict-markers",
    "--tb=short",
    "-v",
]
markers = [
    "slow: marks tests as slow",
    "integration: marks integration tests",
]

[tool.coverage.run]
source = ["src"]
omit = ["tests/*"]

[tool.coverage.report]
fail_under = 80
show_missing = true
```

Run:
```bash
make test-tool T=<name>          # Run all tests
pytest --cov=src --cov-report=term-missing  # With coverage
```

### Go

```bash
go test ./... -v -race -cover
go test ./... -coverprofile=coverage.out
go tool cover -html=coverage.out
```

### Node.js / TypeScript

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage"
  }
}
```

### Shell / Bash

Use `bats` (Bash Automated Testing System):
```bash
#!/usr/bin/env bats

@test "script exits 0 on valid input" {
  run ./src/script.sh valid_input.txt
  [ "$status" -eq 0 ]
}

@test "script exits 2 on missing arguments" {
  run ./src/script.sh
  [ "$status" -eq 2 ]
  [[ "$output" == *"Usage:"* ]]
}
```

### Rust

```bash
cargo test -- --test-threads=1
cargo test -- --nocapture  # Show println output
cargo tarpaulin --out html  # Coverage
```

---

## Test Categories

### 1. Unit Tests (REQUIRED)

- Test individual functions in isolation
- Mock external dependencies
- Fast execution (< 1s per test)
- Cover: happy path, edge cases, error cases

### 2. Integration Tests (WHEN APPLICABLE)

- Test component interactions
- Use real (or realistic) dependencies
- Test CLI argument parsing end-to-end
- Test file I/O with fixture files

### 3. Property-Based Tests (RECOMMENDED for mathematical tools)

```python
from hypothesis import given, strategies as st

@given(st.lists(st.integers()))
def test_sort_preserves_length(data):
    """Sorting should never change the number of elements."""
    assert len(custom_sort(data)) == len(data)

@given(st.lists(st.integers()))
def test_sort_is_idempotent(data):
    """Sorting twice should equal sorting once."""
    assert custom_sort(data) == custom_sort(custom_sort(data))
```

### 4. Example Tests (REQUIRED)

Every file in `examples/` must be runnable and produce expected output:

```bash
# In tool's Makefile
test-examples:
	@echo "Running examples..."
	@for f in examples/*.py; do python "$$f" || exit 1; done
	@echo "All examples passed."
```

---

## Test Quality Checklist

Before considering tests complete:

- [ ] Happy path tested for every public function
- [ ] Edge cases tested (empty input, large input, boundary values)
- [ ] Error cases tested (invalid input, missing dependencies)
- [ ] Spec requirements covered (R-XXX referenced in test names)
- [ ] No flaky tests (deterministic, no race conditions)
- [ ] No hardcoded paths (use fixtures and temp directories)
- [ ] Tests run in isolation (no shared mutable state)
- [ ] Tests are fast (< 30s total for unit tests)
- [ ] Coverage meets threshold (≥ 80%)

---

## CI Test Execution

Tests run automatically in CI via the `tool-ci.yml` reusable workflow:

1. **On PR**: Tests run for changed tools only (path filtering)
2. **On merge to develop**: Full test suite
3. **On merge to main**: Full test suite + coverage report

### Test Failure Protocol

When tests fail in CI:
1. **Read** the full error output (every line)
2. **Reproduce** locally: `make test-tool T=<name>`
3. **Fix** the root cause (not just the symptom)
4. **Add** a regression test if the fix addresses a new edge case
5. **Verify** the fix: `make test-tool T=<name>`
