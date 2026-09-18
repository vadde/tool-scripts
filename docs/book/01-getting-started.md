# Chapter 01 — Getting Started

> _From zero to your first tool in under 10 minutes._

---

## Prerequisites

Before you begin, ensure you have the following installed:

### Required

| Tool | Version | Purpose | Install |
|------|---------|---------|---------|
| Git | 2.40+ | Version control | [git-scm.com](https://git-scm.com) |
| GNU Make | 3.81+ | Task runner | Pre-installed on macOS/Linux |

### Optional (Per Language)

| Language | Tools | Install |
|----------|-------|---------|
| Python | Python 3.10+, pip | [python.org](https://python.org) |
| Go | Go 1.22+ | [go.dev](https://go.dev) |
| Node.js | Node 20+, pnpm | [nodejs.org](https://nodejs.org) |
| Rust | Rust 1.75+, Cargo | [rustup.rs](https://rustup.rs) |
| Shell | Bash 4+, ShellCheck | Pre-installed / `brew install shellcheck` |

### Recommended

| Tool | Purpose | Install |
|------|---------|---------|
| pre-commit | Git hooks | `pip install pre-commit` |
| jq | JSON processing | `brew install jq` |

---

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/vadde/tool-scripts.git
cd tool-scripts
```

### 2. Explore the Structure

```bash
make help    # See all available commands
```

This shows you every target available in the root Makefile — your primary
interface for interacting with the repository.

### 3. Install Pre-Commit Hooks (Optional)

```bash
make setup
```

This installs pre-commit hooks that automatically check your code before
every commit.

---

## Your First Tool

Let's create a simple tool to get familiar with the workflow.

### Step 1: Scaffold

```bash
make new-tool NAME=hello-world
```

This creates:
- `tools/hello-world/` — Tool directory with all required files
- `specs/catalog/hello-world.md` — Specification file

### Step 2: Write the Spec

Edit `specs/catalog/hello-world.md`:

```markdown
## 1. Overview

### 1.1 Problem Statement
Engineers need a quick way to verify their tool-scripts setup is working.

### 1.2 Proposed Solution
A simple "hello world" tool that prints a greeting.

## 2. Requirements

| ID | Requirement | Priority | Status |
|----|------------|----------|--------|
| R-001 | Print a greeting message to stdout | Must | ⬜ |
| R-002 | Accept an optional --name flag | Should | ⬜ |
| R-003 | Exit with code 0 on success | Must | ⬜ |
```

### Step 3: Implement

Create `tools/hello-world/src/main.py`:

```python
"""Hello World — A simple greeting tool.

Implements the hello-world specification.
"""

import argparse
import sys


def greet(name: str = "World") -> str:
    """Generate a greeting message.

    Implements: R-001 (Print greeting), R-002 (Accept name)

    Args:
        name: The name to greet. Defaults to "World".

    Returns:
        The greeting string.
    """
    return f"Hello, {name}! 👋 Welcome to tool-scripts."


def main() -> int:
    """CLI entry point. Implements: R-003 (Exit code 0)."""
    parser = argparse.ArgumentParser(description="A simple greeting tool")
    parser.add_argument("--name", default="World", help="Name to greet")
    args = parser.parse_args()

    print(greet(args.name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

### Step 4: Test

Create `tools/hello-world/tests/test_main.py`:

```python
"""Tests for hello-world tool."""

from src.main import greet


def test_R001_greet_default_returns_hello_world():
    """R-001: Default greeting includes 'Hello, World!'."""
    result = greet()
    assert "Hello, World!" in result


def test_R002_greet_custom_name():
    """R-002: Custom name is used in greeting."""
    result = greet("Alice")
    assert "Alice" in result
```

### Step 5: Run

```bash
cd tools/hello-world
make test    # Run the tests
make run     # Run the tool
```

### Step 6: Commit

```bash
git add tools/hello-world specs/catalog/hello-world.md
git commit -m "feat(hello-world): add initial implementation

Implements R-001 (greeting), R-002 (--name flag), R-003 (exit code).
SDLC Status: in-progress"
```

---

## What's Next?

| Path | Go To |
|------|-------|
| Understand the repo structure | [Chapter 02: Architecture](02-architecture.md) |
| Deep-dive into tool building | [Chapter 03: Tool Development Guide](03-tool-development-guide.md) |
| Learn about specifications | [Chapter 04: Spec-Driven Development](04-spec-driven-development.md) |

---

_[← Preface](00-preface.md) | [Chapter 02: Architecture →](02-architecture.md)_
