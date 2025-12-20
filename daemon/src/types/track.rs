//! Track type representing a generated audio file.
//!
//! A Track represents a successfully generated audio file stored in the cache.
//! Tracks are identified by a deterministic track_id computed from generation parameters.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::SystemTime;

/// A successfully generated audio file stored in the cache.
///
/// Tracks are immutable once created and are uniquely identified by their
/// `track_id`, which is computed from the generation parameters to enable
/// deduplication of identical requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Primary key - SHA256 hash of (prompt + seed + duration + model_version).
    /// Format: 16 hex characters.
    pub track_id: String,

    /// Full filesystem path to the WAV file.
    pub path: PathBuf,

    /// Original text prompt used for generation.
    /// Constraints: 1-1000 characters.
    pub prompt: String,

    /// Actual duration of generated audio in seconds.
    pub duration_sec: f32,

    /// Audio sample rate in Hz. Always 32000 for MusicGen.
    pub sample_rate: u32,

    /// Random seed used for generation.
    pub seed: u64,

    /// Model identifier for reproducibility.
    /// Example: "musicgen-small-fp16-v1"
    pub model_version: String,

    /// Time taken to generate the audio in seconds.
    pub generation_time_sec: f32,

    /// When the track was created (ISO 8601 timestamp).
    #[serde(with = "system_time_serde")]
    pub created_at: SystemTime,
}

impl Track {
    /// Creates a new Track with the given parameters.
    ///
    /// The track_id is automatically computed from the generation parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: PathBuf,
        prompt: String,
        duration_sec: f32,
        seed: u64,
        model_version: String,
        generation_time_sec: f32,
    ) -> Self {
        let track_id = compute_track_id(&prompt, seed, duration_sec, &model_version);
        Self {
            track_id,
            path,
            prompt,
            duration_sec,
            sample_rate: 32000,
            seed,
            model_version,
            generation_time_sec,
            created_at: SystemTime::now(),
        }
    }

    /// Validates that the track meets all constraints.
    ///
    /// Returns an error message if validation fails, None otherwise.
    pub fn validate(&self) -> Option<String> {
        // Track ID must be exactly 16 hex characters
        if self.track_id.len() != 16 {
            return Some(format!(
                "Track ID must be 16 characters, got {}",
                self.track_id.len()
            ));
        }

        if !self.track_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some("Track ID must contain only hex characters".to_string());
        }

        // Path must exist (for cached tracks)
        if !self.path.exists() {
            return Some(format!("Track file does not exist: {:?}", self.path));
        }

        // Duration must be within 5-120 seconds
        if !(5.0..=120.0).contains(&self.duration_sec) {
            return Some(format!(
                "Duration must be between 5 and 120 seconds, got {}",
                self.duration_sec
            ));
        }

        // Prompt must be 1-1000 characters
        if self.prompt.is_empty() {
            return Some("Prompt cannot be empty".to_string());
        }

        if self.prompt.len() > 1000 {
            return Some(format!(
                "Prompt too long: {} characters (max 1000)",
                self.prompt.len()
            ));
        }

        None
    }
}

/// Computes a deterministic track ID from generation parameters.
///
/// The track ID is the first 16 hex characters of the SHA256 hash of:
/// `{prompt}:{seed}:{duration_sec}:{model_version}`
///
/// This enables deduplication: identical generation parameters always
/// produce the same track_id.
pub fn compute_track_id(prompt: &str, seed: u64, duration_sec: f32, model_version: &str) -> String {
    let input = format!("{}:{}:{}:{}", prompt, seed, duration_sec, model_version);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    // Take first 8 bytes (16 hex chars)
    hex::encode(&result[..8])
}

/// Custom serde implementation for SystemTime to use ISO 8601 format.
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        let secs = duration.as_secs();
        // Serialize as Unix timestamp (simpler, widely compatible)
        secs.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

/// Encode bytes as hex string (inline implementation to avoid extra dependency).
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_id_deterministic() {
        let id1 = compute_track_id("lofi beats", 42, 30.0, "musicgen-small-fp16-v1");
        let id2 = compute_track_id("lofi beats", 42, 30.0, "musicgen-small-fp16-v1");
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn track_id_varies_with_params() {
        let id1 = compute_track_id("lofi beats", 42, 30.0, "musicgen-small-fp16-v1");
        let id2 = compute_track_id("lofi beats", 43, 30.0, "musicgen-small-fp16-v1");
        let id3 = compute_track_id("jazz", 42, 30.0, "musicgen-small-fp16-v1");
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn track_id_hex_format() {
        let id = compute_track_id("test", 0, 10.0, "v1");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
