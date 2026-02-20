//! `aether-ledger` — Append-only cryptographic audit ledger (PRD §14).
//!
//! # Design
//! - Blocks are SHA-256 hash-chained for tamper detection
//! - Ed25519 signing optional (can be layered on later)
//! - Tenant-isolated storage
//! - In-memory storage for tests; PostgreSQL + Kafka for production

pub mod block;
pub mod chain;
pub mod storage;
pub mod verify;

pub use block::{LedgerBlockBuilder, block_to_ref};
pub use chain::{compute_block_hash, verify_chain};
pub use storage::{InMemoryLedgerStorage, LedgerStorage};
pub use verify::LedgerVerifier;
