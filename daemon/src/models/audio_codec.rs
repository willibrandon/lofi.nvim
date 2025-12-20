//! Audio codec wrapper for MusicGen.
//!
//! Decodes token sequences into audio samples using EnCodec.

use std::collections::VecDeque;
use std::path::Path;

use half::f16;
use ort::session::Session;
use ort::value::{DynValue, Tensor};

use crate::error::{DaemonError, Result};

/// MusicGen audio codec (EnCodec decoder).
pub struct MusicGenAudioCodec {
    audio_codec: Session,
}

impl MusicGenAudioCodec {
    /// Loads the audio codec from a directory.
    ///
    /// Expects `encodec_decode.onnx` in the directory.
    pub fn load(model_dir: &Path) -> Result<Self> {
        let codec_path = model_dir.join("encodec_decode.onnx");

        let audio_codec = Session::builder()
            .map_err(|e| DaemonError::model_load_failed(format!("Failed to create session: {}", e)))?
            .commit_from_file(&codec_path)
            .map_err(|e| {
                DaemonError::model_load_failed(format!(
                    "Failed to load encodec_decode.onnx: {}",
                    e
                ))
            })?;

        Ok(Self { audio_codec })
    }

    /// Decodes tokens into audio samples.
    ///
    /// Takes an iterator of `[i64; 4]` token arrays (one per timestep, 4 codebooks)
    /// and returns a deque of f32 audio samples.
    pub fn decode(&mut self, tokens: impl IntoIterator<Item = [i64; 4]>) -> Result<VecDeque<f32>> {
        let mut data = vec![];
        for ids in tokens {
            for id in ids {
                data.push(id);
            }
        }

        if data.is_empty() {
            return Ok(VecDeque::new());
        }

        let seq_len = data.len() / 4;

        // Reshape to [1, 1, 4, seq_len] for EnCodec
        // First reshape to [seq_len, 4], then transpose to [4, seq_len]
        let mut transposed = vec![0i64; data.len()];
        for i in 0..seq_len {
            for j in 0..4 {
                transposed[j * seq_len + i] = data[i * 4 + j];
            }
        }

        // Create tensor with shape [1, 1, 4, seq_len]
        let input_tensor = Tensor::from_array(([1usize, 1, 4, seq_len], transposed)).map_err(|e| {
            DaemonError::model_inference_failed(format!("Failed to create token tensor: {}", e))
        })?;

        let mut outputs = self
            .audio_codec
            .run(ort::inputs![input_tensor])
            .map_err(|e| {
                DaemonError::model_inference_failed(format!("Audio codec inference failed: {}", e))
            })?;

        let audio_values: DynValue = outputs.remove("audio_values").ok_or_else(|| {
            DaemonError::model_inference_failed("audio_values not found in output")
        })?;

        // Try f32 first, then f16
        if let Ok((_shape, data)) = audio_values.try_extract_tensor::<f32>() {
            return Ok(data.iter().copied().collect());
        }
        if let Ok((_shape, data)) = audio_values.try_extract_tensor::<f16>() {
            return Ok(data.iter().map(|e| f32::from(*e)).collect());
        }

        Err(DaemonError::model_inference_failed(
            "Audio values must be either f16 or f32",
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_tokens_returns_empty_audio() {
        let tokens: Vec<[i64; 4]> = vec![];
        let mut data = vec![];
        for ids in tokens {
            for id in ids {
                data.push(id);
            }
        }
        assert!(data.is_empty());
    }

    #[test]
    fn token_transpose() {
        let tokens = vec![[1i64, 2, 3, 4], [5, 6, 7, 8]];
        let mut data = vec![];
        for ids in tokens {
            for id in ids {
                data.push(id);
            }
        }

        let seq_len = data.len() / 4;
        let mut transposed = vec![0i64; data.len()];
        for i in 0..seq_len {
            for j in 0..4 {
                transposed[j * seq_len + i] = data[i * 4 + j];
            }
        }

        // After transpose: [1, 5, 2, 6, 3, 7, 4, 8]
        assert_eq!(transposed, vec![1, 5, 2, 6, 3, 7, 4, 8]);
    }
}
