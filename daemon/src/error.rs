//! Error types for the lofi-daemon.
//!
//! Defines all error codes and types used throughout the daemon for
//! consistent error handling and reporting.

use std::fmt;

/// Error codes returned by the daemon in error responses.
///
/// These codes are used in JSON-RPC error responses and allow clients
/// to programmatically handle specific error conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// ONNX model files not found at expected path.
    /// Trigger: Model files missing from cache directory.
    ModelNotFound,

    /// Failed to load ONNX model into memory.
    /// Trigger: Corrupt file, wrong format, or OOM during load.
    ModelLoadFailed,

    /// Failed to download model from remote source.
    /// Trigger: Network error, disk full during download.
    ModelDownloadFailed,

    /// Model inference failed during generation.
    /// Trigger: Numerical instability, OOM during generation.
    ModelInferenceFailed,

    /// Generation queue is at maximum capacity.
    /// Trigger: 10 pending requests already queued.
    QueueFull,

    /// Requested duration is outside valid range.
    /// Trigger: Duration outside 5-120 second range.
    InvalidDuration,

    /// Prompt text is invalid.
    /// Trigger: Empty prompt or exceeds 1000 characters.
    InvalidPrompt,
}

impl ErrorCode {
    /// Returns the string representation of the error code.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::ModelNotFound => "MODEL_NOT_FOUND",
            ErrorCode::ModelLoadFailed => "MODEL_LOAD_FAILED",
            ErrorCode::ModelDownloadFailed => "MODEL_DOWNLOAD_FAILED",
            ErrorCode::ModelInferenceFailed => "MODEL_INFERENCE_FAILED",
            ErrorCode::QueueFull => "QUEUE_FULL",
            ErrorCode::InvalidDuration => "INVALID_DURATION",
            ErrorCode::InvalidPrompt => "INVALID_PROMPT",
        }
    }

    /// Returns a human-readable description of the error.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::ModelNotFound => "ONNX model files not found at expected path",
            ErrorCode::ModelLoadFailed => "Failed to load ONNX model into memory",
            ErrorCode::ModelDownloadFailed => "Failed to download model from remote source",
            ErrorCode::ModelInferenceFailed => "Model inference failed during generation",
            ErrorCode::QueueFull => "Generation queue is at maximum capacity (10 jobs)",
            ErrorCode::InvalidDuration => "Duration must be between 5 and 120 seconds",
            ErrorCode::InvalidPrompt => "Prompt must be non-empty and at most 1000 characters",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Main error type for daemon operations.
#[derive(Debug)]
pub struct DaemonError {
    /// The error code identifying the type of error.
    pub code: ErrorCode,
    /// Human-readable error message with context.
    pub message: String,
    /// Optional underlying cause of the error.
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DaemonError {
    /// Creates a new DaemonError with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new DaemonError with an underlying cause.
    pub fn with_source(
        code: ErrorCode,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a MODEL_NOT_FOUND error.
    pub fn model_not_found(path: &str) -> Self {
        Self::new(
            ErrorCode::ModelNotFound,
            format!("Model files not found at: {}", path),
        )
    }

    /// Creates a MODEL_LOAD_FAILED error.
    pub fn model_load_failed(reason: &str) -> Self {
        Self::new(
            ErrorCode::ModelLoadFailed,
            format!("Failed to load model: {}", reason),
        )
    }

    /// Creates a MODEL_DOWNLOAD_FAILED error.
    pub fn model_download_failed(reason: &str) -> Self {
        Self::new(
            ErrorCode::ModelDownloadFailed,
            format!("Failed to download model: {}", reason),
        )
    }

    /// Creates a MODEL_INFERENCE_FAILED error.
    pub fn model_inference_failed(reason: &str) -> Self {
        Self::new(
            ErrorCode::ModelInferenceFailed,
            format!("Inference failed: {}", reason),
        )
    }

    /// Creates a QUEUE_FULL error.
    pub fn queue_full() -> Self {
        Self::new(
            ErrorCode::QueueFull,
            "Generation queue is full (maximum 10 pending jobs)",
        )
    }

    /// Creates an INVALID_DURATION error.
    pub fn invalid_duration(duration: u32) -> Self {
        Self::new(
            ErrorCode::InvalidDuration,
            format!(
                "Invalid duration: {} seconds (must be between 5 and 120)",
                duration
            ),
        )
    }

    /// Creates an INVALID_PROMPT error for empty prompts.
    pub fn empty_prompt() -> Self {
        Self::new(ErrorCode::InvalidPrompt, "Prompt cannot be empty")
    }

    /// Creates an INVALID_PROMPT error for prompts that are too long.
    pub fn prompt_too_long(len: usize) -> Self {
        Self::new(
            ErrorCode::InvalidPrompt,
            format!(
                "Prompt too long: {} characters (maximum 1000)",
                len
            ),
        )
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for DaemonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Result type alias using DaemonError.
pub type Result<T> = std::result::Result<T, DaemonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_as_str() {
        assert_eq!(ErrorCode::ModelNotFound.as_str(), "MODEL_NOT_FOUND");
        assert_eq!(ErrorCode::ModelLoadFailed.as_str(), "MODEL_LOAD_FAILED");
        assert_eq!(ErrorCode::ModelDownloadFailed.as_str(), "MODEL_DOWNLOAD_FAILED");
        assert_eq!(ErrorCode::ModelInferenceFailed.as_str(), "MODEL_INFERENCE_FAILED");
        assert_eq!(ErrorCode::QueueFull.as_str(), "QUEUE_FULL");
        assert_eq!(ErrorCode::InvalidDuration.as_str(), "INVALID_DURATION");
        assert_eq!(ErrorCode::InvalidPrompt.as_str(), "INVALID_PROMPT");
    }

    #[test]
    fn daemon_error_display() {
        let err = DaemonError::invalid_duration(200);
        assert!(err.to_string().contains("INVALID_DURATION"));
        assert!(err.to_string().contains("200"));
    }
}
