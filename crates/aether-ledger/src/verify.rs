//! High-level verification API — combines chain + storage (PRD §14).
//!
//! `LedgerVerifier` is the entry point for verifying a tenant's full chain
//! or auditing specific block ranges.

use aether_core::error::Result;
use aether_core::ids::TenantId;

use crate::chain::verify_chain;
use crate::storage::LedgerStorage;

/// Verifies a tenant's full ledger chain from storage.
pub struct LedgerVerifier<S: LedgerStorage> {
    storage: S,
}

impl<S: LedgerStorage> LedgerVerifier<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Fetch all blocks for a tenant and verify the hash chain.
    ///
    /// # Errors
    /// Returns `LedgerIntegrityViolation` if chain is broken.
    pub fn verify_tenant(&self, tenant_id: &TenantId) -> Result<VerificationReport> {
        let blocks = self.storage.get_blocks(tenant_id)?;
        let count = blocks.len() as u64;
        verify_chain(&blocks)?;
        Ok(VerificationReport {
            tenant_id: *tenant_id,
            blocks_verified: count,
            intact: true,
        })
    }
}

/// Result of a ledger verification.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub tenant_id: TenantId,
    pub blocks_verified: u64,
    pub intact: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ids::{AgentId, LedgerBlockId, TaskId, TenantId};
    use aether_core::ledger::{BlockHash, LedgerAction, LedgerBlock};
    use crate::chain::compute_block_hash;
    use crate::storage::InMemoryLedgerStorage;
    use chrono::Utc;

    fn make_block(tenant: TenantId, seq: u64, parent: BlockHash) -> LedgerBlock {
        LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: seq,
            timestamp_utc: Utc::now(),
            tenant_id: tenant,
            agent_id: AgentId::new(),
            task_id: TaskId::new(),
            action: LedgerAction::ToolCall,
            tool_id: None,
            input_hash: BlockHash("a".repeat(64)),
            output_hash: BlockHash("b".repeat(64)),
            parent_hash: parent,
            signature: None,
        }
    }

    #[test]
    fn test_empty_tenant_verifies_ok() {
        let storage = InMemoryLedgerStorage::new();
        let verifier = LedgerVerifier::new(storage);
        let t = TenantId::new();
        let report = verifier.verify_tenant(&t).unwrap();
        assert!(report.intact);
        assert_eq!(report.blocks_verified, 0);
    }

    #[test]
    fn test_valid_chain_verifies_ok() {
        let storage = InMemoryLedgerStorage::new();
        let t = TenantId::new();
        let b1 = make_block(t, 1, BlockHash::genesis());
        let b1_hash = crate::chain::compute_block_hash(&b1);
        let b2 = make_block(t, 2, b1_hash);
        storage.append(b1).unwrap();
        storage.append(b2).unwrap();

        let verifier = LedgerVerifier::new(storage);
        let report = verifier.verify_tenant(&t).unwrap();
        assert!(report.intact);
        assert_eq!(report.blocks_verified, 2);
    }

    #[test]
    fn test_broken_chain_fails_verification() {
        let storage = InMemoryLedgerStorage::new();
        let t = TenantId::new();
        // b2's parent_hash intentionally wrong
        let b1 = make_block(t, 1, BlockHash::genesis());
        let b2 = make_block(t, 2, BlockHash("bad_hash".repeat(8))); // wrong
        storage.append(b1).unwrap();
        storage.append(b2).unwrap();

        let verifier = LedgerVerifier::new(storage);
        assert!(verifier.verify_tenant(&t).is_err());
    }
}
