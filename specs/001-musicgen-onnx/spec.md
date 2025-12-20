# Feature Specification: AI Music Generation via MusicGen ONNX Backend

**Feature Branch**: `001-musicgen-onnx`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "AI music generation via MusicGen ONNX backend"

## Clarifications

### Session 2025-12-19

- Q: How do users acquire ONNX model files? → A: Daemon auto-downloads models on first use if missing (with user consent prompt)
- Q: How often are progress notifications sent during generation? → A: Every 5% progress increment (approximately 20 updates per generation)

## Overview

This feature enables AI-powered music generation within the lofi.nvim Neovim plugin using MusicGen-small with ONNX Runtime inference. All processing runs locally on the user's machine with no network dependencies or API keys required. Generation runs asynchronously in a background daemon, never blocking the editor.

**Reference Implementation**: MusicGPT - a working Rust implementation using ONNX Runtime (local clone at `/Users/brandon/src/MusicGPT`).

**Phase 0 Feasibility Checkpoint**: Before building Neovim integration, a standalone Rust CLI must prove it can load MusicGen ONNX models and generate 10 seconds of audio from a text prompt on CPU in under 2 minutes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate Music from Text Prompt (Priority: P1)

A user working in Neovim wants to generate ambient background music to help them focus. They invoke the generation command with a text prompt describing the desired music style (e.g., "lofi hip hop, jazzy piano"), and the system generates a unique audio track matching their description.

**Why this priority**: This is the core value proposition - generating custom music from natural language. Without this, the feature has no purpose.

**Independent Test**: Can be fully tested by invoking the generate command with a prompt and verifying an audio file is created that can be played back.

**Acceptance Scenarios**:

1. **Given** the daemon is running and models are loaded, **When** user requests generation with prompt "lofi hip hop, jazzy piano" and duration 30 seconds, **Then** a track_id is returned immediately and the audio file is generated within expected time bounds
2. **Given** the daemon is running, **When** user provides a seed value with their request, **Then** regenerating with the same prompt, seed, and duration produces identical audio output
3. **Given** the daemon is running, **When** user omits the seed value, **Then** a random seed is generated and included in the response for future reproducibility

---

### User Story 2 - Monitor Generation Progress (Priority: P2)

While a track is being generated, the user wants to see progress updates so they know how long to wait and can continue working with confidence that generation is proceeding.

**Why this priority**: Generation can take 30-90+ seconds. Without progress feedback, users won't know if the system is working or frozen.

**Independent Test**: Can be tested by initiating generation and observing progress notifications with percent complete and estimated time remaining.

**Acceptance Scenarios**:

1. **Given** a generation is in progress, **When** the daemon processes tokens, **Then** progress notifications are sent with track_id, percent (0-99), tokens generated, tokens estimated, and ETA in seconds
2. **Given** a generation is in progress, **When** progress reaches 100%, **Then** a generation_complete notification is sent with the final file path and metadata
3. **Given** a generation encounters an error, **When** the error occurs, **Then** a generation_error notification is sent with the track_id and descriptive error code

---

### User Story 3 - Queue Multiple Generation Requests (Priority: P3)

A user wants to queue several music variations to generate while they work. They submit multiple requests with different prompts, and the system processes them one at a time while reporting queue position.

**Why this priority**: Enables batch workflows but core single-generation must work first. Queue is a convenience feature.

**Independent Test**: Can be tested by submitting multiple generation requests and verifying each is assigned a queue position and processed in order.

**Acceptance Scenarios**:

1. **Given** a generation is already in progress, **When** user submits a new request, **Then** request is queued and response includes queue position
2. **Given** requests are queued, **When** user submits a high-priority request, **Then** the high-priority request moves to front of queue
3. **Given** 10 requests are already pending, **When** user submits another request, **Then** system returns QUEUE_FULL error

---

### User Story 4 - Configure Hardware Acceleration (Priority: P4)

A user with a GPU wants to use it for faster generation. They configure their preferred device (auto, CPU, CUDA, Metal) and the system uses the appropriate hardware.

**Why this priority**: Performance optimization after core functionality works. CPU fallback ensures universal compatibility.

**Independent Test**: Can be tested by setting device configuration and verifying generation uses the specified hardware (or falls back appropriately).

**Acceptance Scenarios**:

1. **Given** device is set to "auto", **When** GPU is available, **Then** system uses GPU acceleration
2. **Given** device is set to "auto", **When** no GPU is available, **Then** system falls back to CPU
3. **Given** device is set to "cuda" but no CUDA GPU exists, **When** generation is requested, **Then** system returns appropriate error or falls back gracefully

---

### Edge Cases

- What happens when model files are missing or corrupted?
  - System returns MODEL_NOT_FOUND or MODEL_LOAD_FAILED error with descriptive message
- What happens when duration is outside 5-120 second range?
  - System returns INVALID_DURATION error immediately without queuing
