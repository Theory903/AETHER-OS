//! Ledger domain model — append-only cryptographic audit (PRD §14).
//!
//! Every tool execution, memory write, agent spawn, and deploy is recorded
//! as an immutable ledger block. Blocks form a hash chain for tamper detection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{AgentId, LedgerBlockId, TaskId, TenantId, ToolId};

/// SHA-256 hash represented as a hex string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockHash(pub String);

impl BlockHash {
    /// Genesis block parent hash — all zeros.
    #[must_use]
    pub fn genesis() -> Self {
        Self("0".repeat(64))
    }

    /// Validate hash length is 64 hex chars (SHA-256).
    pub fn is_valid(&self) -> bool {
        self.0.len() == 64 && self.0.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// The type of action recorded in a ledger block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerAction {
    /// Tool was invoked by an agent.
    ToolCall,
    /// Code was written or modified.
    CodeChange,
    /// Memory was written.
    MemoryWrite,
    /// Memory was deleted.
    MemoryDelete,
    /// Agent was spawned.
    AgentSpawn,
    /// Agent completed or was terminated.
    AgentComplete,
    /// A deployment action was triggered.
    Deploy,
    /// Human approved or rejected an output.
    HumanReview,
    /// A tool was created by the tool factory.
    ToolCreated,
    /// A compensation action was executed.
    Compensation,
}

/// An immutable ledger block.
///
/// Each block records one action and chains to its predecessor via `parent_hash`.
/// Chain integrity is verified by re-hashing the block and comparing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBlock {
    pub id: LedgerBlockId,
    /// Monotonically increasing per tenant.
    pub sequence_number: u64,
    pub timestamp_utc: DateTime<Utc>,
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub action: LedgerAction,
    /// Which tool was invoked (if action = ToolCall).
    pub tool_id: Option<ToolId>,
    /// SHA-256 of the serialized input.
    pub input_hash: BlockHash,
    /// SHA-256 of the serialized output.
    pub output_hash: BlockHash,
    /// Hash of the previous block in this tenant's chain.
    pub parent_hash: BlockHash,
    /// Ed25519 signature of (id + sequence + input_hash + output_hash + parent_hash).
    pub signature: Option<String>,
}

impl LedgerBlock {
    /// Build the string that should be signed/hashed for chain verification.
    ///
    /// Format: `{block_id}|{seq}|{input_hash}|{output_hash}|{parent_hash}`
    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.id,
            self.sequence_number,
            self.input_hash.0,
            self.output_hash.0,
            self.parent_hash.0,
        )
    }
}

/// Compact reference to a ledger block used in tool results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerRef {
    pub block_id: LedgerBlockId,
    pub sequence_number: u64,
    pub action: LedgerAction,
    pub timestamp_utc: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_hash_is_64_chars() {
        let h = BlockHash::genesis();
        assert_eq!(h.0.len(), 64);
        assert!(h.is_valid());
    }

    #[test]
    fn test_invalid_hash_detected() {
        let h = BlockHash("short".into());
        assert!(!h.is_valid());
    }

    #[test]
    fn test_canonical_string_is_deterministic() {
        let block = LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: 42,
            timestamp_utc: Utc::now(),
            tenant_id: TenantId::new(),
            agent_id: AgentId::new(),
            task_id: TaskId::new(),
            action: LedgerAction::ToolCall,
            tool_id: None,
            input_hash: BlockHash("a".repeat(64)),
            output_hash: BlockHash("b".repeat(64)),
            parent_hash: BlockHash::genesis(),
            signature: None,
        };
        let s1 = block.canonical_string();
        let s2 = block.canonical_string();
        assert_eq!(s1, s2);
        assert!(s1.contains("42"));
    }
}
