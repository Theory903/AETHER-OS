//! Session key format — tenant:agent:channel:peer (PRD §16).
//!
//! Format: `tenant:{id}:agent:{id}:channel:{name}:peer:{id}`

use std::fmt;

use aether_core::ids::{AgentId, TenantId};

/// Structured session key.
///
/// Format: `tenant:{tenant_id}:agent:{agent_id}:channel:{channel}:peer:{peer_id}`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionKey {
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub channel: String,
    pub peer_id: String,
}

impl SessionKey {
    pub fn new(
        tenant_id: TenantId,
        agent_id: AgentId,
        channel: impl Into<String>,
        peer_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            agent_id,
            channel: channel.into(),
            peer_id: peer_id.into(),
        }
    }

    /// Parse a session key string back into a `SessionKey`.
    ///
    /// Expected format: `tenant:{id}:agent:{id}:channel:{name}:peer:{id}`
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(8, ':').collect();
        if parts.len() != 8 {
            return None;
        }
        // parts: ["tenant", id, "agent", id, "channel", name, "peer", id]
        if parts[0] != "tenant" || parts[2] != "agent" || parts[4] != "channel" || parts[6] != "peer" {
            return None;
        }
        let tenant_id: TenantId = parts[1].parse().ok()?;
        let agent_id: AgentId = parts[3].parse().ok()?;
        Some(Self {
            tenant_id,
            agent_id,
            channel: parts[5].to_string(),
            peer_id: parts[7].to_string(),
        })
    }
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tenant:{}:agent:{}:channel:{}:peer:{}",
            self.tenant_id, self.agent_id, self.channel, self.peer_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key() -> SessionKey {
        SessionKey::new(TenantId::new(), AgentId::new(), "discord", "user123")
    }

    #[test]
    fn test_display_format() {
        let key = make_key();
        let s = key.to_string();
        assert!(s.starts_with("tenant:"));
        assert!(s.contains(":agent:"));
        assert!(s.contains(":channel:discord:peer:user123"));
    }

    #[test]
    fn test_parse_roundtrip() {
        let key = make_key();
        let s = key.to_string();
        let parsed = SessionKey::parse(&s).expect("parse should succeed");
        assert_eq!(parsed, key);
    }

    #[test]
    fn test_parse_invalid_returns_none() {
        assert!(SessionKey::parse("invalid-key").is_none());
        assert!(SessionKey::parse("").is_none());
    }
}
