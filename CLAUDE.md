# lofi.nvim Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-19

## Active Technologies
- Rust 1.75+ (edition 2021), Lua 5.1 (LuaJIT/Neovim) + ort 2.0.0-rc.9 (ONNX Runtime), tokio, serde/serde_json, ndarray, tokenizers (002-ace-step)
- File-based cache at `~/.cache/lofi.nvim/` (tracks + model weights) (002-ace-step)

- Rust 1.75+ (edition 2021) (001-musicgen-onnx)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+ (edition 2021): Follow standard conventions

## Recent Changes
- 002-ace-step: Added Rust 1.75+ (edition 2021), Lua 5.1 (LuaJIT/Neovim) + ort 2.0.0-rc.9 (ONNX Runtime), tokio, serde/serde_json, ndarray, tokenizers

- 001-musicgen-onnx: Added Rust 1.75+ (edition 2021)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
