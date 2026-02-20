//! Ledger chain integrity verification (PRD §14).
//!
//! Verifies that a sequence of `LedgerBlock`s forms a valid hash chain:
//! each block's `parent_hash` must equal the hash of its predecessor.

use sha2::{Digest, Sha256};

use aether_core::error::{AetherError, Result};
use aether_core::ledger::{BlockHash, LedgerBlock};

/// Compute the block's own hash (used as the `parent_hash` by the next block).
///
/// Hashes the canonical string: `{id}|{seq}|{input_hash}|{output_hash}|{parent_hash}`
pub fn compute_block_hash(block: &LedgerBlock) -> BlockHash {
    let canonical = block.canonical_string();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    BlockHash(format!("{:x}", hasher.finalize()))
}

/// Verify the integrity of an ordered sequence of ledger blocks.
///
/// `blocks` must be sorted by `sequence_number` ascending.
///
/// # Errors
/// Returns `LedgerIntegrityViolation` if any block's parent hash does not
/// match the computed hash of the preceding block.
pub fn verify_chain(blocks: &[LedgerBlock]) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }

    // First block's parent must be the genesis hash
    let genesis = BlockHash::genesis();
    if blocks[0].parent_hash != genesis {
        return Err(AetherError::LedgerIntegrityViolation {
            block_id: blocks[0].id.to_string(),
            reason: "first block parent_hash is not genesis".into(),
        });
    }

    // Each subsequent block's parent_hash = hash of previous block
    for window in blocks.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        let expected_parent = compute_block_hash(prev);
        if curr.parent_hash != expected_parent {
            return Err(AetherError::LedgerIntegrityViolation {
                block_id: curr.id.to_string(),
                reason: format!(
                    "parent_hash mismatch at seq {}: expected {}, got {}",
                    curr.sequence_number, expected_parent.0, curr.parent_hash.0
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ids::{AgentId, LedgerBlockId, TaskId, TenantId};
    use aether_core::ledger::{BlockHash, LedgerAction, LedgerBlock};
    use chrono::Utc;

    fn make_genesis_block() -> LedgerBlock {
        LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: 1,
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
        }
    }

    #[test]
    fn test_empty_chain_is_valid() {
        assert!(verify_chain(&[]).is_ok());
    }

    #[test]
    fn test_single_block_valid_chain() {
        let block = make_genesis_block();
        assert!(verify_chain(&[block]).is_ok());
    }

    #[test]
    fn test_two_block_valid_chain() {
        let b1 = make_genesis_block();
        let b1_hash = compute_block_hash(&b1);

        let b2 = LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: 2,
            timestamp_utc: Utc::now(),
            tenant_id: b1.tenant_id,
            agent_id: b1.agent_id,
            task_id: b1.task_id,
            action: LedgerAction::MemoryWrite,
            tool_id: None,
            input_hash: BlockHash("c".repeat(64)),
            output_hash: BlockHash("d".repeat(64)),
            parent_hash: b1_hash,
            signature: None,
        };
        assert!(verify_chain(&[b1, b2]).is_ok());
    }

    #[test]
    fn test_tampered_chain_detected() {
        let b1 = make_genesis_block();
        let b2 = LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: 2,
            timestamp_utc: Utc::now(),
            tenant_id: b1.tenant_id,
            agent_id: b1.agent_id,
            task_id: b1.task_id,
            action: LedgerAction::MemoryWrite,
            tool_id: None,
            input_hash: BlockHash("c".repeat(64)),
            output_hash: BlockHash("d".repeat(64)),
            parent_hash: BlockHash("0".repeat(64)), // WRONG — tampered
            signature: None,
        };
        assert!(verify_chain(&[b1, b2]).is_err());
    }
}
