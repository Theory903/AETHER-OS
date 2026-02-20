//! Budget enforcement — check → degrade → kill (PRD §12).

use aether_core::error::{AetherError, Result};
use aether_core::ids::TenantId;

use crate::tracker::CostTracker;

/// Budget thresholds — fractions of the total budget.
pub const ALERT_THRESHOLD: f64 = 0.25;     // 75% spent → 25% remaining
pub const DEGRADE_THRESHOLD: f64 = 0.10;   // 90% spent → 10% remaining
pub const KILL_THRESHOLD: f64 = 0.0;       // 100% spent → 0% remaining

/// Enforcement action determined by the limiter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetAction {
    /// Proceed normally.
    Allow,
    /// Proceed but downgrade model to cheaper option.
    Degrade { reason: String },
    /// Alert sent — proceed normally.
    Alert { message: String },
    /// Budget exhausted — block execution.
    Kill { reason: String },
}

/// Budget limiter — checks remaining budget and returns an enforcement action.
pub struct BudgetLimiter {
    tracker: CostTracker,
}

impl BudgetLimiter {
    pub fn new(tracker: CostTracker) -> Self {
        Self { tracker }
    }

    /// Determine the enforcement action for a tenant before a tool/LLM call.
    ///
    /// # Errors
    /// Returns `BudgetExceeded` when the kill threshold is reached.
    pub fn check(&self, tenant_id: &TenantId, limit_usd: f64) -> Result<BudgetAction> {
        let usage = self
            .tracker
            .get_usage(tenant_id)
            .map_err(|e| AetherError::internal(e))?;

        let spent = usage.total_cost_usd;
        let remaining_fraction = self
            .tracker
            .budget_remaining_fraction(tenant_id, limit_usd);

        if remaining_fraction <= KILL_THRESHOLD {
            return Err(AetherError::BudgetExceeded {
                tenant: tenant_id.to_string(),
                spent_usd: spent,
                limit_usd,
            });
        }

        if remaining_fraction <= DEGRADE_THRESHOLD {
            return Ok(BudgetAction::Degrade {
                reason: format!("{:.0}% budget spent — downgrading to cheaper model", 
                    (1.0 - remaining_fraction) * 100.0),
            });
        }

        if remaining_fraction <= ALERT_THRESHOLD {
            return Ok(BudgetAction::Alert {
                message: format!("{:.0}% budget consumed", 
                    (1.0 - remaining_fraction) * 100.0),
            });
        }

        Ok(BudgetAction::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::LlmCost;
    use aether_core::ids::{AgentId, TaskId, TenantId};

    fn limiter() -> (BudgetLimiter, TenantId) {
        (BudgetLimiter::new(CostTracker::new()), TenantId::new())
    }

    fn record(limiter: &BudgetLimiter, tenant: TenantId, usd: f64) {
        limiter.tracker.record(&LlmCost {
            tenant_id: tenant,
            task_id: TaskId::new(),
            agent_id: AgentId::new(),
            model: "claude".into(),
            input_tokens: 100,
            output_tokens: 50,
            cost_usd: usd,
        }).unwrap();
    }

    #[test]
    fn test_allow_when_budget_available() {
        let (l, t) = limiter();
        assert_eq!(l.check(&t, 10.0).unwrap(), BudgetAction::Allow);
    }

    #[test]
    fn test_alert_at_75_pct_spent() {
        let (l, t) = limiter();
        record(&l, t, 7.6); // 76% spent
        match l.check(&t, 10.0).unwrap() {
            BudgetAction::Alert { .. } => {}
            other => panic!("expected Alert, got {other:?}"),
        }
    }

    #[test]
    fn test_degrade_at_90_pct_spent() {
        let (l, t) = limiter();
        record(&l, t, 9.1); // 91% spent
        match l.check(&t, 10.0).unwrap() {
            BudgetAction::Degrade { .. } => {}
            other => panic!("expected Degrade, got {other:?}"),
        }
    }

    #[test]
    fn test_kill_at_100_pct_spent() {
        let (l, t) = limiter();
        record(&l, t, 10.0); // 100% spent
        assert!(l.check(&t, 10.0).is_err());
    }
}
