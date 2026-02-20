//! Session manager — per-session state keyed by SessionKey (PRD §16).

use dashmap::DashMap;

use crate::history::ConversationHistory;
use crate::keys::SessionKey;

/// A single conversation session.
#[derive(Debug, Clone)]
pub struct Session {
    pub key: SessionKey,
    pub history: ConversationHistory,
    pub summary: Option<String>,
    /// Number of messages before history was summarized last.
    pub turns_since_summary: usize,
}

impl Session {
    fn new(key: SessionKey) -> Self {
        Self {
            key,
            history: ConversationHistory::new(),
            summary: None,
            turns_since_summary: 0,
        }
    }
}

/// Thread-safe session store — concurrent access via DashMap (fine-grained sharding).
pub struct SessionManager {
    sessions: DashMap<String, Session>,
    /// Auto-truncate when history exceeds this length.
    max_history: usize,
}

impl SessionManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            sessions: DashMap::new(),
            max_history,
        }
    }

    /// Retrieve an existing session or create a new one.
    pub fn get_or_create(&self, key: &SessionKey) -> Session {
        let k = key.to_string();
        if let Some(s) = self.sessions.get(&k) {
            return s.clone();
        }
        let s = Session::new(key.clone());
        self.sessions.insert(k, s.clone());
        s
    }

    /// Add a user turn and save back.
    pub fn add_user_turn(&self, key: &SessionKey, user: &str, assistant: &str) {
        let k = key.to_string();
        let mut entry = self.sessions.entry(k).or_insert_with(|| Session::new(key.clone()));
        entry.history.add_user(user);
        entry.history.add_assistant(assistant);
        entry.turns_since_summary += 1;
        if entry.history.len() > self.max_history {
            entry.history.truncate_to_last(self.max_history);
        }
    }

    /// Set the summary for a session.
    pub fn set_summary(&self, key: &SessionKey, summary: String) {
        if let Some(mut s) = self.sessions.get_mut(&key.to_string()) {
            s.summary = Some(summary);
        }
    }

    /// Return the current history length for a session.
    pub fn history_len(&self, key: &SessionKey) -> usize {
        self.sessions
            .get(&key.to_string())
            .map(|s| s.history.len())
            .unwrap_or(0)
    }

    /// Delete a session (e.g., after agent completion).
    pub fn remove(&self, key: &SessionKey) {
        self.sessions.remove(&key.to_string());
    }

    /// Total active sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::ids::{AgentId, TenantId};

    fn make_key() -> SessionKey {
        SessionKey::new(TenantId::new(), AgentId::new(), "discord", "user42")
    }

    #[test]
    fn test_get_or_create_new_session() {
        let mgr = SessionManager::new(100);
        let key = make_key();
        let s = mgr.get_or_create(&key);
        assert!(s.history.is_empty());
    }

    #[test]
    fn test_add_turn_grows_history() {
        let mgr = SessionManager::new(100);
        let key = make_key();
        mgr.add_user_turn(&key, "hello", "hi there");
        assert_eq!(mgr.history_len(&key), 2);
    }

    #[test]
    fn test_max_history_truncates() {
        let mgr = SessionManager::new(4);
        let key = make_key();
        for _ in 0..5 {
            mgr.add_user_turn(&key, "msg", "resp");
        }
        // 5 turns = 10 messages, but max_history = 4
        assert!(mgr.history_len(&key) <= 4);
    }

    #[test]
    fn test_remove_session() {
        let mgr = SessionManager::new(100);
        let key = make_key();
        mgr.get_or_create(&key);
        assert_eq!(mgr.len(), 1);
        mgr.remove(&key);
        assert_eq!(mgr.len(), 0);
    }
}
