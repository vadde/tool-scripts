# ADR-001: Monorepo Structure

## Status

✅ Accepted

## Date

2026-09-18

## Context

We need a repository structure to house a growing collection of engineering
tools, scripts, plugins, and workflows. These tools:

- Are written in multiple languages (Python, Go, Rust, Node.js, Shell)
- Range from simple scripts to complex GenAI applications
- Need independent versioning and release cycles
- Must be discoverable by both humans and AI agents
- Require consistent quality standards across all tools

We need to decide between a monorepo (all tools in one repository) and a
multi-repo (each tool in its own repository) approach.

## Options Considered

### Option 1: Monorepo (Single Repository)

- **Pros**:
  - Single place to discover all tools
  - Shared standards, CI/CD, and documentation
  - Atomic changes across multiple tools
  - Easier for AI agents to navigate (single context)
  - Shared tooling (Makefile, pre-commit, workflows)
  - Reduced overhead (one set of repo config)

- **Cons**:
  - Repository size grows over time
  - CI/CD needs path filtering to avoid running everything
  - Permissions are repo-wide (can't restrict per-tool)
  - Git history is shared across all tools

### Option 2: Multi-Repo (Per-Tool Repositories)

- **Pros**:
  - Clean separation of concerns
  - Independent CI/CD per tool
  - Fine-grained access control
  - Smaller, focused repositories

- **Cons**:
  - Tool discovery requires a separate catalog
  - Standards drift between repos
  - Duplicate CI/CD configuration
  - Harder for AI agents (multiple contexts)
  - More overhead per tool (repo setup, README, etc.)
  - Cross-tool changes require coordinated PRs

## Decision

**We chose the monorepo approach** with the following mitigations:

1. **Self-contained tool directories** — Each tool in `tools/<name>/` is
   independent and could be extracted to its own repo if needed
2. **Path-filtered CI/CD** — Workflows only run for changed tools
3. **Per-tool Makefiles** — Each tool has its own build/test commands
4. **SDLC status tracking** — Each tool has independent lifecycle tracking
5. **Canonical template** — New tools start from a shared template

## Consequences

### Positive
- All tools are discoverable in one place
- Shared standards are enforced consistently
- AI agents can navigate the entire ecosystem
- New tool creation is trivial (copy template)

### Negative
- Must maintain path filtering in CI/CD
- Repository will grow large over time
- All contributors have access to all tools

### Neutral
- Each tool still has independent versioning via `STATUS.md`
- Tools can be extracted to separate repos if needed in the future

## References

- [Monorepo best practices (2026)](https://monorepo.tools)
- Google's monorepo approach
- [Nx documentation on monorepo architecture](https://nx.dev)
