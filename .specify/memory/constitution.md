<!--
Sync Impact Report
==================
Version change: 0.0.0 -> 1.0.0 (initial ratification)
Modified principles: N/A (initial version)
Added sections:
  - Core Principles (5 principles)
  - Architecture Constraints
  - Development Workflow
  - Governance
Removed sections: N/A
Templates requiring updates:
  - .specify/templates/plan-template.md: Constitution Check section exists, compatible
  - .specify/templates/spec-template.md: No constitution references, compatible
  - .specify/templates/tasks-template.md: No constitution references, compatible
Follow-up TODOs: None
-->

# lofi.nvim Constitution

> Governing principles for the lofi.nvim Neovim plugin and lofi-daemon backend

## Core Principles

### I. Zero-Latency Experience

The plugin MUST never block Neovim's main thread or delay editor responsiveness.

- Plugin initialization MUST complete in <10ms; all heavy operations defer to first use
- User commands MUST return immediately; long operations run asynchronously
- Progress feedback MUST be non-blocking (cmdline, notifications, or statusline)
- Startup MUST NOT require network, model loading, or daemon spawn

**Rationale:** Users expect their editor to remain responsive. Any perceptible lag
degrades the coding experience and erodes trust in the plugin.

### II. Local & Private

All functionality MUST operate entirely offline with zero data transmission.

- No API keys, accounts, or authentication MUST be required
- No network requests, telemetry, or usage analytics MUST be sent
- All model inference MUST run locally on user hardware
- Cache and configuration MUST remain on local filesystem only

**Rationale:** Developers deserve privacy while coding. A music generation tool
has no legitimate need to phone home or track usage patterns.

### III. Async-First Architecture

All computationally expensive operations MUST execute outside Neovim's main loop.

- Model inference MUST run in the separate lofi-daemon process
- Audio playback MUST be managed entirely by the daemon
- Plugin-to-daemon communication MUST use non-blocking JSON-RPC over stdio
- File I/O for cache operations MUST use vim.loop (libuv) async APIs

**Rationale:** Generation can take 60-90 seconds on CPU. The architecture must
ensure users can continue coding while music generates in the background.

### IV. Minimal Footprint

The plugin MUST maintain a small resource footprint and simple dependency tree.

- Frontend MUST be pure Lua with no external Lua dependencies
- Backend MUST compile to a single static binary (<15MB excluding model weights)
- Model weights MUST be optional and downloaded separately on demand
- Optional integrations (Telescope, fidget.nvim) MUST gracefully degrade when absent

**Rationale:** Neovim plugins should be lightweight. Heavy ML workloads belong
in the separate daemon; the Lua layer should remain thin and fast.

### V. Simplicity & Composability

Code MUST favor simplicity over abstraction, exposing a composable Lua API.

- YAGNI: Solve today's problems; do not build for hypothetical future needs
- Public API MUST be fully documented with type annotations
- All functionality available via commands MUST also be accessible via Lua API
- Events MUST be exposed for user scripting and integration with other plugins
- Avoid abstractions until three concrete use cases justify them

**Rationale:** Users should be able to script lofi.nvim into their workflows.
Over-engineering creates maintenance burden without delivering user value.

## Architecture Constraints

These constraints derive from the Core Principles and MUST be followed.

### Daemon Communication

- All daemon requests MUST include JSON-RPC `id` for response correlation
- Daemon MUST exit gracefully on stdin EOF (Neovim exit/crash)
- Daemon MUST support idle timeout for orphan prevention
- Daemon MUST write PID file for manual cleanup scenarios

### Error Handling

- Daemon errors MUST surface to Lua via `daemon_error` events
- Generation failures MUST NOT crash the daemon; report and continue
- Missing optional dependencies (lame, sox) MUST warn, not error
- Invalid configuration MUST fail loudly at setup() with clear messages

### Cache Management

- Cache format MUST be stable across patch versions
- Track IDs MUST be deterministic (hash of prompt + seed + duration + model_version)
- Cache cleanup MUST use LRU eviction when size limits exceeded
- Cache MUST be readable by multiple Neovim instances concurrently

## Development Workflow

### Testing Philosophy

- Integration tests MUST verify end-to-end daemon-plugin communication
- Contract tests MUST validate JSON-RPC request/response schemas
- Tests are written when explicitly requested or when touching critical paths
- Health check (`:checkhealth lofi`) MUST cover all runtime dependencies

### Code Review Standards

- Changes MUST pass existing tests before merge
- New daemon RPC methods MUST include contract tests
- Configuration changes MUST update both Lua validation and documentation
- Breaking changes MUST be documented in CHANGELOG with migration guidance

### Documentation Requirements

- Public Lua API MUST have LuaCATS type annotations
- User commands MUST be documented in vimdoc (doc/lofi.txt)
- JSON-RPC protocol MUST be documented in design.md
- README MUST include quickstart, installation, and basic configuration

## Governance

### Amendment Process

1. Propose changes via pull request modifying this constitution
2. Document rationale for principle additions, modifications, or removals
3. Update dependent templates if principle changes affect their guidance
4. Increment version according to semantic versioning rules below

### Versioning Policy

- **MAJOR**: Backward-incompatible governance changes or principle removal
- **MINOR**: New principle added or existing principle materially expanded
- **PATCH**: Clarifications, typo fixes, non-semantic refinements

### Compliance Review

- All PRs SHOULD reference relevant principles when making architectural decisions
- Constitution violations MUST be resolved before merge
- Complexity additions MUST be justified against Principle V (Simplicity)

**Version**: 1.0.0 | **Ratified**: 2025-12-19 | **Last Amended**: 2025-12-19