- What happens when system runs out of memory during generation?
  - System returns MODEL_INFERENCE_FAILED error with OOM indication
- What happens when user requests generation before daemon loads models?
  - First generation request triggers model loading; request is queued until models ready
- What happens when daemon crashes mid-generation?
  - Track remains incomplete; next daemon start does not auto-resume incomplete generations
- What happens when model download fails (network error, disk full)?
  - System returns MODEL_DOWNLOAD_FAILED error; user can retry on next generation request

## Requirements *(mandatory)*

### Functional Requirements

#### Generation Request Handling
- **FR-001**: System MUST accept generation requests with prompt (text), duration_sec (default 30, range 5-120), seed (optional), and priority (normal/high)
- **FR-002**: System MUST return a track_id immediately upon receiving a valid generation request
- **FR-003**: System MUST queue generation requests and process them serially (one at a time)
- **FR-004**: System MUST support maximum 10 pending requests in the queue
- **FR-005**: System MUST allow high-priority requests to skip to front of queue

#### Background Processing
- **FR-006**: System MUST load models on first generation request (lazy loading)
- **FR-007**: System MUST keep models loaded in memory until daemon exits
- **FR-008**: System MUST run all inference in background without blocking the editor
- **FR-009**: System MUST NOT support cancellation of in-progress generation

#### Model Acquisition
- **FR-024**: System MUST detect when required ONNX model files are missing on first generation request
- **FR-025**: System MUST prompt user for consent before downloading models (one-time confirmation)
- **FR-026**: System MUST download models from HuggingFace (`gabotechs/music_gen`) when user consents
- **FR-027**: System MUST report download progress during model acquisition

#### Progress Reporting
- **FR-010**: System MUST send progress notifications every 5% increment during generation, including track_id, percent (0-99), tokens_generated, tokens_estimated, and eta_sec
- **FR-011**: System MUST cap progress percent at 99 until generation is fully complete
- **FR-012**: System MUST send generation_complete notification with file path and metadata when finished

#### Reproducibility
- **FR-013**: System MUST produce identical output for same prompt + seed + duration + model_version on the same machine/device
- **FR-014**: System MUST generate a random seed when none provided and include it in the response
- **FR-015**: System MUST compute track_id as a hash of prompt + seed + duration + model_version for cache deduplication

#### Device Selection
- **FR-016**: System MUST support device configuration: "auto", "cpu", "cuda", "metal"
- **FR-017**: System MUST auto-detect available GPU when device is "auto" and prefer it over CPU
- **FR-018**: System MUST support configurable CPU thread count (nil = auto-detect)

#### Error Handling
- **FR-019**: System MUST return MODEL_NOT_FOUND when ONNX model files are not at expected path
- **FR-020**: System MUST return MODEL_LOAD_FAILED for corrupt files, wrong format, or out-of-memory during loading
- **FR-021**: System MUST return MODEL_INFERENCE_FAILED for numerical instability or out-of-memory during generation
- **FR-022**: System MUST return QUEUE_FULL when max 10 pending requests exceeded
- **FR-023**: System MUST return INVALID_DURATION when duration is outside 5-120 second range

### Key Entities

- **Track**: A generated audio file; attributes include track_id (unique identifier/hash), file path, duration, sample rate (32000 Hz), prompt, seed, generation time, and model version
- **Generation Request**: A queued request for music generation; includes prompt, duration, seed, priority, and status (queued/in-progress/complete/failed)
- **Model**: The loaded ONNX model set; includes text encoder, decoder, and audio codec; has quantization variant (fp32/fp16/int8) and version identifier

## Assumptions

- The daemon process is started separately from Neovim (as specified in daemon-lifecycle.md dependency)
- Generated audio files are stored in the cache directory managed by cache-management.md
- Cross-platform byte-identical output is NOT guaranteed due to floating-point variance between different hardware
- Model files are available from `gabotechs/music_gen` on HuggingFace in fp32, fp16, and int8 variants
- Default quantization is fp16 (~250MB total model size)
- Network access is required only for initial model download; all inference runs locally thereafter

## Dependencies

- daemon-lifecycle.md: Daemon must be running to accept generation requests
- cache-management.md: Tracks are stored and managed according to cache policy
- progress-notifications.md: UI handling of progress updates defined there

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Phase 0 feasibility: 10 seconds of audio can be generated on CPU in under 2 minutes
- **SC-002**: Model loading completes in under 10 seconds on modern hardware (Apple Silicon M1+ or equivalent)
- **SC-003**: Progress updates are accurate within 10% of actual completion percentage
- **SC-004**: Same seed produces identical audio output on the same machine and device
- **SC-005**: GPU acceleration (when available) provides greater than 2x speedup over CPU
- **SC-006**: Users can generate and play back custom music within 3 minutes of first request (including initial model load)
- **SC-007**: Queue handles 10 concurrent pending requests without errors or data loss
