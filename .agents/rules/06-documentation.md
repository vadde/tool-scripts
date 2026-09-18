# Rule 06 — Documentation Standards

> **Priority**: 🟡 Important — If it's not documented, it doesn't exist.

---

## Documentation Hierarchy

```
Level 1: README.md (per tool)     — "What is this and how do I use it?"
Level 2: spec.md (per tool)       — "What exactly does this do?"
Level 3: Code comments/docstrings — "How does this work internally?"
Level 4: examples/ (per tool)     — "Show me it working"
Level 5: docs/book/ (repo-wide)   — "Teach me everything"
Level 6: docs/architecture/ (ADRs)— "Why did we decide this?"
```

---

## Tool README Template

Every tool's `README.md` MUST include these sections:

```markdown
# Tool Name

> One-line description of what this tool does.

[![Status](badge)](#) [![Version](badge)](#) [![Language](badge)](#)

## Overview

2-3 sentence description of the problem this tool solves and how it solves it.

## Installation

Step-by-step installation instructions. Include ALL dependencies.

## Usage

### CLI

\`\`\`bash
tool-name [OPTIONS] <input>
\`\`\`

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output` | Output format | `text` |
| `-v, --verbose` | Enable verbose logging | `false` |

### As a Library

\`\`\`python
from tool_name import process
result = process(input_data)
\`\`\`

## Examples

Link to and describe each example in `examples/`.

## Configuration

Document any configuration files or environment variables.

## Specification

See [spec.md](spec.md) for the full technical specification.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

MIT — See [LICENSE](../../LICENSE) for details.
```

---

## Changelog Format (Keep-a-Changelog)

```markdown
# Changelog

All notable changes to this tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New features

### Changed
- Changes in existing functionality

### Deprecated
- Features that will be removed

### Removed
- Features that were removed

### Fixed
- Bug fixes

### Security
- Vulnerability fixes

## [0.1.0] - 2026-XX-XX

### Added
- Initial implementation
```

---

## Code Documentation Standards

### Python — Docstrings (Google Style)

```python
def process(data: list[dict], config: Config | None = None) -> Result:
    """Process input data according to the specified configuration.

    This function applies transformation rules from the config to each
    item in the data list, producing an aggregated result.

    Implements: R-003 (Data processing pipeline)

    Args:
        data: List of dictionaries containing input records.
            Each dict must have 'id' and 'value' keys.
        config: Optional processing configuration. If None,
            uses default settings.

    Returns:
        A Result object containing processed records and metadata.

    Raises:
        ValueError: If data is empty or contains invalid records.
        ConfigError: If config has conflicting settings.

    Example:
        >>> result = process([{"id": 1, "value": 42}])
        >>> result.count
        1
    """
```

### Go — GoDoc Comments

```go
// Process applies transformation rules to the input data.
//
// It iterates through each record, applying the configuration rules,
// and returns an aggregated Result. If config is nil, default settings
// are used.
//
// Implements: R-003 (Data processing pipeline)
func Process(data []Record, config *Config) (*Result, error) {
```

### TypeScript — TSDoc Comments

```typescript
/**
 * Process input data according to the specified configuration.
 *
 * Applies transformation rules from the config to each item in the
 * data array, producing an aggregated result.
 *
 * @implements R-003 (Data processing pipeline)
 *
 * @param data - Array of input records to process.
 * @param config - Optional processing configuration.
 * @returns Processed result with metadata.
 * @throws {ValidationError} If data is empty or invalid.
 *
 * @example
 * ```ts
 * const result = process([{ id: 1, value: 42 }]);
 * console.log(result.count); // 1
 * ```
 */
```

---

## Architecture Decision Records (ADRs)

### When to Write an ADR

Write an ADR when:
- Choosing a technology, library, or framework
- Establishing a pattern or convention
- Making a trade-off that affects multiple tools
- Changing an existing architectural decision

### ADR Template

See `docs/architecture/adr-template.md` for the full template.

```markdown
# ADR-XXX: Title

## Status
Proposed | Accepted | Deprecated | Superseded by ADR-YYY

## Context
What is the issue that we're seeing that is motivating this decision?

## Decision
What is the change that we're proposing and/or doing?

## Consequences
What becomes easier or harder as a result of this change?
```

---

## Documentation Quality Checklist

- [ ] README has all required sections
- [ ] All CLI flags/options documented
- [ ] All public API functions documented
- [ ] At least one runnable example exists
- [ ] CHANGELOG is up to date
- [ ] No broken links
- [ ] No placeholder text (TODO, TBD, etc.)
- [ ] Screenshots/diagrams for visual tools
- [ ] Installation instructions are complete and tested
