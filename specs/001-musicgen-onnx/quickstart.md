# Quickstart: Phase 0 Validation

**Feature**: 001-musicgen-onnx
**Purpose**: Validate MusicGen ONNX feasibility before Neovim integration

## Overview

Phase 0 proves the core hypothesis: MusicGen-small can generate audio via ONNX Runtime in Rust, meeting the performance target of 10 seconds of audio in under 2 minutes on CPU.

## Prerequisites

- Rust 1.75+ installed (`rustup update stable`)
- ~4GB RAM available
- ~500MB disk space for models
- Internet connection (first run only, for model download)

## Step 1: Create Phase 0 CLI Project

```bash
# From repository root
mkdir -p daemon
cd daemon
cargo init --name lofi-daemon
```

Add to `Cargo.toml`:
```toml
[dependencies]
ort = { version = "2.0.0-rc.9", features = ["half", "ndarray"] }
ndarray = "0.16.1"
tokenizers = "0.19.1"
half = "2.4.1"
hound = "3.5.1"
reqwest = { version = "0.12", features = ["blocking", "stream"] }
sha2 = "0.10"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
indicatif = "0.17"  # Progress bars for download
```

## Step 2: Download Models

Models from `gabotechs/music_gen` on HuggingFace:

| File | Size | URL |
|------|------|-----|
| `tokenizer.json` | ~2MB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small/tokenizer.json` |
| `config.json` | ~1KB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small/config.json` |
| `text_encoder.onnx` | ~120MB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small_fp16/text_encoder.onnx` |
| `decoder_model.onnx` | ~80MB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small_fp16/decoder_model.onnx` |
| `decoder_with_past_model.onnx` | ~80MB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small_fp16/decoder_with_past_model.onnx` |
| `encodec_decode.onnx` | ~25MB | `https://huggingface.co/gabotechs/music_gen/resolve/main/small_fp16/encodec_decode.onnx` |

Store in: `~/.cache/lofi/models/musicgen-small-fp16/`

## Step 3: Minimal CLI Implementation

Create `src/main.rs` with:

```rust
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "lofi-phase0")]
#[command(about = "Phase 0: MusicGen ONNX feasibility test")]
struct Cli {
    /// Text prompt for music generation
    #[arg(short, long)]
    prompt: String,

    /// Duration in seconds (5-120)
    #[arg(short, long, default_value = "10")]
    duration: u32,

    /// Output WAV file path
    #[arg(short, long, default_value = "output.wav")]
    output: PathBuf,

    /// Model directory path
    #[arg(short, long)]
    model_dir: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Validate duration
    if cli.duration < 5 || cli.duration > 120 {
        anyhow::bail!("Duration must be 5-120 seconds");
    }

    let model_dir = cli.model_dir.unwrap_or_else(|| {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lofi/models/musicgen-small-fp16")
    });

    println!("Phase 0 Validation: MusicGen ONNX");
    println!("==================================");
    println!("Prompt: {}", cli.prompt);
    println!("Duration: {}s", cli.duration);
    println!("Model dir: {}", model_dir.display());
    println!();

    let start = Instant::now();

    // TODO: Implement in actual Phase 0 task
    // 1. Load ONNX sessions
    // 2. Encode prompt with tokenizer
    // 3. Run text encoder
    // 4. Autoregressive decode with KV cache
    // 5. Decode tokens to audio
    // 6. Write WAV file

    let elapsed = start.elapsed();
    println!("Generation time: {:.1}s", elapsed.as_secs_f64());
    println!("Output: {}", cli.output.display());

    // Phase 0 success criteria
    if cli.duration == 10 && elapsed.as_secs() < 120 {
        println!("\n✓ PHASE 0 PASS: 10s audio in <2 minutes");
    }

    Ok(())
}
```

## Step 4: Validation Criteria

Run the CLI with test prompt:

```bash
cargo run --release -- \
  --prompt "lofi hip hop, jazzy piano, relaxing vibes" \
  --duration 10 \
  --output test_output.wav
```

### Success Criteria (SC-001)

| Metric | Target | How to Verify |
|--------|--------|---------------|
| Generation time | <120s | CLI prints elapsed time |
| Audio duration | 10s | `ffprobe test_output.wav` shows ~10s |
| Audio playable | Yes | `afplay test_output.wav` (macOS) or equivalent |
| Sample rate | 32000 Hz | `ffprobe` shows 32000 Hz |
| No errors | No panics/crashes | CLI exits with code 0 |

### Expected Output

```
Phase 0 Validation: MusicGen ONNX
==================================
Prompt: lofi hip hop, jazzy piano, relaxing vibes
Duration: 10s
Model dir: /Users/user/.cache/lofi/models/musicgen-small-fp16

Loading models...
  text_encoder.onnx: loaded in 1.2s
  decoder_model.onnx: loaded in 0.8s
  decoder_with_past_model.onnx: loaded in 0.7s
  encodec_decode.onnx: loaded in 0.3s

Encoding prompt... done (0.1s)
Generating tokens: [████████████████████] 500/500 (100%)
Decoding audio... done (0.5s)
Writing WAV... done

Generation time: 87.3s
Output: test_output.wav

✓ PHASE 0 PASS: 10s audio in <2 minutes
```

## Step 5: Go/No-Go Decision

| Outcome | Action |
|---------|--------|
| **PASS**: 10s in <120s | Proceed to Phase 1 (full daemon implementation) |
| **FAIL**: 10s in >120s | Investigate: try fp32, check CPU, profile bottlenecks |
| **FAIL**: Runtime errors | Debug ONNX loading, check model compatibility |
| **FAIL**: Audio quality issues | Verify model files, check sampling parameters |

## Phase 0 Verification Checklist

Before marking Phase 0 complete:

```markdown
- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo run --release -- --help` shows CLI usage
- [ ] CLI generates valid WAV file from test prompt
- [ ] Generation completes in <120s for 10s audio
- [ ] Generated audio is playable and sounds like music
- [ ] `grep -rn "TODO\|FIXME" src/` returns empty
- [ ] All Phase 0 code is imported and called (no dead code)
```

## Reference Implementation

Refer to MusicGPT for implementation patterns:
- `MusicGPT/src/musicgen/` - Core inference pipeline
- `MusicGPT/src/musicgen_models.rs` - Model orchestration
- `MusicGPT/src/audio/audio_manager.rs` - WAV writing

Key files to study:
1. `music_gen_text_encoder.rs` - Tokenization + encoding
2. `music_gen_decoder.rs` - KV cache autoregressive loop
3. `delay_pattern_mask_ids.rs` - 4-codebook pattern
4. `logits.rs` - Sampling with classifier-free guidance
