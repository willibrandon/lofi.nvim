//! lofi-daemon: AI music generation daemon using MusicGen ONNX backend.
//!
//! This library provides the core functionality for the lofi-daemon,
//! which generates ambient music using the MusicGen model via ONNX Runtime.
//!
//! # Modules
//!
//! - [`types`]: Core data types (Track, GenerationJob, ModelConfig)
//! - [`config`]: Runtime configuration (DaemonConfig, Device)
//! - [`error`]: Error types and codes (DaemonError, ErrorCode)
//!
//! # Example
//!
//! ```rust,ignore
//! use lofi_daemon::{
//!     config::{DaemonConfig, Device},
//!     types::{GenerationJob, JobPriority, ModelConfig},
//!     error::{DaemonError, ErrorCode},
//! };
//!
//! // Create configuration
//! let config = DaemonConfig {
//!     device: Device::Auto,
//!     ..Default::default()
//! };
//!
//! // Create a generation job
//! let job = GenerationJob::new(
//!     "lofi hip hop beats to relax to".to_string(),
//!     30, // 30 seconds
//!     Some(42), // seed for reproducibility
//!     JobPriority::Normal,
//!     "musicgen-small-fp16-v1",
//! );
//! ```

pub mod config;
pub mod error;
pub mod types;

// Re-export commonly used types at crate root for convenience
pub use config::{DaemonConfig, Device};
pub use error::{DaemonError, ErrorCode, Result};
pub use types::{compute_track_id, GenerationJob, JobPriority, JobStatus, ModelConfig, Track};
