//! Policy evaluation context and result types (PRD §11).
//!
//! `EvaluationContext` is the full input to the policy engine.
//! `PolicyDecision` is the output.

use serde::{Deserialize, Serialize};

use aether_core::ids::{AgentId, TaskId, TenantId, ToolId};
use aether_core::tool::ToolAccessLevel;

use crate::rules::{AgentTier, PolicyAction, PolicySubject};

/// Full context passed to the policy engine for a single evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    pub tenant_id: TenantId,
    pub subject: PolicySubject,
    pub action: PolicyAction,
    pub resource: PolicyResource,
}

impl EvaluationContext {
    /// Shorthand constructor for tool execution checks.
    pub fn tool_execute(
        tenant_id: TenantId,
        agent_id: AgentId,
        task_id: TaskId,
        agent_tier: AgentTier,
        tool_id: ToolId,
        tool_access: ToolAccessLevel,
        budget_remaining_fraction: f64,
        restricted_approved: bool,
    ) -> Self {
        Self {
            tenant_id,
            subject: PolicySubject {
                agent_tier,
                user_role: None,
                budget_remaining_fraction,
                restricted_approved,
            },
            action: PolicyAction::ToolExecute,
            resource: PolicyResource::Tool {
                tool_id,
                access_level: tool_access,
                agent_id,
                task_id,
            },
        }
    }
}

/// The resource being acted upon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyResource {
    Tool {
        tool_id: ToolId,
        access_level: ToolAccessLevel,
        agent_id: AgentId,
        task_id: TaskId,
    },
    Agent {
        agent_id: AgentId,
        tier: AgentTier,
    },
    Workflow {
        workflow_id: aether_core::ids::WorkflowId,
    },
    Memory {
        scope: String,
    },
}

/// Policy decision returned by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub effect: DecisionEffect,
    pub matched_rule: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionEffect {
    Allow,
    Deny,
}

impl PolicyDecision {
    pub fn allow(rule_id: impl Into<String>) -> Self {
        Self {
            effect: DecisionEffect::Allow,
            matched_rule: rule_id.into(),
            reason: "allowed by policy".into(),
        }
    }

    pub fn deny(rule_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            effect: DecisionEffect::Deny,
            matched_rule: rule_id.into(),
            reason: reason.into(),
        }
    }

    pub fn is_allowed(&self) -> bool {
        self.effect == DecisionEffect::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_decision() {
        let d = PolicyDecision::allow("public-allow");
        assert!(d.is_allowed());
    }

    #[test]
    fn test_deny_decision() {
        let d = PolicyDecision::deny("budget-exhausted", "no budget");
        assert!(!d.is_allowed());
        assert_eq!(d.reason, "no budget");
    }
}
