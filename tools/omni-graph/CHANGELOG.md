# Changelog

All notable changes to this tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full Go AST grammar extraction (`tree-sitter-go`): methods (`method_declaration`) with receiver `DECLARES` edge linking, structs and interfaces (`type_spec`), constants/enums (`const_spec`), and package imports (`import_spec`).

### Fixed
- Fixed ingestion bottleneck on large datasets/dumps by pruning non-code directories (`solutions`, `data`, `logs`), applying 512KB file limit, and capping fallback blocks.
- Fixed SurrealDB v2 HNSW delete timeout during workspace re-indexing by temporarily detaching the vector index during bulk record purges.
- Fixed Axum synchronous ingestion cancellation by running ingestion pipelines in detached Tokio tasks.
- Fixed duplicate and uncanonical workspace listings in `/api/workspaces` by coalescing legacy `workspace` into `tool-scripts` and merging AST node/file counts.
- Filtered out empty, `default`, and `global` pseudo-workspaces from API output.
- Hardened React UI `loadWorkspaces` with client-side Set deduplication and filter checks.

### Added
- Initial tool scaffolding with full SDD specification
- Project genesis: 24 functional requirements, 14 acceptance criteria, 11 NFRs
- Research integration from Graphify, CodeGraph, Serena, ECC reference architectures
