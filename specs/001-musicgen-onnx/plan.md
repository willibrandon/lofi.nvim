# Implementation Plan: AI Music Generation via MusicGen ONNX Backend

**Branch**: `001-musicgen-onnx` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-musicgen-onnx/spec.md`

## Summary

Implement AI-powered music generation using MusicGen-small with ONNX Runtime inference in the lofi-daemon Rust backend. The feature follows a proven architecture from MusicGPT: three ONNX models (text encoder, decoder with KV cache, audio codec) orchestrated through a job queue with progress streaming. Phase 0 validates feasibility with a standalone CLI before Neovim integration.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- `ort 2.0.0-rc.9` - ONNX Runtime Rust bindings with fp16/ndarray features
- `ndarray 0.16.1` - Tensor shape manipulation
- `tokenizers 0.19.1` - HuggingFace tokenizer for text encoding
- `half 2.4.1` - Float16 support for fp16 models
- `hound 3.5.1` - WAV file encoding
- `reqwest` - Model download from HuggingFace

**Storage**: Local filesystem cache (`~/.cache/nvim/lofi/` or platform equivalent)
**Testing**: `cargo test` with unit tests for tensor ops, integration tests for model pipeline
**Target Platform**: macOS (Apple Silicon primary), Linux x64, Windows x64
**Project Type**: Single daemon binary with embedded CLI for Phase 0
**Performance Goals**: 10s audio on CPU in <2 minutes; 30s audio in 60-90s; GPU >2x speedup
**Constraints**: <500MB model files (fp16), <2GB peak memory, offline after model download
**Scale/Scope**: Single user, serial generation (1 at a time), queue of 10 max

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Zero-Latency Experience | PASS | Generation runs in daemon; Lua plugin returns track_id immediately |
| II. Local & Private | PASS | All inference local; network only for one-time model download with consent |
| III. Async-First Architecture | PASS | Daemon handles inference; JSON-RPC non-blocking; job queue with progress streaming |
| IV. Minimal Footprint | PASS | Single binary; models downloaded separately on demand (~250MB fp16) |
| V. Simplicity & Composability | PASS | Reusing proven MusicGPT patterns; Lua API mirrors JSON-RPC |
| VI. Complete Implementation | MONITOR | Phase 0 is explicit go/no-go gate; no placeholders permitted |
| VII. No Fallback Implementations | PASS | Phase 0 proves primary approach; no speculative fallbacks |

**Gate Result**: PASS - All principles satisfied or have explicit mitigation.

## Project Structure

### Documentation (this feature)

```text
specs/001-musicgen-onnx/
├── plan.md              # This file
├── research.md          # Phase 0 output: MusicGPT patterns, ort usage
├── data-model.md        # Phase 1 output: Track, Request, Model entities
├── quickstart.md        # Phase 1 output: Phase 0 CLI validation steps
├── contracts/           # Phase 1 output: JSON-RPC schemas
│   ├── generate.json
│   ├── progress.json
│   └── errors.json
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
daemon/
├── Cargo.toml
└── src/
    ├── main.rs                    # Entry point, daemon/CLI mode detection
    ├── cli.rs                     # Phase 0 standalone CLI for validation
    ├── config.rs                  # Device, threads, model path configuration
    ├── rpc/
    │   ├── mod.rs
    │   ├── server.rs              # JSON-RPC over stdio handler
    │   ├── methods.rs             # generate, get_status, etc.
    │   └── types.rs               # Request/Response structs
    ├── generation/
    │   ├── mod.rs
    │   ├── queue.rs               # Job queue with priority, max 10 pending
    │   ├── job.rs                 # GenerationJob state machine
    │   └── progress.rs            # Progress calculation and notification
    ├── models/
    │   ├── mod.rs
    │   ├── loader.rs              # ONNX session loading with device selection
    │   ├── downloader.rs          # HuggingFace model download with consent
    │   ├── text_encoder.rs        # Tokenization + T5 encoding
    │   ├── decoder.rs             # Autoregressive generation with KV cache
    │   ├── audio_codec.rs         # EnCodec token → audio decoding
    │   └── delay_pattern.rs       # 4-codebook delay pattern masking
    ├── audio/
    │   ├── mod.rs
    │   └── wav.rs                 # WAV file writing (hound)
    └── cache/
        ├── mod.rs
        └── tracks.rs              # Track storage and hash-based deduplication

lua/
└── lofi/
    ├── init.lua                   # Public API: generate(), is_generating()
    ├── daemon.lua                 # Daemon spawn/communication
    ├── rpc.lua                    # JSON-RPC client
    └── events.lua                 # generation_start/progress/complete/error

tests/
├── contract/
│   └── json_rpc_test.rs           # JSON-RPC schema validation
├── integration/
│   └── generation_test.rs         # End-to-end generation (requires models)
└── unit/
    ├── delay_pattern_test.rs
    ├── queue_test.rs
    └── progress_test.rs
```

**Structure Decision**: Single daemon binary with modular internal structure. The `generation/` module orchestrates `models/` components. Lua plugin in `lua/lofi/` communicates via JSON-RPC. Phase 0 CLI embedded in daemon binary for validation without Neovim.

## Complexity Tracking

> No violations requiring justification. Architecture follows MusicGPT proven patterns.

## Phase 0: Research Summary

Research completed via MusicGPT codebase analysis. Key findings documented in [research.md](./research.md).

## Phase 1: Design Artifacts

Design artifacts generated:
- [data-model.md](./data-model.md) - Entity definitions
- [contracts/](./contracts/) - JSON-RPC schemas
- [quickstart.md](./quickstart.md) - Phase 0 validation steps
