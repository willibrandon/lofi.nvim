//! Cache module for track storage.
//!
//! Provides LRU-based caching for generated tracks.

pub mod tracks;

// Re-export commonly used types
pub use tracks::TrackCache;
