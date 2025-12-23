# Implementation Order

## Phase 0: AI Feasibility
1. ai-generation.md (Go/No-Go checkpoint - standalone Rust CLI for MusicGen ONNX)
2. ace-step.md (ACE-Step ONNX export and inference validation)
3. lofi-lora-training.md → [lofi-lora repo](https://github.com/willibrandon/lofi-lora) (Data curation, LoRA training, model optimization)

## Phase 1: Core Infrastructure
4. daemon-lifecycle.md
5. plugin-setup.md
6. audio-playback.md
7. cache-management.md

## Phase 2: AI Integration
8. ai-generation.md (full MusicGen integration)
9. ace-step.md (full ACE-Step integration with LoRA support)
10. progress-notifications.md
11. playlist-queue.md
12. prefetch.md

## Phase 3: Polish
13. health-check.md
14. statusline.md
15. telescope.md

---

## Notes

### Removed from scope
- Procedural generation - focusing exclusively on AI-generated lofi

### Phase 0 rationale
The LoRA training (spec: lofi-lora-training.md, impl: [lofi-lora repo](https://github.com/willibrandon/lofi-lora)) is in Phase 0 because:
- Training data acquisition takes time (commissioned tracks, licensing)
- Training can happen in parallel with infrastructure development
- The optimized LoRA model should be ready when integration completes
- Implementation lives in separate repo (Python ML pipeline vs Lua/Rust plugin)

### Phase 1 rationale
Core infrastructure required before any AI features work:
- Daemon lifecycle is the foundation for all daemon communication
- Plugin setup provides config access for all other modules
- Audio playback validates the full audio pipeline works
- Cache management is needed to store generated tracks

### Phase 2 rationale
Full AI integration with both backends:
- MusicGen for quick 30s generations
- ACE-Step for longer 4-minute tracks with LoRA enhancement
- Progress notifications for generation feedback
- Playlist/queue for managing multiple generations
- Prefetch is critical for masking AI generation latency (60-90s)

### Phase 3 rationale
Polish features that enhance UX but aren't blocking:
- Health check validates installation
- Statusline shows real-time state
- Telescope provides rich track management
