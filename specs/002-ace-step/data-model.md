# Data Model: ACE-Step Long-Form Music Generation

**Phase 1 Output** | **Branch**: `002-ace-step` | **Date**: 2025-12-21

## Overview

This document defines the data entities, their relationships, and state transitions for the ACE-Step backend integration. The model extends existing lofi.nvim types to support backend selection and ACE-Step-specific parameters.

---

## Entities

### 1. Backend

Represents an available generation model.

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `type` | `BackendType` | Enum: `MusicGen` or `AceStep` |
| `name` | `string` | Display name (e.g., "MusicGen-Small", "ACE-Step-3.5B") |
| `status` | `BackendStatus` | Current availability status |
| `min_duration_sec` | `u32` | Minimum supported duration (5 for both) |
| `max_duration_sec` | `u32` | Maximum supported duration (120 for MusicGen, 240 for ACE-Step) |
| `sample_rate` | `u32` | Output sample rate (32000 for MusicGen, 48000 for ACE-Step) |
| `model_version` | `string` | Version string for cache invalidation |

**BackendType Enum**:
```
MusicGen   - Autoregressive transformer (Meta)
AceStep    - Diffusion transformer (ACE-Step team)
```

**BackendStatus Enum**:
```
NotInstalled   - Model weights not downloaded
Downloading    - Download in progress
Ready          - Loaded and ready for inference
Loading        - Loading into memory
Error          - Failed to load (with message)
```

**Relationships**:
- Backend → Track: One-to-many (a backend produces many tracks)
- Backend → GenerationRequest: One-to-many (requests specify a backend)

---

### 2. GenerationRequest

Represents a user's request to generate audio.

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `string` | Yes | Generated | Unique request identifier |
| `prompt` | `string` | Yes | - | Text description of desired music |
| `duration_sec` | `u32` | Yes | - | Target duration in seconds |
| `backend` | `BackendType` | No | Config default | Which backend to use |
| `seed` | `u64` | No | Random | Reproducibility seed |
| `priority` | `Priority` | No | `Normal` | Queue priority |

**ACE-Step-specific fields** (optional, only used when backend=AceStep):

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `inference_steps` | `u32` | No | 60 | Number of diffusion steps |
| `scheduler` | `SchedulerType` | No | `Euler` | ODE/SDE solver |
| `guidance_scale` | `f32` | No | 15.0 | Classifier-free guidance strength |

**SchedulerType Enum** (ACE-Step only):
```
Euler      - Fast ODE solver (deterministic)
Heun       - Accurate ODE solver (2x slower, deterministic)
PingPong   - SDE solver (stochastic, best quality)
```

**Priority Enum**:
```
High    - Insert at front of queue
Normal  - Insert at back of queue
```

**Validation Rules**:
- `prompt`: Non-empty, max 512 characters
- `duration_sec`:
  - MusicGen: 5-120 seconds
  - ACE-Step: 5-240 seconds
- `inference_steps`: 1-200 (ACE-Step only)
- `guidance_scale`: 1.0-30.0 (ACE-Step only)
- `seed`: Any u64 value

**State Transitions**:
```
Created → Queued → Generating → Completed
                 ↘ Cancelled
                 ↘ Failed
```

---

### 3. Track

Represents generated audio output.

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string` | Unique track identifier (SHA256 hash) |
| `path` | `string` | Absolute path to audio file |
| `prompt` | `string` | Original generation prompt |
| `duration_sec` | `f32` | Actual audio duration |
| `sample_rate` | `u32` | Audio sample rate (Hz) |
| `seed` | `u64` | Seed used for generation |
| `backend` | `BackendType` | Backend that generated this track |
| `model_version` | `string` | Model version string |
| `generation_time_sec` | `f32` | Time taken to generate |
| `created_at` | `timestamp` | When track was generated |

**Track ID Computation**:
```
id = SHA256(prompt + seed + duration_sec + model_version + backend)
```

**Relationships**:
- Track → Backend: Many-to-one (tracks belong to one backend)
- Track → GenerationRequest: One-to-one (each track from one request)

---

### 4. ProgressUpdate

Represents real-time generation status.

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `track_id` | `string` | Track being generated |
| `percent` | `u8` | Completion percentage (0-100) |
| `current_step` | `u32` | Current step number |
| `total_steps` | `u32` | Total steps to complete |
| `eta_sec` | `f32` | Estimated time remaining |

**MusicGen Steps**: tokens_generated / total_tokens
**ACE-Step Steps**: diffusion_step / inference_steps

**Update Frequency**: Every 5% increment (matches existing behavior)

---

### 5. DaemonConfig

Extended configuration for backend selection.

**New Fields** (additions to existing config):

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_backend` | `BackendType` | `MusicGen` | Default backend when not specified |
| `ace_step_model_dir` | `string` | `~/.cache/lofi.nvim/ace-step` | ACE-Step model location |
| `ace_step_default_steps` | `u32` | 60 | Default inference steps |
| `ace_step_default_scheduler` | `SchedulerType` | `Euler` | Default scheduler |
| `ace_step_default_guidance` | `f32` | 15.0 | Default guidance scale |

