# Rule 02 — Code Standards

> **Priority**: 🔴 Critical — All code in this repository must follow these standards.

---

## Universal Standards (All Languages)

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Tool directories | `kebab-case` | `json-validator`, `csv-merger` |
| Source files | Language convention | See per-language sections |
| Constants | `UPPER_SNAKE_CASE` | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| Config keys | `snake_case` | `output_format`, `max_workers` |

### File Organization

Every tool in `tools/<name>/` MUST have this structure:

```
tools/<name>/
├── README.md           # REQUIRED — Tool documentation
├── spec.md             # REQUIRED — Specification
├── STATUS.md           # REQUIRED — SDLC status
├── CHANGELOG.md        # REQUIRED — Version history
├── Makefile            # REQUIRED — Tool-local make targets
├── src/                # REQUIRED — Source code
│   └── main.<ext>      # Entry point (name per language convention)
├── tests/              # REQUIRED — Test suite
│   └── test_main.<ext> # Tests (name per language convention)
└── examples/           # REQUIRED — Runnable examples
    └── example_*.<ext> # At least one example
```

### Error Handling

1. **Never silently swallow errors** — Always log or propagate
2. **Use structured error types** — Not raw strings
3. **Include context** — What operation failed, what was the input
4. **Fail fast** — Validate inputs at the boundary, not deep inside
5. **Exit codes** — CLI tools must use meaningful exit codes:
   - `0` — Success
   - `1` — General error
   - `2` — Usage/argument error
   - `3` — Configuration error
   - `4` — Dependency error

### Logging

1. **Use structured logging** where possible (JSON format for production tools)
2. **Log levels**: `DEBUG`, `INFO`, `WARN`, `ERROR`
3. **Never log secrets** — Sanitize sensitive data before logging
4. **Include timestamps** and context in log messages

### Documentation Requirements

1. **Every public function/method** must have a docstring/comment
2. **Every module/file** must have a header comment explaining its purpose
3. **Complex algorithms** must have inline comments explaining the logic
4. **Magic numbers** must be replaced with named constants

---

## Python Standards

| Aspect | Standard |
|--------|----------|
| Version | 3.10+ |
| Formatter | `ruff format` (Black-compatible) |
| Linter | `ruff` |
| Type checker | `mypy` (strict mode) |
| Test framework | `pytest` |
| Dependency management | `pyproject.toml` (PEP 621) |
| Import style | `isort`-compatible (via `ruff`) |

### Python File Structure

```python
"""
Module docstring — one-line summary.

Detailed description of what this module does and why.
"""

# Standard library imports
import os
import sys

# Third-party imports
import numpy as np

# Local imports
from .utils import helper_function


# Constants
MAX_RETRIES = 3
DEFAULT_TIMEOUT = 30


def public_function(param: str, count: int = 0) -> list[str]:
    """One-line summary of what this function does.

    Args:
        param: Description of param.
        count: Description of count. Defaults to 0.

    Returns:
        Description of return value.

    Raises:
        ValueError: When param is empty.
    """
    ...
```

### Python `pyproject.toml` Template

```toml
[project]
name = "tool-name"
version = "0.1.0"
description = "Brief description"
requires-python = ">=3.10"
dependencies = []

[project.optional-dependencies]
dev = ["pytest>=8.0", "ruff>=0.6", "mypy>=1.11"]

[tool.ruff]
target-version = "py310"
line-length = 88

[tool.ruff.lint]
select = ["E", "F", "W", "I", "N", "UP", "B", "A", "SIM", "RUF"]

[tool.pytest.ini_options]
testpaths = ["tests"]
```

---

## Go Standards

| Aspect | Standard |
|--------|----------|
| Version | 1.22+ |
| Formatter | `gofmt` / `goimports` |
| Linter | `golangci-lint` |
| Test framework | `testing` (stdlib) + `testify` for assertions |
| Module | `go.mod` per tool |

### Go File Structure

```go
// Package toolname provides a brief description.
//
// Detailed description of the package purpose and usage.
package toolname

import (
    "context"
    "fmt"

    "github.com/external/dependency"
)

// MaxRetries is the maximum number of retry attempts.
const MaxRetries = 3

// Config holds the tool configuration.
type Config struct {
    OutputFormat string `json:"output_format"`
    MaxWorkers   int    `json:"max_workers"`
}

// Process performs the main operation.
// It returns the result or an error if processing fails.
func Process(ctx context.Context, input string) (string, error) {
    ...
}
```

---

## Node.js / TypeScript Standards

| Aspect | Standard |
|--------|----------|
| Runtime | Node.js 20+ / Bun |
| Language | TypeScript preferred, JavaScript acceptable |
| Formatter | `prettier` |
| Linter | `eslint` with `@typescript-eslint` |
| Test framework | `vitest` |
| Package manager | `pnpm` preferred, `npm` acceptable |

### TypeScript File Structure

```typescript
/**
 * Module description — one-line summary.
 *
 * Detailed description of what this module does.
 * @module tool-name
 */

import { type Config } from './types.js';

/** Maximum number of retry attempts. */
const MAX_RETRIES = 3;

/**
 * Process the input and return the result.
 *
 * @param input - The input string to process.
 * @param config - Optional configuration overrides.
 * @returns The processed result.
 * @throws {ValidationError} When input is invalid.
 */
export function process(input: string, config?: Partial<Config>): string {
  ...
}
```

---

## Rust Standards

| Aspect | Standard |
|--------|----------|
| Edition | 2021+ |
| Formatter | `rustfmt` |
| Linter | `clippy` (pedantic) |
| Test framework | Built-in `#[test]` + `proptest` for property testing |
| Dependency management | `Cargo.toml` |

---

## Shell / Bash Standards

| Aspect | Standard |
|--------|----------|
| Shell | Bash 4+ (use `#!/usr/bin/env bash`) |
| Linter | `shellcheck` |
| Style | Google Shell Style Guide |

### Shell Script Template

```bash
#!/usr/bin/env bash
# ===========================================================================
# script-name.sh — Brief description of what this script does
#
# Usage:
#   ./script-name.sh [OPTIONS] <arg1> <arg2>
#
# Options:
#   -h, --help     Show help message
#   -v, --verbose  Enable verbose output
#
# Dependencies:
#   - jq (>= 1.6)
#   - curl
# ===========================================================================

set -euo pipefail
IFS=$'\n\t'

# ─── Constants ──────────────────────────────────────────────────────────────
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"

# ─── Functions ──────────────────────────────────────────────────────────────

# Print an error message to stderr and exit.
# Arguments:
#   $1 - Error message
#   $2 - Exit code (default: 1)
die() {
  echo "ERROR: ${1}" >&2
  exit "${2:-1}"
}

# Print usage information.
usage() {
  sed -n '/^# Usage:/,/^# =====/p' "${BASH_SOURCE[0]}" | head -n -1 | sed 's/^# //'
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
  # Parse arguments
  # Validate inputs
  # Execute logic
  echo "Hello from ${SCRIPT_NAME}"
}

main "$@"
```

---

## Code Review Checklist (Agents Must Self-Review)

Before considering ANY code complete:

- [ ] All public functions have docstrings/comments
- [ ] All error paths are handled
- [ ] All inputs are validated
- [ ] No hardcoded secrets or paths
- [ ] No unused imports or variables
- [ ] Consistent naming conventions
- [ ] Tests cover the happy path AND edge cases
- [ ] Examples in `examples/` are runnable
- [ ] `CHANGELOG.md` is updated
- [ ] `STATUS.md` reflects current state
