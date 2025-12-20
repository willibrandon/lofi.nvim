# Research: MusicGen ONNX Implementation

**Feature**: 001-musicgen-onnx
**Date**: 2025-12-19
**Source**: MusicGPT reference implementation (`/Users/brandon/src/MusicGPT`)

## Executive Summary

MusicGPT demonstrates a production-ready Rust implementation of MusicGen using ONNX Runtime. The architecture is directly applicable to lofi-daemon with minimal adaptation. All technical unknowns have been resolved through code analysis.

## Decision Log

### D1: ONNX Runtime Integration

**Decision**: Use `ort 2.0.0-rc.9` with GitHub-downloaded ONNX Runtime 1.20.1

**Rationale**:
- MusicGPT validates this exact version combination works
- `ort` crate provides safe Rust bindings with async support
- GitHub download approach avoids system dependency requirements
- Features `["half", "ndarray"]` enable fp16 models and tensor manipulation

**Alternatives Considered**:
- System-installed ONNX Runtime: Rejected - adds install complexity for users
- PyTorch/GGML: Rejected - larger binaries, less mature Rust ecosystem

**Reference**: `MusicGPT/Cargo.toml:30`, `MusicGPT/src/onnxruntime_lib.rs:61-167`

### D2: Model Architecture (Three ONNX Files)

**Decision**: Use split three-model architecture from `gabotechs/music_gen`

| Model | Purpose | Input Shape | Output Shape |
|-------|---------|-------------|--------------|
| `text_encoder.onnx` | Tokenize prompt → embeddings | `[1, seq_len]` int64 | `[1, seq_len, 768]` float |
| `decoder_model.onnx` | Autoregressive token generation | embeddings + KV cache | `[8, 1, vocab_size]` logits |
| `encodec_decode.onnx` | Token codes → audio samples | `[1, 1, 4, seq_len]` int64 | `[1, 1, samples]` float |

**Rationale**:
- Pre-exported models available on HuggingFace (no ONNX conversion needed)
- Split architecture enables progress tracking per token
- fp16 variant balances quality (~250MB) vs fp32 (~500MB)

**Alternatives Considered**:
- Single merged model: Rejected - no progress granularity, larger memory footprint
- Export from PyTorch: Rejected - complex, already done by gabotechs

**Reference**: `MusicGPT/src/musicgen_models.rs:66-184`

### D3: KV Cache Strategy

**Decision**: Use split decoder with `decoder_with_past_model.onnx` for cached inference

**Rationale**:
- First token uses full `decoder_model.onnx` (computes encoder attention)
- Subsequent tokens use `decoder_with_past_model.onnx` (reuses cached KV)
- Reduces computation from O(n²) to O(n) for autoregressive generation
- MusicGPT validates this approach with proper cache dimension handling

**Implementation Pattern**:
```rust
// First iteration: run full decoder
let outputs = decoder_model.run(inputs_without_cache)?;

// Extract KV cache from outputs
for layer in 0..num_layers {
    cache.decoder_key[layer] = outputs.present_decoder_key(layer);
    cache.decoder_value[layer] = outputs.present_decoder_value(layer);
}

// Subsequent iterations: run with_past decoder
let outputs = decoder_with_past.run(inputs_with_cache)?;
```

**Reference**: `MusicGPT/src/musicgen/music_gen_decoder.rs:154-256`

### D4: Delay Pattern Masking (4-Codebook)

**Decision**: Implement delay pattern for parallel 4-codebook generation

**Rationale**:
- EnCodec uses 4 vector quantizers (codebooks) for audio compression
- MusicGen generates all 4 in parallel with delay pattern for causality
- Pattern: codebook 0 starts immediately, codebook 1 delayed by 1, etc.

**Implementation Pattern**:
```rust
// After generating 5 tokens:
// Codebook 0: [t0, t1, t2, t3, t4]  - no delay
// Codebook 1: [P,  t0, t1, t2, t3]  - 1 token delay
// Codebook 2: [P,  P,  t0, t1, t2]  - 2 token delay
// Codebook 3: [P,  P,  P,  t0, t1]  - 3 token delay
// (P = pad token)

// Extract diagonal for actual codebook values:
fn last_de_delayed(&self) -> Option<[i64; 4]> {
    // Returns [batch_0[n-4], batch_1[n-3], batch_2[n-2], batch_3[n-1]]
}
```

**Reference**: `MusicGPT/src/musicgen/delay_pattern_mask_ids.rs:1-98`

### D5: Classifier-Free Guidance

**Decision**: Use guidance scale of 3.0 with batch size 8 (4 conditional + 4 unconditional)

