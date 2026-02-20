//! Memory domain model — Mem0-backed tiered memory (PRD §15).
//!
//! This module defines the traits and types that the Mem0 Python client
//! implements. The Rust side uses these types for cross-service communication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{AgentId, SessionId, TenantId};

/// Memory tier determines scope, TTL, and backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryTier {
    /// Redis-backed. Session-scoped. TTL = session lifetime.
    Working,
    /// Mem0-backed (agent_id scope). Permanent. Per-agent knowledge.
    Project,
    /// Mem0-backed (tenant user_id scope). Permanent. Tenant-wide patterns.
    Knowledge,
    /// Neo4j graph store. Permanent. Entity relationships.
    Graph,
}

/// Mem0 memory scope — maps to Mem0 SDK parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScope {
    /// `user_id` in Mem0 — always `tenant:{tenant_id}`.
    pub user_id: String,
    /// `agent_id` in Mem0 — optional, scopes to specific agent.
    pub agent_id: Option<String>,
    /// `run_id` in Mem0 — optional, scopes to specific session.
    pub run_id: Option<String>,
}

impl MemoryScope {
    /// Build a tenant-scoped memory scope (broadest — all agents in tenant).
    #[must_use]
    pub fn for_tenant(tenant_id: &TenantId) -> Self {
        Self {
            user_id: format!("tenant:{tenant_id}"),
            agent_id: None,
            run_id: None,
        }
    }

    /// Build an agent-scoped memory scope.
    #[must_use]
    pub fn for_agent(tenant_id: &TenantId, agent_id: &AgentId) -> Self {
        Self {
            user_id: format!("tenant:{tenant_id}"),
            agent_id: Some(agent_id.to_string()),
            run_id: None,
        }
    }

    /// Build a session-scoped memory scope (narrowest — single conversation).
    #[must_use]
    pub fn for_session(tenant_id: &TenantId, agent_id: &AgentId, session_id: &SessionId) -> Self {
        Self {
            user_id: format!("tenant:{tenant_id}"),
            agent_id: Some(agent_id.to_string()),
            run_id: Some(session_id.to_string()),
        }
    }
}

/// A single memory entry returned from Mem0 search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub memory: String,
    pub score: Option<f32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request to add memories to Mem0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAddRequest {
    pub messages: Vec<MemoryMessage>,
    pub scope: MemoryScope,
}

/// A single message in a memory add request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMessage {
    pub role: MessageRole,
    pub content: String,
}

/// LLM message roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Request to search memories in Mem0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub scope: MemoryScope,
    pub limit: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_scope_user_id_format() {
        let tenant = TenantId::new();
        let scope = MemoryScope::for_tenant(&tenant);
        assert!(scope.user_id.starts_with("tenant:"));
        assert!(scope.agent_id.is_none());
        assert!(scope.run_id.is_none());
    }

    #[test]
    fn test_session_scope_has_all_fields() {
        let t = TenantId::new();
        let a = AgentId::new();
        let s = SessionId::new();
        let scope = MemoryScope::for_session(&t, &a, &s);
        assert!(scope.agent_id.is_some());
        assert!(scope.run_id.is_some());
    }
}
