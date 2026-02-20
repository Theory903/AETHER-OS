//! Tool domain model — shared types for the Tool System (PRD §10).
//!
//! These types are used by `aether-tool-gateway` (registry/execution),
//! `aether-policy` (access control), and the Python runtime (tool definitions).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ids::{AgentId, LedgerBlockId, TenantId, ToolId};

/// 4-tier tool access model (PRD §10).
///
/// Determines which agents can call a tool and what enforcement gates apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolAccessLevel {
    /// All agents — no restrictions.
    Public = 0,
    /// Tier ≥ 2 agents only — tier check enforced.
    Protected = 1,
    /// Pre-approved agents only — policy + sandbox + ledger required.
    Restricted = 2,
    /// Human-in-the-loop only — queued for human approval before execution.
    Critical = 3,
}

/// Where a tool may execute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionScope {
    /// Runs directly in the agent process (only for trivially safe tools).
    Inline,
    /// Runs in a MicroVM or gVisor sandbox.
    Sandbox,
    /// Queued externally (e.g., human review queue, async job).
    External,
}

/// Retry policy embedded in the tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub backoff_ms: Vec<u64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_ms: vec![100, 500, 2_000],
        }
    }
}

/// Immutable tool definition — registered once, used many times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: ToolId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub access: ToolAccessLevel,
    pub scope: ExecutionScope,
    /// JSON Schema for the input (validates before execution).
    pub input_schema: serde_json::Value,
    /// JSON Schema for the output (validates after execution).
    pub output_schema: serde_json::Value,
    /// Tool produces the same output for the same input.
    pub idempotent: bool,
    /// Tool can be rolled back via a compensation action.
    pub reversible: bool,
    pub timeout_ms: u64,
    pub retry_policy: RetryPolicy,
}

impl ToolDefinition {
    /// Validate that name is snake_case and version is semver-ish.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("tool name cannot be empty".into());
        }
        if self.name.contains(' ') {
            return Err(format!("tool name must be snake_case, got: {}", self.name));
        }
        if self.timeout_ms == 0 {
            return Err("timeout_ms must be > 0".into());
        }
        Ok(())
    }
}

/// A single tool call from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// LLM-generated call ID (for correlation in streaming).
    pub call_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub content: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub ledger_block_id: Option<LedgerBlockId>,
}

impl ToolResult {
    #[must_use]
    pub fn success(call_id: impl Into<String>, tool_name: impl Into<String>,
                   content: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            content,
            success: true,
            error: None,
            duration_ms,
            ledger_block_id: None,
        }
    }

    #[must_use]
    pub fn failure(call_id: impl Into<String>, tool_name: impl Into<String>,
                   error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            content: serde_json::Value::Null,
            success: false,
            error: Some(error.into()),
            duration_ms,
            ledger_block_id: None,
        }
    }
}

/// Context passed to every tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub agent_tier: u8,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(access: ToolAccessLevel) -> ToolDefinition {
        ToolDefinition {
            id: ToolId::new(),
            name: "web_search".into(),
            version: "1.0.0".into(),
            description: "Search the web".into(),
            access,
            scope: ExecutionScope::Sandbox,
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "object"}),
            idempotent: true,
            reversible: false,
            timeout_ms: 30_000,
            retry_policy: RetryPolicy::default(),
        }
    }

    #[test]
    fn test_tool_validation_passes_valid() {
        let tool = make_tool(ToolAccessLevel::Public);
        assert!(tool.validate().is_ok());
    }

    #[test]
    fn test_tool_validation_fails_space_in_name() {
        let mut tool = make_tool(ToolAccessLevel::Public);
        tool.name = "web search".into();
        assert!(tool.validate().is_err());
    }

    #[test]
    fn test_access_level_ordering() {
        assert!(ToolAccessLevel::Critical > ToolAccessLevel::Restricted);
        assert!(ToolAccessLevel::Restricted > ToolAccessLevel::Protected);
        assert!(ToolAccessLevel::Protected > ToolAccessLevel::Public);
    }

    #[test]
    fn test_result_success_has_no_error() {
        let r = ToolResult::success("id", "web_search", serde_json::json!({}), 100);
        assert!(r.success);
        assert!(r.error.is_none());
    }

    #[test]
    fn test_result_failure_has_error() {
        let r = ToolResult::failure("id", "nmap", "permission denied", 10);
        assert!(!r.success);
        assert!(r.error.is_some());
    }
}
