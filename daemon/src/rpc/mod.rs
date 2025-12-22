//! JSON-RPC module for daemon communication.
//!
//! Provides the JSON-RPC 2.0 server implementation for:
//! - `generate`: Start music generation
//! - `ping`: Health check
//! - `shutdown`: Graceful shutdown
//!
//! Notifications:
//! - `generation_progress`: Progress updates during generation
//! - `generation_complete`: Successful completion
//! - `generation_error`: Generation failure

pub mod methods;
pub mod server;
pub mod types;

// Re-export commonly used types
pub use server::{run_server, send_notification, BackendStatuses, ServerState};
pub use types::{
    BackendInfo, BackendStatus, GenerateParams, GenerateResult, GenerationCompleteParams,
    GenerationErrorParams, GenerationProgressParams, GenerationStatus, GetBackendsResult,
    JsonRpcError, JsonRpcErrorResponse, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    Priority, RequestId,
};
