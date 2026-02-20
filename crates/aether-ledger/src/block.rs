//! Ledger block construction and hashing (PRD §14).
//!
//! Builds LedgerBlocks from execution events and computes SHA-256 hashes
//! for chain integrity. Ed25519 signing is optional (can be added later).

use chrono::Utc;
use sha2::{Digest, Sha256};

use aether_core::ids::{AgentId, LedgerBlockId, TaskId, TenantId, ToolId};
use aether_core::ledger::{BlockHash, LedgerAction, LedgerBlock, LedgerRef};
use aether_core::error::Result;

/// Builder for `LedgerBlock`.
///
/// Enforces that every required field is set before building.
pub struct LedgerBlockBuilder {
    tenant_id: TenantId,
    agent_id: AgentId,
    task_id: TaskId,
    action: LedgerAction,
    tool_id: Option<ToolId>,
    input: serde_json::Value,
    output: serde_json::Value,
    parent_hash: BlockHash,
    sequence_number: u64,
}

impl LedgerBlockBuilder {
    pub fn new(
        tenant_id: TenantId,
        agent_id: AgentId,
        task_id: TaskId,
        action: LedgerAction,
        parent_hash: BlockHash,
        sequence_number: u64,
    ) -> Self {
        Self {
            tenant_id,
            agent_id,
            task_id,
            action,
            tool_id: None,
            input: serde_json::Value::Null,
            output: serde_json::Value::Null,
            parent_hash,
            sequence_number,
        }
    }

    pub fn tool_id(mut self, id: ToolId) -> Self {
        self.tool_id = Some(id);
        self
    }

    pub fn input(mut self, v: serde_json::Value) -> Self {
        self.input = v;
        self
    }

    pub fn output(mut self, v: serde_json::Value) -> Self {
        self.output = v;
        self
    }

    /// Build the block, computing SHA-256 hashes for input and output.
    pub fn build(self) -> LedgerBlock {
        LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: self.sequence_number,
            timestamp_utc: Utc::now(),
            tenant_id: self.tenant_id,
            agent_id: self.agent_id,
            task_id: self.task_id,
            action: self.action,
            tool_id: self.tool_id,
            input_hash: hash_value(&self.input),
            output_hash: hash_value(&self.output),
            parent_hash: self.parent_hash,
            signature: None,
        }
    }
}

/// Compute SHA-256 hash of a JSON value, returned as lowercase hex string.
fn hash_value(value: &serde_json::Value) -> BlockHash {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    BlockHash(format!("{:x}", hasher.finalize()))
}

/// Build a compact `LedgerRef` from a full block.
pub fn block_to_ref(block: &LedgerBlock) -> LedgerRef {
    LedgerRef {
        block_id: block.id,
        sequence_number: block.sequence_number,
        action: block.action.clone(),
        timestamp_utc: block.timestamp_utc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ledger::BlockHash;

    fn genesis_builder() -> LedgerBlockBuilder {
        LedgerBlockBuilder::new(
            TenantId::new(),
            AgentId::new(),
            TaskId::new(),
            LedgerAction::ToolCall,
            BlockHash::genesis(),
            1,
        )
    }

    #[test]
    fn test_block_hash_is_64_chars() {
        let block = genesis_builder()
            .input(serde_json::json!({"query": "test"}))
            .output(serde_json::json!({"result": "ok"}))
            .build();
        assert!(block.input_hash.is_valid());
        assert!(block.output_hash.is_valid());
    }

    #[test]
    fn test_same_input_same_hash() {
        let input = serde_json::json!({"key": "value"});
        let h1 = hash_value(&input);
        let h2 = hash_value(&input);
        assert_eq!(h1.0, h2.0);
    }

    #[test]
    fn test_different_inputs_different_hashes() {
        let h1 = hash_value(&serde_json::json!({"a": 1}));
        let h2 = hash_value(&serde_json::json!({"a": 2}));
        assert_ne!(h1.0, h2.0);
    }

    #[test]
    fn test_block_ref_preserves_sequence() {
        let block = genesis_builder().build();
        let r = block_to_ref(&block);
        assert_eq!(r.sequence_number, 1);
    }
}
