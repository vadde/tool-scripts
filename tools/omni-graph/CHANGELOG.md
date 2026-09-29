# Changelog

All notable changes to this tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Fixed duplicate and uncanonical workspace listings in `/api/workspaces` by coalescing legacy `workspace` into `tool-scripts` and merging AST node/file counts.
- Filtered out empty, `default`, and `global` pseudo-workspaces from API output.
- Hardened React UI `loadWorkspaces` with client-side Set deduplication and filter checks.

### Added
- Initial tool scaffolding with full SDD specification
- Project genesis: 24 functional requirements, 14 acceptance criteria, 11 NFRs
- Research integration from Graphify, CodeGraph, Serena, ECC reference architectures
