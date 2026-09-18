# Tool Name

> One-line description of what this tool does.

<!-- Badges: update these when the tool is released -->
<!-- ![Status](https://img.shields.io/badge/status-draft-yellow) -->
<!-- ![Version](https://img.shields.io/badge/version-0.1.0-blue) -->
<!-- ![Language](https://img.shields.io/badge/language-python-green) -->

---

## Overview

_2-3 sentence description of the problem this tool solves and how it solves it._

## Installation

### Prerequisites

- _List prerequisites (e.g., Python 3.10+, Go 1.22+)_
- _List external dependencies_

### Setup

```bash
# Clone the repository (if not already done)
git clone https://github.com/vadde/tool-scripts.git
cd tool-scripts/tools/<tool-name>

# Install dependencies
make setup
```

## Usage

### CLI

```bash
# Basic usage
./src/main.py <input>

# With options
./src/main.py --output json --verbose input.txt
```

### Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--output` | `-o` | Output format | `text` |
| `--verbose` | `-v` | Verbose logging | `false` |
| `--help` | `-h` | Show help | — |
| `--version` | | Show version | — |

### As a Library

```python
from tool_name import process

result = process(input_data)
print(result)
```

## Examples

See the [`examples/`](examples/) directory:

| Example | Description |
|---------|-------------|
| `example_basic.py` | Basic usage demonstration |

## Configuration

_Document any configuration files, environment variables, or settings._

## Specification

See [spec.md](spec.md) for the full technical specification.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Development

```bash
make test    # Run tests
make lint    # Run linters
make run     # Run the tool
make help    # Show all targets
```

## License

MIT — See [LICENSE](../../LICENSE) for details.
