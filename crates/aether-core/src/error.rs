//! Structured error hierarchy for AETHER-Ω.
//!
//! Follows STRATOS error format: `{ code, message, context, timestamp, request_id }`.
//! Uses `thiserror` for library errors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::RequestId;

/// AETHER-Ω error codes — stable string identifiers for API consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Auth & Tenant
    Unauthorized,
    Forbidden,
    TenantNotFound,
    TenantQuotaExceeded,
    // Resources
    NotFound,
    AlreadyExists,
    Conflict,
    // Validation
    ValidationFailed,
    InvalidSchema,
    // Execution
    ToolExecutionFailed,
    ToolTimeout,
    ToolAccessDenied,
    SandboxError,
    AgentSpawnFailed,
    MaxDepthExceeded,
    // Budget
    BudgetExceeded,
    BudgetLimitReached,
    // Infrastructure
    StorageError,
    LedgerIntegrityViolation,
    SerializationError,
    // Internal
    Internal,
    NotImplemented,
}

/// Machine-readable error envelope.
///
/// All errors crossing service boundaries are wrapped in this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<RequestId>,
}

impl ErrorEnvelope {
    /// Build an envelope from an `AetherError`.
    pub fn from_error(err: &AetherError, request_id: Option<RequestId>) -> Self {
        Self {
            code: err.code(),
            message: err.to_string(),
            context: err.context(),
            timestamp: Utc::now(),
            request_id,
        }
    }
}

/// Primary error type for all AETHER-Ω Rust crates.
#[derive(Error, Debug)]
pub enum AetherError {
    // Auth
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    // Tenant
    #[error("tenant not found: {id}")]
    TenantNotFound { id: String },

    #[error("tenant quota exceeded: {resource} limit {limit}")]
    TenantQuotaExceeded { resource: String, limit: u64 },

    // Resources
    #[error("not found: {resource} with id {id}")]
    NotFound { resource: &'static str, id: String },

    #[error("already exists: {resource} with id {id}")]
    AlreadyExists { resource: &'static str, id: String },

    // Validation
    #[error("validation failed: {field} — {reason}")]
    ValidationFailed { field: String, reason: String },

    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    // Tool Execution
    #[error("tool execution failed: {tool} — {reason}")]
    ToolExecutionFailed { tool: String, reason: String },

    #[error("tool timeout: {tool} exceeded {timeout_ms}ms")]
    ToolTimeout { tool: String, timeout_ms: u64 },

    #[error("tool access denied: {tool} requires {required_level:?}")]
    ToolAccessDenied {
        tool: String,
        required_level: crate::tool::ToolAccessLevel,
    },

    #[error("sandbox error: {0}")]
    SandboxError(String),

    // Agent
    #[error("agent spawn failed: {reason}")]
    AgentSpawnFailed { reason: String },

    #[error("max subagent depth {max} exceeded")]
    MaxDepthExceeded { max: u8 },

    // Budget
    #[error("budget exceeded for tenant {tenant}: spent {spent_usd:.4} of {limit_usd:.4} USD")]
    BudgetExceeded {
        tenant: String,
        spent_usd: f64,
        limit_usd: f64,
    },

    // Ledger
    #[error("ledger integrity violation: block {block_id} — {reason}")]
    LedgerIntegrityViolation { block_id: String, reason: String },

    // Infrastructure
    #[error("storage error: {0}")]
    StorageError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    // Catch-all
    #[error("internal error: {0}")]
    Internal(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl AetherError {
    /// Return the stable error code.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Unauthorized(_) => ErrorCode::Unauthorized,
            Self::Forbidden(_) => ErrorCode::Forbidden,
            Self::TenantNotFound { .. } => ErrorCode::TenantNotFound,
            Self::TenantQuotaExceeded { .. } => ErrorCode::TenantQuotaExceeded,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::AlreadyExists { .. } => ErrorCode::AlreadyExists,
            Self::ValidationFailed { .. } => ErrorCode::ValidationFailed,
            Self::InvalidSchema(_) => ErrorCode::InvalidSchema,
            Self::ToolExecutionFailed { .. } => ErrorCode::ToolExecutionFailed,
            Self::ToolTimeout { .. } => ErrorCode::ToolTimeout,
            Self::ToolAccessDenied { .. } => ErrorCode::ToolAccessDenied,
            Self::SandboxError(_) => ErrorCode::SandboxError,
            Self::AgentSpawnFailed { .. } => ErrorCode::AgentSpawnFailed,
            Self::MaxDepthExceeded { .. } => ErrorCode::MaxDepthExceeded,
            Self::BudgetExceeded { .. } => ErrorCode::BudgetExceeded,
            Self::LedgerIntegrityViolation { .. } => ErrorCode::LedgerIntegrityViolation,
            Self::StorageError(_) => ErrorCode::StorageError,
            Self::SerializationError(_) => ErrorCode::SerializationError,
            Self::Internal(_) => ErrorCode::Internal,
            Self::NotImplemented(_) => ErrorCode::NotImplemented,
        }
    }

    /// Optional structured context for API responses.
    pub fn context(&self) -> Option<serde_json::Value> {
        match self {
            Self::BudgetExceeded {
                spent_usd,
                limit_usd,
                ..
            } => Some(serde_json::json!({
                "spent_usd": spent_usd,
                "limit_usd": limit_usd,
            })),
            Self::ToolTimeout { timeout_ms, .. } => Some(serde_json::json!({
                "timeout_ms": timeout_ms,
            })),
            Self::MaxDepthExceeded { max } => Some(serde_json::json!({ "max_depth": max })),
            _ => None,
        }
    }

    /// Helpers for common cases.
    pub fn not_found(resource: &'static str, id: impl ToString) -> Self {
        Self::NotFound {
            resource,
            id: id.to_string(),
        }
    }

    pub fn internal(msg: impl ToString) -> Self {
        Self::Internal(msg.to_string())
    }
}

/// Alias for `Result<T, AetherError>`.
pub type Result<T> = std::result::Result<T, AetherError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_matches_variant() {
        let err = AetherError::Unauthorized("bad token".into());
        assert_eq!(err.code(), ErrorCode::Unauthorized);
    }

    #[test]
    fn test_not_found_helper() {
        let err = AetherError::not_found("Agent", "abc-123");
        assert_eq!(err.code(), ErrorCode::NotFound);
        assert!(err.to_string().contains("abc-123"));
    }

    #[test]
    fn test_budget_exceeded_has_context() {
        let err = AetherError::BudgetExceeded {
            tenant: "acme".into(),
            spent_usd: 10.5,
            limit_usd: 10.0,
        };
        let ctx = err.context().unwrap();
        assert_eq!(ctx["spent_usd"], 10.5);
    }

    #[test]
    fn test_envelope_serialization() {
        let err = AetherError::internal("disk full");
        let envelope = ErrorEnvelope::from_error(&err, None);
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("INTERNAL"));
    }
}
