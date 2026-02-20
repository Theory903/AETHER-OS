//! Tenant domain model — multi-tenancy primitives.
//!
//! Every data structure in AETHER-Ω carries a `TenantId`.
//! This module owns the tenant entity and its configuration.

use serde::{Deserialize, Serialize};

use crate::ids::TenantId;

/// Tenant service tier — controls feature access and resource limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantTier {
    /// Free tier — limited agents, tools, and budget.
    #[default]
    Free,
    /// Pro tier — expanded limits.
    Pro,
    /// Enterprise tier — custom limits, dedicated resources.
    Enterprise,
    /// Internal — AETHER-OS system tenant.
    Internal,
}

/// Resource quota per tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Maximum concurrent agents.
    pub max_concurrent_agents: u32,
    /// Maximum requests per minute across all agents.
    pub max_requests_per_minute: u32,
    /// Maximum monthly budget in USD.
    pub max_monthly_budget_usd: f64,
    /// Maximum tool executions per day.
    pub max_tool_executions_per_day: u64,
    /// Maximum workflow definitions stored.
    pub max_workflows: u32,
    /// Maximum session history length (messages).
    pub max_session_history: usize,
}

impl ResourceQuota {
    /// Default quota for the Free tier.
    #[must_use]
    pub fn free() -> Self {
        Self {
            max_concurrent_agents: 3,
            max_requests_per_minute: 60,
            max_monthly_budget_usd: 10.0,
            max_tool_executions_per_day: 500,
            max_workflows: 5,
            max_session_history: 50,
        }
    }

    /// Default quota for the Pro tier.
    #[must_use]
    pub fn pro() -> Self {
        Self {
            max_concurrent_agents: 50,
            max_requests_per_minute: 600,
            max_monthly_budget_usd: 500.0,
            max_tool_executions_per_day: 50_000,
            max_workflows: 100,
            max_session_history: 500,
        }
    }

    /// Default quota for the Enterprise tier.
    #[must_use]
    pub fn enterprise() -> Self {
        Self {
            max_concurrent_agents: 500,
            max_requests_per_minute: 6_000,
            max_monthly_budget_usd: 10_000.0,
            max_tool_executions_per_day: 1_000_000,
            max_workflows: 10_000,
            max_session_history: 5_000,
        }
    }
}

/// RBAC role for human users within a tenant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    /// Read-only access.
    Viewer = 0,
    /// Develop and run agents.
    Developer = 1,
    /// Manage agents and workflows.
    Admin = 2,
    /// Full tenant control.
    Owner = 3,
}

/// Core tenant entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub tier: TenantTier,
    pub quota: ResourceQuota,
    pub active: bool,
}

impl Tenant {
    /// Create a new tenant with default quota for its tier.
    #[must_use]
    pub fn new(id: TenantId, name: impl Into<String>, tier: TenantTier) -> Self {
        let quota = match &tier {
            TenantTier::Free => ResourceQuota::free(),
            TenantTier::Pro => ResourceQuota::pro(),
            TenantTier::Enterprise | TenantTier::Internal => ResourceQuota::enterprise(),
        };
        Self {
            id,
            name: name.into(),
            tier,
            quota,
            active: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tenant_has_low_quota() {
        let t = Tenant::new(TenantId::new(), "acme", TenantTier::Free);
        assert_eq!(t.quota.max_concurrent_agents, 3);
        assert!(t.active);
    }

    #[test]
    fn test_enterprise_tenant_has_high_quota() {
        let t = Tenant::new(TenantId::new(), "bigco", TenantTier::Enterprise);
        assert_eq!(t.quota.max_concurrent_agents, 500);
    }

    #[test]
    fn test_user_role_ordering() {
        assert!(UserRole::Owner > UserRole::Admin);
        assert!(UserRole::Admin > UserRole::Developer);
        assert!(UserRole::Developer > UserRole::Viewer);
    }
}
