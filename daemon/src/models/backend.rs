//! Backend abstraction for multiple music generation models.
//!
//! This module provides a unified interface for MusicGen and ACE-Step backends,
//! allowing seamless switching between generation models.

use serde::{Deserialize, Serialize};

use crate::error::{DaemonError, Result};

use super::ace_step::AceStepModels;
use super::musicgen::MusicGenModels;

/// Available music generation backends.
///
/// Each backend has different capabilities and characteristics:
/// - **MusicGen**: Fast, ~30s max duration, 32kHz output
/// - **AceStep**: Slower, up to 240s duration, 48kHz output, diffusion-based
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    /// MusicGen model - Meta's autoregressive audio generation.
    /// Best for short clips, fast generation.
    #[default]
    MusicGen,

    /// ACE-Step model - Diffusion-based long-form generation.
    /// Supports up to 240 seconds, higher quality, but slower.
    AceStep,
}

impl Backend {
    /// Returns the string representation of the backend.
    pub fn as_str(&self) -> &'static str {
        match self {
            Backend::MusicGen => "musicgen",
            Backend::AceStep => "ace_step",
        }
    }

    /// Parses a backend from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "musicgen" | "music_gen" => Some(Backend::MusicGen),
            "acestep" | "ace_step" | "ace-step" => Some(Backend::AceStep),
            _ => None,
        }
    }

    /// Returns the maximum supported duration in seconds.
    pub fn max_duration_sec(&self) -> u32 {
        match self {
            Backend::MusicGen => 120,
            Backend::AceStep => 240,
        }
    }

    /// Returns the minimum supported duration in seconds.
    pub fn min_duration_sec(&self) -> u32 {
        match self {
            Backend::MusicGen => 5,
            Backend::AceStep => 5,
        }
    }

    /// Returns the output sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        match self {
            Backend::MusicGen => 32000,
            Backend::AceStep => 48000,
        }
    }

    /// Returns whether this backend is installed and ready.
    ///
    /// This is determined by checking if the required model files exist.
    pub fn is_installed(&self, loaded: &LoadedModels) -> bool {
        match self {
            Backend::MusicGen => matches!(loaded, LoadedModels::MusicGen(_)),
            Backend::AceStep => matches!(loaded, LoadedModels::AceStep(_)),
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Loaded models for a specific backend.
///
/// Only one backend's models are loaded at a time to conserve memory.
/// The daemon can switch between backends by unloading one and loading another.
#[derive(Debug)]
pub enum LoadedModels {
    /// No models loaded.
    None,

    /// MusicGen models loaded and ready.
    MusicGen(MusicGenModels),

    /// ACE-Step models loaded and ready.
    /// Placeholder for future implementation.
    AceStep(AceStepModels),
}

impl Default for LoadedModels {
    fn default() -> Self {
        LoadedModels::None
    }
}

impl LoadedModels {
    /// Returns the backend type of the loaded models.
    pub fn backend(&self) -> Option<Backend> {
        match self {
            LoadedModels::None => None,
            LoadedModels::MusicGen(_) => Some(Backend::MusicGen),
            LoadedModels::AceStep(_) => Some(Backend::AceStep),
        }
    }

    /// Returns true if no models are loaded.
    pub fn is_none(&self) -> bool {
        matches!(self, LoadedModels::None)
    }

    /// Returns a reference to the MusicGen models if loaded.
    pub fn as_musicgen(&self) -> Option<&MusicGenModels> {
        match self {
            LoadedModels::MusicGen(models) => Some(models),
            _ => None,
        }
    }

    /// Returns a reference to the ACE-Step models if loaded.
    pub fn as_ace_step(&self) -> Option<&AceStepModels> {
        match self {
            LoadedModels::AceStep(models) => Some(models),
            _ => None,
        }
    }

    /// Returns the model version string.
    pub fn version(&self) -> Option<&str> {
        match self {
            LoadedModels::None => None,
            LoadedModels::MusicGen(models) => Some(models.version()),
            LoadedModels::AceStep(models) => Some(models.version()),
        }
    }

    /// Returns the device name used for inference.
    pub fn device_name(&self) -> Option<&str> {
        match self {
            LoadedModels::None => None,
            LoadedModels::MusicGen(models) => Some(models.device_name()),
            LoadedModels::AceStep(models) => Some(models.device_name()),
        }
    }

    /// Generates audio using the appropriate backend.
    ///
    /// Dispatches to either MusicGen or ACE-Step generation based on which
    /// backend is currently loaded.
    ///
    /// # Arguments
    ///
    /// * `params` - Generation parameters including prompt, duration, etc.
    /// * `on_progress` - Progress callback receiving (current, total) values
    ///
    /// # Returns
    ///
    /// Audio samples at the appropriate sample rate for the backend:
    /// - MusicGen: 32kHz
    /// - ACE-Step: 48kHz
    pub fn generate<F>(&mut self, params: &GenerateDispatchParams, on_progress: F) -> Result<Vec<f32>>
    where
        F: Fn(usize, usize),
    {
        use crate::cli::TOKENS_PER_SECOND;
        use crate::generation::{generate_ace_step, generate_with_models};

        match self {
            LoadedModels::None => Err(DaemonError::model_load_failed("No models loaded")),
            LoadedModels::MusicGen(models) => {
                let max_tokens = params.duration_sec as usize * TOKENS_PER_SECOND;
                generate_with_models(models, &params.prompt, max_tokens, on_progress)
            }
            LoadedModels::AceStep(models) => {
                generate_ace_step(
                    models,
                    &params.prompt,
                    params.duration_sec as f32,
                    params.seed,
                    params.inference_steps.unwrap_or(60),
                    &params.scheduler.clone().unwrap_or_else(|| "euler".to_string()),
                    params.guidance_scale.unwrap_or(15.0),
                    on_progress,
                )
            }
        }
    }
}

/// Parameters for dispatching generation to the appropriate backend.
#[derive(Debug, Clone)]
pub struct GenerateDispatchParams {
    /// Text prompt describing the music to generate.
    pub prompt: String,
    /// Duration in seconds.
    pub duration_sec: u32,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Backend to use (if different from loaded backend).
    pub backend: Backend,
    /// ACE-Step: Number of diffusion steps (1-200).
    pub inference_steps: Option<u32>,
    /// ACE-Step: Scheduler type (euler, heun, pingpong).
    pub scheduler: Option<String>,
    /// ACE-Step: Classifier-free guidance scale.
    pub guidance_scale: Option<f32>,
}

impl GenerateDispatchParams {
    /// Creates new generation dispatch parameters.
    pub fn new(prompt: String, duration_sec: u32, seed: u64, backend: Backend) -> Self {
        Self {
            prompt,
            duration_sec,
            seed,
            backend,
            inference_steps: None,
            scheduler: None,
            guidance_scale: None,
        }
    }

    /// Sets ACE-Step specific parameters.
    pub fn with_ace_step_params(
        mut self,
        inference_steps: Option<u32>,
        scheduler: Option<String>,
        guidance_scale: Option<f32>,
    ) -> Self {
        self.inference_steps = inference_steps;
        self.scheduler = scheduler;
        self.guidance_scale = guidance_scale;
        self
    }
}

// AceStepModels is now defined in ace_step::models and re-exported here

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_parsing() {
        assert_eq!(Backend::parse("musicgen"), Some(Backend::MusicGen));
        assert_eq!(Backend::parse("MusicGen"), Some(Backend::MusicGen));
        assert_eq!(Backend::parse("ace_step"), Some(Backend::AceStep));
        assert_eq!(Backend::parse("ace-step"), Some(Backend::AceStep));
        assert_eq!(Backend::parse("acestep"), Some(Backend::AceStep));
        assert_eq!(Backend::parse("invalid"), None);
    }

    #[test]
    fn backend_display() {
        assert_eq!(Backend::MusicGen.to_string(), "musicgen");
        assert_eq!(Backend::AceStep.to_string(), "ace_step");
    }

    #[test]
    fn backend_duration_limits() {
        assert_eq!(Backend::MusicGen.max_duration_sec(), 120);
        assert_eq!(Backend::AceStep.max_duration_sec(), 240);
        assert_eq!(Backend::MusicGen.min_duration_sec(), 5);
        assert_eq!(Backend::AceStep.min_duration_sec(), 5);
    }

    #[test]
    fn backend_sample_rates() {
        assert_eq!(Backend::MusicGen.sample_rate(), 32000);
        assert_eq!(Backend::AceStep.sample_rate(), 48000);
    }

    #[test]
    fn loaded_models_default() {
        let loaded = LoadedModels::default();
        assert!(loaded.is_none());
        assert!(loaded.backend().is_none());
    }

    #[test]
    fn backend_default() {
        assert_eq!(Backend::default(), Backend::MusicGen);
    }
}
