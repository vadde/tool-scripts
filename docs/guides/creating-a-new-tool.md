# Guide: Creating a New Tool

> Step-by-step guide to creating a new tool in the repository.

---

## Quick Start

```bash
make new-tool NAME=my-tool-name
```

That's it for scaffolding! Now fill in the details.

---

## Detailed Steps

### 1. Choose a Name

- Use **kebab-case**: `json-validator`, `csv-merger`, `token-counter`
- Be descriptive but concise
- Check `tools/README.md` to avoid duplicates

### 2. Scaffold

```bash
make new-tool NAME=my-tool
```

This creates:
```
tools/my-tool/
├── README.md
├── spec.md
├── STATUS.md      ← status: draft
├── CHANGELOG.md
├── Makefile
├── src/
├── tests/
└── examples/
```

And: `specs/catalog/my-tool.md`

### 3. Write the Specification

Edit `specs/catalog/my-tool.md`:

1. Fill in the Overview (problem statement, solution, audience)
2. List all functional requirements (R-001, R-002, ...)
3. Define the interface contract (CLI, API)
4. Write acceptance criteria
5. List dependencies

### 4. Set Up the Language

Edit `tools/my-tool/Makefile` — uncomment the lines for your language.

Create language-specific config files:

| Language | Create |
|----------|--------|
| Python | `pyproject.toml`, `src/__init__.py` |
| Go | `go.mod` |
| Node.js | `package.json`, `tsconfig.json` |
| Rust | `Cargo.toml` |
| Shell | Just write `src/main.sh` |

### 5. Implement

Write your code in `src/`, referencing spec requirements:

```python
def my_function(data: str) -> str:
    """Implements: R-001 (Description from spec)."""
    ...
```

### 6. Test

Write tests in `tests/`, referencing acceptance criteria:

```python
def test_R001_description():
    """AC-001: Given X, when Y, then Z."""
    ...
```

### 7. Document

- Fill in all sections of `tools/my-tool/README.md`
- Create at least one example in `examples/`
- Update `CHANGELOG.md`

### 8. Update Status

Edit `tools/my-tool/STATUS.md`:
```yaml
status: in-progress
```

### 9. Commit

```bash
git add tools/my-tool specs/catalog/my-tool.md
git commit -m "feat(my-tool): initial implementation"
```

---

## Checklist

- [ ] Tool name is kebab-case
- [ ] `specs/catalog/my-tool.md` has numbered requirements
- [ ] `tools/my-tool/STATUS.md` is set correctly
- [ ] `tools/my-tool/Makefile` has working `test` and `lint` targets
- [ ] At least one test exists
- [ ] At least one example exists
- [ ] `tools/README.md` catalog is updated
- [ ] Commit follows conventional format
