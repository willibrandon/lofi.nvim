//! ModelConfig type for MusicGen model parameters.
//!
//! Contains the configuration parameters for the MusicGen ONNX model
//! ensemble, matching the model's architecture requirements.

use serde::{Deserialize, Serialize};

/// Configuration parameters for the MusicGen model architecture.
///
/// These values are derived from the model's config.json and are required
/// for proper tensor shape allocation and inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Token vocabulary size (typically 2048 for MusicGen).
    pub vocab_size: u32,

    /// Number of decoder transformer layers.
    pub num_hidden_layers: u32,

    /// Number of attention heads in each layer.
    pub num_attention_heads: u32,

    /// Hidden dimension size (embedding dimension).
    pub d_model: u32,

    /// Key/value dimension per attention head.
    /// Typically d_model / num_attention_heads.
    pub d_kv: u32,

    /// Number of audio channels (always 1 for mono).
    pub audio_channels: u32,

    /// Audio sample rate in Hz (always 32000 for MusicGen).
    pub sample_rate: u32,

    /// Number of EnCodec codebooks (always 4 for MusicGen).
    pub codebooks: u32,

    /// Padding token ID for the decoder.
    pub pad_token_id: i64,
}

impl ModelConfig {
    /// Creates a ModelConfig for the musicgen-small model.
    ///
    /// This is the default configuration matching the fp16 small model
    /// from gabotechs/music_gen on HuggingFace.
    pub fn musicgen_small() -> Self {
        Self {
            vocab_size: 2048,
            num_hidden_layers: 24,
            num_attention_heads: 16,
            d_model: 1024,
            d_kv: 64, // 1024 / 16 = 64
            audio_channels: 1,
            sample_rate: 32000,
            codebooks: 4,
            pad_token_id: 2048, // vocab_size is used as pad token
        }
    }

    /// Validates the configuration for consistency.
    ///
    /// Returns an error message if validation fails, None otherwise.
    pub fn validate(&self) -> Option<String> {
        if self.vocab_size == 0 {
            return Some("vocab_size must be > 0".to_string());
        }

        if self.num_hidden_layers == 0 {
            return Some("num_hidden_layers must be > 0".to_string());
        }

        if self.num_attention_heads == 0 {
            return Some("num_attention_heads must be > 0".to_string());
        }

        if self.d_model == 0 {
            return Some("d_model must be > 0".to_string());
        }

        // d_kv should typically be d_model / num_attention_heads
        let expected_d_kv = self.d_model / self.num_attention_heads;
        if self.d_kv != expected_d_kv {
            return Some(format!(
                "d_kv ({}) should be d_model / num_attention_heads ({})",
                self.d_kv, expected_d_kv
            ));
        }

        if self.sample_rate != 32000 {
            return Some(format!(
                "sample_rate must be 32000, got {}",
                self.sample_rate
            ));
        }

        if self.codebooks != 4 {
            return Some(format!(
                "codebooks must be 4, got {}",
                self.codebooks
            ));
        }

        None
    }

    /// Returns the total size of the KV cache per layer.
    ///
    /// This is used for pre-allocating cache tensors during inference.
    pub fn kv_cache_size_per_layer(&self, sequence_length: usize) -> usize {
        // Each layer has key and value caches
        // Shape: [batch_size, num_heads, seq_len, d_kv]
        // For batch_size=8 (4 conditional + 4 unconditional for CFG)
        let batch_size = 8;
        batch_size * self.num_attention_heads as usize * sequence_length * self.d_kv as usize
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::musicgen_small()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn musicgen_small_config() {
        let config = ModelConfig::musicgen_small();
        assert_eq!(config.vocab_size, 2048);
        assert_eq!(config.num_hidden_layers, 24);
        assert_eq!(config.sample_rate, 32000);
        assert_eq!(config.codebooks, 4);
        assert!(config.validate().is_none());
    }

    #[test]
    fn config_validation() {
        let mut config = ModelConfig::musicgen_small();
        config.d_kv = 128; // Wrong value
        assert!(config.validate().is_some());
    }

    #[test]
    fn kv_cache_size() {
        let config = ModelConfig::musicgen_small();
        // For sequence length 100, batch 8, 16 heads, d_kv 64
        let size = config.kv_cache_size_per_layer(100);
        assert_eq!(size, 8 * 16 * 100 * 64);
    }
}
