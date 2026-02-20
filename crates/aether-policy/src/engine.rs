//! Policy evaluation engine — core decision loop (PRD §11).
//!
//! The engine evaluates a list of rules against an `EvaluationContext`
//! and returns a `PolicyDecision`. First-matching rule wins.
//!
//! Rule order matters: most-restrictive rules first (budget, critical, restricted)
//! then permissive rules (protected, public).

use aether_core::error::{AetherError, Result};
use aether_core::tool::ToolAccessLevel;

use crate::evaluation::{DecisionEffect, EvaluationContext, PolicyDecision, PolicyResource};
use crate::rules::{AgentTier, PolicyEffect, PolicyRule, RuleCondition, default_rules};

/// Central policy evaluation engine.
///
/// # Single Responsibility
/// Only evaluates rules. Does not store state. Does not call external services.
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl PolicyEngine {
    /// Create engine with custom rules.
    pub fn with_rules(rules: Vec<PolicyRule>) -> Self {
        Self { rules }
    }

    /// Evaluate a policy context against all rules.
    ///
    /// Iterates rules in order; returns the first matching decision.
    /// If no rule matches, defaults to DENY (fail-safe).
    ///
    /// # Errors
    /// Returns `AetherError::Forbidden` if the decision is DENY.
    /// Returns `AetherError::ToolAccessDenied` for tool-specific denials.
    pub fn evaluate(&self, ctx: &EvaluationContext) -> Result<PolicyDecision> {
        let decision = self.decide(ctx);

        if !decision.is_allowed() {
            return Err(self.denial_error(ctx, &decision));
        }
        Ok(decision)
    }

    /// Return the decision without converting to an error.
    /// Use this when you need the decision for audit/logging purposes.
    pub fn decide(&self, ctx: &EvaluationContext) -> PolicyDecision {
        for rule in &self.rules {
            if let Some(decision) = self.evaluate_rule(rule, ctx) {
                return decision;
            }
        }
        // Fail-safe: deny if no rule matched
        PolicyDecision::deny("default-deny", "no matching rule — default deny")
    }

    fn evaluate_rule(
        &self,
        rule: &PolicyRule,
        ctx: &EvaluationContext,
    ) -> Option<PolicyDecision> {
        let matched = match &rule.condition {
            RuleCondition::BudgetAbove { threshold } => {
                ctx.subject.budget_remaining_fraction <= *threshold
            }
            RuleCondition::ToolAccessLevel { required } => {
                let tool_access = self.extract_tool_access(ctx);
                tool_access.map(|a| a >= *required).unwrap_or(false)
            }
            RuleCondition::AgentTierMinimum { minimum } => {
                ctx.subject.agent_tier.0 <= *minimum
            }
            RuleCondition::RestrictedApproved => ctx.subject.restricted_approved,
            RuleCondition::UserRoleMinimum { minimum } => ctx
                .subject
                .user_role
                .as_ref()
                .map(|r| r >= minimum)
                .unwrap_or(false),
            RuleCondition::AlwaysAllow => true,
            RuleCondition::AlwaysDeny => true,
        };

        if !matched {
            return None;
        }

        Some(match &rule.effect {
            PolicyEffect::Allow => PolicyDecision::allow(rule.id),
            PolicyEffect::Deny { reason } => PolicyDecision::deny(rule.id, *reason),
        })
    }

    fn extract_tool_access(&self, ctx: &EvaluationContext) -> Option<ToolAccessLevel> {
        match &ctx.resource {
            PolicyResource::Tool { access_level, .. } => Some(*access_level),
            _ => None,
        }
    }

    fn denial_error(&self, ctx: &EvaluationContext, decision: &PolicyDecision) -> AetherError {
        match &ctx.resource {
            PolicyResource::Tool {
                tool_id,
                access_level,
                ..
            } => AetherError::ToolAccessDenied {
                tool: tool_id.to_string(),
                required_level: *access_level,
            },
            _ => AetherError::Forbidden(decision.reason.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ids::{AgentId, TaskId, TenantId, ToolId};
    use aether_core::tool::ToolAccessLevel;
    use crate::evaluation::EvaluationContext;
    use crate::rules::AgentTier;

    fn make_ctx(tool_access: ToolAccessLevel, tier: AgentTier, budget: f64) -> EvaluationContext {
        EvaluationContext::tool_execute(
            TenantId::new(),
            AgentId::new(),
            TaskId::new(),
            tier,
            ToolId::new(),
            tool_access,
            budget,
            false,
        )
    }

    #[test]
    fn test_public_tool_allowed_for_all_tiers() {
        let engine = PolicyEngine::default();
        let ctx = make_ctx(ToolAccessLevel::Public, AgentTier::SENSOR, 1.0);
        let d = engine.decide(&ctx);
        assert!(d.is_allowed(), "public tools should be allowed for all tiers");
    }

    #[test]
    fn test_critical_tool_always_denied() {
        let engine = PolicyEngine::default();
        let ctx = make_ctx(ToolAccessLevel::Critical, AgentTier::BOSS, 1.0);
        let d = engine.decide(&ctx);
        assert!(!d.is_allowed(), "CRITICAL tools must be denied for agents");
    }

    #[test]
    fn test_restricted_tool_denied_without_approval() {
        let engine = PolicyEngine::default();
        let ctx = make_ctx(ToolAccessLevel::Restricted, AgentTier::BOSS, 1.0);
        let d = engine.decide(&ctx);
        assert!(!d.is_allowed(), "RESTRICTED tools denied without approval");
    }

    #[test]
    fn test_budget_exhausted_denies_all() {
        let engine = PolicyEngine::default();
        // 0.0 = fully exhausted: the budget-exhausted-deny rule fires first
        let ctx = make_ctx(ToolAccessLevel::Public, AgentTier::BOSS, 0.0);
        let d = engine.decide(&ctx);
        assert!(!d.is_allowed(), "exhausted budget should deny everything");
    }

    #[test]
    fn test_evaluate_returns_err_on_deny() {
        let engine = PolicyEngine::default();
        let ctx = make_ctx(ToolAccessLevel::Critical, AgentTier::BOSS, 1.0);
        assert!(engine.evaluate(&ctx).is_err());
    }

    #[test]
    fn test_evaluate_returns_ok_on_allow() {
        let engine = PolicyEngine::default();
        let ctx = make_ctx(ToolAccessLevel::Public, AgentTier::WORKER, 1.0);
        assert!(engine.evaluate(&ctx).is_ok());
    }
}
