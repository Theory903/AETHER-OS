//! In-memory ledger storage (PRD §14).
//!
//! Production: backed by PostgreSQL + Kafka write buffer.
//! This module provides the in-memory implementation for testing
//! and a Storage trait for swapping implementations.

use std::collections::HashMap;
use std::sync::RwLock;

use aether_core::error::{AetherError, Result};
use aether_core::ids::{LedgerBlockId, TenantId};
use aether_core::ledger::LedgerBlock;

/// Storage trait — allows swapping real DB for test double.
pub trait LedgerStorage: Send + Sync {
    /// Append a block. Must be called with monotonically increasing sequence numbers.
    fn append(&self, block: LedgerBlock) -> Result<()>;

    /// Fetch all blocks for a tenant, ordered by sequence number ascending.
    fn get_blocks(&self, tenant_id: &TenantId) -> Result<Vec<LedgerBlock>>;

    /// Get a single block by ID.
    fn get_block(&self, block_id: &LedgerBlockId) -> Result<LedgerBlock>;

    /// Count total blocks for a tenant.
    fn count(&self, tenant_id: &TenantId) -> Result<u64>;
}

/// In-memory ledger storage — suitable for testing.
pub struct InMemoryLedgerStorage {
    /// tenant_id → ordered list of blocks
    blocks: RwLock<HashMap<String, Vec<LedgerBlock>>>,
}

impl InMemoryLedgerStorage {
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryLedgerStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerStorage for InMemoryLedgerStorage {
    fn append(&self, block: LedgerBlock) -> Result<()> {
        let key = block.tenant_id.to_string();
        let mut store = self.blocks.write().map_err(|e| {
            AetherError::internal(format!("ledger lock poisoned: {e}"))
        })?;
        store.entry(key).or_default().push(block);
        Ok(())
    }

    fn get_blocks(&self, tenant_id: &TenantId) -> Result<Vec<LedgerBlock>> {
        let store = self.blocks.read().map_err(|e| {
            AetherError::internal(format!("ledger lock poisoned: {e}"))
        })?;
        Ok(store.get(&tenant_id.to_string()).cloned().unwrap_or_default())
    }

    fn get_block(&self, block_id: &LedgerBlockId) -> Result<LedgerBlock> {
        let store = self.blocks.read().map_err(|e| {
            AetherError::internal(format!("ledger lock poisoned: {e}"))
        })?;
        for blocks in store.values() {
            if let Some(b) = blocks.iter().find(|b| b.id == *block_id) {
                return Ok(b.clone());
            }
        }
        Err(AetherError::not_found("LedgerBlock", block_id))
    }

    fn count(&self, tenant_id: &TenantId) -> Result<u64> {
        let store = self.blocks.read().map_err(|e| {
            AetherError::internal(format!("ledger lock poisoned: {e}"))
        })?;
        Ok(store
            .get(&tenant_id.to_string())
            .map(|v| v.len() as u64)
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ids::{AgentId, LedgerBlockId, TaskId, TenantId};
    use aether_core::ledger::{BlockHash, LedgerAction, LedgerBlock};
    use chrono::Utc;

    fn make_block(tenant_id: TenantId, seq: u64) -> LedgerBlock {
        LedgerBlock {
            id: LedgerBlockId::new(),
            sequence_number: seq,
            timestamp_utc: Utc::now(),
            tenant_id,
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
    fn test_append_and_count() {
        let storage = InMemoryLedgerStorage::new();
        let t = TenantId::new();
        storage.append(make_block(t, 1)).unwrap();
        storage.append(make_block(t, 2)).unwrap();
        assert_eq!(storage.count(&t).unwrap(), 2);
    }

    #[test]
    fn test_get_block_by_id() {
        let storage = InMemoryLedgerStorage::new();
        let t = TenantId::new();
        let block = make_block(t, 1);
        let id = block.id;
        storage.append(block).unwrap();
        let fetched = storage.get_block(&id).unwrap();
        assert_eq!(fetched.id, id);
    }

    #[test]
    fn test_get_block_not_found() {
        let storage = InMemoryLedgerStorage::new();
        let missing = LedgerBlockId::new();
        assert!(storage.get_block(&missing).is_err());
    }

    #[test]
    fn test_tenant_isolation() {
        let storage = InMemoryLedgerStorage::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        storage.append(make_block(t1, 1)).unwrap();
        // t2 has no blocks
        assert_eq!(storage.count(&t2).unwrap(), 0);
        assert_eq!(storage.count(&t1).unwrap(), 1);
    }
}
