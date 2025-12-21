# Quickstart: ACE-Step Long-Form Music Generation

**Phase 1 Output** | **Branch**: `002-ace-step` | **Date**: 2025-12-21

## Overview

This document provides the minimal implementation path to verify the ACE-Step integration works end-to-end. It focuses on the critical path to first successful generation.

---

## Prerequisites

### Hardware Requirements
- **Minimum**: 8GB VRAM (with fp32, longer generation times)
- **Recommended**: 16GB VRAM (full performance)
- **Disk**: ~8GB free for ACE-Step models

### Software Requirements
- Existing lofi.nvim installation with working MusicGen backend
- Rust 1.75+ toolchain
- Python 3.10+ (for ONNX export only)

---

## Phase 0: ONNX Model Export

Before Rust implementation, export ACE-Step models to ONNX format.

### Step 1: Set Up Export Environment

```bash
cd /Users/brandon/src/ACE-Step
python -m venv .venv-export
source .venv-export/bin/activate
pip install torch transformers diffusers onnx onnxruntime
```

### Step 2: Export Text Encoder (UMT5)

```python
# export_text_encoder.py
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import torch

model = AutoModelForSeq2SeqLM.from_pretrained("google/umt5-base")
tokenizer = AutoTokenizer.from_pretrained("google/umt5-base")

# Export encoder only
encoder = model.get_encoder()
dummy_input = tokenizer("lofi hip hop", return_tensors="pt")

torch.onnx.export(
    encoder,
    (dummy_input["input_ids"], dummy_input["attention_mask"]),
    "text_encoder.onnx",
    input_names=["input_ids", "attention_mask"],
    output_names=["hidden_states"],
    dynamic_axes={
        "input_ids": {0: "batch", 1: "seq"},
        "attention_mask": {0: "batch", 1: "seq"},
        "hidden_states": {0: "batch", 1: "seq"}
    },
    opset_version=17
)
```

### Step 3: Export Remaining Components

Export in order:
1. `transformer.onnx` - ACEStepTransformer (largest, ~6.5GB)
2. `dcae_decoder.onnx` - MusicDCAE decoder (~300MB)
3. `vocoder.onnx` - ADaMoSHiFiGAN (~400MB)

Each requires custom export script handling dynamic shapes.

### Step 4: Verify Exports

```bash
python -c "import onnxruntime; print(onnxruntime.InferenceSession('text_encoder.onnx'))"
```

---

## Phase 1: Backend Abstraction

### Step 1: Add Backend Enum

```rust
// daemon/src/models/backend.rs
pub enum Backend {
    MusicGen,
    AceStep,
}

pub enum LoadedModels {
    MusicGen(MusicGenModels),
    AceStep(AceStepModels),
}
```

### Step 2: Restructure Models Directory

```
daemon/src/models/
├── mod.rs
├── backend.rs          # NEW
├── musicgen/           # Move existing files here
│   ├── mod.rs
│   └── ...
└── ace_step/           # NEW
    ├── mod.rs
    ├── models.rs
    └── ...
```

### Step 3: Update Config

```rust
// daemon/src/config.rs
pub struct DaemonConfig {
    // ... existing fields ...
    pub default_backend: Backend,
    pub ace_step_model_dir: PathBuf,
}
```

---

## Phase 2: ACE-Step Models

### Step 1: Define AceStepModels

```rust
// daemon/src/models/ace_step/models.rs
pub struct AceStepModels {
    pub text_encoder: Session,
    pub transformer: Session,
    pub dcae_decoder: Session,
    pub vocoder: Session,
    pub version: String,
}
```

### Step 2: Implement Model Loading

```rust
impl AceStepModels {
    pub fn load(model_dir: &Path, device: Device) -> Result<Self, Error> {
        let text_encoder = Session::builder()?
            .with_execution_providers([device.into()])?
            .commit_from_file(model_dir.join("text_encoder.onnx"))?;
        // ... load other components
    }
}
```

---

## Phase 3: Diffusion Pipeline

### Step 1: Implement Scheduler

```rust
// daemon/src/models/ace_step/scheduler.rs
pub struct EulerScheduler {
    timesteps: Vec<f32>,
    sigmas: Vec<f32>,
}

impl EulerScheduler {
    pub fn new(num_steps: u32) -> Self { ... }
    pub fn step(&self, model_output: &Array, sample: &Array, step: u32) -> Array { ... }
}
```

### Step 2: Implement Generation Loop

```rust
pub fn generate(
    models: &AceStepModels,
    params: &AceStepParams,
    progress: impl Fn(u32, u32),
) -> Result<Vec<f32>, Error> {
    // 1. Encode prompt
    let text_emb = encode_prompt(&models.text_encoder, &params.prompt)?;

    // 2. Initialize latent noise
    let mut latent = random_latent(params.duration_sec, params.seed);

    // 3. Diffusion loop
    let scheduler = EulerScheduler::new(params.inference_steps);
    for step in 0..params.inference_steps {
        progress(step, params.inference_steps);

        // Predict noise
        let noise_pred = predict_noise(&models.transformer, &latent, step, &text_emb)?;

        // Apply CFG
        let guided = apply_cfg(&noise_pred, params.guidance_scale);

        // Scheduler step
        latent = scheduler.step(&guided, &latent, step);
    }

    // 4. Decode to audio
    let mel = decode_latent(&models.dcae_decoder, &latent)?;
    let audio = vocode(&models.vocoder, &mel)?;

    Ok(audio)
}
```

---

## Phase 4: RPC Integration

### Step 1: Extend GenerateParams

```rust
// daemon/src/rpc/types.rs
pub struct GenerateParams {
    // ... existing fields ...
    pub backend: Option<String>,
    pub inference_steps: Option<u32>,
    pub scheduler: Option<String>,
    pub guidance_scale: Option<f32>,
}
```

### Step 2: Add Backend Selection to Handler

```rust
// daemon/src/rpc/methods.rs
fn handle_generate(state: &ServerState, params: GenerateParams) -> Result<...> {
    let backend = params.backend
        .map(|s| Backend::parse(&s))
        .unwrap_or(state.config.default_backend);

    match backend {
        Backend::MusicGen => generate_musicgen(state, params),
        Backend::AceStep => generate_ace_step(state, params),
    }
}
```

---

## Verification Steps

### Step 1: Build and Test

```bash
cd daemon
cargo build --release
cargo test
```

### Step 2: CLI Test (Pre-Daemon)

```bash
./target/release/lofi-daemon --backend ace_step --prompt "lofi beats" --duration 30
```

### Step 3: Full Integration Test

```vim
:lua require('lofi').generate({ prompt = "lofi beats", duration_sec = 30, backend = "ace_step" })
```

---

## Critical Path Summary

1. **Export ONNX models** - Blocks all Rust work
2. **Backend enum + config** - Foundation for multi-backend
3. **AceStepModels struct** - Model loading infrastructure
4. **Scheduler implementation** - Core diffusion logic
5. **Generation pipeline** - End-to-end inference
6. **RPC extensions** - Wire up to Lua interface

Total estimated implementation scope: ~15 Rust files modified/created.

---

## Troubleshooting

### ONNX Export Fails
- Check for unsupported ops (custom attention)
- Try opset 14 instead of 17
- Export components separately to isolate issues

### Memory Errors
- Reduce `inference_steps` (try 30 instead of 60)
- Enable fp16 on CUDA (not macOS)
- Use `--cpu-offload` during development

### Numerical Instability
- Force fp32 on macOS
- Try different seed
- Reduce guidance_scale (try 7.5 instead of 15.0)
