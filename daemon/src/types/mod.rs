//! Core types for the lofi-daemon.
//!
//! This module re-exports all the core data types used throughout the daemon:
//! - [`Track`]: A successfully generated audio file stored in the cache
//! - [`GenerationJob`]: A request for music generation with status tracking
//! - [`ModelConfig`]: Configuration parameters for the MusicGen model

mod config;
mod job;
mod track;

// Re-export all types at the module level
pub use config::ModelConfig;
pub use job::{GenerationJob, JobPriority, JobStatus};
pub use track::{compute_track_id, Track};