**Rationale**:
- MusicGen trained with classifier-free guidance for prompt adherence
- Batch of 8 enables parallel conditional/unconditional computation
- Formula: `guided = uncond + (cond - uncond) * scale`
- Scale 3.0 provides good balance of prompt adherence vs diversity

**Reference**: `MusicGPT/src/musicgen/logits.rs:59-80`

### D6: Audio Output Format

**Decision**: WAV format, 32kHz mono, 32-bit float samples

**Rationale**:
- EnCodec decoder outputs 32kHz sample rate
- Mono output (single channel) matches MusicGen architecture
- Float32 preserves full dynamic range
- WAV is universally playable without additional codecs

**Implementation**: Use `hound` crate for WAV encoding

**Reference**: `MusicGPT/src/audio/audio_manager.rs:76-120`

### D7: Progress Calculation

**Decision**: Progress = tokens_generated / tokens_estimated, where tokens_estimated = duration_sec * 50

**Rationale**:
- MusicGen generates ~50 token frames per second of audio
- Linear relationship enables accurate ETA calculation
- 5% increment notification (per spec) = every ~2.5s for 30s audio

**Reference**: `MusicGPT/src/musicgen_models.rs:265-288` (`INPUT_IDS_BATCH_PER_SECOND = 50`)

### D8: Job Queue Architecture

**Decision**: Channel-based async job queue with CancellationToken

**Rationale**:
- Separates message processing from inference for responsiveness
- `std::sync::mpsc::channel` for token streaming during generation
- `tokio_util::sync::CancellationToken` for clean job cancellation
- Max 10 pending jobs per spec requirement

**Implementation Pattern**:
```rust
pub struct GenerationQueue {
    jobs: Arc<RwLock<VecDeque<Job>>>,
    abort_token: CancellationToken,
}

// Token streaming from decoder
let (tx, rx) = std::sync::mpsc::channel::<Result<[i64; 4]>>();
std::thread::spawn(move || {
    for _ in 0..max_tokens {
        let token = decoder.step()?;
        tx.send(Ok(token))?;
    }
});
```

**Reference**: `MusicGPT/src/backend/audio_generation_backend.rs`

### D9: GPU Execution Provider

**Decision**: Support CoreML (macOS), CUDA (Linux/Windows), CPU fallback

**Rationale**:
- `ort` supports execution providers through feature flags
- CoreML provides optimal Apple Silicon performance
- CUDA covers NVIDIA GPUs on other platforms
- CPU fallback ensures universal compatibility

**Implementation**:
```rust
let providers = match device {
    Device::Auto => detect_available_providers(),
    Device::Cpu => vec![CPUExecutionProvider::default()],
    Device::Cuda => vec![CUDAExecutionProvider::default()],
    Device::Metal => vec![CoreMLExecutionProvider::default()],
};
Session::builder()?.with_execution_providers(providers)?
```

**Reference**: `MusicGPT/src/gpu.rs`

### D10: Model Download Strategy

**Decision**: Async download from HuggingFace with progress reporting and user consent

**Rationale**:
- Models are ~250MB (fp16), too large to bundle
- HuggingFace CDN provides reliable, fast downloads
- User consent required per spec (FR-025)
- Progress reporting enables UI feedback during download

**Implementation**: Use `reqwest` with streaming response, write to temp file, rename on completion

**Reference**: `MusicGPT/src/storage/app_fs.rs`, `MusicGPT/src/musicgen_models.rs:186-205`

## Dependency Matrix

| Crate | Version | Purpose | Required |
|-------|---------|---------|----------|
| `ort` | 2.0.0-rc.9 | ONNX Runtime bindings | Yes |
| `ndarray` | 0.16.1 | Tensor operations | Yes |
| `tokenizers` | 0.19.1 | Text tokenization | Yes |
| `half` | 2.4.1 | Float16 support | Yes (for fp16 models) |
| `hound` | 3.5.1 | WAV file I/O | Yes |
| `reqwest` | 0.12 | HTTP client for downloads | Yes |
| `tokio` | 1.x | Async runtime | Yes |
| `serde`/`serde_json` | 1.x | JSON-RPC serialization | Yes |
| `sha2` | 0.10 | Track ID hashing | Yes |
| `directories` | 5.x | Platform cache paths | Yes |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ONNX Runtime version incompatibility | Low | High | Pin exact versions, test on CI |
| Model download failures | Medium | Medium | Retry logic, resume support, clear error messages |
| Memory exhaustion on low-RAM machines | Medium | High | Document 4GB minimum, add memory checks |
| Apple Silicon CoreML issues | Low | Medium | CPU fallback, user can force device |
| Token generation numerical instability | Low | High | Use fp32 for debugging, add NaN checks |

## Open Questions (None)

All technical questions resolved through MusicGPT analysis. Ready for Phase 1 design.
