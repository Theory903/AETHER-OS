//! Typed UUID newtypes for every domain entity.
//!
//! Using newtypes prevents accidental mixing of IDs at compile time.
//! E.g. passing an `AgentId` where a `TenantId` is expected is a compile error.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generate a new random ID.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wrap an existing UUID.
            #[must_use]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// Return the inner UUID.
            #[must_use]
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

define_id!(TenantId, "Unique identifier for a tenant.");
define_id!(AgentId, "Unique identifier for an agent instance.");
define_id!(TaskId, "Unique identifier for a task/intent.");
define_id!(ToolId, "Unique identifier for a registered tool.");
define_id!(WorkflowId, "Unique identifier for a workflow definition.");
define_id!(LedgerBlockId, "Unique identifier for a ledger block.");
define_id!(SessionId, "Unique identifier for a conversation session.");
define_id!(WorkerId, "Unique identifier for a VM/worker.");
define_id!(RequestId, "Unique identifier for an API request (tracing).");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ids_are_unique() {
        let a = TenantId::new();
        let b = TenantId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_display_is_uuid_format() {
        let id = AgentId::new();
        let s = id.to_string();
        assert_eq!(s.len(), 36); // UUID v4 format
    }

    #[test]
    fn test_roundtrip_through_str() {
        let id = TaskId::new();
        let parsed: TaskId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_serde_roundtrip() {
        let id = WorkflowId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: WorkflowId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn test_type_safety_compile_time() {
        // This verifies that different ID types cannot be interchanged.
        // The following would fail to compile if uncommented:
        // let tenant: TenantId = AgentId::new(); // type mismatch
        let _ = TenantId::new();
        let _ = AgentId::new();
    }
}
