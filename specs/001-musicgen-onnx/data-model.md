# Data Model: MusicGen ONNX

**Feature**: 001-musicgen-onnx
**Date**: 2025-12-19

## Entity Relationship Overview

```
┌─────────────────┐     generates     ┌─────────────┐
│ GenerationJob   │─────────────────▶│   Track     │
└────────┬────────┘                   └──────┬──────┘
         │                                   │
         │ uses                              │ stored in
         ▼                                   ▼
┌─────────────────┐                   ┌─────────────┐
│   ModelSet      │                   │    Cache    │
└─────────────────┘                   └─────────────┘
```

## Entities

### Track

A successfully generated audio file stored in the cache.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `track_id` | string | Primary key, hex string | SHA256 hash of (prompt + seed + duration + model_version) |
| `path` | string | Absolute path | Full filesystem path to WAV file |
| `prompt` | string | 1-1000 chars | Original text prompt |
| `duration_sec` | float | Actual duration | Precise duration of generated audio |
| `sample_rate` | int | Always 32000 | Audio sample rate in Hz |
| `seed` | int | u64 | Random seed used for generation |
| `model_version` | string | e.g., "musicgen-small-fp16-v1" | Model identifier for reproducibility |
| `generation_time_sec` | float | > 0 | Time taken to generate in seconds |
| `created_at` | timestamp | ISO 8601 | When track was created |

**Computed Fields**:
- `track_id = sha256(prompt + ":" + seed + ":" + duration_sec + ":" + model_version).hex()[0:16]`

**Lifecycle**:
```
[not exists] ──generate──▶ [generating] ──complete──▶ [cached]
                               │
                               └──error──▶ [failed]
```

**Validation Rules**:
- Track ID must be exactly 16 hex characters
- Path must exist and be readable
- Duration must be within 5-120 seconds

---

### GenerationJob

A request for music generation, tracked from submission through completion.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `job_id` | string | UUID v4 | Unique job identifier (not same as track_id) |
| `track_id` | string | Derived | Computed track_id for deduplication |
| `prompt` | string | 1-1000 chars | Text description of desired music |
| `duration_sec` | int | 5-120, default 30 | Requested audio duration |
| `seed` | int | Optional, u64 | If null, system generates random |
| `priority` | enum | "normal" \| "high" | Queue priority |
| `status` | enum | See states below | Current job state |
| `queue_position` | int | 0-9, nullable | Position in queue (null if not queued) |
| `progress_percent` | int | 0-99 | Generation progress (99 max until complete) |
| `tokens_generated` | int | >= 0 | Number of token frames generated |
| `tokens_estimated` | int | > 0 | duration_sec * 50 |
| `eta_sec` | float | >= 0 | Estimated seconds remaining |
| `error_code` | string | Nullable | Error code if failed |
| `error_message` | string | Nullable | Human-readable error |
| `created_at` | timestamp | ISO 8601 | When job was submitted |
| `started_at` | timestamp | Nullable | When generation started |
| `completed_at` | timestamp | Nullable | When generation finished |

**Status States**:
```
                                    ┌──────────────────┐
                                    ▼                  │
[pending] ──queue──▶ [queued] ──start──▶ [generating] ─┴──▶ [complete]
    │                    │                    │
    │                    │                    └──▶ [failed]
    └──validate_error────┴─────────────────────────▶ [rejected]
```

| Status | Description |
|--------|-------------|
| `pending` | Job received, validating |
| `queued` | Validated, waiting in queue |
| `generating` | Actively generating audio |
| `complete` | Generation successful |
| `failed` | Generation failed mid-process |
| `rejected` | Invalid request (bad duration, queue full) |

**Validation Rules**:
- Prompt must be non-empty, max 1000 characters
- Duration must be 5-120 seconds
- Only one job can be `generating` at a time
- Max 10 jobs in `queued` status

---

### ModelSet

The loaded ONNX model ensemble for inference.

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | e.g., "musicgen-small-fp16-v1" |
| `quantization` | enum | "fp32" \| "fp16" \| "int8" |
| `text_encoder` | Session | Loaded ONNX session |
| `decoder` | Session | Loaded ONNX session |
| `decoder_with_past` | Session | Loaded ONNX session (KV cache) |
| `audio_codec` | Session | Loaded ONNX session |
| `tokenizer` | Tokenizer | HuggingFace tokenizer |
| `config` | ModelConfig | Model parameters |
| `device` | enum | "cpu" \| "cuda" \| "metal" |
| `loaded_at` | timestamp | When models were loaded |

**ModelConfig Sub-entity**:
| Field | Type | Description |
|-------|------|-------------|
| `vocab_size` | int | Token vocabulary size |
| `num_hidden_layers` | int | Decoder layer count |
| `num_attention_heads` | int | Attention head count |
| `d_model` | int | Hidden dimension |
| `d_kv` | int | Key/value dimension per head |
| `audio_channels` | int | Always 1 (mono) |
| `sample_rate` | int | Always 32000 |
| `codebooks` | int | Always 4 |
| `pad_token_id` | int | Padding token ID |

**Lifecycle**:
```
[not loaded] ──load──▶ [loading] ──ready──▶ [loaded]
                           │
                           └──error──▶ [failed]
```

---

### DownloadProgress

Progress state for model download operation.

| Field | Type | Description |
|-------|------|-------------|
| `file_name` | string | Current file being downloaded |
| `bytes_downloaded` | int | Bytes received |
| `bytes_total` | int | Total file size |
| `files_completed` | int | Files fully downloaded |
| `files_total` | int | Total files to download |
| `status` | enum | "pending" \| "downloading" \| "complete" \| "failed" |

---

### Cache

Track storage and management.

| Field | Type | Description |
|-------|------|-------------|
| `base_path` | string | Cache directory path |
| `max_size_bytes` | int | Maximum cache size |
| `current_size_bytes` | int | Current used size |
| `track_count` | int | Number of cached tracks |

**Operations**:
- `get(track_id) -> Option<Track>`: Retrieve cached track
- `put(track) -> Result<()>`: Store track, evict LRU if needed
- `evict_lru() -> Option<Track>`: Remove least recently used track
- `contains(track_id) -> bool`: Check if track exists (for deduplication)

---

## Error Codes

| Code | Trigger | Description |
|------|---------|-------------|
| `MODEL_NOT_FOUND` | FR-019 | ONNX files not at expected path |
| `MODEL_LOAD_FAILED` | FR-020 | Corrupt file, wrong format, OOM during load |
| `MODEL_DOWNLOAD_FAILED` | Edge case | Network error, disk full during download |
| `MODEL_INFERENCE_FAILED` | FR-021 | Numerical instability, OOM during generation |
| `QUEUE_FULL` | FR-022 | 10 pending requests already queued |
| `INVALID_DURATION` | FR-023 | Duration outside 5-120 range |
| `INVALID_PROMPT` | Validation | Empty or >1000 char prompt |
| `GENERATION_CANCELLED` | Future | Job cancelled (not implemented per FR-009) |

---

## Index Recommendations

For the Track cache manifest (JSON file or similar):

| Index | Fields | Purpose |
|-------|--------|---------|
| Primary | `track_id` | Fast lookup by ID |
| LRU | `created_at` | Eviction ordering |
| Size | `path` → file size | Cache size calculation |
