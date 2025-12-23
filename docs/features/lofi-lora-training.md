# Lofi LoRA Training: The Definitive AI Lofi Beats Model

> **Implementation Repository**: [github.com/willibrandon/lofi-lora](https://github.com/willibrandon/lofi-lora)
>
> This document serves as the feature specification. The training scripts, data processing tools, and configs are implemented in the [lofi-lora](https://github.com/willibrandon/lofi-lora) repository. Trained models are hosted on [willibrandon/lofi-models](https://huggingface.co/willibrandon/lofi-models).

---

## Executive Summary

This feature implements a comprehensive pipeline for curating, training, and deploying a custom ACE-Step LoRA fine-tuned specifically for lofi music generation. The goal is to produce the highest-quality locally-deployable AI lofi beats model ever created, covering all sub-genres, seasonal themes, and stylistic variations of lofi music.

**Inspiration**: The [RapMachine LoRA](https://github.com/ace-step/ACE-Step/blob/main/ZH_RAP_LORA.md) demonstrates that meticulous data curation and targeted LoRA training can dramatically improve generation quality for specific styles. We apply the same methodology to lofi music.

## Vision

> "To create the Stable Diffusion moment for lofi music" - adapting ACE-Step's philosophy to the lofi genre.

The resulting LoRA will enable:
- **Superior lofi aesthetics**: Warm, vintage sound with authentic vinyl crackle and tape characteristics
- **Style diversity**: From classic ChilledCow-style beats to seasonal Christmas lofi and everything between
- **Vocal capability**: Optional chopped vocal samples and full vocal lofi tracks
- **Fine-grained control**: Prompt-based control over mood, instruments, tempo, and theme
- **Local deployment**: ONNX export for integration with lofi.nvim daemon

## Technical Foundation

### ACE-Step LoRA Training Pipeline

From [TRAIN_INSTRUCTION.md](https://github.com/ace-step/ACE-Step/blob/main/TRAIN_INSTRUCTION.md):

```
Data Preparation → HuggingFace Dataset → LoRA Training → Validation → Export
```

### Required Files Per Track

| File | Format | Purpose |
|------|--------|---------|
| `{name}.mp3` | Audio file | Source audio (30s-240s optimal) |
| `{name}_prompt.txt` | Comma-separated tags | Style descriptors |
| `{name}_lyrics.txt` | Song structure text | Lyrics or `[Instrumental]` marker |

### Hardware Requirements

| Component | Specification | Notes |
|-----------|---------------|-------|
| GPU | RTX 4090 (24GB VRAM) | Sufficient for r=256 LoRA |
| System RAM | 64GB+ | Dataset loading, preprocessing |
| Storage | 2TB+ NVMe | Raw audio + processed datasets |
| Training Time | ~48-72 hours | 40,000+ steps for convergence |

---

## Phase 1: Data Curation Strategy

### 1.1 Target Dataset Size

| Category | Track Count | Total Duration |
|----------|-------------|----------------|
| Core lofi styles | 200 tracks | ~10 hours |
| Seasonal variations | 100 tracks | ~5 hours |
| Instrumental-focused | 100 tracks | ~5 hours |
| Vocal lofi | 50 tracks | ~2.5 hours |
| Themed/niche | 50 tracks | ~2.5 hours |
| **Total** | **500 tracks** | **~25 hours** |

With `repeat_count=200`, this produces 100,000 training samples.

### 1.2 Legal Data Sources

#### Tier 1: Commissioned Original Content (Highest Quality)

**Strategy**: Commission professional lofi producers to create exclusive training data.

| Source | Cost Estimate | Tracks | Notes |
|--------|---------------|--------|-------|
| [Fiverr Pro Producers](https://www.fiverr.com/categories/music-audio/producers) | $50-150/track | 100 | Custom stems available |
| [SoundBetter](https://soundbetter.com/) | $100-300/track | 50 | Professional producers |
| [Direct Artist Commissions](https://twitter.com/search?q=lofi%20producer) | $30-100/track | 100 | Negotiate training rights |

**Budget**: ~$15,000-25,000 for 250 commissioned tracks

**Advantages**:
- 100% legal clarity for AI training
- Can request specific styles/themes
- Stems available for analysis
- Higher audio quality (lossless masters)

#### Tier 2: Royalty-Free Libraries with AI Training Rights

| Library | License Type | Cost | Notes |
|---------|--------------|------|-------|
| [Epidemic Sound](https://www.epidemicsound.com/) | Commercial subscription | $299/year | Check AI training terms |
| [Artlist](https://artlist.io/) | Universal license | $199/year | Lofi collection available |
| [Uppbeat](https://uppbeat.io/) | Royalty-free | $99/year | Growing lofi catalog |
| [Pond5](https://www.pond5.com/) | Per-track license | ~$20-50/track | Large selection |
| [Musicbed](https://www.musicbed.com/) | Commercial sync | $500-2000/yr | Premium quality |

**Important**: Contact each service to confirm AI training usage is permitted under their license terms.

#### Tier 3: Creative Commons Sources

| Source | License | Quality | Notes |
|--------|---------|---------|-------|
| [Free Music Archive](https://freemusicarchive.org/genre/Lo-Fi/) | CC BY/CC0 | Variable | Verify each track's license |
| [ccMixter](http://ccmixter.org/) | CC | Good | Remix-friendly community |
| [Bandcamp CC](https://bandcamp.com/tag/creative-commons) | CC BY/NC | Variable | Contact artists directly |
| [SoundCloud CC](https://soundcloud.com/search?q=lofi%20creative%20commons) | CC | Variable | Filter by license |

#### Tier 4: Artist Partnerships

**Strategy**: Reach out to lofi artists/labels for training data licensing agreements.

| Target | Contact Method | Proposed Terms |
|--------|----------------|----------------|
| [Lofi Girl](https://lofigirl.com/) | Business inquiry | Licensing agreement |
| [Chillhop Music](https://chillhop.com/) | Label contact | Per-track or catalog |
| [College Music](https://college.lnk.to/) | Label contact | AI training rights |
| [Dreamhop Music](https://dreamhopmusic.com/) | Direct contact | Indie-friendly |
| Individual producers | Twitter/Instagram DM | Per-track licensing |

**Template Outreach Message**:
```
Subject: AI Training Data Licensing Inquiry - Lofi Music Research

Hi [Artist/Label],

I'm developing an open-source AI music generation tool for the developer
community. I'm interested in licensing [X] tracks from your catalog
specifically for AI model training purposes.

This would involve:
- One-time licensing fee for training data rights only
- No redistribution of original audio
- Credit in model documentation
- Generated outputs would be clearly marked as AI-generated

Would you be open to discussing terms?

Best,
[Name]
```

### 1.3 Quality Requirements

| Criterion | Requirement | Verification Method |
|-----------|-------------|---------------------|
| Audio format | 320kbps MP3 or lossless | File inspection |
| Duration | 30s - 180s (optimal: 60-120s) | Script validation |
| Clipping | No digital clipping | Waveform analysis |
| Compression | Minimal limiting artifacts | Spectral analysis |
| Style accuracy | Authentic lofi characteristics | Human review |
| Noise floor | Clean recording (vinyl noise OK if stylistic) | Listening test |

---

## Phase 2: Style Taxonomy

### 2.1 Core Lofi Styles

| Style ID | Name | Key Characteristics | Example Tags |
|----------|------|---------------------|--------------|
| `CORE-01` | Classic Lofi Hip Hop | Boom-bap drums, jazz samples, vinyl crackle | `lofi hip hop, chill beats, vinyl crackle, boom bap drums, jazz samples` |
| `CORE-02` | Jazz Lofi | Heavy piano/keys, saxophone, complex chords | `jazz lofi, piano, rhodes, saxophone, complex harmony, swing rhythm` |
| `CORE-03` | Ambient Lofi | Atmospheric pads, minimal drums, reverb-heavy | `ambient lofi, atmospheric, pads, reverb, ethereal, minimal drums` |
| `CORE-04` | Lofi House | Uptempo (110-120bpm), house influence, filtered loops | `lofi house, filtered, house beat, uptempo, disco samples` |
| `CORE-05` | Chillhop | Upbeat lofi, positive mood, funk elements | `chillhop, upbeat, positive, funk bassline, groovy` |
| `CORE-06` | Dark Lofi | Minor keys, melancholic, deeper bass | `dark lofi, melancholic, minor key, deep bass, moody` |
| `CORE-07` | Bedroom Pop Lofi | Indie influence, dreamy vocals, lo-fi production | `bedroom pop, indie, dreamy, lo-fi production, intimate` |

### 2.2 Seasonal & Holiday Themes

| Style ID | Name | Key Characteristics | Season |
|----------|------|---------------------|--------|
| `SEASON-01` | Winter Lofi | Warm pads, sleigh bells, cozy atmosphere | December-February |
| `SEASON-02` | Christmas Lofi | Holiday melodies, bells, festive but chill | December |
| `SEASON-03` | Spring Lofi | Light, airy, nature sounds, renewal themes | March-May |
| `SEASON-04` | Summer Lofi | Beach vibes, tropical elements, bright | June-August |
| `SEASON-05` | Autumn/Fall Lofi | Warm tones, nostalgic, acoustic elements | September-November |
| `SEASON-06` | Rainy Day Lofi | Rain ambience, melancholic, contemplative | Year-round |
| `SEASON-07` | Halloween Lofi | Spooky elements, minor keys, eerie samples | October |
| `SEASON-08` | Valentine Lofi | Romantic, soft, love-themed samples | February |
| `SEASON-09` | New Year Lofi | Reflective, hopeful, transition themes | December-January |

### 2.3 Thematic/Setting Variations

| Style ID | Name | Key Characteristics |
|----------|------|---------------------|
| `THEME-01` | Coffee Shop Lofi | Café ambience, acoustic guitar, morning vibes |
| `THEME-02` | Late Night Lofi | Dark atmosphere, slow tempo, urban feel |
| `THEME-03` | Study Lofi | Focus-oriented, minimal distraction, steady rhythm |
| `THEME-04` | Sleep/Relaxation Lofi | Ultra-chill, ambient elements, drone-like |
| `THEME-05` | City Night Lofi | Urban soundscape, neon aesthetic, synthwave influence |
| `THEME-06` | Nature Lofi | Forest/ocean sounds, organic instruments |
| `THEME-07` | Gaming Lofi | 8-bit influence, chiptune elements, retro |
| `THEME-08` | Anime Lofi | Japanese influence, nostalgic, visual kei elements |
| `THEME-09` | Sunset/Golden Hour Lofi | Warm, golden tones, transitional mood |
| `THEME-10` | Sunday Morning Lofi | Lazy, peaceful, gentle awakening |

### 2.4 Instrumental Focus

| Style ID | Name | Primary Instrument |
|----------|------|-------------------|
| `INST-01` | Piano Lofi | Acoustic/electric piano lead |
| `INST-02` | Guitar Lofi | Acoustic or clean electric guitar |
| `INST-03` | Synth Lofi | Analog synth textures, pad-heavy |
| `INST-04` | Rhodes/Wurlitzer Lofi | Electric piano, warm keys |
| `INST-05` | Saxophone Lofi | Jazz saxophone featured |
| `INST-06` | Strings Lofi | Orchestral string samples |
| `INST-07` | Flute Lofi | Woodwind elements, airy |

### 2.5 Vocal Variations

| Style ID | Name | Vocal Type |
|----------|------|------------|
| `VOCAL-01` | Chopped Vocals | Sliced/processed vocal samples |
| `VOCAL-02` | Japanese Lofi | Japanese lyrics, city pop influence |
| `VOCAL-03` | R&B Lofi | Soulful vocals, R&B influence |
| `VOCAL-04` | Spoken Word Lofi | Poetry, narration overlays |
| `VOCAL-05` | Humming/Wordless | Non-lexical vocalization |

---

## Phase 3: Prompt Engineering System

### 3.1 Tag Categories

Following ACE-Step's proven tagging approach from ZH_RAP_LORA.md:

```python
PROMPT_CATEGORIES = {
    "genre": [
        "lofi hip hop", "chillhop", "jazz lofi", "ambient lofi",
        "lofi house", "dark lofi", "bedroom pop lofi"
    ],
    "mood": [
        "chill", "relaxing", "melancholic", "nostalgic", "dreamy",
        "cozy", "peaceful", "introspective", "romantic", "hopeful"
    ],
    "instruments": [
        "piano", "rhodes", "wurlitzer", "acoustic guitar", "electric guitar",
        "saxophone", "flute", "strings", "synth pads", "bass guitar",
        "drum machine", "vinyl crackle", "tape hiss"
    ],
    "tempo": [
        "slow tempo", "70 bpm", "80 bpm", "90 bpm", "medium tempo"
    ],
    "production": [
        "vinyl crackle", "tape saturation", "lo-fi production",
        "warm compression", "analog warmth", "side-chain compression"
    ],
    "atmosphere": [
        "rainy", "late night", "coffee shop", "urban", "nature",
        "winter", "summer", "autumn", "spring"
    ],
    "vocal_type": [
        "instrumental", "chopped vocals", "female vocal sample",
        "male vocal sample", "humming", "wordless vocals"
    ]
}
```

### 3.2 Prompt Template

Each track's `*_prompt.txt` file follows this structure:

```
{genre}, {mood}, {instruments...}, {tempo}, {production...}, {atmosphere}, {vocal_type}
```

**Examples**:

```
# Classic lofi hip hop
lofi hip hop, chill, relaxing, piano, rhodes, boom bap drums, vinyl crackle,
tape saturation, 85 bpm, late night, instrumental

# Christmas lofi
jazz lofi, cozy, festive, piano, sleigh bells, warm bass, vinyl crackle,
soft drums, 78 bpm, winter, christmas, holiday, instrumental

# Vocal lofi with Japanese influence
lofi hip hop, nostalgic, dreamy, rhodes, acoustic guitar, female vocal sample,
japanese influence, vinyl crackle, 82 bpm, city night, chopped vocals

# Rainy day ambient lofi
ambient lofi, melancholic, introspective, synth pads, piano, rain ambience,
minimal drums, reverb heavy, slow tempo, 65 bpm, rainy, instrumental

# Summer beach lofi
chillhop, upbeat, tropical, acoustic guitar, bongos, warm bass,
filtered samples, 95 bpm, summer, beach vibes, sunset, instrumental
```

### 3.3 Lyrics File Format

For **instrumental tracks**:
```
[Instrumental]
```

For **vocal tracks** (following ACE-Step structure):
```
[Intro]
(ambient sounds, vinyl crackle)

[Verse]
Walking through the city lights
Memories fade into the night
Coffee steam and neon glow
Time moves fast but feels so slow

[Chorus]
Let it go, let it flow
Like the music soft and low
In this moment, we are free
Just the beat and you and me

[Outro]
(fade out with vinyl crackle)
```

---

## Repository Setup

The lofi-lora repository requires:

- `requirements.txt` - Python dependencies (torch, diffusers, peft, librosa, etc.)
- `config/lofi_lora_config.json` - LoRA hyperparameters
- `scripts/` - Audio processing and dataset conversion utilities
- `train.sh` - Wrapper script invoking ACE-Step's trainer.py
- `.gitignore` - Exclude data/, checkpoints/, and generated datasets

ACE-Step should be installed as an editable dependency from its local clone.

---

## Phase 4: Data Processing Pipeline

### 4.1 Directory Structure

```
lofi-training-data/
├── raw/                          # Original source files
│   ├── commissioned/             # Tier 1: Commissioned tracks
│   ├── royalty-free/            # Tier 2: Licensed library tracks
│   ├── creative-commons/        # Tier 3: CC-licensed tracks
│   └── partnerships/            # Tier 4: Artist partnerships
├── processed/                    # Cleaned and validated
│   ├── core/                    # CORE-01 through CORE-07
│   ├── seasonal/                # SEASON-01 through SEASON-09
│   ├── thematic/                # THEME-01 through THEME-10
│   ├── instrumental/            # INST-01 through INST-07
│   └── vocal/                   # VOCAL-01 through VOCAL-05
├── data/                        # Final training data
│   ├── track_001.mp3
│   ├── track_001_prompt.txt
│   ├── track_001_lyrics.txt
│   └── ...
├── validation/                  # Hold-out set (10%)
└── metadata/
    ├── sources.json             # Licensing info per track
    ├── style_mapping.json       # Style ID assignments
    └── quality_scores.json      # Human evaluation scores
```

### 4.2 Audio Processing Script

```python
#!/usr/bin/env python3
"""
lofi_audio_processor.py
Standardizes audio files for ACE-Step LoRA training.
"""

import os
import subprocess
from pathlib import Path
import librosa
import soundfile as sf
import numpy as np

class LofiAudioProcessor:
    """Processes raw audio into training-ready format."""

    TARGET_SR = 48000  # ACE-Step native sample rate
    TARGET_DURATION_MIN = 30
    TARGET_DURATION_MAX = 180
    TARGET_DURATION_OPTIMAL = 90

    def __init__(self, input_dir: str, output_dir: str):
        self.input_dir = Path(input_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def process_file(self, audio_path: Path) -> dict:
        """Process single audio file."""
        # Load audio
        y, sr = librosa.load(audio_path, sr=self.TARGET_SR, mono=False)

        # Convert to stereo if mono
        if y.ndim == 1:
            y = np.stack([y, y])

        duration = len(y[0]) / sr

        # Validate duration
        if duration < self.TARGET_DURATION_MIN:
            return {"status": "skipped", "reason": "too_short", "duration": duration}

        # Trim to max duration if needed
        if duration > self.TARGET_DURATION_MAX:
            samples = int(self.TARGET_DURATION_MAX * sr)
            y = y[:, :samples]
            duration = self.TARGET_DURATION_MAX

        # Normalize peak to -1dB
        peak = np.max(np.abs(y))
        target_peak = 10 ** (-1 / 20)  # -1dB
        y = y * (target_peak / peak)

        # Quality checks
        quality = self._assess_quality(y, sr)
        if not quality["passed"]:
            return {"status": "failed", "reason": quality["reason"]}

        # Export as high-quality MP3
        output_path = self.output_dir / f"{audio_path.stem}.mp3"
        self._export_mp3(y, sr, output_path)

        return {
            "status": "success",
            "output_path": str(output_path),
            "duration": duration,
            "quality_score": quality["score"]
        }

    def _assess_quality(self, y: np.ndarray, sr: int) -> dict:
        """Assess audio quality."""
        # Check for clipping
        clipping_threshold = 0.99
        clipping_samples = np.sum(np.abs(y) > clipping_threshold)
        clipping_ratio = clipping_samples / y.size

        if clipping_ratio > 0.001:  # More than 0.1% clipping
            return {"passed": False, "reason": "excessive_clipping", "score": 0}

        # Check for silence (>50% of track)
        silence_threshold = 0.01
        silence_ratio = np.sum(np.abs(y) < silence_threshold) / y.size

        if silence_ratio > 0.5:
            return {"passed": False, "reason": "excessive_silence", "score": 0}

        # Calculate quality score (0-100)
        score = 100 - (clipping_ratio * 1000) - (silence_ratio * 50)
        score = max(0, min(100, score))

        return {"passed": True, "score": score, "reason": None}

    def _export_mp3(self, y: np.ndarray, sr: int, output_path: Path):
        """Export audio as 320kbps MP3."""
        # First export as WAV
        temp_wav = output_path.with_suffix('.wav')
        sf.write(temp_wav, y.T, sr)

        # Convert to MP3 using ffmpeg
        subprocess.run([
            'ffmpeg', '-y', '-i', str(temp_wav),
            '-codec:a', 'libmp3lame', '-b:a', '320k',
            str(output_path)
        ], check=True, capture_output=True)

        # Remove temp WAV
        temp_wav.unlink()

    def process_directory(self):
        """Process all audio files in input directory."""
        results = []
        audio_extensions = {'.mp3', '.wav', '.flac', '.m4a', '.ogg'}

        for audio_path in self.input_dir.rglob('*'):
            if audio_path.suffix.lower() in audio_extensions:
                result = self.process_file(audio_path)
                result["source"] = str(audio_path)
                results.append(result)

        return results
```

### 4.3 Prompt Generation Tool

```python
#!/usr/bin/env python3
"""
lofi_prompt_generator.py
Interactive tool for generating consistent prompt tags.
"""

import json
from pathlib import Path

STYLE_PRESETS = {
    "CORE-01": {
        "base_tags": ["lofi hip hop", "chill", "boom bap drums", "vinyl crackle"],
        "instruments": ["piano", "rhodes", "bass guitar"],
        "production": ["tape saturation", "analog warmth", "side-chain compression"]
    },
    "CORE-02": {
        "base_tags": ["jazz lofi", "sophisticated", "complex harmony"],
        "instruments": ["piano", "saxophone", "upright bass", "brushed drums"],
        "production": ["warm compression", "vinyl crackle", "room reverb"]
    },
    "SEASON-02": {
        "base_tags": ["christmas lofi", "cozy", "festive", "holiday"],
        "instruments": ["piano", "sleigh bells", "music box", "soft strings"],
        "production": ["vinyl crackle", "warm", "gentle compression"]
    },
    # ... additional presets for all style IDs
}

MOOD_TAGS = [
    "chill", "relaxing", "melancholic", "nostalgic", "dreamy",
    "cozy", "peaceful", "introspective", "romantic", "hopeful",
    "contemplative", "serene", "wistful", "bittersweet", "warm"
]

TEMPO_MAPPING = {
    "very_slow": "60 bpm, very slow tempo",
    "slow": "70 bpm, slow tempo",
    "medium_slow": "80 bpm, laid-back tempo",
    "medium": "85 bpm, medium tempo",
    "medium_fast": "90 bpm, groovy tempo",
    "upbeat": "100 bpm, upbeat tempo"
}

def generate_prompt(style_id: str, mood: str, tempo: str,
                   extra_instruments: list = None,
                   atmosphere: str = None,
                   is_vocal: bool = False) -> str:
    """Generate a complete prompt string for a track."""

    preset = STYLE_PRESETS.get(style_id, STYLE_PRESETS["CORE-01"])

    tags = []
    tags.extend(preset["base_tags"])
    tags.append(mood)
    tags.extend(preset["instruments"])
    if extra_instruments:
        tags.extend(extra_instruments)
    tags.extend(preset["production"])
    tags.append(TEMPO_MAPPING.get(tempo, "80 bpm"))

    if atmosphere:
        tags.append(atmosphere)

    tags.append("chopped vocals" if is_vocal else "instrumental")

    return ", ".join(tags)


def create_track_files(track_name: str, audio_path: Path,
                       prompt: str, lyrics: str = "[Instrumental]"):
    """Create the required training files for a track."""

    base_path = audio_path.parent / track_name

    # Write prompt file
    prompt_path = Path(f"{base_path}_prompt.txt")
    prompt_path.write_text(prompt)

    # Write lyrics file
    lyrics_path = Path(f"{base_path}_lyrics.txt")
    lyrics_path.write_text(lyrics)

    return prompt_path, lyrics_path
```

---

## Phase 5: Training Configuration

### 5.1 LoRA Configuration

Create `config/lofi_lora_config.json`:

```json
{
    "r": 256,
    "lora_alpha": 32,
    "target_modules": [
        "speaker_embedder",
        "linear_q",
        "linear_k",
        "linear_v",
        "to_q",
        "to_k",
        "to_v",
        "to_out.0"
    ],
    "use_rslora": true
}
```

**Configuration rationale**:
- `r=256`: High rank captures stylistic nuances (same as RapMachine)
- `lora_alpha=32`: Standard alpha for stable training
- `speaker_embedder`: Enables vocal characteristics learning
- Attention modules: Full attention pathway for style transfer
- `use_rslora=true`: Rank-stabilized LoRA for better convergence

### 5.2 Training Parameters

```bash
python trainer.py \
    --dataset_path "./lofi_lora_dataset" \
    --exp_name "lofi_beats_v1" \
    --lora_config_path "config/lofi_lora_config.json" \
    --checkpoint_dir "~/.cache/ace-step/checkpoints" \
    --learning_rate 1e-4 \
    --max_steps 100000 \
    --every_n_train_steps 5000 \
    --every_plot_step 5000 \
    --num_workers 8 \
    --precision "bf16-mixed" \
    --gradient_clip_val 0.5 \
    --devices 1 \
    --logger_dir "./exps/logs/"
```

**Hyperparameter rationale**:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `learning_rate` | 1e-4 | Standard for LoRA, proven in RapMachine |
| `max_steps` | 100,000 | 500 tracks × 200 repeats = 100k samples |
| `every_n_train_steps` | 5,000 | Checkpoint every 5k for recovery |
| `every_plot_step` | 5,000 | Generate samples for quality monitoring |
| `precision` | bf16-mixed | RTX 4090 supports bfloat16, saves VRAM |
| `gradient_clip_val` | 0.5 | Prevent gradient explosion |

### 5.3 Training Timeline (RTX 4090)

| Phase | Steps | Duration | Notes |
|-------|-------|----------|-------|
| Initial convergence | 0-20,000 | ~12 hours | Loss stabilizes |
| Style learning | 20,000-60,000 | ~24 hours | Lofi characteristics emerge |
| Refinement | 60,000-100,000 | ~24 hours | Fine details improve |
| **Total** | 100,000 | ~60 hours | 2.5 days continuous |

### 5.4 VRAM Optimization (if needed)

If 24GB VRAM is insufficient with r=256:

```json
{
    "r": 128,
    "lora_alpha": 16,
    "target_modules": [
        "linear_q",
        "linear_k",
        "linear_v",
        "to_q",
        "to_k",
        "to_v",
        "to_out.0"
    ],
    "use_rslora": true
}
```

Note: Removing `speaker_embedder` reduces vocal learning but saves VRAM.

---

## Phase 6: Quality Validation

### 6.1 Objective Metrics

| Metric | Tool | Target |
|--------|------|--------|
| FAD (Fréchet Audio Distance) | `frechet_audio_distance` | < 5.0 vs reference lofi |
| CLAP Score | `laion/clap-htsat-unfused` | > 0.7 prompt alignment |
| Audio Quality | PESQ/ViSQOL | > 4.0 MOS |
| Style Consistency | Embedding clustering | Tight lofi cluster |

### 6.2 Subjective Evaluation

**Listening Test Protocol**:

1. Generate 50 samples across all style categories
2. Mix with 50 real lofi tracks (blind test)
3. Rate each on:
   - Lofi authenticity (1-5)
   - Production quality (1-5)
   - Prompt adherence (1-5)
   - Musicality (1-5)
4. Target: AI tracks indistinguishable from real (p > 0.05)

### 6.3 Checkpoint Selection

At each 5,000-step checkpoint:

1. Generate test suite (one sample per style category)
2. Compute FAD against validation set
3. Human review of generated samples
4. Track best checkpoint by combined score

---

## Phase 7: ONNX Export & Integration

### 7.1 LoRA Merging

After training, merge LoRA weights into base model:

```python
from peft import PeftModel
from acestep.pipeline_ace_step import ACEStepPipeline

# Load base model
pipeline = ACEStepPipeline(checkpoint_dir)
pipeline.load_checkpoint()

# Load and merge LoRA
model = PeftModel.from_pretrained(
    pipeline.ace_step_transformer,
    "exps/logs/.../checkpoints/epoch=0-step=100000_lora"
)
merged_model = model.merge_and_unload()

# Save merged model
merged_model.save_pretrained("./lofi_ace_step_merged")
```

### 7.2 ONNX Export

Export merged model to ONNX (same process as base ACE-Step):

```python
import torch

# Export transformer
torch.onnx.export(
    merged_model,
    dummy_inputs,
    "lofi_ace_step_transformer.onnx",
    opset_version=17,
    input_names=["hidden_states", "attention_mask", "encoder_text_hidden_states",
                 "text_attention_mask", "speaker_embeds", "lyric_token_idx",
                 "lyric_mask", "timestep"],
    dynamic_axes={"hidden_states": {3: "seq_len"}, "attention_mask": {1: "seq_len"}}
)
```

### 7.3 lofi.nvim Integration

Update daemon to support custom LoRA models:

```lua
-- lua/lofi/init.lua
lofi.setup({
    model = {
        backend = "ace-step",
        lora = "lofi-beats-v1",  -- Custom LoRA identifier
    }
})
```

```rust
// daemon/src/models/ace_step/config.rs
pub struct AceStepConfig {
    pub checkpoint_dir: PathBuf,
    pub lora_path: Option<PathBuf>,  // Optional LoRA overlay
}
```

---

## Phase 8: Distribution

### 8.1 Model Hosting

| Platform | Purpose | Access |
|----------|---------|--------|
| HuggingFace Hub | Primary distribution | [`willibrandon/lofi-models`](https://huggingface.co/willibrandon/lofi-models) |
| GitHub Releases | Versioned downloads | Attached to lofi.nvim releases |
| Mirror (optional) | Backup hosting | Self-hosted CDN |

### 8.2 Model Card

```yaml
# lofi-ace-step-lora-v1/README.md
---
license: apache-2.0
tags:
  - music-generation
  - lofi
  - ace-step
  - lora
datasets:
  - custom-lofi-dataset
language:
  - en
pipeline_tag: text-to-audio
---

# Lofi Beats LoRA for ACE-Step

Fine-tuned LoRA adapter for ACE-Step v1-3.5B, specialized in lofi hip hop
and related genres.

## Usage

```python
from acestep.pipeline_ace_step import ACEStepPipeline

pipeline = ACEStepPipeline()
pipeline.load_lora("willibrandon/lofi-models")

audio = pipeline.generate(
    prompt="lofi hip hop, chill, piano, vinyl crackle, 80 bpm, late night",
    duration=120,
    infer_step=27
)
```

## Training Data

- 500 curated lofi tracks
- 25 hours total audio
- Professionally licensed/commissioned
- Covers 30+ style variations

## Prompt Guidelines

See [PROMPTING.md](./PROMPTING.md) for detailed tag recommendations.
```

---

## Phase 9: Model Size Optimization

### 9.1 Current Model Footprint

| Component | Size | Parameters | Purpose |
|-----------|------|------------|---------|
| ACE-Step Transformer | 6.2GB | ~3.5B | Diffusion denoiser |
| UMT5-base | 1.1GB | ~300M | Text encoder |
| MusicDCAE | 299MB | ~100M | Latent decoder |
| Vocoder | 197MB | ~50M | Mel → Audio |
| **Total** | **7.7GB** | **~4B** | |

### 9.2 Optimization Strategy

Based on [PTQ4ADM](https://arxiv.org/abs/2409.13894) research demonstrating 70% model size reduction with <5% quality loss on audio diffusion models, and [Post-Training Quantization for Audio Diffusion Transformers](https://arxiv.org/html/2510.00313) showing INT8 models remain perceptually close to baseline.

#### Tier 1: FP16 Baseline (Conservative)

**Target: 3.85GB (50% reduction)**

| Component | Before | After | Method |
|-----------|--------|-------|--------|
| Transformer | 6.2GB | 3.1GB | FP32→FP16 |
| UMT5 | 1.1GB | 550MB | FP32→FP16 |
| DCAE | 299MB | 150MB | FP32→FP16 |
| Vocoder | 197MB | 99MB | FP32→FP16 |
| **Total** | 7.7GB | **~3.9GB** | |

- **Quality impact**: None
- **Effort**: 1 day (re-export with `torch.float16`)
- **Status**: Ship as default

#### Tier 2: Mixed INT8/FP16 (Recommended)

**Target: 2.0-2.5GB (68-74% reduction)**

| Component | Precision | Size | Rationale |
|-----------|-----------|------|-----------|
| Transformer | W8A16 | ~1.55GB | Main savings, diffusion-safe |
| UMT5 | INT8 | ~275MB | Text encoders quantize well |
| Lyric Encoder | INT8 | ~20MB | Conformer architecture, INT8-safe |
| DCAE | FP16 | 150MB | Audio-critical, preserve precision |
| Vocoder | FP16 | 99MB | Audio-critical, preserve precision |
| **Total** | Mixed | **~2.1GB** | |

- **Quality impact**: <5% FAD increase (per PTQ4ADM findings)
- **Effort**: ~1 week (calibration dataset required)
- **Hardware**: Full ONNX Runtime support, works on CPU

#### Tier 3: Aggressive INT4/INT8 (Maximum Compression)

**Target: 1.2-1.5GB (80-85% reduction)**

Based on [TinyMusician](https://arxiv.org/html/2509.00914) mixed-precision approach.

| Component | Precision | Size | Rationale |
|-----------|-----------|------|-----------|
| Transformer backbone | W4A8 | ~775MB | AWQ calibration |
| Transformer attention | INT8 | (included) | Attention more sensitive |
| UMT5 | INT8 | ~275MB | Standard text encoder |
| Lyric Encoder | INT8 | ~20MB | Preserve vocal capability |
| DCAE | FP16 | 150MB | Must preserve for audio quality |
| Vocoder | FP16 | 99MB | Must preserve for audio quality |
| **Total** | Mixed | **~1.3GB** | |

- **Quality impact**: ~10-15% FAD increase, requires validation
- **Effort**: ~2 weeks (careful layer selection, AWQ calibration)
- **Hardware**: Requires ONNX opset 21+, ONNX Runtime 1.20+

### 9.3 Quantization Implementation

#### Calibration Dataset

Create a lofi-specific calibration set for optimal quantization:

```python
CALIBRATION_PROMPTS = [
    # Core styles
    "lofi hip hop, chill, piano, vinyl crackle, 80 bpm, late night, instrumental",
    "jazz lofi, saxophone, rhodes, brushed drums, warm, 75 bpm, coffee shop",
    "ambient lofi, pads, minimal drums, reverb, ethereal, slow tempo",

    # Seasonal
    "christmas lofi, cozy, piano, sleigh bells, festive, winter, 78 bpm",
    "summer lofi, beach vibes, acoustic guitar, bright, upbeat, 95 bpm",
    "rainy day lofi, melancholic, piano, rain ambience, contemplative",

    # Vocal (important to include for lyric encoder calibration)
    "lofi hip hop, female vocal, dreamy, chopped vocals, nostalgic, 82 bpm",
    "japanese lofi, male vocal, city pop influence, neon, night, 85 bpm",

    # ... 100+ diverse prompts covering all style categories
]
```

#### ONNX Export with Quantization

```python
from onnxruntime.quantization import quantize_dynamic, QuantType
import onnx

def quantize_ace_step_model(fp32_model_path: str, output_path: str, tier: str = "tier2"):
    """
    Quantize ACE-Step ONNX model with tier-specific settings.
    """
    if tier == "tier1":
        # FP16 conversion
        from onnxconverter_common import float16
        model = onnx.load(fp32_model_path)
        model_fp16 = float16.convert_float_to_float16(model)
        onnx.save(model_fp16, output_path)

    elif tier == "tier2":
        # INT8 weights, FP16 activations
        quantize_dynamic(
            model_input=fp32_model_path,
            model_output=output_path,
            weight_type=QuantType.QInt8,
            extra_options={
                "ActivationSymmetric": False,
                "WeightSymmetric": True,
            }
        )

    elif tier == "tier3":
        # INT4 with AWQ calibration (requires modelopt)
        from modelopt.onnx.quantization import quantize_int4
        quantize_int4(
            onnx_path=fp32_model_path,
            output_path=output_path,
            calibration_method="awq_lite",
            # ... calibration data config
        )
```

### 9.4 Quality Validation Protocol

After quantization, validate quality hasn't degraded unacceptably:

| Metric | FP32 Baseline | Tier 1 Target | Tier 2 Target | Tier 3 Target |
|--------|---------------|---------------|---------------|---------------|
| FAD Score | X | X ± 0% | X + 5% max | X + 15% max |
| CLAP Alignment | Y | Y ± 0% | Y - 3% max | Y - 10% max |
| Vocal Clarity | Subjective | Same | Same | -1 MOS max |
| Lyric Alignment | Z | Z ± 0% | Z - 5% max | Z - 15% max |

**A/B Testing Protocol**:
1. Generate 50 samples with FP32 model
2. Generate same 50 samples (same seeds) with quantized model
3. Blind listening test: "Which sounds better?"
4. Accept if quantized wins >40% (within noise margin)

### 9.5 Component-Specific Notes

#### Preserving Vocal/Lyric Capability

The lyric encoder (Conformer architecture) and speaker embedder are **kept at INT8 minimum** to preserve:
- Lyric-to-audio alignment accuracy
- Vocal timbre characteristics
- Pronunciation clarity (especially for lofi vocal tracks)

```python
# LoRA config includes speaker_embedder for vocal learning
{
    "target_modules": [
        "speaker_embedder",  # Keep for vocal capability
        "linear_q", "linear_k", "linear_v",
        "to_q", "to_k", "to_v", "to_out.0"
    ]
}
```

#### Audio-Critical Components (No INT4)

DCAE decoder and Vocoder remain at **FP16 minimum**:
- Direct impact on audio waveform quality
- Mel spectrogram artifacts propagate to final audio
- Vocoder distortion is perceptually obvious

### 9.6 Deployment Options

| Variant | Size | Quality | Use Case |
|---------|------|---------|----------|
| `lofi-ace-step-fp16` | 3.9GB | 100% | Default, quality-first |
| `lofi-ace-step-int8` | 2.1GB | ~95% | Balanced, recommended |
| `lofi-ace-step-int4` | 1.3GB | ~85% | Mobile/edge, experimental |

lofi.nvim configuration:

```lua
lofi.setup({
    model = {
        backend = "ace-step",
        lora = "lofi-beats-v1",
        precision = "int8",  -- "fp16" | "int8" | "int4"
    }
})
```

### 9.7 Size Optimization Summary

| Optimization | Size | Reduction | Quality | Effort |
|--------------|------|-----------|---------|--------|
| Baseline FP32 | 7.7GB | - | 100% | - |
| **Tier 1: FP16** | 3.9GB | 50% | 100% | 1 day |
| **Tier 2: INT8 Mixed** | 2.1GB | 73% | ~95% | 1 week |
| **Tier 3: INT4 Mixed** | 1.3GB | 83% | ~85% | 2 weeks |

**Primary Target**: Tier 2 (2.1GB) - Best balance of size, quality, and compatibility.

---

## Budget Summary

| Category | Low Estimate | High Estimate | Notes |
|----------|--------------|---------------|-------|
| **Data Acquisition** | | | |
| Commissioned tracks (250) | $7,500 | $25,000 | $30-100/track |
| Royalty-free licenses | $500 | $2,000 | Annual subscriptions |
| Artist partnerships | $2,000 | $10,000 | Bulk licensing deals |
| **Infrastructure** | | | |
| Cloud GPU (if needed) | $0 | $500 | RTX 4090 available locally |
| Storage (2TB NVMe) | $150 | $300 | If additional needed |
| **Human Evaluation** | | | |
| Listening test participants | $500 | $1,000 | MTurk/Prolific |
| **Contingency** | $1,000 | $3,000 | Unexpected costs |
| **Total** | **$11,650** | **$41,800** | |

**Recommended budget**: ~$20,000 for high-quality result

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1-2 | Data acquisition | 100 commissioned tracks ordered |
| 2-3 | Licensing | Royalty-free subscriptions, artist outreach |
| 3-4 | Data collection | All 500 tracks acquired |
| 4-5 | Processing | Audio cleaned, prompts generated |
| 5-6 | Dataset creation | HuggingFace dataset ready |
| 6-7 | Training | 100k steps completed |
| 7-8 | Validation | Quality metrics, human evaluation |
| 8-9 | ONNX export | Integration with lofi.nvim |
| 9-10 | Release | HuggingFace upload, documentation |

**Total timeline**: 10 weeks

---

## Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| FAD score | < 5.0 | vs. held-out lofi validation set |
| CLAP alignment | > 0.7 | Prompt-to-audio similarity |
| Human preference | > 45% | Indistinguishable from real lofi |
| Style coverage | 100% | All 30+ styles generate correctly |
| Vocal/lyric alignment | > 0.8 | Lyric-to-audio sync accuracy |
| Generation speed | < 5s/min | On RTX 4090 at 27 steps |
| ONNX inference | Working | Full pipeline in lofi.nvim daemon |
| Model size (INT8) | ≤ 2.5GB | 73%+ reduction from FP32 |
| Quantized quality | ≥ 95% | FAD within 5% of FP32 baseline |

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Licensing disputes | Low | High | Document all sources, prefer commissioned |
| Training divergence | Medium | Medium | Frequent checkpoints, early stopping |
| VRAM limitations | Low | Medium | Reduce r value, gradient checkpointing |
| Style collapse | Medium | High | Diverse dataset, balanced sampling |
| ONNX export failures | Low | High | Test export before training ends |
| Quantization quality loss | Medium | Medium | A/B testing, tier fallback options |
| Vocal degradation (INT4) | Medium | High | Keep lyric encoder at INT8 minimum |
| ONNX opset compatibility | Low | Medium | Target opset 17 for INT8, 21 for INT4 |

---

## References

### Training & Fine-tuning
- [ACE-Step Training Instructions](https://github.com/ace-step/ACE-Step/blob/main/TRAIN_INSTRUCTION.md)
- [RapMachine LoRA](https://github.com/ace-step/ACE-Step/blob/main/ZH_RAP_LORA.md)
- [ACE-Step Technical Report](https://arxiv.org/abs/2506.00045)
- [LoRA: Low-Rank Adaptation](https://arxiv.org/abs/2106.09685)
- [PEFT Library](https://github.com/huggingface/peft)

### Model Optimization & Quantization
- [PTQ4ADM: Post-Training Quantization for Audio Diffusion Models](https://arxiv.org/abs/2409.13894)
- [Post-Training Quantization for Audio Diffusion Transformers](https://arxiv.org/html/2510.00313)
- [TinyMusician: On-Device Music Generation](https://arxiv.org/html/2509.00914)
- [ONNX Runtime Quantization Guide](https://onnxruntime.ai/docs/performance/model-optimizations/quantization.html)
- [NVIDIA TensorRT INT8 for Diffusion Models](https://developer.nvidia.com/blog/tensorrt-accelerates-stable-diffusion-nearly-2x-faster-with-8-bit-post-training-quantization/)

---

## Appendix A: Complete Style Tag Reference

See [STYLE_TAGS.md](./lofi-style-tags.md) for the complete taxonomy with example prompts for each of the 30+ style variations.

## Appendix B: Data Source Tracking Template

```json
{
    "track_id": "track_001",
    "filename": "track_001.mp3",
    "source": {
        "type": "commissioned",
        "provider": "Fiverr",
        "artist": "ChillBeatsProducer",
        "license": "AI Training Rights",
        "cost": 75.00,
        "date_acquired": "2025-01-15"
    },
    "style_ids": ["CORE-01", "THEME-03"],
    "quality_score": 92,
    "human_verified": true
}
```

## Appendix C: Vocal Lofi Lyrics Templates

For tracks requiring generated lyrics with ACE-Step:

```
[Verse 1]
{4 lines, introspective theme}
{AABB or ABAB rhyme scheme}
{Imagery: urban, night, memory}

[Chorus]
{2-4 lines, emotional hook}
{Repetition-friendly}

[Verse 2]
{4 lines, development}
{Same rhyme scheme as Verse 1}

[Outro]
{Fade with ambient sounds}
```

Example:
```
[Verse]
City lights blur through the rain
Coffee cold, I feel the same
Headphones on, the world fades out
Lost in beats, no room for doubt

[Chorus]
Let the lofi take me home
Through the static, never alone

[Outro]
(vinyl crackle, rain sounds fade)
```
