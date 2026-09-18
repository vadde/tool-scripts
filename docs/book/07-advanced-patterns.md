# Chapter 07 — Advanced Patterns

> _Beyond the basics: GenAI tools, plugin architectures, and tool composition._

---

## GenAI & Machine Learning Tools

This repository is designed to house tools across the entire GenAI spectrum:

### Tool Categories

| Category | Examples | Typical Language |
|----------|----------|-----------------|
| **Prompt Engineering** | Template engines, prompt validators | Python, TypeScript |
| **Model Evaluation** | Benchmark suites, quality metrics | Python |
| **Data Pipelines** | Preprocessing, embedding generation | Python, Go |
| **LLM Utilities** | Token counters, cost estimators | Python, Rust |
| **Agent Frameworks** | Tool orchestrators, workflow engines | Python, TypeScript |
| **RAG Components** | Chunkers, retrievers, rankers | Python |

### Spec Considerations for GenAI Tools

GenAI tools have unique requirements:

```markdown
## Non-Functional Requirements

| ID | Requirement | Metric |
|----|------------|--------|
| NF-001 | Latency | < 200ms p99 for sync calls |
| NF-002 | Cost | < $0.01 per invocation |
| NF-003 | Determinism | Same input → same output (where applicable) |
| NF-004 | Token efficiency | Minimize token usage |
```

---

## Plugin Architecture

Tools can be extended via plugins. The pattern:

```
tools/plugin-host/
├── src/
│   ├── main.py
│   └── plugin_loader.py     # Dynamic plugin discovery
├── plugins/                  # Plugin directory
│   ├── plugin_a.py
│   └── plugin_b.py
└── plugin_spec.md            # Plugin interface contract
```

### Plugin Interface Contract

```python
"""All plugins must implement this interface."""

from abc import ABC, abstractmethod

class PluginBase(ABC):
    """Base class for all plugins."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Plugin name."""

    @property
    @abstractmethod
    def version(self) -> str:
        """Plugin version (SemVer)."""

    @abstractmethod
    def execute(self, input_data: dict) -> dict:
        """Execute the plugin's main logic."""

    def validate(self, input_data: dict) -> bool:
        """Optional: validate input before execution."""
        return True
```

---

## Tool Composition

Tools can depend on and compose with each other:

```
tools/data-pipeline/
├── spec.md
│   └── Dependencies:
│       ├── tools/csv-parser (R-001)
│       └── tools/json-formatter (R-003)
```

### Composition Patterns

1. **CLI Piping**: `tool-a | tool-b | tool-c`
2. **Library Import**: Tool B imports from Tool A's `src/`
3. **Config Reference**: Tool B references Tool A's output format
4. **Workflow**: An orchestrator tool calls multiple tools in sequence

---

## Property-Based Testing for Mathematical Tools

Mathematical tools benefit from property-based testing:

```python
from hypothesis import given, strategies as st

@given(st.lists(st.floats(allow_nan=False, allow_infinity=False), min_size=1))
def test_mean_is_bounded(data):
    """The mean must be between min and max of the data."""
    result = compute_mean(data)
    assert min(data) <= result <= max(data)

@given(st.integers(), st.integers())
def test_addition_is_commutative(a, b):
    """a + b must equal b + a."""
    assert add(a, b) == add(b, a)
```

---

## Performance Optimization Patterns

For performance-critical tools:

| Pattern | When to Use | Example |
|---------|------------|---------|
| Streaming | Large files (> 1GB) | Process line-by-line |
| Parallelism | CPU-bound + many cores | `multiprocessing`, goroutines |
| Caching | Repeated computations | `functools.lru_cache`, memoization |
| Lazy evaluation | Don't need all results | Generators, iterators |
| Native extensions | Hot inner loops | Rust/C via FFI |

---

## Multi-Language Tool Patterns

Some tools benefit from multiple languages:

```
tools/hybrid-tool/
├── src/
│   ├── core.rs           # Performance-critical core in Rust
│   ├── bindings.py       # Python bindings via PyO3
│   └── cli.py            # User-facing CLI in Python
├── Cargo.toml
└── pyproject.toml
```

---

_[← CI/CD and Releases](06-ci-cd-and-releases.md) | [Chapter 08: Appendix →](08-appendix.md)_
