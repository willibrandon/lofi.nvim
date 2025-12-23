# lofi-vision: AI-Generated Lofi Scenes

> **Proposed Repository**: [github.com/willibrandon/lofi-vision](https://github.com/willibrandon/lofi-vision)
>
> This document proposes a new project for generating animated lofi scenes using AnimateDiff and related video generation technologies. This is a companion to [lofi-lora](https://github.com/willibrandon/lofi-lora) (audio) to complete the lofi experience.

---

## Executive Summary

lofi-vision generates the visual component of the lofi experience: cozy animated scenes with subtle looping motion. Think "lofi girl studying" - a static illustration brought to life with gentle hair movement, rain on windows, steam from coffee, flickering candles.

**Approach**: AnimateDiff with custom style LoRAs and motion LoRAs, optimized for infinite seamless loops.

## Vision

> "The visual half of the ultimate lofi experience"

Combined with lofi.nvim's audio generation, lofi-vision enables:
- **Complete lofi streams**: Generated audio + generated visuals
- **Infinite variety**: New scenes on demand, never the same twice
- **Local & private**: No cloud dependencies after model download
- **Style control**: Seasonal themes, time of day, weather, room types

## Why AnimateDiff Over Full Video Generation

| Aspect | AnimateDiff | CogVideoX/Wan/Hunyuan |
|--------|-------------|----------------------|
| Use case | Subtle animation of static scenes | Full motion video |
| VRAM | 8-12GB | 24-80GB |
| Loop-friendly | Excellent (designed for it) | Requires post-processing |
| Training data | Images + short clips | Full video datasets |
| Inference speed | Fast (seconds) | Slow (minutes) |
| Lofi aesthetic fit | Perfect | Overkill |

The lofi aesthetic is defined by **subtle, looping motion** - exactly what AnimateDiff excels at.

---

## Technical Architecture

### Generation Pipeline

```
Text Prompt
    ↓
Stable Diffusion (with Style LoRA)
    ↓
AnimateDiff (with Motion LoRA)
    ↓
Temporal smoothing + loop blending
    ↓
Output: Seamless looping video (5-30 seconds)
```

### Model Components

| Component | Purpose | Size |
|-----------|---------|------|
| SD 1.5 / SDXL | Base image generation | 2-7GB |
| Style LoRA | Lofi aesthetic (cozy, anime, illustration) | 50-200MB |
| AnimateDiff Motion Module | Adds temporal consistency | ~1.8GB |
| Motion LoRA | Specific movement patterns (rain, hair, steam) | 50-100MB |

### Target Output Specs

| Property | Value |
|----------|-------|
| Resolution | 512x512 (SD1.5) or 1024x1024 (SDXL) |
| Frame rate | 8-12 fps (lofi aesthetic, not cinematic) |
| Duration | 5-30 seconds (seamless loop) |
| Format | MP4/GIF/WebM |

---

## Style Taxonomy

### Scene Types

| ID | Scene | Key Elements |
|----|-------|--------------|
| `SCENE-01` | Study Room | Desk, lamp, books, window, figure studying |
| `SCENE-02` | Coffee Shop | Counter, steam, rain outside, warm lighting |
| `SCENE-03` | Bedroom Night | Bed, fairy lights, city view, cozy |
| `SCENE-04` | Rainy Window | Window focus, rain drops, blurred interior |
| `SCENE-05` | Balcony Sunset | City skyline, plants, warm colors |
| `SCENE-06` | Library | Bookshelves, reading nook, dust particles |
| `SCENE-07` | Kitchen Morning | Coffee brewing, sunlight, plants |
| `SCENE-08` | Rooftop Night | City lights, stars, figure relaxing |

### Motion Types (Motion LoRAs)

| ID | Motion | Application |
|----|--------|-------------|
| `MOTION-01` | Hair sway | Subtle character hair movement |
| `MOTION-02` | Rain drops | Window rain, puddle ripples |
| `MOTION-03` | Steam/smoke | Coffee steam, incense, breath |
| `MOTION-04` | Flickering light | Candles, fairy lights, monitors |
| `MOTION-05` | Curtain drift | Window curtains, fabric movement |
| `MOTION-06` | Particle float | Dust motes, snow, leaves |
| `MOTION-07` | Water reflection | Puddles, windows, mirrors |
| `MOTION-08` | Breathing | Subtle character breathing motion |

### Seasonal/Thematic Variations

Same taxonomy as lofi-lora audio:
- Winter/Christmas, Spring, Summer, Autumn
- Rainy, Sunny, Night, Golden hour
- Urban, Nature, Cozy indoor

---

## Data Curation Strategy

### Style LoRA Training Data

**Source**: Static images (not video)

| Source | Type | Estimated Count |
|--------|------|-----------------|
| Lofi album covers | Illustration | 200-500 |
| Anime background art | Screencaps | 300-500 |
| Commissioned illustrations | Custom | 50-100 |
| DeviantArt/ArtStation (CC licensed) | Fan art | 200-300 |

**Requirements**:
- High resolution (1024x1024+)
- Consistent aesthetic (cozy, warm, anime-influenced)
- No text/logos
- Proper licensing for AI training

### Motion LoRA Training Data

**Source**: Short video clips (2-5 seconds each)

| Motion Type | Source | Clips Needed |
|-------------|--------|--------------|
| Hair movement | Anime clips, stock video | 20-50 |
| Rain | Stock footage, anime | 30-50 |
| Steam/smoke | Stock footage | 20-30 |
| Flickering | Candle videos, fairy lights | 20-30 |
| Particles | Dust/snow footage | 20-30 |

**Total**: ~150-200 short clips for motion LoRAs

### Legal Sources

| Source | License | Use Case |
|--------|---------|----------|
| Pexels/Pixabay | CC0 | Motion reference clips |
| Commissioned artists | Work-for-hire | Style training images |
| Open-source anime datasets | Various | Background reference |
| Personal photography | Owned | Scene composition reference |

---

## Hardware Requirements

### Training

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| GPU | RTX 3090 (24GB) | RTX 4090 (24GB) or A100 (40GB) |
| System RAM | 32GB | 64GB |
| Storage | 500GB SSD | 1TB NVMe |
| Training time (Style LoRA) | ~4-8 hours | ~2-4 hours |
| Training time (Motion LoRA) | ~8-16 hours | ~4-8 hours |

### Inference

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| GPU | RTX 3060 (12GB) | RTX 4070+ (16GB+) |
| VRAM for SD1.5 + AnimateDiff | 8GB | 12GB |
| VRAM for SDXL + AnimateDiff | 12GB | 16GB |
| Generation time (24 frames) | ~30-60s | ~10-20s |

---

## Training Pipeline

### Phase 1: Style LoRA

Train a Stable Diffusion LoRA on lofi aesthetic images:

```
1. Curate 500-1000 lofi-style images
2. Caption with consistent tags (lofi, cozy, anime style, warm lighting, etc.)
3. Train SD LoRA (rank 32-64, 2000-4000 steps)
4. Validate: generate test images, check aesthetic consistency
```

### Phase 2: Motion LoRAs

Train specialized motion LoRAs using AnimateDiff-MotionDirector:

```
1. Collect 20-50 clips per motion type
2. Process: trim to 2-3 seconds, consistent resolution
3. Train motion LoRA per category
4. Validate: test motion isolation, loop quality
```

### Phase 3: Integration & Loop Optimization

```
1. Combine style LoRA + motion LoRA in ComfyUI
2. Implement seamless loop blending (last frame → first frame)
3. Test temporal consistency
4. Optimize for target hardware
```

---

## Repository Structure

```
lofi-vision/
├── docs/
│   └── lofi-vision.md              # This document
├── config/
│   ├── style_lora_config.yaml      # Style LoRA training config
│   └── motion_lora_config.yaml     # Motion LoRA training config
├── scripts/
│   ├── prepare_images.py           # Image dataset preparation
│   ├── prepare_clips.py            # Video clip preparation
│   ├── train_style_lora.py         # Style LoRA training
│   └── train_motion_lora.py        # Motion LoRA training (wraps MotionDirector)
├── workflows/
│   └── comfyui/                    # ComfyUI workflow JSON files
├── data/                           # Training data (gitignored)
├── requirements.txt
├── train_style.sh
├── train_motion.sh
├── .gitignore
└── README.md
```

---

## Integration with lofi.nvim

### Future Vision

```lua
-- Hypothetical lofi.nvim integration
lofi.setup({
    audio = {
        backend = "ace-step",
        lora = "lofi-beats-v1",
    },
    video = {
        enabled = true,
        backend = "animatediff",
        style_lora = "lofi-scenes-v1",
        motion_lora = "lofi-rain-v1",
    }
})

-- Generate synchronized audio + video
lofi.generate({
    prompt = "rainy night, coffee shop, jazz lofi",
    duration = 120,  -- 2 minute loop
    video = true,    -- generate matching visuals
})
```

### Standalone Usage

lofi-vision can also run independently via ComfyUI for users who just want visuals.

---

## Budget Estimate

| Category | Low | High | Notes |
|----------|-----|------|-------|
| **Training Data** | | | |
| Commissioned illustrations | $2,000 | $5,000 | 50-100 custom pieces |
| Stock video clips | $200 | $500 | Motion reference |
| **Hardware** (if needed) | | | |
| Additional GPU | $0 | $2,000 | If 4090 insufficient |
| Cloud GPU time | $100 | $500 | Overflow training |
| **Total** | **$2,300** | **$8,000** | |

Lower than lofi-lora because:
- Image data is cheaper/easier to source than licensed audio
- Shorter training times
- Fewer total training samples needed

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Data curation | 2-3 weeks | 500+ images, 150+ clips |
| Style LoRA training | 1 week | lofi-style-v1 LoRA |
| Motion LoRA training | 2 weeks | 5-8 motion LoRAs |
| Integration & testing | 1-2 weeks | ComfyUI workflows |
| Documentation | 1 week | Usage guides, examples |
| **Total** | **7-9 weeks** | |

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Style consistency | Generated images match lofi aesthetic |
| Loop seamlessness | No visible jump at loop point |
| Motion quality | Natural, subtle movement |
| VRAM usage | <12GB for SD1.5 pipeline |
| Generation speed | <60s for 24-frame loop on RTX 4090 |
| Prompt adherence | Scene matches text description |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AnimateDiff quality limitations | Medium | High | Fall back to CogVideoX if needed |
| Motion LoRA overfitting | Medium | Medium | Diverse training data, early stopping |
| Loop artifacts | Medium | Medium | Post-processing, frame blending |
| VRAM constraints | Low | Medium | Model offloading, quantization |
| Style LoRA doesn't generalize | Low | High | More diverse training data |

---

## References

### AnimateDiff Ecosystem
- [ComfyUI-AnimateDiff-Evolved](https://github.com/Kosinkadink/ComfyUI-AnimateDiff-Evolved)
- [ComfyUI-ADMotionDirector](https://github.com/kijai/ComfyUI-ADMotionDirector) - Motion LoRA training
- [AnimateDiff](https://github.com/guoyww/AnimateDiff) - Original paper/code

### Alternative Video Models (if AnimateDiff insufficient)
- [CogVideoX](https://github.com/zai-org/CogVideo) - Full video generation with LoRA support
- [HunyuanVideo](https://github.com/Tencent-Hunyuan/HunyuanVideo) - Tencent's open model
- [Wan 2.1](https://github.com/Wan-Video/Wan2.1) - Alibaba's efficient video model
- [finetrainers](https://github.com/a-r-r-o-w/finetrainers) - Video model fine-tuning library

### Training Resources
- [Diffusers LoRA Training](https://huggingface.co/docs/diffusers/training/lora)
- [CogVideoX Fine-tuning](https://huggingface.co/docs/diffusers/en/training/cogvideox)

---

## Next Steps

1. Create `lofi-vision` repository
2. Set up basic structure and documentation
3. Begin style image curation
4. Prototype with existing AnimateDiff + public LoRAs
5. Evaluate quality before committing to full training pipeline

---

## Appendix: Example Prompts

```
# Study room scene
lofi style, cozy study room, girl studying at desk, warm lamp light,
rain outside window, books stacked, plants on windowsill, night time,
anime illustration style, soft colors, peaceful atmosphere

# Coffee shop scene
lofi style, coffee shop interior, steam rising from cup, rainy day outside,
wooden tables, warm lighting, cozy atmosphere, no people, anime background style

# Rainy window scene
lofi style, close up window, rain drops on glass, blurred city lights outside,
warm interior glow, night time, melancholic mood, anime style
```
