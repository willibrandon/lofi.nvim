# Feature Specification: ACE-Step Long-Form Music Generation

**Feature Branch**: `002-ace-step`
**Created**: 2025-12-21
**Status**: Draft
**Input**: User description: "ACE-Step: Long-form music generation via flow matching - a 3.5B parameter diffusion model for generating up to 240 seconds of instrumental audio"

## Clarifications

### Session 2025-12-21

- Q: What is the maximum queue size for pending generation requests? → A: 3-5 requests maximum
- Q: Can users cancel an in-progress generation? → A: Yes, allow cancellation with immediate stop

## User Scenarios & Testing

### User Story 1 - Generate Long-Form Instrumental Track (Priority: P1)

A user wants to generate a 2-minute lofi instrumental track without audio gaps or artifacts. They provide a text prompt describing the desired style (e.g., "lofi hip hop, jazzy piano, vinyl crackle") and receive a seamless audio file.

**Why this priority**: This is the core value proposition—generating long-form music (up to 4 minutes) without the gaps inherent in MusicGen's 30-second limit. Without this capability, there's no reason to use ACE-Step.

**Independent Test**: Can be fully tested by generating a 120-second track with a style prompt and verifying the output is a valid audio file without audible gaps or artifacts.

**Acceptance Scenarios**:

1. **Given** the ACE-Step model is loaded, **When** user requests generation with prompt "lofi hip hop, chill beats" and duration 120 seconds, **Then** system returns a 120-second audio file with no audible gaps
2. **Given** a generation is in progress, **When** user requests progress, **Then** system reports accurate percentage complete and estimated time remaining
3. **Given** user specifies a seed value, **When** generation completes, **Then** regenerating with the same seed produces identical output

---

### User Story 2 - Switch Between Generation Backends (Priority: P2)

A user wants to choose between MusicGen (faster, smaller, 30s limit) and ACE-Step (slower, larger, 240s limit) based on their current needs. They can configure which backend to use and switch between them.

**Why this priority**: Users need flexibility to choose the right tool for the job. Quick previews may use MusicGen; final renders use ACE-Step. This enables a practical workflow.

**Independent Test**: Can be fully tested by switching backend configuration and verifying the correct model is used for generation.

**Acceptance Scenarios**:

1. **Given** both backends are installed, **When** user configures backend to "ace-step", **Then** subsequent generations use ACE-Step model
2. **Given** user configures backend to "musicgen", **When** user requests generation, **Then** system uses MusicGen model
3. **Given** only MusicGen is installed, **When** user attempts to switch to ACE-Step, **Then** system informs user to install ACE-Step first

---

### User Story 3 - Download and Setup ACE-Step Model (Priority: P2)

A user wants to install the ACE-Step model weights to enable long-form generation. They run a setup command that downloads the required files (7.7GB total) to the local cache.

**Why this priority**: Users cannot use ACE-Step without first downloading the model. This is a one-time setup but blocks all ACE-Step functionality.

**Independent Test**: Can be fully tested by running the setup command and verifying all model components are downloaded to the expected locations.

**Acceptance Scenarios**:

1. **Given** ACE-Step is not installed, **When** user runs setup command, **Then** system downloads all required model components to cache directory
2. **Given** download is in progress, **When** user checks status, **Then** system shows download progress for each component
3. **Given** partial download exists, **When** user runs setup again, **Then** system resumes download rather than restarting

---

### User Story 4 - Monitor Generation Progress (Priority: P3)

A user wants real-time feedback on generation progress since long-form generation can take significant time. They see percentage complete, current step, and estimated time remaining.

**Why this priority**: Without progress feedback, users have no way to know if generation is working or how long to wait. This is essential for user experience but not for core functionality.

**Independent Test**: Can be fully tested by starting a generation and verifying progress notifications are received with accurate information.

**Acceptance Scenarios**:

1. **Given** generation starts, **When** progress occurs, **Then** system emits progress notification with percentage, step number, and ETA
2. **Given** generation is 50% complete, **When** user checks progress, **Then** reported percentage is within 10% of actual completion
3. **Given** generation completes, **When** completion event fires, **Then** notification includes final track path, duration, and generation time

---

### User Story 5 - Configure Generation Parameters (Priority: P3)

A user wants to fine-tune generation quality by adjusting parameters like inference steps (speed/quality tradeoff), scheduler type, and guidance scale.

**Why this priority**: Advanced users benefit from tuning, but reasonable defaults work for most users. This enhances but doesn't enable the core use case.

**Independent Test**: Can be fully tested by generating with non-default parameters and verifying the output reflects the configuration.

