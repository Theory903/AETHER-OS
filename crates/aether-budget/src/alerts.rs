//! Budget alert notification types (PRD §12).

use aether_core::ids::TenantId;

/// A budget alert event — emitted when a threshold is crossed.
#[derive(Debug, Clone)]
pub struct BudgetAlert {
    pub tenant_id: TenantId,
    pub alert_type: AlertType,
    pub spent_usd: f64,
    pub limit_usd: f64,
    pub pct_used: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertType {
    /// 75% of budget consumed.
    Warning,
    /// 90% of budget consumed — model degraded.
    Critical,
    /// 100% consumed — all tasks killed.
    Exhausted,
}

impl BudgetAlert {
    pub fn new(tenant_id: TenantId, spent_usd: f64, limit_usd: f64) -> Self {
        let pct = if limit_usd > 0.0 {
            (spent_usd / limit_usd * 100.0).min(100.0)
        } else {
            100.0
        };
        let alert_type = if pct >= 100.0 {
            AlertType::Exhausted
        } else if pct >= 90.0 {
            AlertType::Critical
        } else {
            AlertType::Warning
        };
        Self {
            tenant_id,
            alert_type,
            spent_usd,
            limit_usd,
            pct_used: pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_type_at_75_pct() {
        let a = BudgetAlert::new(TenantId::new(), 7.5, 10.0);
        assert_eq!(a.alert_type, AlertType::Warning);
    }

    #[test]
    fn test_alert_type_at_90_pct() {
        let a = BudgetAlert::new(TenantId::new(), 9.0, 10.0);
        assert_eq!(a.alert_type, AlertType::Critical);
    }

    #[test]
    fn test_alert_type_at_100_pct() {
        let a = BudgetAlert::new(TenantId::new(), 10.0, 10.0);
        assert_eq!(a.alert_type, AlertType::Exhausted);
    }
}
