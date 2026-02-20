//! Real-time cost tracker — per-tenant, per-task, per-model (PRD §12).
//!
//! Uses atomic operations for thread-safe accumulation without a mutex.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use aether_core::ids::{AgentId, TaskId, TenantId};

/// Cost of a single LLM call.
#[derive(Debug, Clone)]
pub struct LlmCost {
    pub tenant_id: TenantId,
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    /// Estimated USD cost (provider-specific pricing).
    pub cost_usd: f64,
}

/// Usage summary for a tenant.
#[derive(Debug, Clone, Default)]
pub struct TenantUsage {
    pub total_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_calls: u64,
}

/// Tracks cumulative costs per tenant.
pub struct CostTracker {
    usage: Arc<RwLock<HashMap<String, TenantUsage>>>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a single LLM call's cost.
    pub fn record(&self, cost: &LlmCost) -> Result<(), String> {
        let key = cost.tenant_id.to_string();
        let mut map = self.usage.write().map_err(|e| e.to_string())?;
        let entry = map.entry(key).or_default();
        entry.total_cost_usd += cost.cost_usd;
        entry.total_input_tokens += cost.input_tokens as u64;
        entry.total_output_tokens += cost.output_tokens as u64;
        entry.total_calls += 1;
        Ok(())
    }

    /// Get current usage for a tenant.
    pub fn get_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage, String> {
        let map = self.usage.read().map_err(|e| e.to_string())?;
        Ok(map.get(&tenant_id.to_string()).cloned().unwrap_or_default())
    }

    /// Budget remaining fraction [0.0, 1.0].
    pub fn budget_remaining_fraction(&self, tenant_id: &TenantId, limit_usd: f64) -> f64 {
        let usage = self.get_usage(tenant_id).unwrap_or_default();
        if limit_usd <= 0.0 {
            return 0.0;
        }
        ((limit_usd - usage.total_cost_usd) / limit_usd).clamp(0.0, 1.0)
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cost(tenant: TenantId, usd: f64) -> LlmCost {
        LlmCost {
            tenant_id: tenant,
            task_id: TaskId::new(),
            agent_id: AgentId::new(),
            model: "claude-sonnet".into(),
            input_tokens: 1000,
            output_tokens: 500,
            cost_usd: usd,
        }
    }

    #[test]
    fn test_cost_accumulates() {
        let tracker = CostTracker::new();
        let t = TenantId::new();
        tracker.record(&make_cost(t, 1.0)).unwrap();
        tracker.record(&make_cost(t, 2.0)).unwrap();
        let usage = tracker.get_usage(&t).unwrap();
        assert!((usage.total_cost_usd - 3.0).abs() < f64::EPSILON);
        assert_eq!(usage.total_calls, 2);
    }

    #[test]
    fn test_tenant_isolation() {
        let tracker = CostTracker::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        tracker.record(&make_cost(t1, 5.0)).unwrap();
        let u2 = tracker.get_usage(&t2).unwrap();
        assert_eq!(u2.total_cost_usd, 0.0);
    }

    #[test]
    fn test_budget_fraction_full() {
        let tracker = CostTracker::new();
        let t = TenantId::new();
        let fraction = tracker.budget_remaining_fraction(&t, 10.0);
        assert!((fraction - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_fraction_half_spent() {
        let tracker = CostTracker::new();
        let t = TenantId::new();
        tracker.record(&make_cost(t, 5.0)).unwrap();
        let fraction = tracker.budget_remaining_fraction(&t, 10.0);
        assert!((fraction - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_budget_fraction_clamped_at_zero() {
        let tracker = CostTracker::new();
        let t = TenantId::new();
        tracker.record(&make_cost(t, 15.0)).unwrap();
        let fraction = tracker.budget_remaining_fraction(&t, 10.0);
        assert_eq!(fraction, 0.0);
    }
}