**Acceptance Scenarios**:

1. **Given** user specifies 60 inference steps, **When** generation runs, **Then** system uses 60 steps (higher quality, slower)
2. **Given** user specifies "euler" scheduler, **When** generation runs, **Then** system uses Euler scheduler
3. **Given** user specifies invalid parameter value, **When** generation is requested, **Then** system returns clear error message

---

### Edge Cases

- What happens when user requests duration outside 5-240 second range?
  - System rejects with INVALID_DURATION error and specifies valid range
- What happens when insufficient VRAM is available?
  - System reports MODEL_LOAD_FAILED with clear message about memory requirements
- What happens when model files are corrupted?
  - System reports MODEL_LOAD_FAILED and suggests re-running setup
- What happens when generation fails mid-way due to numerical instability?
  - System reports MODEL_INFERENCE_FAILED with suggestion to retry with different seed
- What happens when queue is full?
  - System reports QUEUE_FULL with current queue position and estimated wait
- What happens on macOS with bf16 precision issues?
  - System automatically uses fp32 on Apple Silicon to avoid precision errors
- What happens when user cancels an in-progress generation?
  - System immediately stops inference, releases resources, and emits cancellation event

## Requirements

### Functional Requirements

- **FR-001**: System MUST generate audio from text prompts between 5 and 240 seconds duration
- **FR-002**: System MUST support instrumental generation without requiring lyrics input
- **FR-003**: System MUST download and cache model weights to local filesystem (~7.7GB total)
- **FR-004**: System MUST provide real-time progress notifications during generation
- **FR-005**: System MUST support switching between MusicGen and ACE-Step backends
- **FR-006**: System MUST produce deterministic output when given the same seed on the same hardware
- **FR-007**: System MUST run all inference locally without network access after initial model download
- **FR-008**: System MUST queue generation requests (maximum 5 pending) and process them sequentially
- **FR-009**: System MUST validate duration parameter is within acceptable range before starting generation
- **FR-010**: System MUST report clear error codes and messages for all failure scenarios
- **FR-011**: System MUST detect available backends at startup and use configured preference
- **FR-012**: System MUST support configurable inference parameters (steps, scheduler, guidance scale)
- **FR-013**: System MUST resample output audio to 48kHz standard sample rate
- **FR-014**: System MUST use fp32 precision on macOS to avoid Apple Silicon bf16 issues
- **FR-015**: System MUST allow users to cancel an in-progress generation with immediate stop

### Key Entities

- **Generation Request**: Represents a user's request to generate audio, including prompt, duration, parameters, and seed
- **Track**: Represents generated audio output, including file path, duration, sample rate, and generation metadata
- **Backend**: Represents an available generation model (MusicGen or ACE-Step) with its capabilities and status
- **Progress Update**: Represents real-time generation status including percentage, step, total steps, and ETA

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can generate 120 seconds of audio in under 15 seconds on high-end hardware (RTX 3090 equivalent)
- **SC-002**: Model loads and becomes ready in under 30 seconds on modern hardware
- **SC-003**: Progress updates reflect actual completion within 10% accuracy
- **SC-004**: Same seed produces bit-identical output on the same machine and device
- **SC-005**: Generated audio has no audible gaps, clicks, or artifacts when played back
- **SC-006**: Users can switch between backends without restarting the daemon
- **SC-007**: System handles generation requests up to 240 seconds without failure
- **SC-008**: Download progress is reported with per-component status and overall percentage
- **SC-009**: Error messages are actionable—users understand what went wrong and how to fix it

## Assumptions

- Users have hardware capable of running the model (minimum 8GB VRAM for degraded mode, 16GB for optimal)
- Model weights are available for download from HuggingFace (ACE-Step/ACE-Step-v1-3.5B)
- Users accept the 7.7GB storage requirement for ACE-Step model weights
- Network access is available for initial model download
- The daemon process has permissions to write to the cache directory (~/.cache/lofi.nvim/)
- MusicGen backend implementation already exists and this extends the system with an additional backend

## Dependencies

- Daemon process must be running to handle generation requests
- Cache management system must support storing and retrieving large model files
- Progress notification system must support real-time updates to the editor
- Existing backend abstraction layer allows plugging in ACE-Step alongside MusicGen

## Out of Scope

- Lyric-based generation (this spec focuses on instrumental only)
- Audio-to-audio style transfer (future capability)
- LoRA fine-tuning support (future capability)
- Selective segment regeneration/repainting (future capability)
- Extending existing tracks (future capability)
- Quantization/int4 mode for reduced VRAM (future optimization)
