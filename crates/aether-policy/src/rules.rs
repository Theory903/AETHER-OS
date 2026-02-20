//! Policy rule definitions — RBAC, ABAC, temporal, budget-gated (PRD §11).
//!
//! Rules are evaluated by the `PolicyEngine` to produce ALLOW/DENY decisions.
//!
//! Reference: PicoClaw `pkg/auth/` (PKCE, token checks) → Adapted for multi-tenant
//! RBAC/ABAC with budget gating.

use serde::{Deserialize, Serialize};

use aether_core::tenant::UserRole;
use aether_core::tool::ToolAccessLevel;

/// An agent's operational tier (maps to PRD §22 T1/T2/T3/T4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgentTier(pub u8);

impl AgentTier {
    pub const BOSS: Self = Self(1);
    pub const SPECIALIST: Self = Self(2);
    pub const WORKER: Self = Self(3);
    pub const SENSOR: Self = Self(4);

    /// Returns true when this tier meets the minimum required for access.
    ///
    /// Lower number = higher privilege (Boss = 1, Sensor = 4).
    pub fn meets(&self, required: AgentTier) -> bool {
        self.0 <= required.0
    }
}

/// Which action is being evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    ToolExecute,
    AgentSpawn,
    MemoryWrite,
    MemoryDelete,
    WorkflowRun,
    WorkflowCreate,
    WorkflowDelete,
    CronCreate,
    HumanReviewApprove,
    SystemConfig,
}

/// The subject making the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySubject {
    pub agent_tier: AgentTier,
    pub user_role: Option<UserRole>,
    /// Budget remaining as a fraction [0.0, 1.0].
    pub budget_remaining_fraction: f64,
    /// Whether this agent has been explicitly approved for RESTRICTED tools.
    pub restricted_approved: bool,
}

/// A single policy rule.
///
/// Rules are evaluated in order; first match wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: &'static str,
    pub description: &'static str,
    pub condition: RuleCondition,
    pub effect: PolicyEffect,
}

/// The logical condition that must be true for a rule to fire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    /// Requires the subject's agent tier to meet a minimum.
    AgentTierMinimum { minimum: u8 },
    /// Requires a specific tool access level to match the subject's clearance.
    ToolAccessLevel { required: ToolAccessLevel },
    /// Requires the budget remaining fraction to be above a threshold.
    BudgetAbove { threshold: f64 },
    /// User must hold at least this role.
    UserRoleMinimum { minimum: UserRole },
    /// Agent must be explicitly approved for restricted operations.
    RestrictedApproved,
    /// Always-true sentinel for default rules.
    AlwaysAllow,
    /// Always-false sentinel for deny-by-default.
    AlwaysDeny,
}

/// Whether the rule permits or denies the action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny { reason: &'static str },
}

/// Built-in ruleset.
pub fn default_rules() -> Vec<PolicyRule> {
    vec![
        PolicyRule {
            id: "budget-exhausted-deny",
            description: "Deny all tool calls when budget is fully exhausted",
            condition: RuleCondition::BudgetAbove { threshold: 0.0 },
            effect: PolicyEffect::Deny {
                reason: "budget exhausted",
            },
        },
        PolicyRule {
            id: "critical-tool-deny-agent",
            description: "CRITICAL tools require human approval — agents cannot self-approve",
            condition: RuleCondition::ToolAccessLevel {
                required: ToolAccessLevel::Critical,
            },
            effect: PolicyEffect::Deny {
                reason: "CRITICAL tools require human-in-the-loop approval",
            },
        },
        PolicyRule {
            id: "restricted-tool-requires-approval",
            description: "RESTRICTED tools require explicit pre-approval flag",
            condition: RuleCondition::ToolAccessLevel {
                required: ToolAccessLevel::Restricted,
            },
            effect: PolicyEffect::Deny {
                reason: "RESTRICTED tool requires policy approval",
            },
        },
        PolicyRule {
            id: "protected-tool-tier-2-minimum",
            description: "PROTECTED tools require Tier ≤ 2 agents",
            condition: RuleCondition::AgentTierMinimum { minimum: 2 },
            effect: PolicyEffect::Allow,
        },
        PolicyRule {
            id: "public-tool-allow-all",
            description: "PUBLIC tools are available to all agents",
            condition: RuleCondition::AlwaysAllow,
            effect: PolicyEffect::Allow,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_tier_ordering() {
        assert!(AgentTier::BOSS.meets(AgentTier::SPECIALIST));
        assert!(!AgentTier::WORKER.meets(AgentTier::SPECIALIST));
        assert!(!AgentTier::SENSOR.meets(AgentTier::BOSS));
    }

    #[test]
    fn test_default_rules_non_empty() {
        let rules = default_rules();
        assert!(!rules.is_empty());
    }
}
