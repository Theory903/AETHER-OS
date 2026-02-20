//! `aether-budget` — Cost governance with kill switch (PRD §12).

pub mod alerts;
pub mod limiter;
pub mod tracker;

pub use alerts::{AlertType, BudgetAlert};
pub use limiter::{BudgetAction, BudgetLimiter, ALERT_THRESHOLD, DEGRADE_THRESHOLD, KILL_THRESHOLD};
pub use tracker::{CostTracker, LlmCost, TenantUsage};
