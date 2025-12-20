//! Audio output module.
//!
//! Provides WAV file writing for generated audio.

pub mod wav;

// Re-export commonly used items
pub use wav::{samples_to_duration, write_wav, write_wav_to_buffer, CHANNELS, SAMPLE_RATE};
