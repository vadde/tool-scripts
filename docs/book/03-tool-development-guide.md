# Chapter 03 — Tool Development Guide

> _The complete lifecycle of building a tool, from idea to release._

---

## The Tool Lifecycle

Every tool follows this lifecycle:

```
💡 Idea → 📋 Spec → 🔨 Build → 🧪 Test → 👀 Review → 🚀 Release
```

This chapter walks you through each stage.

---

## Stage 1: Ideate

Before writing any code, answer these questions:

1. **What problem does this tool solve?**
2. **Who will use it?** (Engineers, data scientists, DevOps, agents?)
3. **What language fits best?** (CPU-bound → Go/Rust, data → Python, web → TypeScript)
4. **Does a similar tool already exist?** (Check the [Tool Catalog](../../tools/README.md))
5. **What category does it belong to?** (Math, GenAI, DevOps, Data, Utilities, etc.)

## Stage 2: Specify

### Create the Spec

```bash
make new-tool NAME=my-tool
```

This creates both the tool directory and the spec file. Edit the spec:

```bash
$EDITOR specs/catalog/my-tool.md
```

### Spec Completeness Checklist

- [ ] Problem statement is clear and specific
- [ ] All functional requirements are numbered (R-001, R-002...)
- [ ] Non-functional requirements defined (performance, portability)
- [ ] Interface contract specified (CLI flags, API signatures)
- [ ] Input/output formats documented
- [ ] Error codes defined
- [ ] Acceptance criteria are testable
- [ ] Dependencies listed

📎 See [Chapter 04: Spec-Driven Development](04-spec-driven-development.md) for detailed guidance.

## Stage 3: Implement

### Set Up the Language Environment

Uncomment the appropriate lines in `tools/my-tool/Makefile`:

**Python:**
```makefile
test:
	python -m pytest tests/ -v --tb=short

lint:
	ruff check src/ && ruff format --check src/
```

**Go:**
```makefile
test:
	go test ./... -v -race

lint:
	golangci-lint run
```

**Node.js:**
```makefile
test:
	pnpm test

lint:
	pnpm lint
```

### Write the Code

Every source file should:
1. **Start with a module docstring** explaining its purpose
2. **Reference spec requirements** in function docstrings: `Implements: R-001`
3. **Validate inputs** at the boundary
4. **Handle errors** with context (never silently fail)
5. **Use meaningful names** for variables, functions, and classes

### File Naming by Language

| Language | Entry Point | Test File | Config |
|----------|------------|-----------|--------|
| Python | `src/main.py` | `tests/test_main.py` | `pyproject.toml` |
| Go | `src/main.go` | `src/main_test.go` | `go.mod` |
| Node.js | `src/index.ts` | `tests/index.test.ts` | `package.json` |
| Rust | `src/main.rs` | `tests/test_main.rs` | `Cargo.toml` |
| Shell | `src/main.sh` | `tests/test_main.bats` | — |

## Stage 4: Test

### Write Tests That Map to Spec

```python
def test_R001_validate_accepts_valid_json():
    """R-001: Validator accepts well-formed JSON."""
    assert validate('{"key": "value"}') is True

def test_R001_validate_rejects_invalid_json():
    """R-001: Validator rejects malformed JSON."""
    assert validate('{bad json}') is False

def test_R002_cli_outputs_json_format():
    """R-002: --format=json flag produces JSON output."""
    result = run_cli(["--format", "json", "input.txt"])
    assert json.loads(result.stdout)
```

### Run Tests

```bash
make test-tool T=my-tool    # From repo root
cd tools/my-tool && make test   # From tool directory
```

### Check Coverage

```bash
# Python
cd tools/my-tool && python -m pytest tests/ --cov=src --cov-report=term-missing

# Go
cd tools/my-tool && go test ./... -cover
```

📎 See [Chapter 05: Testing Strategy](../../.agents/rules/05-testing-strategy.md) for coverage targets.

## Stage 5: Document

### Update Tool README

Fill in all sections of `tools/my-tool/README.md`:
- Installation & prerequisites
- Usage examples (CLI and programmatic)
- Configuration options
- All CLI flags documented in a table

### Create Examples

Every tool needs at least one runnable example in `examples/`:

```
tools/my-tool/examples/
├── example_basic.py      # Simplest usage
├── example_advanced.py   # Complex usage with options
└── README.md             # Explains each example
```

### Update CHANGELOG

```markdown
## [Unreleased]

### Added
- Initial implementation of my-tool
- CLI with --format and --verbose flags
- Support for JSON and CSV input
```

## Stage 6: Review & Release

1. Update `STATUS.md` → `status: review`
2. Self-review against the [Code Standards](../../.agents/rules/02-code-standards.md)
3. Run final checks: `make test-tool T=my-tool && make lint-tool T=my-tool`
4. Follow the [Release Skill](../../.agents/skills/release-tool/SKILL.md) for the release procedure

---

## Quick Reference: Make Targets

```bash
make new-tool NAME=x       # Scaffold a new tool
make test-tool T=x         # Test a specific tool
make lint-tool T=x         # Lint a specific tool
make test                  # Test ALL tools
make lint                  # Lint ALL tools
make catalog               # Regenerate tool catalog
make status                # Show SDLC dashboard
```

---

_[← Architecture](02-architecture.md) | [Chapter 04: Spec-Driven Development →](04-spec-driven-development.md)_
