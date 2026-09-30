# Changelog

All notable changes to this tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Fixed critical frontend initialization failure on initial landing where `initLiveWatch` was nested inside `initEventListeners()`, causing an unhandled `ReferenceError: initLiveWatch is not defined` that blocked `loadSessions()` from running.
- Defaulted initial date filter to Last 7 Days (`7d`) with pre-filled inputs and active chip highlight, eliminating the stuck "Loading..." badge on landing and allowing smooth toggling to "All Time" and custom dates.
- Initialized `groupBy` in app state from `localStorage` to ensure consistent active state on the segmented control.
- Re-synchronized date preset chip active states dynamically upon programmatic preset switches and manual input clearance.
- Fixed critical workspace accumulation bug where `idx.stats` was not reinitialized during background rescans, causing workspaces list to balloon to 3,000+ entries with duplicates.
- Fixed workspace path leakage where filenames (`all_mermaids.txt`), usernames (`aparv`), and internal subfolders (`_templates`, `docs`, `scripts`, `rules`, `analytics`) were mistakenly recognized as workspaces.
- Added strict `isValidWorkspaceName` filter rejecting non-workspace paths, files with dots, and system directories (`library`, `applications`, `system`, `volumes`).
- Added client-side Set deduplication and change detection in `renderWorkspaceOptions` to eliminate UI glitches and duplicate options.
- Added unit tests `TestRescanStatsDeduplication` and `TestExtractProjectName_Sanitization`.

### Added
- Initial tool scaffolding