---

## State Diagrams

### Generation Request Lifecycle

```
┌─────────┐
│ Created │
└────┬────┘
     │ submit()
     ▼
┌─────────┐     ┌───────────┐
│ Queued  │────►│ Cancelled │
└────┬────┘     └───────────┘
     │ pop()         ▲
     ▼               │ cancel()
┌────────────┐       │
│ Generating │───────┤
└─────┬──────┘       │
      │              │
      ├──────────────┤
      │ error()      │
      ▼              │
┌─────────┐          │
│ Failed  │          │
└─────────┘          │
      │              │
      │ complete()   │
      ▼              │
┌───────────┐        │
│ Completed │────────┘
└───────────┘
```

### Backend Status Lifecycle

```
┌──────────────┐
│ NotInstalled │
└──────┬───────┘
       │ download()
       ▼
┌─────────────┐
│ Downloading │
└──────┬──────┘
       │
       ├─────────────┐
       │ success()   │ error()
       ▼             ▼
┌─────────┐    ┌─────────┐
│  Ready  │    │  Error  │
└────┬────┘    └─────────┘
     │ load()
     ▼
┌─────────┐
│ Loading │
└────┬────┘
     │ success()
     ▼
┌─────────┐
│  Ready  │ (in-memory)
└─────────┘
```

---

## Relationships Diagram

```
┌────────────────────┐
│   DaemonConfig     │
│                    │
│ default_backend ───┼──────────────────┐
│ ace_step_model_dir │                  │
└────────────────────┘                  │
                                        ▼
┌────────────────────┐         ┌───────────────┐
│ GenerationRequest  │         │    Backend    │
│                    │  uses   │               │
│ backend ───────────┼────────►│ type          │
│ prompt             │         │ status        │
│ duration_sec       │         │ max_duration  │
│ seed               │         │ sample_rate   │
│ inference_steps    │         └───────┬───────┘
│ scheduler          │                 │
│ guidance_scale     │                 │ produces
└────────┬───────────┘                 │
         │                             ▼
         │ generates         ┌───────────────┐
         └──────────────────►│     Track     │
                             │               │
                             │ id            │
                             │ path          │
                             │ backend       │
                             │ model_version │
                             └───────┬───────┘
                                     │
                                     │ emits
                                     ▼
                             ┌───────────────┐
                             │ ProgressUpdate│
                             │               │
                             │ track_id      │
                             │ percent       │
                             │ eta_sec       │
                             └───────────────┘
```

---

## Validation Summary

### GenerationRequest Validation

| Field | MusicGen | ACE-Step |
|-------|----------|----------|
| duration_sec | 5-120 | 5-240 |
| inference_steps | N/A | 1-200 |
| scheduler | N/A | Euler, Heun, PingPong |
| guidance_scale | N/A | 1.0-30.0 |
| prompt | 1-512 chars | 1-512 chars |

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_DURATION` | Duration outside backend's supported range |
| `INVALID_BACKEND` | Requested backend not installed |
| `BACKEND_NOT_READY` | Backend loading or in error state |
| `INVALID_INFERENCE_STEPS` | Steps outside 1-200 range |
| `INVALID_GUIDANCE_SCALE` | Scale outside 1.0-30.0 range |
| `INVALID_SCHEDULER` | Unknown scheduler type |
| `MODEL_LOAD_FAILED` | Failed to load backend models |
| `MODEL_INFERENCE_FAILED` | Inference error during generation |
| `GENERATION_CANCELLED` | User cancelled in-progress generation |
| `QUEUE_FULL` | Queue at maximum capacity (5) |
