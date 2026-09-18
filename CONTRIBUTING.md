# Contributing to tool-scripts

Thank you for contributing to **tool-scripts**! This guide will help you get
started quickly and ensure your contributions meet our standards.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Commit Conventions](#commit-conventions)
- [Pull Request Process](#pull-request-process)
- [Style Guide](#style-guide)

---

## Code of Conduct

By participating in this project, you agree to abide by our
[Code of Conduct](CODE_OF_CONDUCT.md).

---

## Getting Started

### Prerequisites

- Git 2.40+
- GNU Make 3.81+
- Language toolchains as needed (Python 3.10+, Go 1.22+, Node 20+, Rust 1.75+)

### Setup

```bash
git clone https://github.com/vadde/tool-scripts.git
cd tool-scripts
make setup  # Installs pre-commit hooks
```

---

## Development Workflow

We follow **Spec-Driven Development (SDD)**. The workflow is:

### 1. Write the Spec First

Before writing any code, create or update the specification:

```bash
# For a new tool
make new-tool NAME=my-tool
# Edit the spec
$EDITOR specs/catalog/my-tool.md
```

### 2. Branch from `develop`

```bash
git checkout develop
git pull origin develop
git checkout -b feat/my-tool/initial-implementation
```

### 3. Implement

- Write code in `tools/my-tool/src/`
- Write tests in `tools/my-tool/tests/`
- Create examples in `tools/my-tool/examples/`

### 4. Test

```bash
make test-tool T=my-tool
make lint-tool T=my-tool
```

### 5. Document

- Fill in all sections of `tools/my-tool/README.md`
- Update `tools/my-tool/CHANGELOG.md`
- Update `tools/my-tool/STATUS.md`

### 6. Submit PR

```bash
git add .
git commit -m "feat(my-tool): initial implementation"
git push origin feat/my-tool/initial-implementation
```

---

## Commit Conventions

We use **[Conventional Commits](https://www.conventionalcommits.org/)**.

### Format

```
type(scope): description

[optional body]

[optional footer]
```

### Types

| Type | Purpose |
|------|---------|
| `feat` | New feature or tool |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `test` | Test additions/changes |
| `refactor` | Code restructuring |
| `chore` | Maintenance tasks |
| `ci` | CI/CD changes |
| `style` | Code style/formatting |
| `perf` | Performance improvements |

### Scope

The scope is the **tool name** or a repo area (`docs`, `ci`, `agents`, `repo`).

### Examples

```bash
feat(json-validator): add schema validation support
fix(csv-merger): handle empty rows gracefully
docs(book): add chapter on advanced patterns
test(file-hasher): add edge case tests for empty files
chore(deps): update ruff to v0.7.0
```

---

## Branch Naming

```
feat/<tool>/<description>      # New feature
fix/<tool>/<description>       # Bug fix
docs/<description>              # Documentation
refactor/<tool>/<description>  # Refactoring
test/<tool>/<description>      # Tests
chore/<description>             # Maintenance
```

---

## Pull Request Process

1. **Fill in the PR template** — All sections are required
2. **Link to the spec** — Reference `specs/catalog/<tool>.md`
3. **Ensure CI passes** — All tests and lints must pass
4. **Update documentation** — README, CHANGELOG, STATUS
5. **Keep it small** — PRs over 500 lines should be split

### PR Checklist

- [ ] Spec reference included
- [ ] Tests written and passing
- [ ] Lint passing
- [ ] README and examples updated
- [ ] CHANGELOG entry added
- [ ] STATUS.md updated
- [ ] No secrets in the diff
- [ ] PR title follows conventional commit format

---

## Style Guide

We support multiple languages. See the full
[Code Standards](.agents/rules/02-code-standards.md) for details.

### Universal Rules

1. Every public function has a docstring/comment
2. Every tool has tests (≥80% coverage target)
3. Every tool has at least one runnable example
4. No hardcoded secrets or file paths
5. Consistent naming within each language's conventions

### Language References

| Language | Style Guide | Formatter | Linter |
|----------|------------|-----------|--------|
| Python | Google Style | `ruff format` | `ruff` |
| Go | Effective Go | `gofmt` | `golangci-lint` |
| Node.js | Standard | `prettier` | `eslint` |
| Rust | Rust API Guidelines | `rustfmt` | `clippy` |
| Shell | Google Shell Guide | — | `shellcheck` |

---

## Questions?

- Read the [Documentation](docs/README.md)
- Check the [Roadmap](sdlc/ROADMAP.md)
- Open an [Issue](.github/ISSUE_TEMPLATE/config.yml)

---

Thank you for helping make **tool-scripts** better! 🚀
