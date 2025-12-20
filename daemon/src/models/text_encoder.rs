//! Text encoder wrapper for MusicGen.
//!
//! Handles tokenization and T5 text encoding for text prompts.

use std::path::Path;

use ort::session::Session;
use ort::value::{DynValue, Tensor};
use tokenizers::Tokenizer;

use crate::error::{DaemonError, Result};

/// MusicGen text encoder combining tokenizer and T5 encoder.
pub struct MusicGenTextEncoder {
    tokenizer: Tokenizer,
    text_encoder: Session,
}

impl MusicGenTextEncoder {
    /// Creates a new text encoder from model directory.
    ///
    /// Loads `tokenizer.json` and `text_encoder.onnx` from the given directory.
    pub fn load(model_dir: &Path) -> Result<Self> {
        let tokenizer_path = model_dir.join("tokenizer.json");
        let encoder_path = model_dir.join("text_encoder.onnx");

        let mut tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| {
            DaemonError::model_load_failed(format!("Failed to load tokenizer: {}", e))
        })?;

        tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(|e| {
                DaemonError::model_load_failed(format!("Failed to configure tokenizer: {}", e))
            })?;

        let text_encoder = Session::builder()
            .map_err(|e| DaemonError::model_load_failed(format!("Failed to create session: {}", e)))?
            .commit_from_file(&encoder_path)
            .map_err(|e| {
                DaemonError::model_load_failed(format!(
                    "Failed to load text_encoder.onnx: {}",
                    e
                ))
            })?;

        Ok(Self {
            tokenizer,
            text_encoder,
        })
    }

    /// Encodes text into embeddings and attention mask.
    ///
    /// Returns a tuple of (last_hidden_state, attention_mask) as DynValue tensors.
    pub fn encode(&mut self, text: &str) -> Result<(DynValue, DynValue)> {
        let tokens = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| {
                DaemonError::model_inference_failed(format!("Tokenization failed: {}", e))
            })?
            .get_ids()
            .iter()
            .map(|e| *e as i64)
            .collect::<Vec<_>>();

        let tokens_len = tokens.len();

        // Create input tensors
        let input_ids = Tensor::from_array(([1, tokens_len], tokens)).map_err(|e| {
            DaemonError::model_inference_failed(format!("Failed to create input tensor: {}", e))
        })?;

        let attention_mask_data: Vec<i64> = vec![1; tokens_len];
        let attention_mask = Tensor::from_array(([1, tokens_len], attention_mask_data)).map_err(|e| {
            DaemonError::model_inference_failed(format!("Failed to create attention mask: {}", e))
        })?;

        // Run the text encoder
        let mut output = self
            .text_encoder
            .run(ort::inputs![input_ids, attention_mask])
            .map_err(|e| {
                DaemonError::model_inference_failed(format!("Text encoder inference failed: {}", e))
            })?;

        let last_hidden_state = output
            .remove("last_hidden_state")
            .ok_or_else(|| {
                DaemonError::model_inference_failed(
                    "last_hidden_state not found in output",
                )
            })?;

        // Create attention mask for decoder
        let decoder_attention_mask_data: Vec<i64> = vec![1; tokens_len];
        let decoder_attention_mask = Tensor::from_array(([1, tokens_len], decoder_attention_mask_data))
            .map_err(|e| {
                DaemonError::model_inference_failed(format!("Failed to create decoder attention mask: {}", e))
            })?;

        Ok((last_hidden_state, decoder_attention_mask.into_dyn()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder_test() {
        // Model loading tests require actual model files
        assert!(true);
    }
}
