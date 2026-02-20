//! `aether-policy` — Centralized RBAC/ABAC policy engine (PRD §11).
//!
//! # Usage
//! ```rust,no_run
//! use aether_policy::engine::PolicyEngine;
//! use aether_policy::evaluation::EvaluationContext;
//!
//! let engine = PolicyEngine::default();
//! // let ctx = EvaluationContext::tool_execute(tenant, agent, task, tier, tool, access, budget, approved);
//! // let decision = engine.decide(&ctx);
//! ```

pub mod engine;
pub mod evaluation;
pub mod rules;

pub use engine::PolicyEngine;
pub use evaluation::{DecisionEffect, EvaluationContext, PolicyDecision, PolicyResource};
pub use rules::{AgentTier, PolicyAction, PolicyEffect, PolicyRule, PolicySubject, default_rules};
