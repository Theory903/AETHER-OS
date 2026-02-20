//! `aether-core` — Shared kernel for AETHER-Ω.
//!
//! This crate owns all domain types used across Rust crates.
//! No business logic lives here — only pure types, traits, and errors.
//!
//! # Module Map
//! - [`ids`] — Typed UUID newtypes (TenantId, AgentId, …)
//! - [`error`] — AetherError enum + ErrorEnvelope
//! - [`tenant`] — Tenant entity, ResourceQuota, UserRole
//! - [`tool`] — ToolDefinition, ToolCall, ToolResult, ToolAccessLevel
//! - [`memory`] — MemoryScope, MemoryEntry, MemoryMessage
//! - [`ledger`] — LedgerBlock, LedgerAction, BlockHash

pub mod error;
pub mod ids;
pub mod ledger;
pub mod memory;
pub mod tenant;
pub mod tool;

// Re-export most commonly used items at crate root.
pub use error::{AetherError, ErrorCode, ErrorEnvelope, Result};
pub use ids::{AgentId, LedgerBlockId, RequestId, SessionId, TaskId, TenantId, ToolId, WorkflowId, WorkerId};
pub use tenant::{ResourceQuota, Tenant, TenantTier, UserRole};
pub use tool::{
    ExecutionScope, RetryPolicy, ToolAccessLevel, ToolCall, ToolDefinition, ToolExecutionContext,
    ToolResult,
};
